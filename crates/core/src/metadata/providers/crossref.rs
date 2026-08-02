use std::{borrow::Cow, sync::Arc};

use async_trait::async_trait;
use color_eyre::eyre::{Context, bail};
use reqwest::Client;

use crate::metadata::{
    common_metadata::{AuthorInput, ItemMetadata, ItemType},
    proxy::MetadataFetcher,
};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct CrossrefWorkResponse {
    pub message: CrossrefItem,
}

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
    #[serde(rename = "abstract", default)]
    abstract_text: Option<String>,
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
    fn title(&self) -> Cow<'_, str> {
        match self.title.first().map(std::string::String::as_str) {
            Some(title) => Cow::Borrowed(title),
            None => Cow::default(),
        }
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

    fn isbn(&self) -> Option<Cow<'_, str>> {
        None
    }

    fn doi(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(&self.doi))
    }

    fn publication_date(&self) -> Option<Cow<'_, str>> {
        self.issued.as_ref().and_then(|issued| {
            issued.date_parts.first().map(|parts| {
                let t = parts
                    .iter()
                    .filter_map(|p| p.map(|n| n.to_string()))
                    .collect::<Vec<_>>()
                    .join("-");
                Cow::Owned(t)
            })
        })
    }

    fn cover_image_url(&self) -> Option<Cow<'_, str>> {
        None
    }

    fn source(&self) -> Cow<'_, str> {
        Cow::Borrowed("Crossref")
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        self.abstract_text
            .as_deref()
            .map(strip_jats_tags)
            .filter(|s| !s.is_empty())
            .map(Cow::Owned)
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

    fn container(&self) -> Option<Cow<'_, str>> {
        self.container_title
            .first()
            .map(|c| Cow::Borrowed(c.as_str()))
    }
}

#[derive(Clone, Debug)]
pub struct CrossrefManager {}

impl CrossrefManager {
    /// Construct a [`CrossrefManager`].
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for CrossrefManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MetadataFetcher for CrossrefManager {
    async fn fetch(
        &self,
        client: Arc<Client>,
        title: String,
    ) -> color_eyre::Result<Vec<Box<dyn ItemMetadata>>> {
        let base_url: &str = "https://api.crossref.org/works/";
        let res = client
            .get(base_url)
            .query(&[("query.title", title.as_str()), ("rows", "5")])
            .send()
            .await
            .context("Crosser API call");
        match res {
            Ok(res) => {
                let parsed: CrossrefResponse = res.json().await.context("Crossrer JSON parsing")?;
                let items = parsed
                    .message
                    .items
                    .into_iter()
                    .filter(|i| !i.doi.is_empty() && !i.title.is_empty())
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

    async fn fetch_abstract(
        &self,
        client: Arc<Client>,
        _title: String,
        doi: Option<String>,
        _isbn: Option<String>,
    ) -> color_eyre::Result<Option<String>> {
        let Some(doi) = doi else { return Ok(None) };
        let url = format!("https://api.crossref.org/works/{doi}");
        let res = client
            .get(&url)
            .send()
            .await
            .context("Crossref DOI lookup")?;
        if res.status().is_success() {
            let parsed: CrossrefWorkResponse =
                res.json().await.context("Crossref DOI json parsing")?;
            Ok(parsed.message.description().map(|d| d.to_string()))
        } else {
            Ok(None)
        }
    }
}

fn strip_jats_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for c in input.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}
