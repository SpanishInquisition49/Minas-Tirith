use std::{borrow::Cow, str::FromStr};

use color_eyre::eyre::bail;
use crossterm::event::Event;
use ratatui::style::{Modifier, Style};
use ratatui_textarea::{TextArea, WrapMode};
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    metadata::common_metadata::{ItemMetadata, ItemType},
    schema::item::DatabaseItem,
};

pub const FIELD_LABELS: [&str; 8] = [
    "Title",
    "Description",
    "Container",
    "DOI",
    "ISBN",
    "Publication Date",
    "Tags",
    "Cover URL",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Field {
    Title,
    Description,
    Container,
    Doi,
    Isbn,
    PublicationDate,
    Tags,
    CoverUrl,
}

impl FromStr for Field {
    type Err = color_eyre::eyre::Error;
    fn from_str(s: &str) -> color_eyre::Result<Self> {
        Ok(match s.to_lowercase().as_str() {
            "title" => Field::Title,
            "description" => Field::Description,
            "container" => Field::Container,
            "doi" => Field::Doi,
            "isbn" => Field::Isbn,
            "publication date" => Field::PublicationDate,
            "tags" => Field::Tags,
            "cover url" => Field::CoverUrl,
            _ => bail!(format!("Could not convert {s} to Field")),
        })
    }
}

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
    fn title(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.title)
    }

    fn description(&self) -> Option<Cow<'_, str>> {
        Some(Cow::Borrowed(&self.description))
    }

    fn item_type(&self) -> ItemType {
        self.item_type
    }

    fn authors(&self) -> Vec<String> {
        self.authors.clone()
    }

    fn isbn(&self) -> Option<Cow<'_, str>> {
        if self.isbn.is_empty() {
            None
        } else {
            Some(Cow::Borrowed(&self.isbn))
        }
    }

    fn doi(&self) -> Option<Cow<'_, str>> {
        if self.doi.is_empty() {
            None
        } else {
            Some(Cow::Borrowed(&self.doi))
        }
    }

    fn publication_date(&self) -> Option<Cow<'_, str>> {
        if self.publication_date.is_empty() {
            None
        } else {
            Some(Cow::Borrowed(&self.publication_date))
        }
    }

    fn cover_image_url(&self) -> Option<Cow<'_, str>> {
        self.cover_image_url
            .as_deref()
            .map(|url| Cow::Borrowed(url))
    }

    fn source(&self) -> Cow<'_, str> {
        Cow::default()
    }

    fn tags(&self) -> Vec<String> {
        self.tags
            .iter()
            .filter_map(|t| {
                if t.trim().is_empty() {
                    None
                } else {
                    Some(t.trim().to_string())
                }
            })
            .collect()
    }

    fn container(&self) -> Option<Cow<'_, str>> {
        self.container.as_deref().map(|c| Cow::Borrowed(c))
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
    pub cover_image_url: Input,
    pub authors: Vec<String>,
    pub field: Field,
    pub editing: bool,
    pub container: Input,
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
            cover_image_url: Input::default(),
            authors: Vec::new(),
            field: Field::Title,
            editing: false,
            container: Input::default(),
        }
    }

    pub fn from_candidate(candidate: &dyn ItemMetadata) -> Self {
        Self {
            title: candidate.title().to_string().into(),
            description: TextArea::new(
                candidate
                    .description()
                    .unwrap_or_default()
                    .split("\n")
                    .map(|l| l.to_string())
                    .collect::<Vec<_>>(),
            ),
            doi: candidate.doi().unwrap_or_default().to_string().into(),
            isbn: candidate.isbn().unwrap_or_default().to_string().into(),
            publication_date: candidate
                .publication_date()
                .unwrap_or_default()
                .to_string()
                .into(),
            tags: Input::new("".to_string()),
            item_type: candidate.item_type(),
            cover_image_url: candidate
                .cover_image_url()
                .unwrap_or_default()
                .to_string()
                .into(),
            authors: candidate.authors(),
            field: Field::Title,
            editing: false,
            container: candidate.container().unwrap_or_default().to_string().into(),
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
            cover_image_url: item
                .fields
                .cover_image_url
                .clone()
                .unwrap_or_default()
                .into(),
            authors: item.authors.iter().map(|a| a.name.clone()).collect(),
            field: Field::Title,
            editing: false,
            container: item.fields.container.clone().unwrap_or_default().into(),
        }
    }

    pub fn snapshot(self) -> FormSnapshot {
        let cover_image_url = if self.cover_image_url.value().trim().is_empty() {
            None
        } else {
            Some(self.cover_image_url.value().trim().to_string())
        };
        let container = if self.container.value().trim().is_empty() {
            None
        } else {
            Some(self.container.value().trim().to_string())
        };
        FormSnapshot {
            title: self.title.value().to_string(),
            description: self.description.lines().join("\n"),
            item_type: self.item_type,
            doi: self.doi.value().to_string(),
            isbn: self.isbn.value().to_string(),
            publication_date: self.publication_date.value().to_string(),
            tags: self
                .tags
                .value()
                .split(",")
                .map(|t| t.trim().to_string())
                .collect(),
            authors: self.authors,
            container,
            cover_image_url,
        }
    }

    pub fn field_value(&self, field: &Field) -> String {
        match field {
            Field::Title => self.title.to_string(),
            Field::Description => self.description.lines().join("\n"),
            Field::Container => self.container.to_string(),
            Field::Doi => self.doi.to_string(),
            Field::Isbn => self.isbn.to_string(),
            Field::PublicationDate => self.publication_date.to_string(),
            Field::Tags => self.tags.to_string(),
            Field::CoverUrl => self.cover_image_url.to_string(),
        }
    }

    pub fn field_title(&self, field: &Field) -> String {
        match field {
            Field::Title => "Title".to_string(),
            Field::Description => "Description".to_string(),
            Field::Container => match self.item_type {
                ItemType::Book => "Publisher".to_string(),
                ItemType::Article => "Journal".to_string(),
                ItemType::Report => "Institution".to_string(),
                ItemType::Thesis => "University".to_string(),
                ItemType::Misc => "How Published".to_string(),
            },
            Field::Doi => "DOI".to_string(),
            Field::Isbn => "ISBN".to_string(),
            Field::PublicationDate => "Publication Date".to_string(),
            Field::Tags => "Tags".to_string(),
            Field::CoverUrl => "Cover URL".to_string(),
        }
    }

    pub fn next_field(&mut self) {
        self.field = match self.field {
            Field::Title => Field::Description,
            Field::Description => Field::Container,
            Field::Container => Field::Doi,
            Field::Doi => Field::Isbn,
            Field::Isbn => Field::PublicationDate,
            Field::PublicationDate => Field::Tags,
            Field::Tags => Field::CoverUrl,
            Field::CoverUrl => Field::Title,
        };
    }

    pub fn prev_field(&mut self) {
        self.field = match self.field {
            Field::Title => Field::CoverUrl,
            Field::Description => Field::Title,
            Field::Container => Field::Description,
            Field::Doi => Field::Container,
            Field::Isbn => Field::Doi,
            Field::PublicationDate => Field::Isbn,
            Field::Tags => Field::PublicationDate,
            Field::CoverUrl => Field::Tags,
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
        match self.field {
            Field::Title => self.title.handle_event(event),
            Field::Description => {
                if let Event::Key(key) = event {
                    self.description.input(*key);
                }
                None
            }
            Field::Container => self.container.handle_event(event),
            Field::Doi => self.doi.handle_event(event),
            Field::Isbn => self.isbn.handle_event(event),
            Field::PublicationDate => self.publication_date.handle_event(event),
            Field::Tags => self.tags.handle_event(event),
            Field::CoverUrl => self.cover_image_url.handle_event(event),
        };
    }
}
