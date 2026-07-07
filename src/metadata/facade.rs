use reqwest::Client;

use crate::metadata::{common_metadata::ItemMetadata, proxy::MetadataFetcher};

#[derive(Clone, Debug)]
pub struct MetadataProvider<T: ItemMetadata + Sized> {
    client: Client,
    providers: Vec<Box<dyn MetadataFetcher<T>>>,
}

impl MetadataProvider {
    pub fn new() {}

    pub async fn fetch(&self, title: &str) -> Vec<Box<dyn ItemMetadata>> {
        vec![]
    }
}
