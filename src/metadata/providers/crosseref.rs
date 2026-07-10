use color_eyre::eyre::Context;
use reqwest::Client;

use crate::metadata::{
    common_metadata::{AuthorInput, ItemMetadata, ItemType},
    proxy::MetadataFetcher,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CrossrefResponse {
    message: CrossrefMessage,
}

#[derive(Debug, Deserialize)]
struct CrossrefMessage {
    items: Vec<CrossrefItem>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CrossrefItem {
    #[serde(rename = "DOI", default)]
    doi: String,
    #[serde(default)]
    title: Vec<String>,
    author: Option<Vec<CrossrefAuthor>>,
    #[serde(rename = "type", default)]
    work_type: String,
    issued: Option<CrossrefDate>,
    #[serde(rename = "container-title", default)]
    container_title: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CrossrefAuthor {
    given: Option<String>,
    family: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct CrossrefDate {
    #[serde(rename = "date-parts", default)]
    date_parts: Vec<Vec<Option<i32>>>,
}
impl ItemMetadata for CrossrefItem {
    fn title(&self) -> String {
        self.title.clone().into_iter().next().unwrap_or_default()
    }

    fn item_type(&self) -> ItemType {
        match self.work_type.as_str() {
            "journal-article" | "proceedings-article" | "conference-paper" => ItemType::Article,
            "book" | "monograph" | "edited-book" => ItemType::Book,
            "report" | "report-series" => ItemType::Report,
            "dissertation" => ItemType::Thesis,
            _ => ItemType::Misc,
        }
    }

    fn authors(&self) -> Vec<String> {
        if let Some(authors) = &self.author {
            authors
                .iter()
                .map(|a| {
                    let given = a.given.clone().unwrap_or_default();
                    let family = a.family.clone().unwrap_or_default();
                    format!("{given} {family}").trim().to_string()
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn isbn(&self) -> Option<String> {
        None
    }

    fn doi(&self) -> Option<String> {
        Some(self.doi.clone())
    }

    fn publication_date(&self) -> Option<String> {
        self.issued.as_ref().and_then(|issued| {
            issued.date_parts.first().map(|parts| {
                parts
                    .iter()
                    .filter_map(|p| p.map(|n| n.to_string()))
                    .collect::<Vec<_>>()
                    .join("-")
            })
        })
    }

    fn cover_image_url(&self) -> Option<String> {
        None
    }

    fn source(&self) -> String {
        "crossref".to_string()
    }

    fn description(&self) -> Option<String> {
        None
    }

    fn tags(&self) -> Vec<String> {
        vec![]
    }

    fn authors_structured(&self) -> Vec<AuthorInput> {
        let Some(authors) = &self.author else {
            return vec![];
        };
        authors
            .iter()
            .map(|a| {
                let given = a.given.clone().unwrap_or_default();
                let family = a.family.clone().unwrap_or_default();
                let full_name = format!("{given} {family}").trim().to_string();
                AuthorInput {
                    given_name: Some(given).filter(|g| !g.is_empty()),
                    family_name: Some(family).filter(|f| !f.is_empty()),
                    full_name,
                }
            })
            .collect()
    }

    fn container(&self) -> Option<String> {
        self.container_title.first().cloned()
    }
}

#[derive(Clone, Debug)]
pub struct CrossrefManager {}

impl CrossrefManager {
    pub fn new() -> Self {
        Self {}
    }
}

impl MetadataFetcher for CrossrefManager {
    async fn fetch(&self, client: &Client, title: &str) -> color_eyre::Result<Vec<CrossrefItem>> {
        let base_url: &str = "https://api.crossref.org/works/";
        let res = client
            .get(base_url)
            .query(&[("query.title", title), ("rows", "5")])
            .send()
            .await
            .context("Crosser API call")?;
        let parsed: CrossrefResponse = res.json().await.context("Crossrer JSON parsing")?;
        let items = parsed
            .message
            .items
            .into_iter()
            .filter(|i| !i.doi.is_empty() && !i.title.is_empty())
            .collect();
        Ok(items)
    }

    type Item = CrossrefItem;
}
