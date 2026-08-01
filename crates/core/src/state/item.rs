use std::{path::PathBuf, sync::Arc};

use color_eyre::eyre::Result;
use minastirith_core_derive::Selectable;

use crate::{
    database::Archive, metadata::common_metadata::ItemType, schema::item::DatabaseItem,
    traits::Selectable,
};

pub const TABS_LABELS: [&str; 6] = ["All", "Book", "Article", "Thesis", "Report", "Misc"];

#[derive(Selectable, Debug)]
#[select(items, selected_item)]
pub struct ItemState {
    archive: Arc<Archive>,
    items: Vec<DatabaseItem>,
    selected_item: Option<usize>,
    selected_tab: usize,
}

impl ItemState {
    pub fn new(archive: Arc<Archive>) -> Self {
        Self {
            archive,
            items: Vec::default(),
            selected_item: None,
            selected_tab: 0,
        }
    }

    pub fn items_mut(&mut self) -> &mut [DatabaseItem] {
        self.items.as_mut_slice()
    }

    pub async fn refresh(&mut self, collection_id: Option<i32>) -> Result<()> {
        let active_item_type = ItemType::try_from(TABS_LABELS[self.selected_tab]).ok();
        self.items = self
            .archive
            .get_items(active_item_type, collection_id)
            .await?;

        if self.selected_item.is_none() && !self.items.is_empty() {
            self.selected_index_mut().replace(0);
        }
        Ok(())
    }

    pub fn tabs_prev(&mut self) {
        self.selected_tab = if self.selected_tab == 0 {
            TABS_LABELS.len() - 1
        } else {
            self.selected_tab - 1
        }
    }

    pub fn tabs_next(&mut self) {
        self.selected_tab = if self.selected_tab + 1 == TABS_LABELS.len() {
            0
        } else {
            self.selected_tab + 1
        }
    }

    pub fn selected_tab(&self) -> usize {
        self.selected_tab
    }

    pub fn selected_item(&self) -> Option<&DatabaseItem> {
        self.items.get(self.selected_item?)
    }

    pub fn selected_item_mut(&mut self) -> Option<&mut DatabaseItem> {
        self.items.get_mut(self.selected_item?)
    }

    pub fn open_item(&self) -> Result<()> {
        let Some(item) = self.selected_item() else {
            return Ok(());
        };
        opener::open(PathBuf::from(&item.path))?;
        Ok(())
    }
}
