use color_eyre::Result;
use crossterm::event::Event;
use std::sync::Arc;

use minastirith_core::{
    database::Archive, schema::collection::Collection, state::collection::CollectionState,
    traits::Selectable,
};
use ratatui::widgets::ListState;
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::traits::SelectableSync;

pub struct CollectionComponent {
    core: CollectionState,
    list_state: ListState,
    name_input: Input,
}

impl CollectionComponent {
    pub fn new(archive: Arc<Archive>) -> Self {
        let core = CollectionState::new(archive);
        Self {
            core,
            list_state: ListState::default(),
            name_input: Input::default(),
        }
    }

    pub async fn refresh(&mut self) -> Result<()> {
        self.core.refresh().await?;
        Ok(())
    }

    pub fn items(&self) -> &[Collection] {
        self.core.items()
    }

    pub fn selected(&self) -> Option<&Collection> {
        self.core
            .selected_index()
            .and_then(|i| self.core.items().get(i))
    }

    pub fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    pub fn core(&self) -> &CollectionState {
        &self.core
    }

    pub fn core_mut(&mut self) -> &mut CollectionState {
        &mut self.core
    }

    pub fn open_create(&mut self) {
        self.name_input.reset();
    }

    pub async fn confirm_create(&mut self) -> Result<()> {
        let name = self.name_input.value_and_reset();
        let name = name.trim();
        if name.is_empty() {
            return Ok(());
        }
        self.core.create_collection(name).await?;
        self.core.refresh().await?;
        self.resync_after_refresh();
        Ok(())
    }

    pub fn name_input(&self) -> &Input {
        &self.name_input
    }

    pub fn handle_event(&mut self, event: &Event) {
        self.name_input.handle_event(event);
    }
}

impl SelectableSync for CollectionComponent {
    type Inner = CollectionState;

    fn inner(&mut self) -> &mut Self::Inner {
        self.core_mut()
    }

    fn sync(&mut self) {
        let i = self.inner().selected_index();
        self.list_state.select(i);
    }
}
