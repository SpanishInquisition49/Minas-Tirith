use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use color_eyre::eyre::Context;
use reqwest::Client;
use serde::Deserialize;

use crate::metadata::{
    common_metadata::{ItemMetadata, ItemType},
    proxy::MetadataFetcher,
};

pub struct OpenAlexManager {}

impl OpenAlexManager {
    const BASE_URL: &str = "https://api.openalex.org/works";
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl MetadataFetcher for OpenAlexManager {
    async fn fetch(
        &self,
        client: Arc<Client>,
        title: String,
    ) -> color_eyre::Result<Vec<Box<dyn ItemMetadata>>> {
        let query = vec![("search", title), ("per-page", "5".to_string())];
        let res = client
            .get(Self::BASE_URL)
            .query(&query)
            .send()
            .await
            .context("OpenAlex API call")?;

        let parsed: OpenAlexResponse = res.json().await.context("OpenAlex JSON parsing")?;
        let items = parsed
            .results
            .into_iter()
            .map(|i| Box::new(i) as Box<dyn ItemMetadata>)
            .collect();
        Ok(items)
    }

    async fn fetch_abstract(
        &self,
        client: Arc<Client>,
        _title: String,
        doi: Option<String>,
        _isbn: Option<String>,
    ) -> color_eyre::Result<Option<String>> {
        let Some(doi) = doi else { return Ok(None) };
        let url = format!("{}/doi:{doi}", Self::BASE_URL);
        if let Ok(res) = client.get(&url).send().await
            && res.status().is_success()
            && let Ok(item) = res.json::<OpenAlexItem>().await
            && let Some(r#abstract) = item.reconstruct_abstract()
        {
            return Ok(Some(r#abstract));
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Deserialize)]
struct OpenAlexResponse {
    #[serde(default)]
    results: Vec<OpenAlexItem>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAlexItem {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    doi: Option<String>,
    #[serde(default)]
    publication_year: Option<i32>,
    #[serde(rename = "type", default)]
    work_type: String,
    #[serde(default)]
    primary_location: Option<OpenAlexLocation>,
    #[serde(default)]
    authorships: Vec<OpenAlexAuthorship>,
    #[serde(rename = "abstract_inverted_index", default)]
    abstract_inverted_index: Option<HashMap<String, Vec<u32>>>,
}

#[derive(Debug, Deserialize, Default)]
struct OpenAlexLocation {
    source: Option<OpenAlexSource>,
}

#[derive(Debug, Deserialize, Default)]
struct OpenAlexSource {
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAlexAuthorship {
    author: Option<OpenAlexAuthor>,
    #[serde(default)]
    institutions: Vec<OpenAlexInstitution>,
}

#[derive(Debug, Deserialize)]
struct OpenAlexAuthor {
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAlexInstitution {
    display_name: Option<String>,
}

impl OpenAlexItem {
    fn reconstruct_abstract(&self) -> Option<String> {
        let index = self.abstract_inverted_index.as_ref()?;
        let mut words: Vec<(u32, &str)> = index
            .iter()
            .flat_map(|(word, positions)| positions.iter().map(move |&pos| (pos, word.as_str())))
            .collect();
        words.sort_by_key(|(pos, _)| *pos);
        if words.is_empty() {
            None
        } else {
            Some(
                words
                    .into_iter()
                    .map(|(_, w)| w)
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        }
    }

    fn institution(&self) -> Option<String> {
        self.authorships
            .iter()
            .find_map(|a| a.institutions.first())
            .and_then(|i| i.display_name.clone())
    }
}

impl ItemMetadata for OpenAlexItem {
    fn title(&self) -> String {
        self.title.clone().unwrap_or_default()
    }

    fn description(&self) -> Option<String> {
        self.reconstruct_abstract()
    }

    fn item_type(&self) -> crate::metadata::common_metadata::ItemType {
        match self.work_type.as_str() {
            "article" | "preprint" | "paratext" => ItemType::Article,
            "book" | "book-chapter" | "monograph" | "edited-book" => ItemType::Book,
            "report" | "standard" => ItemType::Report,
            "dissertation" => ItemType::Thesis,
            _ => ItemType::Misc,
        }
    }

    fn authors(&self) -> Vec<String> {
        self.authorships
            .iter()
            .filter_map(|a| a.author.as_ref())
            .filter_map(|a| a.display_name.clone())
            .collect()
    }

    fn isbn(&self) -> Option<String> {
        None
    }

    fn doi(&self) -> Option<String> {
        self.doi
            .as_ref()
            .map(|d| d.trim_start_matches("https://doi.org/").to_string())
    }

    fn publication_date(&self) -> Option<String> {
        self.publication_year.map(|y| y.to_string())
    }

    fn cover_image_url(&self) -> Option<String> {
        None
    }

    fn source(&self) -> String {
        "openalex".to_string()
    }

    fn tags(&self) -> Vec<String> {
        vec![]
    }

    fn container(&self) -> Option<String> {
        self.primary_location
            .as_ref()
            .and_then(|l| l.source.as_ref())
            .and_then(|s| s.display_name.clone())
            .or_else(|| self.institution())
    }
}
