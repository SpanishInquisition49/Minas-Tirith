use std::sync::Arc;

use reqwest::Client;

use crate::metadata::{
    common_metadata::ItemMetadata,
    providers::{
        crosseref::CrossrefManager, openalex::OpenAlexManager, openlibrary::OpenLibraryManager,
    },
    proxy::MetadataFetcher,
};

/// Facade that hides all the metadata providers, aggregate their results
/// with a generic simple API
pub struct MetadataProvider {
    client: Arc<Client>,
    providers: Vec<Arc<dyn MetadataFetcher>>,
}

impl MetadataProvider {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Client::new()),
            providers: vec![
                Arc::new(OpenLibraryManager::new()),
                Arc::new(CrossrefManager::new()),
                Arc::new(OpenAlexManager::new()),
            ],
        }
    }

    pub async fn fetch(&self, title: &str) -> Vec<Box<dyn ItemMetadata>> {
        let mut res = Vec::new();
        let mut tasks = tokio::task::JoinSet::new();
        for provider in self.providers.iter() {
            let client = self.client.clone();
            let title = title.to_string();
            let p = provider.clone();
            tasks.spawn(async move { p.fetch(client, title).await });
        }

        for metadatas in tasks.join_all().await.into_iter().flatten() {
            res.extend(metadatas);
        }
        res
    }
}
