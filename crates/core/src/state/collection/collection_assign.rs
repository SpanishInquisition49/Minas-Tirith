use color_eyre::Result;
use std::{
    collections::{HashSet, hash_set::Difference},
    hash::RandomState,
};

use crate::{
    schema::{collection::Collection, item::DatabaseItem},
    state::collection::CollectionState,
};

#[derive(Clone, Copy, Debug)]
pub enum AssignMode {
    /// Assign items to the selected collection
    Items,
    /// Assign collections to the selected item
    Collections,
}

pub struct CollectionAssignState {
    mode: AssignMode,
    id: i32,
    original: HashSet<i32>,
    selected: HashSet<i32>,
    selected_index: Option<usize>,
}

impl CollectionAssignState {
    /// Start an assignment session in `mode` for the entity identified by
    /// `id`, with `original` as the set of currently associated ids.
    #[must_use]
    pub fn new(mode: AssignMode, id: i32, original: HashSet<i32>) -> Self {
        let selected = original.clone();
        Self {
            mode,
            id,
            original,
            selected,
            selected_index: Some(0),
        }
    }

    /// Whether this session assigns items to a collection or collections
    /// to an item.
    #[must_use]
    pub fn mode(&self) -> AssignMode {
        self.mode
    }

    /// Id of the collection or item this session is assigning to.
    #[must_use]
    pub fn id(&self) -> i32 {
        self.id
    }

    /// Ids currently selected in this assignment session.
    #[must_use]
    pub fn selected(&self) -> &HashSet<i32> {
        &self.selected
    }

    /// Mutable access to the ids currently selected in this assignment
    /// session.
    pub fn selected_mut(&mut self) -> &mut HashSet<i32> {
        &mut self.selected
    }

    /// Ids that are newly selected and need to be added.
    pub fn add_list(&self) -> Difference<'_, i32, RandomState> {
        self.selected.difference(&self.original)
    }

    /// Ids that were originally selected and need to be removed.
    pub fn remove_list(&self) -> Difference<'_, i32, RandomState> {
        self.original.difference(&self.selected)
    }

    /// Index of the currently highlighted row in the assignment list.
    #[must_use]
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    /// Mutable access to the index of the currently highlighted row in the
    /// assignment list.
    pub fn selected_index_mut(&mut self) -> &mut Option<usize> {
        &mut self.selected_index
    }
}

impl CollectionState {
    /// Start an item-assignment session for the currently selected
    /// collection. Returns `false` if `items` is empty, no collection is
    /// selected, or the trivial "All" collection is selected.
    pub fn open_item_assign(&mut self, items: &[DatabaseItem]) -> bool {
        if items.is_empty() {
            return false;
        }
        let Some(id) = self.selected_collection_id() else {
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
        self.assign = Some(CollectionAssignState::new(AssignMode::Items, id, original));
        true
    }

    /// Start a collection-assignment session for the item identified by
    /// `item_id`, with `original` as its currently assigned collection ids.
    pub fn open_collection_assign(&mut self, item_id: i32, original: HashSet<i32>) {
        self.assign = Some(CollectionAssignState::new(
            AssignMode::Collections,
            item_id,
            original,
        ));
    }

    /// Move the assignment highlight to the next row, wrapping around.
    /// No-op if no assignment session is active.
    pub fn assign_next(&mut self, items_len: usize) {
        let Some(assign) = self.assign.as_mut() else {
            return;
        };
        let len = match assign.mode() {
            AssignMode::Items => items_len,
            // NOTE: ignore the "All" collection
            AssignMode::Collections => self.collections.len() - 1,
        };
        if len == 0 {
            return;
        }
        let i = match assign.selected_index() {
            Some(i) if i + 1 < len => i + 1,
            _ => 0,
        };
        assign.selected_index_mut().replace(i);
    }

    /// Move the assignment highlight to the previous row, wrapping around.
    /// No-op if no assignment session is active.
    pub fn assign_prev(&mut self, items_len: usize) {
        let Some(assign) = self.assign.as_mut() else {
            return;
        };
        let len = match assign.mode() {
            AssignMode::Items => items_len,
            // NOTE: ignore the "All" collection
            AssignMode::Collections => self.collections.len() - 1,
        };
        if len == 0 {
            return;
        }
        let i = match assign.selected_index() {
            Some(i) if i > 0 => i - 1,
            _ => len - 1,
        };
        assign.selected_index_mut().replace(i);
    }

    /// Toggle selection of the currently highlighted row in the active
    /// assignment session.
    pub fn assign_toggle_current(&mut self, items: &[DatabaseItem]) {
        let Some(assign) = self.assign.as_mut() else {
            return;
        };
        let Some(index) = assign.selected_index() else {
            return;
        };

        let id = match assign.mode() {
            AssignMode::Items => {
                let Some(id) = items.get(index).map(|i| i.id) else {
                    return;
                };
                id
            }
            AssignMode::Collections => {
                let Some(id) = self.collections.get(index + 1).map(|c| c.id) else {
                    return;
                };
                id
            }
        };

        if !assign.selected_mut().remove(&id) {
            assign.selected_mut().insert(id);
        }
    }

    /// Discard the active assignment session, if any.
    pub fn cancel_assign(&mut self) {
        self.assign = None;
    }

    /// Commit the active assignment session, adding/removing associations
    /// as needed.
    /// # Errors
    /// Returns an error if any add/remove operation against the archive
    /// fails.
    pub async fn confirm_assign(&mut self) -> Result<()> {
        let Some(assign) = self.assign.take() else {
            return Ok(());
        };

        match assign.mode() {
            AssignMode::Items => {
                for item_id in assign.add_list() {
                    self.archive
                        .add_item_to_collection(*item_id, assign.id())
                        .await?;
                }
                for item_id in assign.remove_list() {
                    self.archive
                        .remove_item_from_collection(*item_id, assign.id())
                        .await?;
                }
            }
            AssignMode::Collections => {
                for collection_id in assign.add_list() {
                    self.archive
                        .add_item_to_collection(assign.id(), *collection_id)
                        .await?;
                }

                for collection_id in assign.remove_list() {
                    self.archive
                        .remove_item_from_collection(assign.id(), *collection_id)
                        .await?;
                }
            }
        }
        Ok(())
    }
}
