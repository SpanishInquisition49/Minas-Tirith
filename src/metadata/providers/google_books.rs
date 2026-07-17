use std::{borrow::Cow, sync::Arc};

use async_trait::async_trait;
use color_eyre::eyre::Context;
use reqwest::Client;
use serde::Deserialize;

use crate::metadata::{
    common_metadata::{ItemMetadata, ItemType},
    proxy::MetadataFetcher,
};

#[derive(Debug, Deserialize, Default)]
struct GoogleBooksResponse {
    #[serde(default)]
    items: Vec<GoogleBooksItem>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleBooksItem {
    #[serde(rename = "volumeInfo")]
    volume_info: VolumeInfo,
}

#[derive(Debug, Deserialize, Default)]
struct VolumeInfo {
    title: Option<String>,
    #[serde(default)]
    authors: Vec<String>,
    #[serde(rename = "publishedDate")]
    published_date: Option<String>,
    description: Option<String>,
    #[serde(rename = "industryIdentifiers", default)]
    industry_identifiers: Vec<IndustryIdentifier>,
    #[serde(rename = "imageLinks")]
    image_links: Option<ImageLinks>,
    publisher: Option<String>,
    #[serde(default)]
    categories: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct IndustryIdentifier {
    #[serde(rename = "type")]
    id_type: String,
    identifier: String,
}

#[derive(Debug, Deserialize, Default)]
struct ImageLinks {
    thumbnail: Option<String>,
}

impl ItemMetadata for GoogleBooksItem {
    fn title(&self) -> Cow<'_, str> {
        self.volume_info
            .title
            .as_deref()
            .map_or_else(Cow::default, Cow::Borrowed)
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        self.volume_info.description.as_deref().map(Cow::Borrowed)
    }

    fn item_type(&self) -> ItemType {
        ItemType::Book
    }

    fn authors(&self) -> Vec<String> {
        self.volume_info.authors.clone()
    }

    fn isbn(&self) -> Option<Cow<'_, str>> {
        self.volume_info
            .industry_identifiers
            .iter()
            .find(|i| i.id_type == "ISBN_13")
            .or_else(|| {
                self.volume_info
                    .industry_identifiers
                    .iter()
                    .find(|i| i.id_type == "ISBN_10")
            })
            .map(|i| Cow::Borrowed(i.identifier.as_str()))
    }

    fn doi(&self) -> Option<Cow<'_, str>> {
        None
    }

    fn publication_date(&self) -> Option<Cow<'_, str>> {
        self.volume_info
            .published_date
            .as_deref()
            .map(Cow::Borrowed)
    }

    fn cover_image_url(&self) -> Option<Cow<'_, str>> {
        self.volume_info
            .image_links
            .as_ref()
            .and_then(|l| l.thumbnail.as_deref())
            .map(|u| Cow::Owned(u.replacen("http://", "https://", 1)))
    }

    fn source(&self) -> Cow<'_, str> {
        Cow::Borrowed("google_books")
    }

    fn tags(&self) -> Vec<String> {
        self.volume_info.categories.clone()
    }

    fn container(&self) -> Option<Cow<'_, str>> {
        self.volume_info.publisher.as_deref().map(Cow::Borrowed)
    }
}

#[derive(Clone, Debug)]
pub struct GoogleBooksManager {
    key: String,
}

impl GoogleBooksManager {
    pub fn new(key: String) -> Self {
        Self { key }
    }
}

#[async_trait]
impl MetadataFetcher for GoogleBooksManager {
    async fn fetch(
        &self,
        client: Arc<Client>,
        title: String,
    ) -> color_eyre::Result<Vec<Box<dyn ItemMetadata>>> {
        const BASE_URL: &str = "https://www.googleapis.com/books/v1/volumes";
        let query = format!("intitle:{title}");
        let mut retries: usize = 0;
        while retries < 10 {
            let res = client
                .get(BASE_URL)
                .query(&[
                    ("key", self.key.as_str()),
                    ("q", query.as_str()),
                    ("maxResults", "5"),
                ])
                .send()
                .await
                .context("Google Books API call");

            match res {
                Ok(res) => {
                    if !res.status().is_success() {
                        retries += 1;
                        continue;
                    }
                    let parsed: GoogleBooksResponse =
                        res.json().await.context("Google Books JSON parsing")?;
                    let items = parsed
                        .items
                        .into_iter()
                        .filter(|i| i.volume_info.title.is_some())
                        .map(|i| Box::new(i) as Box<dyn ItemMetadata>)
                        .collect();
                    return Ok(items);
                }
                Err(e) => {
                    tracing::error!(errore = %e, provider = self.name(), work_title = title, "Failed to fetch metadata");
                    retries += 1;
                }
            }
        }

        Ok(vec![])
    }
}
