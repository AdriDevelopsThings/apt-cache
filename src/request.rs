use std::{
    path::Path,
    time::{Duration, SystemTime},
};

use async_stream::stream;
use bytes::Bytes;
use futures_core::{stream::BoxStream, Stream};
use futures_util::{pin_mut, StreamExt};
use reqwest::StatusCode;
use sha1::{Digest, Sha1};
use tokio::{
    fs::{metadata, read_dir, remove_file, File},
    io::AsyncWriteExt,
    sync::mpsc,
    time::sleep,
};
use tokio_util::io::ReaderStream;

use crate::{config::Config, error::AptCacheError};

fn get_cache_file_path(config: &Config, repository: &str, path: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(path.as_bytes());
    let hash = hasher.finalize();
    format!("{}/{repository}/{:x}", config.cache_directory, hash)
}

// create a cache file
async fn cache_file(
    config: &Config,
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>>,
    repository: &str,
    path: &str,
) -> impl Stream<Item = Bytes> {
    let config_path = get_cache_file_path(config, repository, path);
    let mut file = File::create(config_path).await.unwrap();

    let (tx, mut rx) = mpsc::channel::<Bytes>(1024);

    tokio::spawn(async move {
        while let Some(bytes) = rx.recv().await {
            file.write_all(&bytes).await.unwrap();
        }
    });

    stream! {
        pin_mut!(stream);
        while let Some(item) = stream.next().await {
            let bytes = item.unwrap();
            tx.send(bytes.clone()).await.unwrap();
            yield bytes;
        }
    }
}

async fn check_file_ttl(config: &Config, cache_file: &Path) -> bool {
    let modified = metadata(cache_file).await.unwrap().modified().unwrap();
    let duration = SystemTime::now().duration_since(modified).unwrap();
    let ttl = Duration::from_secs(config.cache_ttl * 60);
    if duration >= ttl {
        remove_file(cache_file).await.unwrap();
        return false;
    }
    true
}

async fn get_cached_file(
    config: &Config,
    repository: &str,
    path: &str,
) -> Option<impl Stream<Item = Bytes>> {
    let cache_path = get_cache_file_path(config, repository, path);
    let cache_path = Path::new(&cache_path);
    if cache_path.exists() && check_file_ttl(config, cache_path).await {
        let file = File::open(cache_path).await.unwrap();
        let mut stream = ReaderStream::new(file);
        Some(stream! {
            while let Some(bytes) = stream.next().await {
                yield bytes.unwrap();
            }
        })
    } else {
        None
    }
}

// this function should return a stream that will be written to the response body
pub async fn request_file<'a>(
    config: &Config,
    repository: &str,
    path: &str,
) -> Result<(StatusCode, bool, BoxStream<'a, Bytes>), AptCacheError> {
    let repo = config
        .get_repository(repository)
        .ok_or(AptCacheError::RepositoryNotFound)?;
    if let Some(cached) = get_cached_file(config, repository, path).await {
        return Ok((StatusCode::OK, true, Box::pin(cached)));
    }
    let url = format!("{}{}", repo.url, path);
    let response = reqwest::get(&url).await.unwrap();
    let status = response.status();
    let mut stream = response.bytes_stream();
    if status != StatusCode::OK {
        let stream = stream! {
            while let Some(bytes) = stream.next().await {
                yield bytes.unwrap();
            }
        };
        return Ok((status, false, Box::pin(stream)));
    }
    Ok((
        StatusCode::OK,
        false,
        Box::pin(cache_file(config, stream, repository, path).await),
    ))
}

pub async fn run_cache_ttl_worker(config: Config) {
    loop {
        let mut repositories = read_dir(&config.cache_directory).await.unwrap();
        while let Some(repository) = repositories.next_entry().await.unwrap() {
            if repository.metadata().await.unwrap().is_dir() {
                let mut cache_files = read_dir(repository.path()).await.unwrap();
                while let Some(cache_file) = cache_files.next_entry().await.unwrap() {
                    if cache_file.metadata().await.unwrap().is_file() {
                        check_file_ttl(&config, cache_file.path().as_path()).await;
                    }
                }
            }
        }
        sleep(Duration::from_secs(60)).await;
    }
}
