use crate::{
    metadata::common_metadata::{ItemMetadata, ItemType},
    schema::item::{DatabaseItem, Item},
};

pub const FIELD_LABELS: [&str; 6] = [
    "Title",
    "Description",
    "DOI",
    "ISBN",
    "Publication Date",
    "Tags",
];

#[derive(Debug)]
pub struct MetadataForm {
    pub title: String,
    pub description: String,
    pub doi: String,
    pub isbn: String,
    pub publication_date: String,
    pub tags: String,
    pub item_type: ItemType,
    pub cover_image_url: Option<String>,
    pub authors: Vec<String>,
    pub filed_index: usize,
    pub editing: bool,
}

impl MetadataForm {
    pub fn from_candidate(candidate: &dyn ItemMetadata) -> Self {
        Self {
            title: candidate.title(),
            description: candidate.description().unwrap_or_default(),
            doi: candidate.doi().unwrap_or_default(),
            isbn: candidate.isbn().unwrap_or_default(),
            publication_date: candidate.publication_date().unwrap_or_default(),
            tags: String::new(),
            item_type: candidate.item_type(),
            cover_image_url: candidate.cover_image_url(),
            authors: candidate.authors(),
            filed_index: 0,
            editing: false,
        }
    }

    pub fn from_item(item: &DatabaseItem) -> Self {
        let item_type = ItemType::try_from(item.fields.r#type.as_str()).unwrap_or(ItemType::Misc);
        Self {
            title: item.fields.title.clone(),
            description: item.fields.description.clone().unwrap_or_default(),
            doi: item.fields.doi.clone().unwrap_or_default(),
            isbn: item.fields.isbn.clone().unwrap_or_default(),
            publication_date: item.fields.publication_date.clone().unwrap_or_default(),
            tags: item
                .tags
                .iter()
                .map(|t| t.name.clone())
                .collect::<Vec<_>>()
                .join(", "),
            item_type,
            cover_image_url: item.fields.cover_image_url.clone(),
            authors: item.authors.iter().map(|a| a.name.clone()).collect(),
            filed_index: 0,
            editing: false,
        }
    }

    pub fn current_field_mut(&mut self) -> &mut String {
        match self.filed_index {
            0 => &mut self.title,
            1 => &mut self.description,
            2 => &mut self.doi,
            3 => &mut self.isbn,
            4 => &mut self.publication_date,
            5 => &mut self.tags,
            _ => unreachable!(),
        }
    }

    pub fn field_value(&self, index: usize) -> &str {
        match index {
            0 => &self.title,
            1 => &self.description,
            2 => &self.doi,
            3 => &self.isbn,
            4 => &self.publication_date,
            5 => &self.tags,
            _ => unreachable!(),
        }
    }

    pub fn next_field(&mut self) {
        self.filed_index = (self.filed_index + 1) % FIELD_LABELS.len();
    }

    pub fn prev_field(&mut self) {
        self.filed_index = if self.filed_index == 0 {
            FIELD_LABELS.len() - 1
        } else {
            self.filed_index - 1
        }
    }

    pub fn cycle_item_type(&mut self) {
        self.item_type = match self.item_type {
            ItemType::Book => ItemType::Article,
            ItemType::Article => ItemType::Report,
            ItemType::Report => ItemType::Thesis,
            ItemType::Thesis => ItemType::Misc,
            ItemType::Misc => ItemType::Book,
        }
    }

    pub fn tags_vec(&self) -> Vec<String> {
        self.tags
            .split(",")
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect()
    }

    fn opt(s: &str) -> Option<String> {
        let s = s.trim();
        if s.is_empty() {
            None
        } else {
            Some(s.to_string())
        }
    }

    pub fn description_opt(&self) -> Option<String> {
        Self::opt(&self.description)
    }

    pub fn doi_opt(&self) -> Option<String> {
        Self::opt(&self.doi)
    }

    pub fn isbn_opt(&self) -> Option<String> {
        Self::opt(&self.isbn)
    }

    pub fn publication_date_opt(&self) -> Option<String> {
        Self::opt(&self.publication_date)
    }
}
