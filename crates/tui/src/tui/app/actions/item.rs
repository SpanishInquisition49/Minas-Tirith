use color_eyre::Result;
use minastirith_core::{schema::item::DatabaseItem, traits::Selectable};
use ratatui::widgets::ListState;
use ratatui_notifications::Level;
use tui_input::Input;

use crate::{
    traits::SelectableSync,
    tui::app::{App, Mode},
};

impl App {
    /// All items currently loaded in the items list.
    pub fn items(&self) -> &[DatabaseItem] {
        self.items_component.items()
    }

    /// The currently selected item, if any.
    pub fn selected_item(&self) -> Option<&DatabaseItem> {
        self.items_component.core().selected_item()
    }

    /// Index of the currently selected item, if any.
    pub fn selected_item_index(&self) -> Option<usize> {
        self.items_component.core().selected_index()
    }

    /// Select the item at `index`.
    pub fn select_item(&mut self, index: usize) {
        self.items_component
            .core_mut()
            .selected_index_mut()
            .replace(index);
    }

    /// Select the next item and sync dependent UI state.
    pub fn next_item(&mut self) {
        self.items_component.core_mut().select_next();
        self.items_component.sync();
    }

    /// Select the previous item and sync dependent UI state.
    pub fn prev_item(&mut self) {
        self.items_component.core_mut().select_prev();
        self.items_component.sync();
    }

    /// Index of the currently selected tab.
    pub fn selected_tab(&self) -> usize {
        self.items_component.core().selected_tab()
    }

    /// Move to the next tab and refresh the item list for it.
    /// # Errors
    /// Returns an error if refreshing the item list fails.
    pub async fn next_tab(&mut self) -> Result<()> {
        self.items_component.core_mut().tabs_next();
        let collection = self.collection_component.selected();
        self.items_component.refresh(collection).await
    }

    /// Move to the previous tab and refresh the item list for it.
    /// # Errors
    /// Returns an error if refreshing the item list fails.
    pub async fn prev_tab(&mut self) -> Result<()> {
        self.items_component.core_mut().tabs_prev();
        let collection = self.collection_component.selected();
        self.items_component.refresh(collection).await
    }

    /// Mutable access to the item list's ratatui `ListState`.
    pub fn item_list_state_mut(&mut self) -> &mut ListState {
        self.items_component.list_state_mut()
    }

    /// The item search input widget.
    pub fn item_search_input(&self) -> &Input {
        self.items_component.search_input()
    }

    /// Mutable access to the item search input widget.
    pub fn item_search_input_mut(&mut self) -> &mut Input {
        self.items_component.search_input_mut()
    }

    /// Reopen the file picker at its current working directory and switch
    /// to insert mode.
    /// # Errors
    /// Returns an error if the file explorer cannot be rebuilt.
    pub fn request_open_file_picker(&mut self) -> Result<()> {
        let cwd = self.file_explorer.cwd();
        self.file_explorer = Self::build_explorer(Some(cwd))?;
        self.mode = Mode::Insert;
        Ok(())
    }

    /// Switch to search mode.
    pub fn open_search(&mut self) {
        self.mode = Mode::Search;
    }

    /// Apply the current search input as the active item filter and
    /// return to normal mode.
    /// # Errors
    /// Currently always returns `Ok`; search failures are shown as a
    /// notification instead of being propagated.
    pub async fn confirm_search(&mut self) -> Result<()> {
        let collection = self.collection_component.selected();
        match self.items_component.confirm_search(collection).await {
            Ok(applied) => {
                if applied {
                    self.collection_component
                        .core_mut()
                        .selected_index_mut()
                        .replace(0);
                    self.collection_component.sync();
                }
                self.mode = Mode::Normal;
            }
            Err(e) => self.notify(e.to_string(), " Search ".to_string(), Level::Error),
        }
        Ok(())
    }

    /// Cancel the active search, clear the filter, and refresh the item
    /// list.
    /// # Errors
    /// Returns an error if clearing the search or refreshing the item list
    /// fails.
    pub async fn cancel_search(&mut self) -> Result<()> {
        self.mode = Mode::Normal;
        self.items_component.cancel_search()?;
        self.items_component
            .refresh(self.collection_component.selected())
            .await?;
        Ok(())
    }
}
