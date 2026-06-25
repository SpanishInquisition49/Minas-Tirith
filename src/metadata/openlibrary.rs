use anyhow::Context;
use reqwest::Client;
use serde::Deserialize;

use crate::metadata::{
    common_metadata::{ItemMetadata, ItemType},
    proxy::MetadataFetcher,
};

pub struct OpenLibraryManager {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct OpenLibraryResponse {
    docs: Vec<OpenLibraryDoc>,
}

#[derive(Debug, Deserialize)]
struct OpenLibraryDoc {
    title: String,
    author_name: Option<Vec<String>>,
    first_publish_year: Option<i32>,
    cover_i: Option<i64>,
    key: String,
    ia: Option<Vec<String>>,
}

fn extract_isbn_from_ia(ia: &Option<Vec<String>>) -> Option<String> {
    ia.as_ref()?
        .iter()
        .find_map(|s| s.strip_prefix("isbn_").map(|isbn| isbn.to_string()))
}

impl From<OpenLibraryDoc> for ItemMetadata {
    fn from(value: OpenLibraryDoc) -> Self {
        Self {
            title: value.title,
            item_type: ItemType::Book,
            authors: value.author_name.unwrap_or_default(),
            isbn: extract_isbn_from_ia(&value.ia),
            doi: None,
            publication_date: value.first_publish_year.map(|y| y.to_string()),
            cover_image_url: value
                .cover_i
                .map(|id| format!("https://covers.openlibrary.org/b/id/{}-L.jpg", id)),
            source: "openlibrary".to_string(),
            source_id: Some(value.key),
        }
    }
}

impl OpenLibraryManager {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl MetadataFetcher for OpenLibraryManager {
    const BASE_URL: &str = "https://openlibrary.org/search.json?title=";

    async fn fetch(&self, title: &str) -> anyhow::Result<Vec<ItemMetadata>> {
        let res = self
            .client
            .get(Self::BASE_URL)
            .query(&[("title", title), ("limit", "1")])
            .send()
            .await
            .context("Open Library API call")?;

        let parsed: OpenLibraryResponse = res.json().await.context("Open Library JSON parsing")?;

        Ok(parsed.docs.into_iter().map(ItemMetadata::from).collect())
    }
}
