use std::{env, path::Path};

use serde::Deserialize;
use tokio::fs::{create_dir, read_to_string};

fn default_cache_directory() -> String {
    "cache".to_string()
}

fn default_cache_ttl() -> u64 {
    60 * 24
}

#[derive(Deserialize, Clone)]
pub struct Repository {
    pub name: String,
    pub url: String,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    #[serde(default, rename = "repository")]
    pub repositories: Vec<Repository>,
    #[serde(default = "default_cache_directory")]
    pub cache_directory: String,
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,
}

impl Config {
    pub fn get_repository(&self, name: &str) -> Option<&Repository> {
        self.repositories.iter().find(|&repo| repo.name == name)
    }

    async fn initialize(&mut self) {
        if let Ok(cache_path) = env::var("CACHE_PATH") {
            if self.cache_directory == default_cache_directory() {
                self.cache_directory = cache_path;
            }
        }
        
        let path = Path::new(&self.cache_directory);
        if !path.exists() {
            create_dir(path)
                .await
                .expect("Error while creating cache directory");
        }

        for repository in self.repositories.iter_mut() {
            let repository_cache_path = &format!("{}/{}", self.cache_directory, repository.name);
            let repository_cache_path = Path::new(&repository_cache_path);
            if !repository_cache_path.exists() {
                create_dir(repository_cache_path)
                    .await
                    .expect("Error while creating repository directory inside the cache");
            }

            if !repository.url.ends_with('/') {
                repository.url = format!("{}/", repository.url);
            }
        }
    }
}

pub async fn read_config() -> Config {
    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
    let config_toml = read_to_string(config_path)
        .await
        .expect("Error while reading config file");
    let mut config: Config = toml::from_str(&config_toml).expect("Error while parsing config file");
    config.initialize().await;
    config
}
