use std::{borrow::Cow, sync::Arc};

use async_trait::async_trait;
use color_eyre::eyre::{Context, Result, bail};
use reqwest::Client;
use rustix::path::Arg;
use serde::Deserialize;

use crate::metadata::{
    common_metadata::{ItemMetadata, ItemType},
    proxy::MetadataFetcher,
};

#[derive(Clone, Debug)]
pub struct CoreManager {
    key: String,
}

impl CoreManager {
    pub fn new(key: String) -> Self {
        Self { key }
    }
}

#[async_trait]
impl MetadataFetcher for CoreManager {
    async fn fetch(
        &self,
        client: Arc<Client>,
        title: String,
    ) -> Result<Vec<Box<dyn ItemMetadata>>> {
        let query = format!("title:{title}");
        let res = client
            .get("https://api.core.ac.uk/v3/search/works")
            .query(&[("q", query.as_str()), ("limit", "5")])
            .header("Authorization", format!("Bearer {}", self.key))
            .send()
            .await
            .context("CORE API call");
        match res {
            Ok(res) => {
                let parsed: CoreResponse = res.json().await.context("CORE JSON parsing")?;
                let items = parsed
                    .results
                    .into_iter()
                    .filter(|i| i.doi.is_some() && !i.title.is_empty())
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

#[derive(Deserialize, Debug)]
struct CoreResponse {
    results: Vec<CoreItem>,
}

#[derive(Deserialize, Debug)]
struct CoreAuthor {
    pub name: String,
}

#[derive(Deserialize, Debug)]
struct CoreItem {
    title: String,
    #[serde(rename = "abstract")]
    abstract_text: String,
    #[serde(rename = "yearPublished")]
    year_published: String,
    doi: Option<String>,
    authors: Vec<CoreAuthor>,
    #[serde(rename = "fieldOfStudy")]
    field_of_study: String,
    publisher: String,
}

impl ItemMetadata for CoreItem {
    fn title(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.title)
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        if self.abstract_text.is_empty() {
            None
        } else {
            Some(Cow::Borrowed(&self.abstract_text))
        }
    }

    fn item_type(&self) -> ItemType {
        // TODO: figure out how to infer this information from the response
        ItemType::Misc
    }

    fn authors(&self) -> Vec<String> {
        self.authors.iter().map(|a| a.name.clone()).collect()
    }

    fn isbn(&self) -> Option<Cow<'_, str>> {
        None
    }

    fn doi(&self) -> Option<Cow<'_, str>> {
        self.doi.as_deref().map(Cow::Borrowed)
    }

    fn publication_date(&self) -> Option<Cow<'_, str>> {
        if !self.year_published.trim().is_empty() {
            Some(Cow::Borrowed(&self.year_published))
        } else {
            None
        }
    }

    fn cover_image_url(&self) -> Option<Cow<'_, str>> {
        None
    }

    fn source(&self) -> Cow<'_, str> {
        Cow::Borrowed("CORE")
    }

    fn tags(&self) -> Vec<String> {
        Vec::new()
    }

    fn container(&self) -> Option<Cow<'_, str>> {
        None
    }
}
