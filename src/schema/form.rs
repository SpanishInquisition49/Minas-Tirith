use crossterm::event::Event;
use ratatui::style::{Modifier, Style};
use ratatui_textarea::{TextArea, WrapMode};
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

#[derive(Clone, Debug)]
pub struct FormSnapshot {
    pub title: String,
    pub description: String,
    pub item_type: ItemType,
    pub doi: String,
    pub isbn: String,
    pub publication_date: String,
    pub tags: Vec<String>,
    pub authors: Vec<String>,
    pub container: Option<String>,
    pub cover_image_url: Option<String>,
}

impl ItemMetadata for FormSnapshot {
    fn title(&self) -> String {
        self.title.to_string()
    }

    fn description(&self) -> Option<String> {
        Some(self.description.clone())
    }

    fn item_type(&self) -> ItemType {
        self.item_type.clone()
    }

    fn authors(&self) -> Vec<String> {
        self.authors.clone()
    }

    fn isbn(&self) -> Option<String> {
        if self.isbn.is_empty() {
            None
        } else {
            Some(self.isbn.to_string())
        }
    }

    fn doi(&self) -> Option<String> {
        if self.doi.is_empty() {
            None
        } else {
            Some(self.doi.to_string())
        }
    }

    fn publication_date(&self) -> Option<String> {
        if self.publication_date.is_empty() {
            None
        } else {
            Some(self.publication_date.to_string())
        }
    }

    fn cover_image_url(&self) -> Option<String> {
        self.cover_image_url.clone()
    }

    fn source(&self) -> String {
        todo!()
    }

    fn tags(&self) -> Vec<String> {
        self.tags.iter().map(|t| t.to_string()).collect()
    }

    fn container(&self) -> Option<String> {
        self.container.clone()
    }
}

#[derive(Debug)]
pub struct MetadataForm {
    pub title: Input,
    pub description: TextArea<'static>,
    pub doi: Input,
    pub isbn: Input,
    pub publication_date: Input,
    pub tags: Input,
    pub item_type: ItemType,
    pub cover_image_url: Option<String>,
    pub authors: Vec<String>,
    pub field_index: usize,
    pub editing: bool,
    pub container: Option<String>,
}

impl MetadataForm {
    pub fn new() -> Self {
        Self {
            title: Input::default(),
            description: TextArea::default(),
            doi: Input::default(),
            isbn: Input::default(),
            publication_date: Input::default(),
            tags: Input::default(),
            item_type: ItemType::Misc,
            cover_image_url: None,
            authors: Vec::new(),
            field_index: 0,
            editing: false,
            container: None,
        }
    }

    pub fn from_candidate(candidate: &dyn ItemMetadata) -> Self {
        Self {
            title: candidate.title().into(),
            description: TextArea::new(
                candidate
                    .description()
                    .unwrap_or_default()
                    .split("\n")
                    .map(|l| l.to_string())
                    .collect::<Vec<_>>(),
            ),
            doi: candidate.doi().unwrap_or_default().into(),
            isbn: candidate.isbn().unwrap_or_default().into(),
            publication_date: candidate.publication_date().unwrap_or_default().into(),
            tags: Input::new("".to_string()),
            item_type: candidate.item_type(),
            cover_image_url: candidate.cover_image_url(),
            authors: candidate.authors(),
            field_index: 0,
            editing: false,
            container: candidate.container(),
        }
    }

    pub fn from_item(item: &DatabaseItem) -> Self {
        let mut text_area = TextArea::new(
            item.fields
                .description
                .clone()
                .unwrap_or_default()
                .split("\n")
                .map(|l| l.to_string())
                .collect::<Vec<_>>(),
        );
        text_area.set_wrap_mode(WrapMode::Word);
        text_area.set_cursor_style(Style::default().add_modifier(Modifier::BOLD));
        let item_type =
            ItemType::try_from(item.fields.r#type.as_str()).unwrap_or(ItemType::default());
        Self {
            title: item.fields.title.clone().into(),
            description: text_area,
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
            container: item.fields.container.clone(),
        }
    }

    pub fn snapshot(&self) -> FormSnapshot {
        FormSnapshot {
            title: self.title.value().to_string(),
            description: self.description.lines().join("\n"),
            item_type: self.item_type.clone(),
            doi: self.doi.value().to_string(),
            isbn: self.isbn.value().to_string(),
            publication_date: self.publication_date.value().to_string(),
            tags: self
                .tags
                .value()
                .split(",")
                .map(|t| t.to_string())
                .collect(),
            authors: self.authors.clone(),
            container: self.container.clone(),
            cover_image_url: self.cover_image_url.clone(),
        }
    }

    pub fn field_value(&self, index: usize) -> String {
        match index {
            0 => self.title.to_string(),
            1 => self
                .description
                .lines()
                .iter()
                .map(|l| l.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
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
            1 => {
                if let Event::Key(key) = event {
                    self.description.input(*key);
                }
                None
            }
            2 => self.doi.handle_event(event),
            3 => self.isbn.handle_event(event),
            4 => self.publication_date.handle_event(event),
            5 => self.tags.handle_event(event),
            _ => unreachable!(),
        };
    }
}
