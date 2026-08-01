use crossterm::event::Event;
use minastirith_core::{
    metadata::common_metadata::{ItemMetadata, ItemType},
    schema::form::{Field, FormSnapshot},
    traits::MetadataForm,
};
use ratatui_textarea::TextArea;
use tui_input::{Input, backend::crossterm::EventHandler};

#[derive(Debug, Default)]
pub struct TuiMetadataForm {
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

impl MetadataForm for TuiMetadataForm {
    fn snapshot(self) -> FormSnapshot {
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

    fn from_candidate(candidate: &dyn ItemMetadata) -> Self {
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

    fn field_value(&self, field: &Field) -> String {
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

    fn field_title(&self, field: &Field) -> String {
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

    fn next_field(&mut self) {
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

    fn prev_field(&mut self) {
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

    fn cycle_item_type(&mut self) {
        self.item_type = match self.item_type {
            ItemType::Book => ItemType::Article,
            ItemType::Article => ItemType::Report,
            ItemType::Report => ItemType::Thesis,
            ItemType::Thesis => ItemType::Misc,
            ItemType::Misc => ItemType::Book,
        }
    }
}

impl TuiMetadataForm {
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
