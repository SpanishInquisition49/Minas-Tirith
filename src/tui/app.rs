use ratatui::widgets::ListState;

use crate::{database::archive::Archive, schema::item::DatabaseItem};

pub enum Mode {
    Normal,
    Search,
}

pub struct App {
    pub archive: Archive,
    pub mode: Mode,
    pub items: Vec<DatabaseItem>,
    pub list_state: ListState,
    pub search_query: String,
    pub quit: bool,
}

impl App {
    pub async fn new(archive: Archive) -> color_eyre::Result<Self> {
        let items = archive.get_all_items().await?;
        let mut list = ListState::default();
        if !items.is_empty() {
            list.select_first();
        }
        Ok(Self {
            archive,
            mode: Mode::Normal,
            items,
            list_state: list,
            search_query: String::new(),
            quit: false,
        })
    }

    pub fn select_prev(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) if i > 0 => i - 1,
            Some(i) => i,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) if i < self.items.len() => i + 1,
            Some(i) => i,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn selected_item(&self) -> Option<&DatabaseItem> {
        self.list_state.selected().and_then(|i| self.items.get(i))
    }
}
