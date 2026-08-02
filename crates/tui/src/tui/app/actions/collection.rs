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
    /// All collections currently loaded in the collection list.
    pub fn collections(&self) -> &[Collection] {
        self.collection_component.items()
    }

    /// Reload the collection list and sync dependent UI state.
    /// # Errors
    /// Returns an error if refreshing collections fails.
    pub async fn request_refresh_collections(&mut self) -> Result<()> {
        self.collection_component.refresh().await?;
        self.collection_component.sync();
        Ok(())
    }

    /// Select the previous collection.
    pub fn select_collection_prev(&mut self) {
        self.collection_component.select_prev();
    }

    /// Select the next collection.
    pub fn select_collection_next(&mut self) {
        self.collection_component.select_next();
    }

    /// Mutable access to the collection list's ratatui `ListState`.
    pub fn collection_list_state_mut(&mut self) -> &mut ListState {
        self.collection_component.list_state_mut()
    }

    /// Commit the highlighted collection as the active selection and
    /// refresh the item list for it.
    /// # Errors
    /// Returns an error if refreshing the item list fails.
    pub async fn confirm_collection_selection(&mut self) -> Result<()> {
        self.collection_component.core_mut().confirm_selection();
        let collection = self.collection_component.selected();
        self.items_component.refresh(collection).await?;
        Ok(())
    }

    /// Delete the currently selected collection and remove it from any
    /// items that referenced it locally.
    /// # Errors
    /// Returns an error if deleting the collection or refreshing the
    /// collection list fails.
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

    /// The input widget for the new-collection name field.
    pub fn collection_input_field(&self) -> &Input {
        self.collection_component.name_input()
    }

    /// Open the collection-creation dialog.
    pub fn open_collection_create(&mut self) {
        self.collection_component.open_create();
        self.mode = Mode::CollectionCreate;
    }

    /// Close the collection-creation dialog without creating a
    /// collection.
    pub fn close_collection_create(&mut self) {
        self.mode = Mode::Normal;
    }

    /// Create the collection from the create dialog's current input and
    /// return to normal mode.
    /// # Errors
    /// Returns an error if the collection cannot be created.
    pub async fn confirm_collection_create(&mut self) -> Result<()> {
        self.collection_component.confirm_create().await?;
        self.mode = Mode::Normal;
        Ok(())
    }

    /// Mutable access to the active item/collection assignment state, if
    /// an assignment is in progress.
    pub fn collection_assign_state_mut(&mut self) -> Option<&mut CollectionAssignState> {
        self.collection_component.core_mut().assign_mut()
    }

    /// The active item/collection assignment state, if an assignment is
    /// in progress.
    pub fn collection_assign_state(&self) -> Option<&CollectionAssignState> {
        self.collection_component.core().assign()
    }

    /// Start assigning items to the currently selected collection,
    /// switching to collection-assign mode.
    pub fn open_item_assign_for_selected_collection(&mut self) {
        if self
            .collection_component
            .core_mut()
            .open_item_assign(self.items_component.items())
        {
            self.mode = Mode::CollectionAssign;
        }
    }

    /// Start assigning collections to the currently selected item,
    /// switching to collection-assign mode.
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

    /// Move the assignment highlight to the next row.
    pub fn collection_assign_next(&mut self) {
        self.collection_component
            .core_mut()
            .assign_next(self.items_component.items().len());
    }

    /// Move the assignment highlight to the previous row.
    pub fn collection_assign_prev(&mut self) {
        self.collection_component
            .core_mut()
            .assign_prev(self.items_component.items().len());
    }

    /// Toggle selection of the currently highlighted row in the active
    /// assignment session.
    pub fn collection_assign_toggle_current(&mut self) {
        self.collection_component
            .core_mut()
            .assign_toggle_current(self.items_component.items());
    }

    /// Discard the active assignment session and return to normal mode.
    pub fn cancel_collection_assign(&mut self) {
        self.collection_component.core_mut().cancel_assign();
        self.mode = Mode::Normal;
    }

    /// Commit the active assignment session and refresh the item list.
    /// # Errors
    /// Returns an error if committing the assignment or refreshing the
    /// item list fails.
    pub async fn confirm_collection_assign(&mut self) -> Result<()> {
        self.collection_component
            .core_mut()
            .confirm_assign()
            .await?;
        self.mode = Mode::Normal;
        let collection = self.collection_component.selected();
        self.items_component.refresh(collection).await?;
        Ok(())
    }
}
