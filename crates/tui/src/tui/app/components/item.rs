use std::sync::Arc;

use color_eyre::eyre::Result;
use crossterm::event::Event;
use minastirith_core::{
    database::Archive,
    schema::{collection::Collection, item::DatabaseItem},
    state::item::ItemState,
    traits::Selectable,
};
use ratatui::widgets::ListState;
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::traits::SelectableSync;

pub struct ItemComponent {
    core: ItemState,
    list_state: ListState,
    search_input: Input,
}

impl ItemComponent {
    pub fn new(archive: Arc<Archive>) -> Self {
        Self {
            core: ItemState::new(archive),
            list_state: ListState::default(),
            search_input: Input::default(),
        }
    }

    pub async fn refresh(&mut self, collection: Option<&Collection>) -> Result<()> {
        self.core.refresh(collection).await?;
        self.sync();
        Ok(())
    }

    pub fn core(&self) -> &ItemState {
        &self.core
    }

    pub fn core_mut(&mut self) -> &mut ItemState {
        &mut self.core
    }

    pub fn items(&self) -> &[DatabaseItem] {
        self.core.items()
    }

    pub fn items_mut(&mut self) -> &mut [DatabaseItem] {
        self.core.items_mut()
    }

    pub fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    pub fn search_input(&self) -> &Input {
        &self.search_input
    }

    pub fn search_input_mut(&mut self) -> &mut Input {
        &mut self.search_input
    }

    pub fn handle_search_event(&mut self, event: &Event) {
        self.search_input.handle_event(event);
    }

    /// Parse the current search input and apply it.
    /// The error is returned on parse failure so the caller can surface it.
    pub async fn confirm_search(&mut self, collection: Option<&Collection>) -> Result<bool> {
        let raw = self.search_input.value();
        let applied = self.core.set_search(raw)?;
        self.refresh(collection).await?;
        Ok(applied)
    }

    pub fn cancel_search(&mut self) -> Result<()> {
        self.core_mut().set_search("")?;
        self.search_input.reset();
        Ok(())
    }
}

impl SelectableSync for ItemComponent {
    type Inner = ItemState;

    fn inner(&mut self) -> &mut Self::Inner {
        &mut self.core
    }

    fn sync(&mut self) {
        let i = self.inner().selected_index();
        self.list_state.select(i);
    }
}
