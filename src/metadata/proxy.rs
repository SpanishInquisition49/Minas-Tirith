use crate::metadata::common_metadata::ItemMetadata;

/// Common Interface for every metadata provider
pub trait MetadataFetcher<T: ItemMetadata + Sized> {
    const BASE_URL: &str;
    async fn fetch(&self, title: &str) -> color_eyre::Result<Vec<T>>;
}
