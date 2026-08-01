use color_eyre::Result;
use minastirith_core_derive::{Cyclable, Focusable};

use crate::{peer2peer::library::SharedItemEntry, state::library::LibraryState, traits::Focusable};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Cyclable, Default)]
pub enum BrowseFocus {
    #[default]
    Subscriptions,
    Items,
}

#[derive(Focusable, Default)]
#[focus(focus)]
pub struct LibraryBrowseState {
    selected_subscriptions_index: Option<usize>,
    selected_item_index: Option<usize>,
    focus: BrowseFocus,
}

impl LibraryBrowseState {
    pub fn selected_subscriptions_index(&self) -> Option<usize> {
        self.selected_subscriptions_index
    }

    pub fn selected_item_index(&self) -> Option<usize> {
        self.selected_item_index
    }

    pub fn selected_subscriptions_index_mut(&mut self) -> &mut Option<usize> {
        &mut self.selected_subscriptions_index
    }

    pub fn selected_item_index_mut(&mut self) -> &mut Option<usize> {
        &mut self.selected_item_index
    }
}

impl LibraryState {
    pub fn get_browse_state(&self) -> &Option<LibraryBrowseState> {
        &self.browse
    }

    pub fn get_browse_state_mut(&mut self) -> &mut Option<LibraryBrowseState> {
        &mut self.browse
    }

    pub fn current_namespace(&self) -> Option<&str> {
        let s = self.browse.as_ref()?;
        let i = s.selected_item_index?;
        self.subscriptions
            .get(i)
            .map(|sub| sub.namespace_id.as_str())
    }

    pub fn current_items(&self) -> &[SharedItemEntry] {
        todo!()
    }

    /// Refresh the item for the selected namespace in the browse list
    pub async fn refresh_current(&mut self) -> Result<()> {
        let Some(namespace) = self.current_namespace().map(str::to_string) else {
            return Ok(());
        };
        let Some(doc) = self.open_docs.get(&namespace) else {
            return Ok(());
        };
        let items = self.share_node.list_items(doc).await?;
        self.browsed_items.insert(namespace, items);

        Ok(())
    }

    pub fn open_browse(&mut self) {
        let mut browse = LibraryBrowseState::default();
        if !self.subscriptions.is_empty() {
            browse.selected_item_index_mut().replace(0);
        }
        self.browse = Some(browse);
    }

    pub fn close_browse(&mut self) {
        self.browse = None;
    }

    pub fn browse_select_next(&mut self) {
        let items_len = self.current_items().len();
        let Some(browse) = &mut self.browse else {
            return;
        };

        let len = match browse.current_focus() {
            BrowseFocus::Subscriptions => self.subscriptions.len(),
            BrowseFocus::Items => items_len,
        };

        if len == 0 {
            return;
        }

        match browse.current_focus() {
            BrowseFocus::Subscriptions => {
                let i = match browse.selected_subscriptions_index() {
                    Some(i) if i + 1 < len => i + 1,
                    _ => 0,
                };
                browse.selected_subscriptions_index_mut().replace(i);
            }
            BrowseFocus::Items => {
                let i = match browse.selected_item_index() {
                    Some(i) if i + 1 < len => i + 1,
                    _ => 0,
                };
                browse.selected_item_index_mut().replace(i);
            }
        }
    }

    pub fn browse_select_prev(&mut self) {
        let items_len = self.current_items().len();
        let Some(browse) = &mut self.browse else {
            return;
        };

        let len = match browse.current_focus() {
            BrowseFocus::Subscriptions => self.subscriptions.len(),
            BrowseFocus::Items => items_len,
        };

        if len == 0 {
            return;
        }

        match browse.current_focus() {
            BrowseFocus::Subscriptions => {
                let i = match browse.selected_subscriptions_index() {
                    Some(i) if i > 0 => i - 1,
                    _ => len - 1,
                };
                browse.selected_subscriptions_index_mut().replace(i);
            }
            BrowseFocus::Items => {
                let i = match browse.selected_item_index() {
                    Some(i) if i > 0 => i - 1,
                    _ => len - 1,
                };
                browse.selected_item_index_mut().replace(i);
            }
        }
    }

    pub fn browse_confirm_import(&mut self) {
        let Some(browse) = &self.browse else {
            return;
        };
        if browse.current_focus() != BrowseFocus::Items {
            return;
        }

        let Some(i) = browse.selected_item_index() else {
            return;
        };

        let Some(entry) = self.current_items().get(i).cloned() else {
            return;
        };
        self.request_import(entry);
    }
}
