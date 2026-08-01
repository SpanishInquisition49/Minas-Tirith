use std::collections::HashSet;

use color_eyre::Result;
use minastirith_core::{
    schema::collection::Collection, state::collection::collection_assign::CollectionAssignState,
};
use ratatui::widgets::ListState;
use tui_input::Input;

use crate::{
    traits::SelectableSync,
    tui::app::{App, Mode},
};

impl App {
    pub fn collections(&self) -> &[Collection] {
        self.collection_component.items()
    }

    pub async fn request_refresh_collections(&mut self) -> Result<()> {
        self.collection_component.refresh().await?;
        self.collection_component.sync();
        Ok(())
    }

    pub fn select_collection_prev(&mut self) {
        self.collection_component.select_prev();
    }

    pub fn select_collection_next(&mut self) {
        self.collection_component.select_next();
    }

    pub fn collection_list_state_mut(&mut self) -> &mut ListState {
        self.collection_component.list_state_mut()
    }

    pub async fn confirm_collection_selection(&mut self) -> Result<()> {
        self.collection_component.core_mut().confirm_selection();
        let collection_id = self.collection_component.selected().map(|c| c.id);
        self.items_component.refresh(collection_id).await?;
        Ok(())
    }

    pub async fn delete_collection(&mut self) -> Result<()> {
        if let Some(collection_id) = self
            .collection_component
            .core_mut()
            .delete_selected()
            .await?
        {
            self.request_refresh_collections().await?;
            self.items_component
                .items_mut()
                .iter_mut()
                .for_each(|i| i.collections.retain(|c| c.id != collection_id));
        }
        Ok(())
    }

    pub fn collection_input_field(&self) -> &Input {
        self.collection_component.name_input()
    }

    pub fn open_collection_create(&mut self) {
        self.collection_component.open_create();
        self.mode = Mode::CollectionCreate;
    }

    pub fn close_collection_create(&mut self) {
        self.mode = Mode::Normal;
    }

    pub async fn confirm_collection_create(&mut self) -> Result<()> {
        self.collection_component.confirm_create().await?;
        self.mode = Mode::Normal;
        Ok(())
    }

    pub fn collection_assign_state_mut(&mut self) -> &mut Option<CollectionAssignState> {
        self.collection_component.core_mut().assign_mut()
    }

    pub fn collection_assign_state(&self) -> &Option<CollectionAssignState> {
        self.collection_component.core().assign()
    }

    pub fn open_item_assign_for_selected_collection(&mut self) {
        if self
            .collection_component
            .core_mut()
            .open_item_assign(&self.items_component.items())
        {
            self.mode = Mode::CollectionAssign;
        }
    }

    pub fn open_collection_assign_for_selected(&mut self) {
        let Some(item) = self.items_component.core().selected_item() else {
            return;
        };
        let item_id = item.id;
        let collection_ids: HashSet<i32> = item.collections.iter().map(|c| c.id).collect();
        self.collection_component
            .core_mut()
            .open_collection_assign(item_id, collection_ids);
        self.mode = Mode::CollectionAssign;
    }

    pub fn collection_assign_next(&mut self) {
        self.collection_component
            .core_mut()
            .assign_next(self.items_component.items().len());
    }

    pub fn collection_assign_prev(&mut self) {
        self.collection_component
            .core_mut()
            .assign_prev(self.items_component.items().len());
    }

    pub fn collection_assign_toggle_current(&mut self) {
        self.collection_component
            .core_mut()
            .assign_toggle_current(&self.items_component.items());
    }

    pub fn cancel_collection_assign(&mut self) {
        self.collection_component.core_mut().cancel_assign();
        self.mode = Mode::Normal;
    }

    pub async fn confirm_collection_assign(&mut self) -> Result<()> {
        self.collection_component
            .core_mut()
            .confirm_assign()
            .await?;
        self.mode = Mode::Normal;
        let collection_id = self.collection_component.selected().map(|c| c.id);
        self.items_component.refresh(collection_id).await?;
        Ok(())
    }
}
