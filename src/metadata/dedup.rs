use std::{borrow::Cow, collections::HashMap};

use rustix::path::Arg;
use slug::slugify;

use crate::metadata::common_metadata::{ItemMetadata, ItemType};

#[derive(Clone, Debug)]
pub struct MergedCandidate {
    pub title: String,
    pub description: Option<String>,
    pub item_type: ItemType,
    pub authors: Vec<String>,
    pub isbn: Option<String>,
    pub doi: Option<String>,
    pub publication_date: Option<String>,
    pub cover_image_url: Option<String>,
    pub tags: Vec<String>,
    pub container: Option<String>,
    pub sources: Vec<String>,
}

impl MergedCandidate {
    const PROVIDER_PRIORITY: &[&str] = &[
        "Crossref",
        "OpenAlex",
        "SemanticScholar",
        "Google Books",
        "CORE",
        "OpenLibrary",
    ];

    pub fn merge_candidates(items: Vec<Box<dyn ItemMetadata>>) -> Vec<MergedCandidate> {
        let mut groups: HashMap<String, Vec<Box<dyn ItemMetadata>>> = HashMap::new();
        for item in items {
            groups
                .entry(Self::dedup_key(item.as_ref()))
                .or_default()
                .push(item);
        }

        let mut merged: Vec<MergedCandidate> = groups
            .into_values()
            .map(|mut group| {
                group.sort_by_key(|i| Self::provider_rank(&i.source()));

                let mut out = MergedCandidate {
                    title: group[0].title().to_string(),
                    item_type: group[0].item_type(),
                    description: None,
                    authors: vec![],
                    isbn: None,
                    doi: None,
                    publication_date: None,
                    cover_image_url: None,
                    tags: vec![],
                    container: None,
                    sources: vec![],
                };

                for item in &group {
                    let item_source = item.source().to_string();
                    if !out.sources.contains(&item_source) {
                        out.sources.push(item_source);
                    }

                    match (item.description(), &out.description) {
                        (Some(d), None) => out.description = Some(d.to_string()),
                        // NOTE: prefer longer description/abstracts over shorter ones
                        (Some(d), Some(current)) if d.len() > current.len() => {
                            out.description = Some(d.to_string())
                        }
                        _ => {}
                    }

                    if out.authors.is_empty() {
                        out.authors = item.authors();
                    }

                    out.isbn = out
                        .isbn
                        .take()
                        .or_else(|| item.isbn().map(|v| v.to_string()));
                    out.doi = out.doi.take().or_else(|| item.doi().map(|v| v.to_string()));
                    out.publication_date = out
                        .publication_date
                        .take()
                        .or_else(|| item.publication_date().map(|v| v.to_string()));
                    out.cover_image_url = out
                        .cover_image_url
                        .take()
                        .or_else(|| item.cover_image_url().map(|v| v.to_string()));

                    for tag in item.tags() {
                        if !out.tags.contains(&tag) {
                            out.tags.push(tag);
                        }
                    }

                    out.container = out
                        .container
                        .take()
                        .or_else(|| item.container().map(|v| v.to_string()));
                }
                out
            })
            .collect();
        merged.sort_by(|a, b| a.title.cmp(&b.title));
        merged
    }

    fn normalize_doi(doi: &str) -> String {
        doi.trim()
            .to_lowercase()
            .trim_start_matches("https://doi.org/")
            .trim_start_matches("doi:")
            .to_string()
    }

    fn normalize_isbn(isbn: &str) -> String {
        isbn.chars()
            .filter(|c| c.is_ascii_alphabetic())
            .collect::<String>()
            .to_lowercase()
    }

    fn provider_rank(source: &str) -> usize {
        Self::PROVIDER_PRIORITY
            .iter()
            .position(|p| *p == source)
            .unwrap_or(usize::MAX)
    }

    fn dedup_key(item: &dyn ItemMetadata) -> String {
        if let Some(doi) = item.doi().filter(|d| !d.is_empty()) {
            return format!("doi:{}", Self::normalize_doi(&doi));
        }
        if let Some(isbn) = item.isbn().filter(|i| !i.is_empty()) {
            return format!("isbn:{}", Self::normalize_isbn(&isbn));
        }
        let title_slug = slugify(item.title());
        let author_slug = item.authors().first().map(slugify).unwrap_or_default();
        format!("title:{title_slug}:{author_slug}")
    }
}

impl ItemMetadata for MergedCandidate {
    fn title(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.title)
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        self.description.as_deref().map(Cow::Borrowed)
    }

    fn item_type(&self) -> ItemType {
        self.item_type
    }

    fn authors(&self) -> Vec<String> {
        self.authors.clone()
    }

    fn isbn(&self) -> Option<Cow<'_, str>> {
        self.isbn.as_deref().map(Cow::Borrowed)
    }

    fn doi(&self) -> Option<Cow<'_, str>> {
        self.doi.as_deref().map(Cow::Borrowed)
    }

    fn publication_date(&self) -> Option<Cow<'_, str>> {
        self.publication_date.as_deref().map(Cow::Borrowed)
    }

    fn cover_image_url(&self) -> Option<Cow<'_, str>> {
        self.cover_image_url.as_deref().map(Cow::Borrowed)
    }

    fn source(&self) -> Cow<'_, str> {
        self.sources
            .first()
            .map(|s| Cow::Owned(s.clone()))
            .unwrap_or_default()
    }

    fn tags(&self) -> Vec<String> {
        self.tags.clone()
    }

    fn container(&self) -> Option<Cow<'_, str>> {
        self.container.as_deref().map(Cow::Borrowed)
    }
}
