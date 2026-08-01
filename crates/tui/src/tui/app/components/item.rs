use std::sync::Arc;

use color_eyre::eyre::Result;
use minastirith_core::{
    database::Archive, schema::item::DatabaseItem, state::item::ItemState, traits::Selectable,
};
use ratatui::widgets::ListState;

use crate::traits::SelectableSync;

pub struct ItemComponent {
    core: ItemState,
    list_state: ListState,
}

impl ItemComponent {
    pub fn new(archive: Arc<Archive>) -> Self {
        Self {
            core: ItemState::new(archive),
            list_state: ListState::default(),
        }
    }

    pub async fn refresh(&mut self, collection_id: Option<i32>) -> Result<()> {
        self.core.refresh(collection_id).await?;
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
