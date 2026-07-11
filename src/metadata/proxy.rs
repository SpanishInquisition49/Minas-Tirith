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
}
