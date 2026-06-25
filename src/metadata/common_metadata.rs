use core::fmt;

use serde::{Deserialize, Serialize};

/// Common interface for metadata from various providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemMetadata {
    pub title: String,
    pub item_type: ItemType,
    pub authors: Vec<String>,
    pub isbn: Option<String>,
    pub doi: Option<String>,
    pub publication_date: Option<String>,
    pub cover_image_url: Option<String>,
    pub source: String,
    pub source_id: Option<String>,
}

impl fmt::Display for ItemMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        str.push_str(&format!("Title: {}\n", self.title));
        str.push_str(&format!("Type: {}\n", self.item_type));

        str.push_str("Authors:\n");
        for a in &self.authors {
            str.push_str(&format!("\t- {a}\n"));
        }
        if let Some(isbn) = &self.isbn {
            str.push_str(&format!("ISBN: {}\n", isbn));
        }
        if let Some(doi) = &self.doi {
            str.push_str(&format!("DOI: {}\n", doi));
        }
        if let Some(d) = &self.publication_date {
            str.push_str(&format!("Publiciation Date: {}\n", d));
        }

        if let Some(url) = &self.cover_image_url {
            str.push_str(&format!("Cover URL: {}\n", url));
        }
        str.push_str(&format!("Source: {}\n", self.source));
        if let Some(sid) = &self.source_id {
            str.push_str(&format!("Source ID: {}\n", sid));
        }
        write!(f, "{str}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemType {
    Book,
    Article,
    Report,
    Thesis,
    Misc,
}

impl fmt::Display for ItemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ItemType::Book => write!(f, "book"),
            ItemType::Article => write!(f, "article"),
            ItemType::Report => write!(f, "report"),
            ItemType::Thesis => write!(f, "thesis"),
            ItemType::Misc => write!(f, "misc"),
        }
    }
}
