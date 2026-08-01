use std::{path::PathBuf, sync::Arc};

use color_eyre::eyre::Result;
use minastirith_core_derive::Selectable;

use crate::{
    database::{
        Archive,
        query::{
            ast::{Expr, Field, Op, Value},
            parser,
        },
    },
    metadata::common_metadata::ItemType,
    schema::{collection::Collection, item::DatabaseItem},
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
    search_query: Option<Expr>,
}

impl ItemState {
    pub fn new(archive: Arc<Archive>) -> Self {
        Self {
            archive,
            items: Vec::default(),
            selected_item: None,
            selected_tab: 0,
            search_query: None,
        }
    }

    pub fn items_mut(&mut self) -> &mut [DatabaseItem] {
        self.items.as_mut_slice()
    }

    /// Parse `raw` and, on success, make it the active filter,
    /// overriding tab/collection, whose selectors reset to "All"
    /// *Note:* resetting the collection sidebar is a UI-layer concern.
    /// An empty `raw` clears the search and falls back to tab/collection filtering.
    /// Return `true` if the filter is applied, `false` otherwise
    pub fn set_search(&mut self, raw: &str) -> Result<bool> {
        let raw = raw.trim();
        if raw.is_empty() {
            self.search_query = None;
            return Ok(false);
        }
        self.search_query = Some(parser::parse(raw)?);
        self.selected_tab = 0;
        Ok(true)
    }

    fn resolve_filter(&self, collection: Option<&Collection>) -> Option<Expr> {
        if let Some(search) = &self.search_query {
            return Some(search.clone());
        }

        let type_expr = ItemType::try_from(TABS_LABELS[self.selected_tab])
            .ok()
            .map(|t| Expr::Compare {
                field: Field::Type,
                op: Op::Eq,
                value: Value::Single(t.to_string()),
            });

        let collection_expr = collection
            .filter(|c| !Collection::is_trivial_collection(c.id))
            .map(|c| Expr::Compare {
                field: Field::Collection,
                op: Op::Eq,
                value: Value::Single(c.name.clone()),
            });

        match (type_expr, collection_expr) {
            (None, None) => None,
            (None, Some(c)) => Some(c),
            (Some(t), None) => Some(t),
            (Some(t), Some(c)) => Some(Expr::And(Box::new(t), Box::new(c))),
        }
    }

    pub async fn refresh(&mut self, collection: Option<&Collection>) -> Result<()> {
        let filter = self.resolve_filter(collection);
        self.items = self.archive.get_items(filter.as_ref()).await?;
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
