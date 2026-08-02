pub mod collection_assign;

use color_eyre::Result;
use std::sync::Arc;

use minastirith_core_derive::Selectable;

use crate::{
    database::Archive, schema::collection::Collection,
    state::collection::collection_assign::CollectionAssignState,
};

#[derive(Selectable)]
#[select(collections, collection_selected_index)]
pub struct CollectionState {
    archive: Arc<Archive>,
    collections: Vec<Collection>,
    collection_selected_index: Option<usize>,
    assign: Option<CollectionAssignState>,
    selected: Option<i32>,
}

impl CollectionState {
    /// Construct a [`CollectionState`] backed by `archive`, with no
    /// collections loaded yet.
    #[must_use]
    pub fn new(archive: Arc<Archive>) -> Self {
        Self {
            archive,
            collections: Vec::new(),
            collection_selected_index: None,
            assign: None,
            selected: None,
        }
    }

    /// Create a collection named `collection_name`.
    /// # Errors
    /// Returns an error if the collection cannot be created.
    pub async fn create_collection(&self, collection_name: &str) -> Result<()> {
        self.archive.create_collection(collection_name).await?;
        Ok(())
    }

    /// Reload all collections from `archive`, prepending the trivial "All"
    /// collection. Selects the first entry if none was selected yet.
    /// # Errors
    /// Returns an error if fetching collections from the archive fails.
    pub async fn refresh(&mut self) -> Result<()> {
        self.collections.clear();
        self.collections.push(Collection::trivial_collection());
        self.collections
            .extend(self.archive.get_all_collections().await?);
        if self.collection_selected_index.is_none() && !self.collections.is_empty() {
            self.collection_selected_index.replace(0);
        }
        Ok(())
    }

    fn selected_collection_id(&self) -> Option<i32> {
        self.collections
            .get(self.collection_selected_index?)
            .map(|c| c.id)
    }

    /// Commit the currently highlighted collection as the active selection.
    pub fn confirm_selection(&mut self) {
        self.selected = self.selected_collection_id();
    }

    /// Delete the currently selected collection, unless it is the trivial
    /// "All" collection. Returns the deleted collection's id, or `None` if
    /// nothing was deleted.
    /// # Errors
    /// Returns an error if the delete fails.
    pub async fn delete_selected(&mut self) -> Result<Option<i32>> {
        let Some(id) = self.selected_collection_id() else {
            return Ok(None);
        };

        if Collection::is_trivial_collection(id) {
            return Ok(None);
        }

        self.archive.delete_collection(id).await?;

        Ok(Some(id))
    }

    /// The active item/collection assignment state, if an assignment is in
    /// progress.
    #[must_use]
    pub fn assign(&self) -> Option<&CollectionAssignState> {
        self.assign.as_ref()
    }

    /// Mutable access to the active item/collection assignment state, if an
    /// assignment is in progress.
    pub fn assign_mut(&mut self) -> Option<&mut CollectionAssignState> {
        self.assign.as_mut()
    }
}
