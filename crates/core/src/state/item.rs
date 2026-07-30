use std::sync::Arc;

use minastirith_core_derive::Selectable;

use crate::{database::Archive, schema::item::DatabaseItem};

#[derive(Selectable)]
#[select(items, selected_item)]
pub struct ItemState {
    archive: Arc<Archive>,
    pub items: Vec<DatabaseItem>,
    pub selected_item: Option<usize>,
}
