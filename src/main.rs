use std::{env, io, net::SocketAddr, str::FromStr};

use axum::{body::StreamBody, extract, http::HeaderMap, routing, Router, Server};
use bytes::Bytes;
use config::{read_config, Config};
use error::AptCacheError;
use futures_core::Stream;
use futures_util::StreamExt;
use request::request_file;
use reqwest::StatusCode;

use crate::request::run_cache_ttl_worker;

mod config;
mod error;
mod request;

async fn get_repositry(
    extract::Path((repository_name, path)): extract::Path<(String, String)>,
    extract::State(config): extract::State<Config>,
) -> Result<
    (
        StatusCode,
        HeaderMap,
        StreamBody<impl Stream<Item = io::Result<Bytes>>>,
    ),
    AptCacheError,
> {
    let (status_code, cached, stream) = request_file(&config, &repository_name, &path).await?;
    let stream = stream.map(io::Result::Ok);
    let mut headers = HeaderMap::new();
    headers.insert(
        "X-Cached",
        match cached {
            true => "true",
            false => "false",
        }
        .parse()
        .unwrap(),
    );
    Ok((status_code, headers, StreamBody::new(stream)))
}

#[tokio::main]
async fn main() {
    let config = read_config().await;
    let app = Router::new()
        .route("/:repository_name/*path", routing::get(get_repositry))
        .with_state(config.clone());
    let listen_address =
        env::var("LISTEN_ADDRESS").unwrap_or_else(|_| String::from("127.0.0.1:8000"));
    let addr = SocketAddr::from_str(&listen_address).expect("Invalid listen address");
    tokio::spawn(run_cache_ttl_worker(config));
    println!("Server listening on {listen_address}");
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Error while listening");
}
