use color_eyre::Result;
use minastirith_core_derive::{Cyclable, Focusable};

use crate::{state::library::LibraryState, traits::Focusable};

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
    /// Index of the currently highlighted subscription row.
    #[must_use]
    pub fn selected_subscriptions_index(&self) -> Option<usize> {
        self.selected_subscriptions_index
    }

    /// Index of the currently highlighted item row.
    #[must_use]
    pub fn selected_item_index(&self) -> Option<usize> {
        self.selected_item_index
    }

    /// Mutable access to the currently highlighted subscription row index.
    pub fn selected_subscriptions_index_mut(&mut self) -> &mut Option<usize> {
        &mut self.selected_subscriptions_index
    }

    /// Mutable access to the currently highlighted item row index.
    pub fn selected_item_index_mut(&mut self) -> &mut Option<usize> {
        &mut self.selected_item_index
    }
}

impl LibraryState {
    /// The active browse state, if the browse view is open.
    #[must_use]
    pub fn get_browse_state(&self) -> Option<&LibraryBrowseState> {
        self.browse.as_ref()
    }

    /// Mutable access to the active browse state, if the browse view is
    /// open.
    pub fn get_browse_state_mut(&mut self) -> &mut Option<LibraryBrowseState> {
        &mut self.browse
    }

    /// Namespace id of the subscription currently highlighted in the
    /// browse view, if any.
    #[must_use]
    pub fn current_namespace(&self) -> Option<&str> {
        let s = self.browse.as_ref()?;
        let i = s.selected_subscriptions_index?;
        self.subscriptions
            .get(i)
            .map(|sub| sub.namespace_id.as_str())
    }

    /// Refresh the items for the selected namespace in the browse list
    /// # Errors
    /// Returns an error if fetching items from the shared library fails.
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

    /// Open the browse view, selecting the first subscription if any exist.
    pub fn open_browse(&mut self) {
        let mut browse = LibraryBrowseState::default();
        if !self.subscriptions.is_empty() {
            browse.selected_subscriptions_index_mut().replace(0);
        }
        self.browse = Some(browse);
    }

    /// Close the browse view.
    pub fn close_browse(&mut self) {
        self.browse = None;
    }

    /// Move the browse highlight to the next row in the focused pane,
    /// wrapping around. No-op if the browse view isn't open.
    pub fn browse_select_next(&mut self) {
        let Some(namespace) = self.current_namespace() else {
            return;
        };
        let items_len = self
            .browsed_items
            .get(namespace)
            .map_or(0, std::vec::Vec::len);
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

    /// Move the browse highlight to the previous row in the focused pane,
    /// wrapping around. No-op if the browse view isn't open.
    pub fn browse_select_prev(&mut self) {
        let Some(namespace) = self.current_namespace() else {
            return;
        };
        let items_len = self
            .browsed_items
            .get(namespace)
            .map_or(0, std::vec::Vec::len);
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

    /// Import the currently highlighted item in the browse view. No-op
    /// unless the items pane is focused and an item is highlighted.
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

        let Some(current_namespace) = self.current_namespace() else {
            return;
        };

        let Some(entry) = self
            .browsed_items
            .get(current_namespace)
            .and_then(|vec| vec.get(i).cloned())
        else {
            return;
        };
        self.request_import(entry);
    }
}
