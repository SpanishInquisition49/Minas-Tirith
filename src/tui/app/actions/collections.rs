use std::collections::HashSet;

use crate::{
    schema::collection::Collection,
    tui::app::{App, Mode, components::collection::CollectionAssignState, traits::ListWidget},
};
use color_eyre::Result;
use ratatui::widgets::ListState;
use tui_input::Input;

impl App {
    pub fn collections(&self) -> &[Collection] {
        self.collections.items()
    }

    pub async fn request_refresh_collections(&mut self) -> Result<()> {
        self.collections.refresh().await
    }

    pub fn select_collection_prev(&mut self) {
        self.collections.select_prev();
    }

    pub fn select_collection_next(&mut self) {
        self.collections.select_next();
    }

    pub fn colletion_list_state(&self) -> &ListState {
        &self.collections.list_state
    }

    pub fn collection_list_state_mut(&mut self) -> &mut ListState {
        &mut self.collections.list_state
    }

    pub fn confirm_collection_selection(&mut self) {
        self.collections.confirm_selection();
        self.items_list_state = ListState::default();
        if !self.items.is_empty() {
            self.items_list_state.select(Some(0));
        }
    }

    pub async fn delete_collection(&mut self) -> Result<()> {
        if let Some(collection_id) = self.collections.delete_highlighted().await? {
            self.items
                .iter_mut()
                .for_each(|i| i.collections.retain(|c| c.id != collection_id));
        }
        Ok(())
    }

    pub fn collection_input_field(&self) -> &Input {
        &self.collections.name_input
    }

    pub fn open_collection_create(&mut self) {
        self.collections.open_create();
        self.mode = Mode::CollectionCreate;
    }

    pub fn close_collection_create(&mut self) {
        self.mode = Mode::Normal;
    }

    pub async fn confirm_collection_create(&mut self) -> Result<()> {
        self.collections.confirm_create().await?;
        self.mode = Mode::Normal;
        Ok(())
    }

    pub fn collection_assign_state_mut(&mut self) -> Option<&mut CollectionAssignState> {
        self.collections.assign.as_mut()
    }

    pub fn collection_assign_state(&self) -> Option<&CollectionAssignState> {
        self.collections.assign.as_ref()
    }

    pub fn open_item_assign_for_selected_collection(&mut self) {
        if self.collections.open_item_assign(&self.items) {
            self.mode = Mode::CollectionAssign;
        }
    }

    pub fn open_collection_assign_for_selected(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let item_id = item.id;
        let collection_ids: HashSet<i32> = item.collections.iter().map(|c| c.id).collect();
        self.collections
            .open_collection_assign(item_id, collection_ids);
        self.mode = Mode::CollectionAssign;
    }

    pub fn collection_assign_next(&mut self) {
        self.collections.assign_next(self.items.len());
    }

    pub fn collection_assign_prev(&mut self) {
        self.collections.assign_prev(self.items.len());
    }

    pub fn collection_assign_toggle_current(&mut self) {
        self.collections.assign_toggle_current(&self.items);
    }

    pub fn cancel_collection_assign(&mut self) {
        self.collections.cancel_assign();
        self.mode = Mode::Normal;
    }

    pub async fn confirm_collection_assign(&mut self) -> Result<()> {
        self.collections.confirm_assign().await?;
        self.mode = Mode::Normal;
        self.request_refresh_item_list().await?;
        Ok(())
    }
}
