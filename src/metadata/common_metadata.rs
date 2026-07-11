use core::fmt;

use color_eyre::eyre::eyre;
use human_name::Name;
use slug::slugify;

/// Common interface for metadata from various providers
pub trait ItemMetadata: Send {
    fn title(&self) -> String;
    fn description(&self) -> Option<String>;
    fn item_type(&self) -> ItemType;
    fn authors(&self) -> Vec<String>;
    fn isbn(&self) -> Option<String>;
    fn doi(&self) -> Option<String>;
    fn publication_date(&self) -> Option<String>;
    fn cover_image_url(&self) -> Option<String>;
    fn source(&self) -> String;
    fn tags(&self) -> Vec<String>;
    fn container(&self) -> Option<String>;

    fn slug(&self) -> String {
        slugify(self.title())
    }

    fn authors_structured(&self) -> Vec<AuthorInput> {
        self.authors()
            .into_iter()
            .map(|full_name| {
                let parsed = Name::parse(&full_name);
                AuthorInput {
                    given_name: parsed
                        .as_ref()
                        .and_then(|n| n.given_name())
                        .map(&str::to_string),
                    family_name: parsed
                        .as_ref()
                        .map(|n| n.surnames().join(" "))
                        .filter(|s| !s.is_empty()),
                    full_name,
                }
            })
            .collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum ItemType {
    Book,
    Article,
    Report,
    Thesis,
    #[default]
    Misc,
}

impl TryFrom<&str> for ItemType {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value.to_lowercase().as_str() {
            "book" => ItemType::Book,
            "article" => ItemType::Article,
            "report" => ItemType::Report,
            "thesis" => ItemType::Thesis,
            "misc" => ItemType::Misc,
            _ => Err(eyre!("Cannot convert {value} to ItemType"))?,
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

#[derive(Clone, Debug)]
pub struct AuthorInput {
    pub full_name: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
}
