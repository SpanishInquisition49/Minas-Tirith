use color_eyre::Result;
use minastirith_core::{schema::item::DatabaseItem, traits::Selectable};
use ratatui::widgets::ListState;

use crate::tui::app::{App, Mode};

impl App {
    pub fn items(&self) -> &[DatabaseItem] {
        self.items_component.items()
    }

    pub fn selected_item(&self) -> Option<&DatabaseItem> {
        self.items_component.core().selected_item()
    }

    pub fn selected_item_index(&self) -> Option<usize> {
        self.items_component.core().selected_index()
    }

    pub fn select_item(&mut self, index: usize) {
        self.items_component
            .core_mut()
            .selected_index_mut()
            .replace(index);
    }

    pub fn selected_item_mut(&mut self) -> Option<&mut DatabaseItem> {
        self.items_component.core_mut().selected_item_mut()
    }

    pub fn selected_tab(&self) -> usize {
        self.items_component.core().selected_tab()
    }

    pub fn item_list_state_mut(&mut self) -> &mut ListState {
        self.items_component.list_state_mut()
    }

    pub fn request_open_file_picker(&mut self) -> Result<()> {
        let cwd = self.file_explorer.cwd();
        self.file_explorer = Self::build_explorer(Some(cwd))?;
        self.mode = Mode::Insert;
        Ok(())
    }
}
