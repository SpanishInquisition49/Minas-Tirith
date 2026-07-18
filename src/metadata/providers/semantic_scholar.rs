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
struct SemanticScholarSearchResponse {
    #[serde(default)]
    data: Vec<SemanticScholarItem>,
}

#[derive(Debug, Deserialize)]
pub struct SemanticScholarItem {
    title: Option<String>,
    r#abstract: Option<String>,
    year: Option<i32>,
    #[serde(rename = "externalIds", default)]
    external_ids: ExternalIds,
    #[serde(default)]
    authors: Vec<SemanticScholarAuthor>,
    venue: Option<String>,
    #[serde(rename = "publicationTypes", default)]
    publication_types: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct ExternalIds {
    #[serde(rename = "DOI")]
    doi: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SemanticScholarAuthor {
    name: Option<String>,
}

impl ItemMetadata for SemanticScholarItem {
    fn title(&self) -> Cow<'_, str> {
        self.title
            .as_deref()
            .map_or_else(Cow::default, Cow::Borrowed)
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        self.r#abstract.as_deref().map(Cow::Borrowed)
    }

    fn item_type(&self) -> ItemType {
        if self
            .publication_types
            .iter()
            .any(|t| t == "Book" || t == "BookSection")
        {
            ItemType::Book
        } else if self.publication_types.is_empty() {
            ItemType::Misc
        } else {
            ItemType::Article
        }
    }

    fn authors(&self) -> Vec<String> {
        self.authors.iter().filter_map(|a| a.name.clone()).collect()
    }

    fn isbn(&self) -> Option<Cow<'_, str>> {
        None
    }

    fn doi(&self) -> Option<Cow<'_, str>> {
        self.external_ids.doi.as_deref().map(Cow::Borrowed)
    }

    fn publication_date(&self) -> Option<Cow<'_, str>> {
        self.year.map(|y| Cow::Owned(y.to_string()))
    }

    fn cover_image_url(&self) -> Option<Cow<'_, str>> {
        None
    }

    fn source(&self) -> Cow<'_, str> {
        Cow::Borrowed("Semantic Scholar")
    }

    fn tags(&self) -> Vec<String> {
        vec![]
    }

    fn container(&self) -> Option<Cow<'_, str>> {
        self.venue
            .as_deref()
            .filter(|v| !v.is_empty())
            .map(Cow::Borrowed)
    }
}

#[derive(Debug, Deserialize)]
struct SemanticScholarPaperResponse {
    r#abstract: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SemanticScholarManager {}

impl SemanticScholarManager {
    const BASE_URL: &str = "https://api.semanticscholar.org/graph/v1/paper";

    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl MetadataFetcher for SemanticScholarManager {
    async fn fetch(
        &self,
        client: Arc<Client>,
        title: String,
    ) -> color_eyre::Result<Vec<Box<dyn ItemMetadata>>> {
        let url = format!("{}/search", Self::BASE_URL);
        let mut retries: usize = 0;
        while retries < 10 {
            let res = client
                .get(&url)
                .query(&[
                    ("query", title.to_string().as_str()),
                    (
                        "fields",
                        "title,abstract,year,externalIds,authors,venue,publicationTypes",
                    ),
                    ("limit", "5"),
                ])
                .send()
                .await
                .context("Semantic Scholar API call");

            match res {
                Ok(res) => {
                    if !res.status().is_success() {
                        retries += 1;
                        continue;
                    }
                    let parsed: SemanticScholarSearchResponse =
                        res.json().await.context("Semantic Scholar JSON parsing")?;
                    let items = parsed
                        .data
                        .into_iter()
                        .filter(|i| i.title.is_some())
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

    async fn fetch_abstract(
        &self,
        client: Arc<Client>,
        _title: String,
        doi: Option<String>,
        _isbn: Option<String>,
    ) -> color_eyre::Result<Option<String>> {
        let Some(doi) = doi else { return Ok(None) };
        let url = format!("{}/DOI:{doi}", Self::BASE_URL);
        let res = client
            .get(&url)
            .query(&[("fields", "abstract")])
            .send()
            .await
            .context("Semantic Scholar DOI lookup")?;

        if !res.status().is_success() {
            Ok(None)
        } else {
            let parsed: SemanticScholarPaperResponse = res
                .json()
                .await
                .context("Semantic Scholar DOI json parsing")?;
            Ok(parsed.r#abstract)
        }
    }
}
