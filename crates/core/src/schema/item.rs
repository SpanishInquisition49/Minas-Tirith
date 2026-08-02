use core::fmt;
use std::borrow::Cow;
use std::fmt::Write;

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
        let _ = writeln!(str, "Id: {}", self.id);
        let _ = write!(str, "{}", self.fields);
        let _ = writeln!(str, "Created At: {}", self.created_at);
        let _ = writeln!(str, "Updated At: {}", self.updated_at);
        write!(f, "{str}")
    }
}

impl DatabaseItem {
    /// Render this item as a BibTeX entry, using `override_key` as the
    /// cite key if given, otherwise deriving one from the author and
    /// publication year.
    #[must_use]
    pub fn to_bibtex(&self, override_key: Option<String>) -> String {
        let item_type =
            ItemType::try_from(self.fields.r#type.as_str()).unwrap_or(ItemType::default());
        let key = match override_key {
            None => self.cite_key(),
            Some(key) => key,
        };
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
                fields.push(("note", Some(&description)));
            }
        } else if !description.is_empty() {
            fields.push(("abstract", Some(&description)));
        }

        let mut bibtex = format!("@{entry_type}{{{key},\n");
        for (name, value) in fields {
            if let Some(v) = value {
                let _ = writeln!(bibtex, "\t{name} = {{{v}}},");
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

    /// Derive a BibTeX cite key from the first author's surname and the
    /// publication year, falling back to the item's title slug.
    pub fn cite_key(&self) -> String {
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

impl ItemMetadata for DatabaseItem {
    fn title(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.fields.title)
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        self.fields.description.as_deref().map(Cow::Borrowed)
    }

    fn item_type(&self) -> ItemType {
        ItemType::try_from(self.fields.r#type.as_str()).unwrap_or(ItemType::default())
    }

    fn authors(&self) -> Vec<String> {
        self.authors.iter().map(|a| a.name.clone()).collect()
    }

    fn isbn(&self) -> Option<Cow<'_, str>> {
        self.fields.isbn.as_deref().map(Cow::Borrowed)
    }

    fn doi(&self) -> Option<Cow<'_, str>> {
        self.fields.doi.as_deref().map(Cow::Borrowed)
    }

    fn publication_date(&self) -> Option<Cow<'_, str>> {
        self.fields.publication_date.as_deref().map(Cow::Borrowed)
    }

    fn cover_image_url(&self) -> Option<Cow<'_, str>> {
        self.fields.cover_image_url.as_deref().map(Cow::Borrowed)
    }

    fn source(&self) -> Cow<'_, str> {
        Cow::Owned("database".to_string())
    }

    fn tags(&self) -> Vec<String> {
        self.tags.iter().map(|t| t.name.clone()).collect()
    }

    fn container(&self) -> Option<Cow<'_, str>> {
        self.fields.container.as_deref().map(Cow::Borrowed)
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
    pub shared_paper_id: Option<String>,
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        let _ = writeln!(str, "Title: {}", self.title);
        if let Some(desc) = &self.description {
            let _ = write!(str, "Description:\n{desc}");
        }
        let _ = writeln!(str, "Type: {}", self.r#type);
        if let Some(doi) = &self.doi {
            let _ = writeln!(str, "DOI: {doi}");
        }
        if let Some(isbn) = &self.isbn {
            let _ = writeln!(str, "ISBN: {isbn}");
        }
        if let Some(date) = &self.publication_date {
            let _ = writeln!(str, "Publication Date: {date}");
        }
        let _ = writeln!(str, "Slug: {}", self.slug);
        if let Some(url) = &self.cover_image_url {
            let _ = writeln!(str, "Cover URL: {url}");
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
            shared_paper_id: value.shared_paper_id().map(|id| id.to_string()),
        }
    }
}
