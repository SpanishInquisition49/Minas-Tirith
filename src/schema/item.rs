use core::fmt;

use sqlx::types::chrono::{DateTime, Utc};

use crate::metadata::common_metadata::ItemMetadata;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DatabaseItem {
    pub id: i32,
    #[sqlx(flatten)]
    pub fields: Item,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl fmt::Display for DatabaseItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        str.push_str(&format!("Id: {}\n", self.id));
        str.push_str(&format!("{}", self.fields));
        str.push_str(&format!("Created At: {}\n", self.created_at));
        str.push_str(&format!("Updated At: {}\n", self.updated_at));
        write!(f, "{str}")
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Item {
    pub title: String,
    pub description: Option<String>,
    pub r#type: String,
    pub doi: Option<String>,
    pub isbn: Option<String>,
    pub publication_date: Option<String>,
    pub slug: String,
    pub cover_image_url: Option<String>,
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        str.push_str(&format!("Title: {}\n", self.title));
        if let Some(desc) = &self.description {
            str.push_str(&format!("Description:\n{desc}"));
        }
        str.push_str(&format!("Type: {}\n", self.r#type));
        if let Some(doi) = &self.doi {
            str.push_str(&format!("DOI: {doi}\n"));
        }
        if let Some(isbn) = &self.isbn {
            str.push_str(&format!("ISBN: {isbn}\n"));
        }
        if let Some(date) = &self.publication_date {
            str.push_str(&format!("Publication Date: {date}\n"));
        }
        str.push_str(&format!("Slug: {}\n", self.slug));
        if let Some(url) = &self.cover_image_url {
            str.push_str(&format!("Cover URL: {url}\n"));
        }

        write!(f, "{str}")
    }
}

impl From<&ItemMetadata> for Item {
    fn from(value: &ItemMetadata) -> Self {
        let slug = value
            .title
            .replace(",", "")
            .replace(" ", "-")
            .to_lowercase();
        Self {
            title: value.title.clone(),
            description: None,
            r#type: value.item_type.to_string(),
            doi: value.doi.clone(),
            isbn: value.isbn.clone(),
            publication_date: value.publication_date.clone(),
            slug,
            cover_image_url: value.cover_image_url.clone(),
        }
    }
}
