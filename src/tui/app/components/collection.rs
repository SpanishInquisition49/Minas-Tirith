use std::{collections::HashSet, sync::Arc};

use color_eyre::eyre::Result;
use crossterm::event::Event;
use ratatui::widgets::ListState;
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{
    database::archive::Archive,
    schema::{collection::Collection, item::DatabaseItem},
    tui::app::traits::ListWidget,
};

#[derive(Clone, Copy, Debug)]
pub enum AssignMode {
    /// Assign items to the selected collection
    Items,
    /// Assign collections to the selected item
    Collections,
}

pub struct CollectionAssignState {
    pub(in crate::tui::app) mode: AssignMode,
    pub(in crate::tui::app) id: i32,
    pub(in crate::tui::app) original: HashSet<i32>,
    pub(in crate::tui::app) selected: HashSet<i32>,
    pub(in crate::tui::app) list_state: ListState,
}

impl CollectionAssignState {
    pub fn mode(&self) -> AssignMode {
        self.mode
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn selected(&self) -> &HashSet<i32> {
        &self.selected
    }

    pub fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }
}

pub struct CollectionState {
    archive: Arc<Archive>,
    selected: Option<i32>,
    pub items: Vec<Collection>,
    pub list_state: ListState,
    pub name_input: Input,
    pub assign: Option<CollectionAssignState>,
}

impl ListWidget<Collection> for CollectionState {
    fn items(&self) -> &[Collection] {
        self.items.as_ref()
    }

    fn list_state(&self) -> &ListState {
        &self.list_state
    }

    fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }
}

impl CollectionState {
    pub fn new(archive: Arc<Archive>) -> Self {
        Self {
            archive,
            selected: None,
            items: Vec::new(),
            list_state: ListState::default(),
            name_input: Input::default(),
            assign: None,
        }
    }

    pub async fn refresh(&mut self) -> Result<()> {
        self.items.clear();
        // NOTE: Element 0 is the trivial collection "All"
        self.items.push(Collection::trivial_collection());
        self.items.extend(self.archive.get_all_collections().await?);
        if self.list_state.selected().is_none() && !self.items.is_empty() {
            self.list_state.select(Some(0));
        }
        Ok(())
    }

    pub fn highlighted_id(&self) -> Option<i32> {
        let i = self.list_state.selected()?;
        self.items.get(i).map(|i| i.id)
    }

    pub fn confirm_selection(&mut self) {
        self.selected = self.highlighted_id();
    }

    pub async fn delete_highlighted(&mut self) -> Result<Option<i32>> {
        let Some(id) = self.highlighted_id() else {
            return Ok(None);
        };

        // NOTE: we don't need to delete anything when the "All" collection is selected
        if Collection::is_trivial_collection(id) {
            return Ok(None);
        }

        self.archive.delete_collection(id).await?;
        self.refresh().await?;
        Ok(Some(id))
    }

    pub fn open_create(&mut self) {
        self.name_input = Input::default();
    }

    pub async fn confirm_create(&mut self) -> Result<()> {
        let name = self.name_input.value().trim().to_string();
        if !name.is_empty() {
            self.archive.create_collection(&name).await?;
        }
        self.refresh().await?;
        Ok(())
    }

    pub fn open_item_assign(&mut self, items: &[DatabaseItem]) -> bool {
        let Some(id) = self.highlighted_id() else {
            return false;
        };

        if Collection::is_trivial_collection(id) {
            return false;
        }

        let original: HashSet<i32> = items
            .iter()
            .filter_map(|i| {
                if i.collections.iter().any(|c| c.id == id) {
                    Some(i.id)
                } else {
                    None
                }
            })
            .collect();
        let selected = original.clone();
        let mut list_state = ListState::default();
        if !items.is_empty() {
            list_state.select(Some(0));
        }
        self.assign = Some(CollectionAssignState {
            mode: AssignMode::Items,
            id,
            original,
            selected,
            list_state,
        });
        true
    }

    pub fn open_collection_assign(&mut self, item_id: i32, original: HashSet<i32>) {
        let selected = original.clone();
        let mut list_state = ListState::default();
        if !self.items.is_empty() {
            list_state.select(Some(0));
        }
        self.assign = Some(CollectionAssignState {
            mode: AssignMode::Collections,
            id: item_id,
            original,
            selected,
            list_state,
        });
    }

    pub fn assign_next(&mut self, items_len: usize) {
        let Some(state) = &mut self.assign else {
            return;
        };
        let len = match state.mode {
            AssignMode::Items => items_len,
            AssignMode::Collections => self.items.len() - 1, // NOTE: ignore the "All" collection
        };
        if len == 0 {
            return;
        }
        let i = match state.list_state.selected() {
            Some(i) if i + 1 < len => i + 1,
            _ => 0,
        };
        state.list_state.select(Some(i));
    }

    pub fn assign_prev(&mut self, items_len: usize) {
        let Some(state) = &mut self.assign else {
            return;
        };
        let len = match state.mode {
            AssignMode::Items => items_len,
            AssignMode::Collections => self.items.len() - 1, // NOTE: ignore the "All" collection
        };
        if len == 0 {
            return;
        }
        let i = match state.list_state.selected() {
            Some(i) if i > 0 => i - 1,
            _ => len - 1,
        };
        state.list_state.select(Some(i));
    }

    pub fn assign_toggle_current(&mut self, items: &[DatabaseItem]) {
        let Some(state) = &mut self.assign else {
            return;
        };
        let Some(index) = state.list_state.selected() else {
            return;
        };

        let id = match state.mode {
            AssignMode::Items => {
                let Some(id) = items.get(index).map(|i| i.id) else {
                    return;
                };
                id
            }
            AssignMode::Collections => {
                let Some(id) = self.items.get(index + 1).map(|c| c.id) else {
                    return;
                };
                id
            }
        };
        if !state.selected.remove(&id) {
            state.selected.insert(id);
        }
    }

    pub fn cancel_assign(&mut self) {
        self.assign = None
    }

    pub async fn confirm_assign(&mut self) -> Result<()> {
        let Some(state) = &self.assign else {
            return Ok(());
        };

        match state.mode {
            AssignMode::Items => {
                for &item_id in state.selected.difference(&state.original) {
                    self.archive
                        .add_item_to_collection(item_id, state.id)
                        .await?;
                }
                for &item_id in state.original.difference(&state.selected) {
                    self.archive
                        .remove_item_from_collection(item_id, state.id)
                        .await?;
                }
            }
            AssignMode::Collections => {
                for &collection_id in state.selected.difference(&state.original) {
                    self.archive
                        .add_item_to_collection(state.id, collection_id)
                        .await?;
                }
                for &collection_id in state.original.difference(&state.selected) {
                    self.archive
                        .remove_item_from_collection(state.id, collection_id)
                        .await?;
                }
            }
        }
        Ok(())
    }

    pub fn handle_event(&mut self, event: &Event) {
        self.name_input.handle_event(event);
    }
}
