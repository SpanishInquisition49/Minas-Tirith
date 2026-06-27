use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
};

use color_eyre::eyre::Context;
use reqwest::Client;

#[derive(Clone, Debug)]
pub struct ImageCache {
    client: Client,
    cache_path: PathBuf,
}

impl ImageCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            client: Client::new(),
            cache_path: cache_dir,
        }
    }

    fn cache_path(&self, url: &str) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let hash = hasher.finish();

        let extension = url
            .rsplit('.')
            .next()
            .filter(|e| e.len() < 4 && e.chars().all(|c| c.is_ascii_alphabetic()))
            .unwrap_or(".jpg");

        self.cache_path.join(format!("{hash:x}.{extension}"))
    }

    pub async fn get_or_download(&self, url: &str) -> color_eyre::Result<PathBuf> {
        let path = self.cache_path(url);
        if path.exists() {
            return Ok(path);
        }
        let bytes = self
            .client
            .get(url)
            .send()
            .await
            .context("Downloading Cover Image")?
            .error_for_status()
            .context("Cover Image request returned an error status")?
            .bytes()
            .await
            .context("Reading cover image bytes")?;

        let _ = tokio::fs::write(&path, &bytes)
            .await
            .context("Writing cover image to cache");
        Ok(path)
    }
}
