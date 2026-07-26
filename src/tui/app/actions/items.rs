use std::path::PathBuf;

use crate::{
    metadata::common_metadata::ItemType,
    schema::{collection::Collection, item::DatabaseItem},
    tui::app::{App, Focus, Mode, TABS_LABELS, traits::ListWidget},
};

use color_eyre::Result;
use ratatui::widgets::ListState;

impl App {
    pub fn tabs_prev(&mut self) {
        self.selected_tab = if self.selected_tab == 0 {
            TABS_LABELS.len() - 1
        } else {
            self.selected_tab - 1
        };
    }

    pub fn tabs_next(&mut self) {
        self.selected_tab = if self.selected_tab + 1 == TABS_LABELS.len() {
            0
        } else {
            self.selected_tab + 1
        };
    }

    pub fn selected_tab(&self) -> usize {
        self.selected_tab
    }

    pub fn select_prev(&mut self) {
        let len = self
            .items
            .iter()
            .fold(0, |acc, i| if self.keep_items(i) { acc + 1 } else { acc });
        let i = match self.items_list_state.selected() {
            Some(i) if i > 0 => i - 1,
            Some(_) if len > 0 => len - 1,
            _ => 0,
        };
        self.items_list_state.select(Some(i));
    }

    pub fn select_next(&mut self) {
        let len = self
            .items
            .iter()
            .fold(0, |acc, i| if self.keep_items(i) { acc + 1 } else { acc });
        let i = match self.items_list_state.selected() {
            Some(i) if i + 1 < len => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.items_list_state.select(Some(i));
    }

    pub fn keep_items(&self, item: &DatabaseItem) -> bool {
        let keep = match self.collections.selected() {
            Some(c) if item.collections.is_empty() && !Collection::is_trivial_collection(c.id) => {
                false
            }
            Some(collection) if !Collection::is_trivial_collection(collection.id) => {
                item.collections.iter().any(|c| c.id == collection.id)
            }
            _ => true,
        };
        if self.selected_tab == 0 {
            return keep;
        }
        let Ok(active_item_type) = ItemType::try_from(TABS_LABELS[self.selected_tab]) else {
            return keep;
        };
        let item_type =
            ItemType::try_from(item.fields.r#type.as_str()).unwrap_or(ItemType::default());
        keep && active_item_type == item_type
    }

    pub fn items(&self) -> &[DatabaseItem] {
        self.items.as_slice()
    }

    pub fn items_list_state(&self) -> &ListState {
        &self.items_list_state
    }

    pub fn items_list_state_mut(&mut self) -> &mut ListState {
        &mut self.items_list_state
    }

    pub fn selected_item(&self) -> Option<&DatabaseItem> {
        let slug = {
            let filtered_items = self
                .items
                .iter()
                .filter(|i| self.keep_items(i))
                .collect::<Vec<_>>();
            let index = self.items_list_state.selected()?;
            filtered_items.get(index)?.fields.slug.clone()
        };
        self.items.iter().find(|i| i.fields.slug == slug)
    }

    pub async fn request_refresh_item_list(&mut self) -> Result<()> {
        self.items = self.archive.get_all_items().await?;
        if !self.items.is_empty() && self.items_list_state.selected().is_none() {
            self.items_list_state.select(Some(0));
        }
        Ok(())
    }

    pub fn request_file_opening(&self) -> Result<()> {
        let Some(index) = self.items_list_state.selected() else {
            return Ok(());
        };
        let filtered = self
            .items
            .iter()
            .filter(|i| self.keep_items(i))
            .collect::<Vec<_>>();
        let Some(item) = filtered.get(index) else {
            return Ok(());
        };
        opener::open(PathBuf::from(&item.path))?;
        Ok(())
    }

    pub fn request_open_file_picker(&mut self) -> Result<()> {
        let cwd = self.file_explorer.cwd();
        self.file_explorer = Self::build_explorer(Some(cwd))?;
        self.mode = Mode::Insert;
        Ok(())
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Items => Focus::Collections,
            Focus::Collections => Focus::Items,
        }
    }
}
