use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Client;

use crate::metadata::common_metadata::ItemMetadata;

#[async_trait]
pub trait MetadataFetcher: Send + Sync {
    async fn fetch(
        &self,
        client: Arc<Client>,
        title: String,
    ) -> color_eyre::Result<Vec<Box<dyn ItemMetadata>>>;

    async fn fetch_abstract(
        &self,
        _client: Arc<Client>,
        _title: String,
        _doi: Option<String>,
        _isbn: Option<String>,
    ) -> color_eyre::Result<Option<String>> {
        Ok(None)
    }
}
