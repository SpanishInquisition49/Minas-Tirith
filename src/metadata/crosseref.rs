use color_eyre::eyre::Context;
use reqwest::Client;
use slug::slugify;

use crate::metadata::{
    common_metadata::{ItemMetadata, ItemType},
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

#[derive(Debug, Deserialize)]
pub struct CrossrefItem {
    #[serde(rename = "DOI")]
    doi: String,
    title: Vec<String>,
    author: Option<Vec<CrossrefAuthor>>,
    #[serde(rename = "type")]
    work_type: String,
    issued: Option<CrossrefDate>,
}

#[derive(Debug, Deserialize)]
struct CrossrefAuthor {
    given: Option<String>,
    family: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CrossrefDate {
    #[serde(rename = "date-parts")]
    date_parts: Vec<Vec<i32>>,
}

impl ItemMetadata for CrossrefItem {
    fn title(&self) -> String {
        self.title.clone().into_iter().next().unwrap_or_default()
    }

    fn item_type(&self) -> super::common_metadata::ItemType {
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
        if let Some(issued) = &self.issued {
            issued.date_parts.clone().into_iter().next().map(|parts| {
                parts
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join("-")
            })
        } else {
            None
        }
    }

    fn cover_image_url(&self) -> Option<String> {
        None
    }

    fn source(&self) -> String {
        "crossref".to_string()
    }

    fn source_id(&self) -> Option<String> {
        Some(self.doi.clone())
    }

    fn description(&self) -> Option<String> {
        None
    }

    fn slug(&self) -> String {
        slugify(self.title())
    }
}

pub struct CrossrefManager {
    client: Client,
}

impl CrossrefManager {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl MetadataFetcher<CrossrefItem> for CrossrefManager {
    const BASE_URL: &str = "https://api.crossref.org/works/";

    async fn fetch(&self, title: &str) -> color_eyre::Result<Vec<CrossrefItem>> {
        let res = self
            .client
            .get(Self::BASE_URL)
            .query(&[("query.title", title), ("rows", "1")])
            .send()
            .await
            .context("Crosser API call")?;
        let parsed: CrossrefResponse = res.json().await.context("Crossrer JSON parsing")?;
        let items = parsed.message.items;
        Ok(items)
    }
}
