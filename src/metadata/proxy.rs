use std::pin::Pin;

use reqwest::Client;

use crate::metadata::common_metadata::ItemMetadata;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[trait_variant::make(Send)]
pub trait MetadataFetcher: Send + Sync {
    type Item: ItemMetadata;
    async fn fetch(&self, client: &Client, title: &str) -> color_eyre::Result<Vec<Self::Item>>;
}

/// Common Interface for every metadata provider
pub trait GenericMetadataFetcher: Send + Sync {
    fn fetch_metadata<'a>(
        &'a self,
        client: &'a Client,
        title: &'a str,
    ) -> BoxFuture<'a, color_eyre::Result<Vec<Box<dyn ItemMetadata>>>>;
}

impl<F> GenericMetadataFetcher for F
where
    F: MetadataFetcher + Send + Sync + 'static,
{
    fn fetch_metadata<'a>(
        &'a self,
        client: &'a Client,
        title: &'a str,
    ) -> BoxFuture<'a, color_eyre::Result<Vec<Box<dyn ItemMetadata>>>> {
        Box::pin(async move {
            let items = self.fetch(client, title).await?;
            Ok(items
                .into_iter()
                .map(|i| Box::new(i) as Box<dyn ItemMetadata>)
                .collect())
        })
    }
}
