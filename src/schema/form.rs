use crossterm::event::Event;
use slug::slugify;
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    metadata::common_metadata::{ItemMetadata, ItemType},
    schema::item::DatabaseItem,
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
    pub title: Input,
    pub description: Input,
    pub doi: Input,
    pub isbn: Input,
    pub publication_date: Input,
    pub tags: Input,
    pub item_type: ItemType,
    pub cover_image_url: Option<String>,
    pub authors: Vec<String>,
    pub field_index: usize,
    pub editing: bool,
}

impl ItemMetadata for MetadataForm {
    fn title(&self) -> String {
        self.title.to_string()
    }

    fn description(&self) -> Option<String> {
        Self::opt(&self.description.to_string())
    }

    fn item_type(&self) -> ItemType {
        self.item_type.clone()
    }

    fn authors(&self) -> Vec<String> {
        self.authors.clone()
    }

    fn isbn(&self) -> Option<String> {
        Self::opt(&self.isbn.to_string())
    }

    fn doi(&self) -> Option<String> {
        Self::opt(&self.doi.to_string())
    }

    fn publication_date(&self) -> Option<String> {
        Self::opt(&self.publication_date.to_string())
    }

    fn cover_image_url(&self) -> Option<String> {
        self.cover_image_url.clone()
    }

    fn slug(&self) -> String {
        slugify(self.title.to_string())
    }

    fn source(&self) -> String {
        "internal".to_string()
    }

    fn tags(&self) -> Vec<String> {
        self.tags_vec()
    }
}

impl MetadataForm {
    pub fn new() -> Self {
        Self {
            title: Input::default(),
            description: Input::default(),
            doi: Input::default(),
            isbn: Input::default(),
            publication_date: Input::default(),
            tags: Input::default(),
            item_type: ItemType::Misc,
            cover_image_url: None,
            authors: Vec::new(),
            field_index: 0,
            editing: false,
        }
    }

    pub fn from_candidate(candidate: &dyn ItemMetadata) -> Self {
        Self {
            title: candidate.title().into(),
            description: candidate.description().unwrap_or_default().into(),
            doi: candidate.doi().unwrap_or_default().into(),
            isbn: candidate.isbn().unwrap_or_default().into(),
            publication_date: candidate.publication_date().unwrap_or_default().into(),
            tags: Input::new("".to_string()),
            item_type: candidate.item_type(),
            cover_image_url: candidate.cover_image_url(),
            authors: candidate.authors(),
            field_index: 0,
            editing: false,
        }
    }

    pub fn from_item(item: &DatabaseItem) -> Self {
        let item_type = ItemType::try_from(item.fields.r#type.as_str()).unwrap_or(ItemType::Misc);
        Self {
            title: item.fields.title.clone().into(),
            description: item.fields.description.clone().unwrap_or_default().into(),
            doi: item.fields.doi.clone().unwrap_or_default().into(),
            isbn: item.fields.isbn.clone().unwrap_or_default().into(),
            publication_date: item
                .fields
                .publication_date
                .clone()
                .unwrap_or_default()
                .into(),
            tags: item
                .tags
                .iter()
                .map(|t| t.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
                .into(),
            item_type,
            cover_image_url: item.fields.cover_image_url.clone(),
            authors: item.authors.iter().map(|a| a.name.clone()).collect(),
            field_index: 0,
            editing: false,
        }
    }

    pub fn field_value(&self, index: usize) -> String {
        match index {
            0 => self.title.to_string(),
            1 => self.description.to_string(),
            2 => self.doi.to_string(),
            3 => self.isbn.to_string(),
            4 => self.publication_date.to_string(),
            5 => self.tags.to_string(),
            _ => unreachable!(),
        }
    }

    pub fn next_field(&mut self) {
        self.field_index = (self.field_index + 1) % FIELD_LABELS.len();
    }

    pub fn prev_field(&mut self) {
        self.field_index = if self.field_index == 0 {
            FIELD_LABELS.len() - 1
        } else {
            self.field_index - 1
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

    pub fn handle_event(&mut self, event: &Event) {
        match self.field_index {
            0 => self.title.handle_event(event),
            1 => self.description.handle_event(event),
            2 => self.doi.handle_event(event),
            3 => self.isbn.handle_event(event),
            4 => self.publication_date.handle_event(event),
            5 => self.tags.handle_event(event),
            _ => unreachable!(),
        };
    }

    fn tags_vec(&self) -> Vec<String> {
        self.tags
            .to_string()
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
}
