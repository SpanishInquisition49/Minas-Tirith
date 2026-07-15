use core::fmt;

use human_name::Name;
use serde::Deserialize;
use slug::slugify;
use sqlx::types::{
    Json,
    chrono::{DateTime, Utc},
};

use crate::{
    metadata::common_metadata::{ItemMetadata, ItemType},
    schema::{author::Author, collection::Collection, tag::Tag},
};

#[derive(Clone, sqlx::FromRow, Deserialize, Debug)]
pub struct RawItemRow {
    pub id: i32,
    pub path: String,
    #[sqlx(flatten)]
    pub fields: Item,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub authors: Option<sqlx::types::Json<Vec<Author>>>,
    pub tags: Option<sqlx::types::Json<Vec<Tag>>>,
    pub collections: Option<sqlx::types::Json<Vec<Collection>>>,
}

#[derive(Debug, Clone)]
pub struct DatabaseItem {
    pub id: i32,
    pub path: String,
    pub fields: Item,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub authors: Vec<Author>,
    pub tags: Vec<Tag>,
    pub collections: Vec<Collection>,
}

impl From<RawItemRow> for DatabaseItem {
    fn from(value: RawItemRow) -> Self {
        Self {
            id: value.id,
            path: value.path,
            fields: value.fields,
            created_at: value.created_at,
            updated_at: value.updated_at,
            authors: value.authors.map(|Json(v)| v).unwrap_or_default(),
            tags: value.tags.map(|Json(v)| v).unwrap_or_default(),
            collections: value.collections.map(|Json(v)| v).unwrap_or_default(),
        }
    }
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

impl DatabaseItem {
    pub fn to_bibtex(&self) -> String {
        let item_type =
            ItemType::try_from(self.fields.r#type.as_str()).unwrap_or(ItemType::default());
        let key = self.cite_key();
        let year = self.year();
        let authors = self.autors_bibtex();
        let title = Self::escape_bibtex(&self.fields.title);
        let description =
            Self::escape_bibtex(self.fields.description.as_deref().unwrap_or_default());
        let mut fields: Vec<(&str, Option<&str>)> = vec![
            ("title", Some(&title)),
            ("authors", authors.as_deref()),
            ("year", year.as_deref()),
            ("doi", self.fields.doi.as_deref()),
        ];

        let entry_type = match item_type {
            ItemType::Book => {
                fields.push(("publisher", self.fields.container.as_deref()));
                fields.push(("isbn", self.fields.isbn.as_deref()));
                "book"
            }
            ItemType::Article => {
                fields.push(("journal", self.fields.container.as_deref()));
                "article"
            }
            ItemType::Report => {
                fields.push(("institution", self.fields.container.as_deref()));
                "techreport"
            }
            ItemType::Thesis => {
                fields.push(("school", self.fields.container.as_deref()));
                "phdthesis"
            }
            ItemType::Misc => {
                fields.push(("howpublished", self.fields.container.as_deref()));
                "misc"
            }
        };

        if item_type == ItemType::Misc && !description.is_empty() {
            if !description.is_empty() {
                fields.push(("abstract", Some(&description)));
            }
        } else if !description.is_empty() {
            fields.push(("note", Some(&description)));
        }

        let mut bibtex = format!("@{entry_type}{{{key},\n");
        for (name, value) in fields {
            if let Some(v) = value {
                bibtex.push_str(&format!("\t{name} = {{{v}}},\n"));
            }
        }
        bibtex.push_str("}\n");
        bibtex
    }

    fn escape_bibtex(value: &str) -> String {
        let mut out = String::with_capacity(value.len());
        for c in value.chars() {
            match c {
                '$' | '&' | '%' | '#' | '_' | '{' | '}' => {
                    out.push('\\');
                    out.push(c);
                }
                '~' => out.push_str("\\textasciitilde{}"),
                '^' => out.push_str("\\textasciicircum{}"),
                '\\' => out.push_str("\\textasciibackslash{}"),
                _ => out.push(c),
            }
        }
        out
    }

    fn autors_bibtex(&self) -> Option<String> {
        if self.authors.is_empty() {
            None
        } else {
            Some(
                self.authors
                    .iter()
                    .map(|a| a.name.clone())
                    .collect::<Vec<_>>()
                    .join(" and "),
            )
        }
    }

    fn year(&self) -> Option<String> {
        self.fields
            .publication_date
            .as_ref()
            .and_then(|d| d.split(['-', '/']).next())
            .filter(|s| s.chars().all(|c| c.is_ascii_digit()) && s.len() == 4)
            .map(str::to_string)
    }

    fn cite_key(&self) -> String {
        let author_last_name = self
            .authors
            .first()
            .map(|a| {
                a.family_name.clone().unwrap_or_else(|| {
                    let Some(parsed) = Name::parse(&a.name) else {
                        // NOTE: if everything fails use the full name of the author
                        return a.name.clone();
                    };
                    parsed.surnames().join(" ")
                })
            })
            .map(slugify)
            .filter(|s| !s.is_empty());

        match (author_last_name, self.year()) {
            (Some(a), Some(y)) => format!("{a}{y}"),
            (Some(a), None) => a,
            (None, _) => slugify(&self.fields.title),
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow, Deserialize)]
pub struct Item {
    pub title: String,
    pub description: Option<String>,
    pub r#type: String,
    pub doi: Option<String>,
    pub isbn: Option<String>,
    pub publication_date: Option<String>,
    pub slug: String,
    pub cover_image_url: Option<String>,
    pub container: Option<String>,
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

impl<T: ItemMetadata + Sized> From<&T> for Item {
    fn from(value: &T) -> Self {
        Self {
            title: value.title().to_string(),
            description: value.description().map(|d| d.to_string()),
            r#type: value.item_type().to_string(),
            doi: value.doi().map(|d| d.to_string()),
            isbn: value.isbn().map(|i| i.to_string()),
            publication_date: value.publication_date().map(|d| d.to_string()),
            slug: value.slug(),
            cover_image_url: value.cover_image_url().map(|u| u.to_string()),
            container: value.container().map(|c| c.to_string()),
        }
    }
}
