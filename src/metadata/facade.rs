use reqwest::Client;

use crate::metadata::{
    common_metadata::ItemMetadata,
    providers::{
        crosseref::CrossrefManager, openalex::OpenAlexManager, openlibrary::OpenLibraryManager,
    },
    proxy::GenericMetadataFetcher,
};

/// Facade that hides all the metadata providers, aggregate their results
/// with a generic simple API
pub struct MetadataProvider {
    client: Client,
    providers: Vec<Box<dyn GenericMetadataFetcher>>,
}

impl MetadataProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            providers: vec![
                Box::new(OpenLibraryManager::new()),
                Box::new(CrossrefManager::new()),
                Box::new(OpenAlexManager::new()),
            ],
        }
    }

    pub async fn fetch(&self, title: &str) -> Vec<Box<dyn ItemMetadata>> {
        let mut res = Vec::new();
        for provider in &self.providers {
            let request = provider.fetch_metadata(&self.client, title);
            if let Ok(metadata) = request.await {
                res.extend(metadata)
            }
        }
        res
    }
}
