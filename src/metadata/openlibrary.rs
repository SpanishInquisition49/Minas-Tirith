use color_eyre::eyre::Context;
use reqwest::Client;
use serde::Deserialize;
use slug::slugify;

use crate::metadata::{
    common_metadata::{ItemMetadata, ItemType},
    proxy::MetadataFetcher,
};

pub struct OpenLibraryManager {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct OpenLibraryResponse {
    docs: Vec<OpenLibraryItem>,
}

#[derive(Debug, Deserialize)]
pub struct OpenLibraryItem {
    title: String,
    author_name: Option<Vec<String>>,
    first_publish_year: Option<i32>,
    #[serde(rename = "cover_i")]
    cover_url: Option<i64>,
    key: String,
    ia: Option<Vec<String>>,
}

impl ItemMetadata for OpenLibraryItem {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn item_type(&self) -> ItemType {
        // NOTE: maybe this is too specific
        ItemType::Book
    }

    fn authors(&self) -> Vec<String> {
        match &self.author_name {
            Some(a) => a.clone(),
            None => vec![],
        }
    }

    fn isbn(&self) -> Option<String> {
        if let Some(ia) = &self.ia {
            ia.iter()
                .find_map(|s| s.strip_prefix("isbn_"))
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    fn doi(&self) -> Option<String> {
        None
    }

    fn publication_date(&self) -> Option<String> {
        self.first_publish_year.map(|y| y.to_string())
    }

    fn cover_image_url(&self) -> Option<String> {
        self.cover_url
            .map(|id| format!("https://covers.openlibrary.org/b/id/{id}-L.jpg"))
    }

    fn source(&self) -> String {
        "openlibrary".to_string()
    }

    fn source_id(&self) -> Option<String> {
        Some(self.key.clone())
    }

    fn description(&self) -> Option<String> {
        None
    }

    fn slug(&self) -> String {
        slugify(&self.title)
    }
}

impl OpenLibraryManager {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl MetadataFetcher<OpenLibraryItem> for OpenLibraryManager {
    const BASE_URL: &str = "https://openlibrary.org/search.json";

    async fn fetch(&self, title: &str) -> color_eyre::Result<Vec<OpenLibraryItem>> {
        let res = self
            .client
            .get(Self::BASE_URL)
            .query(&[("title", title), ("limit", "1")])
            .send()
            .await
            .context("Open Library API call")?;

        let parsed: OpenLibraryResponse = res.json().await.context("Open Library JSON parsing")?;
        let items = parsed.docs;

        Ok(items)
    }
}
