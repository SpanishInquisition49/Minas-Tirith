use core::fmt;

/// Common interface for metadata from various providers
pub trait ItemMetadata {
    fn title(&self) -> &str;
    fn item_type(&self) -> ItemType;
    fn authors(&self) -> Vec<String>;
    fn isbn(&self) -> Option<&str>;
    fn doi(&self) -> Option<&str>;
    fn publication_date(&self) -> Option<String>;
    fn cover_image_url(&self) -> Option<String>;
    fn source(&self) -> &str;
    fn source_id(&self) -> Option<&str>;
}

#[derive(Debug, Clone)]
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
