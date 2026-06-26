use core::fmt;

use anyhow::anyhow;

/// Common interface for metadata from various providers
pub trait ItemMetadata {
    fn title(&self) -> String;
    fn description(&self) -> Option<String>;
    fn item_type(&self) -> ItemType;
    fn authors(&self) -> Vec<String>;
    fn isbn(&self) -> Option<String>;
    fn doi(&self) -> Option<String>;
    fn publication_date(&self) -> Option<String>;
    fn cover_image_url(&self) -> Option<String>;
    fn slug(&self) -> String;
    fn source(&self) -> String;
    fn source_id(&self) -> Option<String>;
}

#[derive(Debug, Clone)]
pub enum ItemType {
    Book,
    Article,
    Report,
    Thesis,
    Misc,
}

impl TryFrom<&str> for ItemType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "book" => ItemType::Book,
            "article" => ItemType::Article,
            "report" => ItemType::Report,
            "thesis" => ItemType::Thesis,
            "misc" => ItemType::Misc,
            _ => Err(anyhow!("Cannot convert {value} to ItemType"))?,
        })
    }
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
