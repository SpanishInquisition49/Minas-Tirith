use std::{borrow::Cow, str::FromStr};

use color_eyre::eyre::bail;

use crate::metadata::common_metadata::{ItemMetadata, ItemType};

pub const FIELD_LABELS: [&str; 8] = [
    "Title",
    "Description",
    "Container",
    "DOI",
    "ISBN",
    "Publication Date",
    "Tags",
    "Cover URL",
];

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Field {
    #[default]
    Title,
    Description,
    Container,
    Doi,
    Isbn,
    PublicationDate,
    Tags,
    CoverUrl,
}

impl FromStr for Field {
    type Err = color_eyre::eyre::Error;
    fn from_str(s: &str) -> color_eyre::Result<Self> {
        Ok(match s.to_lowercase().as_str() {
            "title" => Field::Title,
            "description" => Field::Description,
            "container" => Field::Container,
            "doi" => Field::Doi,
            "isbn" => Field::Isbn,
            "publication date" => Field::PublicationDate,
            "tags" => Field::Tags,
            "cover url" => Field::CoverUrl,
            _ => bail!(format!("Could not convert {s} to Field")),
        })
    }
}

#[derive(Clone, Debug)]
pub struct FormSnapshot {
    pub title: String,
    pub description: String,
    pub item_type: ItemType,
    pub doi: String,
    pub isbn: String,
    pub publication_date: String,
    pub tags: Vec<String>,
    pub authors: Vec<String>,
    pub container: Option<String>,
    pub cover_image_url: Option<String>,
}

impl ItemMetadata for FormSnapshot {
    fn title(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.title)
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(&self.description))
    }

    fn item_type(&self) -> ItemType {
        self.item_type
    }

    fn authors(&self) -> Vec<String> {
        self.authors.clone()
    }

    fn isbn(&self) -> Option<Cow<'_, str>> {
        if self.isbn.is_empty() {
            None
        } else {
            Some(Cow::Borrowed(&self.isbn))
        }
    }

    fn doi(&self) -> Option<Cow<'_, str>> {
        if self.doi.is_empty() {
            None
        } else {
            Some(Cow::Borrowed(&self.doi))
        }
    }

    fn publication_date(&self) -> Option<Cow<'_, str>> {
        if self.publication_date.is_empty() {
            None
        } else {
            Some(Cow::Borrowed(&self.publication_date))
        }
    }

    fn cover_image_url(&self) -> Option<Cow<'_, str>> {
        self.cover_image_url.as_deref().map(Cow::Borrowed)
    }

    fn source(&self) -> Cow<'_, str> {
        Cow::default()
    }

    fn tags(&self) -> Vec<String> {
        self.tags
            .iter()
            .filter_map(|t| {
                if t.trim().is_empty() {
                    None
                } else {
                    Some(t.trim().to_string())
                }
            })
            .collect()
    }

    fn container(&self) -> Option<Cow<'_, str>> {
        self.container.as_deref().map(Cow::Borrowed)
    }
}
