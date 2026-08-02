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
    /// Construct a [`CollectionComponent`] backed by `archive`, with no
    /// collections loaded yet.
    pub fn new(archive: Arc<Archive>) -> Self {
        let core = CollectionState::new(archive);
        Self {
            core,
            list_state: ListState::default(),
            name_input: Input::default(),
        }
    }

    /// Reload the collection list from the archive.
    /// # Errors
    /// Returns an error if fetching collections fails.
    pub async fn refresh(&mut self) -> Result<()> {
        self.core.refresh().await?;
        Ok(())
    }

    /// All collections currently loaded.
    pub fn items(&self) -> &[Collection] {
        self.core.items()
    }

    /// The currently selected collection, if any.
    pub fn selected(&self) -> Option<&Collection> {
        self.core
            .selected_index()
            .and_then(|i| self.core.items().get(i))
    }

    /// Mutable access to the collection list's ratatui `ListState`.
    pub fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    /// The underlying [`CollectionState`].
    pub fn core(&self) -> &CollectionState {
        &self.core
    }

    /// Mutable access to the underlying [`CollectionState`].
    pub fn core_mut(&mut self) -> &mut CollectionState {
        &mut self.core
    }

    /// Reset the new-collection name input, ready for the create dialog.
    pub fn open_create(&mut self) {
        self.name_input.reset();
    }

    /// Create a collection from the current name input, then refresh and
    /// resync the list. No-op if the input is blank.
    /// # Errors
    /// Returns an error if creating or refreshing the collection list
    /// fails.
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

    /// The input widget for the new-collection name field.
    pub fn name_input(&self) -> &Input {
        &self.name_input
    }

    /// Route `event` to the new-collection name input.
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
