use std::sync::Arc;

use minastirith_core_derive::Selectable;

use crate::{database::Archive, schema::collection::Collection};

#[derive(Selectable)]
#[select(collections, selected_collection)]
pub struct CollectionState {
    archive: Arc<Archive>,
    pub collections: Vec<Collection>,
    pub selected_collection: Option<usize>,
    pub assign: Option<String>,
}
