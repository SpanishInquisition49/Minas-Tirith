use std::{borrow::Cow, sync::Arc};

use async_trait::async_trait;
use color_eyre::eyre::{Context, bail};
use reqwest::Client;
use serde::Deserialize;

use crate::metadata::{
    common_metadata::{ItemMetadata, ItemType},
    proxy::MetadataFetcher,
};

#[derive(Clone, Debug)]
pub struct OpenLibraryManager {}

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
    ia: Option<Vec<String>>,
    #[serde(default)]
    publisher: Vec<String>,
}

impl ItemMetadata for OpenLibraryItem {
    fn title(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.title)
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

    fn isbn(&self) -> Option<Cow<'_, str>> {
        if let Some(ia) = &self.ia {
            ia.iter()
                .find_map(|s| s.strip_prefix("isbn_"))
                .map(Cow::Borrowed)
        } else {
            None
        }
    }

    fn doi(&self) -> Option<Cow<'_, str>> {
        None
    }

    fn publication_date(&self) -> Option<Cow<'_, str>> {
        self.first_publish_year.map(|y| Cow::Owned(y.to_string()))
    }

    fn cover_image_url(&self) -> Option<Cow<'_, str>> {
        self.cover_url
            .map(|id| Cow::Owned(format!("https://covers.openlibrary.org/b/id/{id}-L.jpg")))
    }

    fn source(&self) -> Cow<'_, str> {
        Cow::Borrowed("OpenLibrary")
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        None
    }

    fn tags(&self) -> Vec<String> {
        vec![]
    }

    fn container(&self) -> Option<Cow<'_, str>> {
        self.publisher.first().map(|p| Cow::Borrowed(p.as_str()))
    }
}

impl OpenLibraryManager {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for OpenLibraryManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MetadataFetcher for OpenLibraryManager {
    async fn fetch(
        &self,
        client: Arc<Client>,
        title: String,
    ) -> color_eyre::Result<Vec<Box<dyn ItemMetadata>>> {
        const BASE_URL: &str = "https://openlibrary.org/search.json";
        let res = client
            .get(BASE_URL)
            .query(&[("title", title.as_str()), ("limit", "5")])
            .send()
            .await
            .context("Open Library API call");

        match res {
            Ok(res) => {
                let parsed: OpenLibraryResponse =
                    res.json().await.context("Open Library JSON parsing")?;
                let items = parsed
                    .docs
                    .into_iter()
                    .map(|i| Box::new(i) as Box<dyn ItemMetadata>)
                    .collect();

                Ok(items)
            }
            Err(e) => {
                tracing::error!(errore = %e, provider = self.name(), work_title = title, "Failed to fetch metadata");
                bail!(e)
            }
        }
    }
}
