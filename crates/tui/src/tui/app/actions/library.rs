use color_eyre::Result;
use ratatui_notifications::Level;
use std::path::PathBuf;

use minastirith_core::{
    database::query::ast::{Expr, Field, Op, Value},
    metadata::shared_library::{LibrarySubscription, SharedLibrary},
    peer2peer::library::SharedItemEntry,
    schema::collection::Collection,
    state::{
        library::{
            browse::{BrowseFocus, LibraryBrowseState},
            manage::LibraryManageState,
            publish::LibraryPublishState,
            subscribe::LibrarySubscribeState,
        },
        metadata::EditContext,
    },
    traits::{Focusable, MetadataForm},
};
use ratatui::widgets::ListState;
use tui_input::Input;

use crate::{
    schema::tui_metadata_form::TuiMetadataForm,
    tui::app::{App, Mode},
};

impl App {
    /// The active publish dialog state, if it's open.
    pub fn get_publish_state(&self) -> Option<&LibraryPublishState> {
        self.library_component.core().get_publish_state()
    }

    /// The active subscribe dialog state, if it's open.
    pub fn get_subscribe_state(&self) -> Option<&LibrarySubscribeState> {
        self.library_component.core().get_subscribe_state()
    }

    /// Mutable access to the active manage-libraries state, if it's open.
    pub fn get_library_manage_state_mut(&mut self) -> Option<&mut LibraryManageState> {
        self.library_component.core_mut().get_manage_state_mut()
    }

    /// The active browse state, if the browse view is open.
    pub fn get_library_browse_state_mut(&mut self) -> Option<&LibraryBrowseState> {
        self.library_component.core_mut().get_browse_state()
    }

    /// The active browse state, if the browse view is open.
    pub fn get_browse_state(&self) -> Option<&LibraryBrowseState> {
        self.library_component.core().get_browse_state()
    }

    /// The input widget for the publish dialog's collection-name field.
    pub fn collection_name_input(&self) -> &Input {
        self.library_component.collection_name()
    }

    /// The input widget for the publish dialog's collection-description
    /// field.
    pub fn collection_description_input(&self) -> &Input {
        self.library_component.collection_description()
    }

    /// The input widget for the subscribe dialog's ticket field.
    pub fn subscribe_ticket(&self) -> &Input {
        self.library_component.subscribe_ticket()
    }

    /// The input widget for the subscribe dialog's nickname field.
    pub fn subscribe_nickname(&self) -> &Input {
        self.library_component.subscribe_nickname()
    }

    /// The locally known library subscriptions.
    pub fn get_subscriptions(&self) -> &[LibrarySubscription] {
        self.library_component.core().subscriptions()
    }

    /// The libraries published from this node.
    pub fn get_shared_libraries(&self) -> &[SharedLibrary] {
        self.library_component.core().shared_libraries()
    }

    /// Items browsed so far for the currently selected subscription
    /// namespace.
    pub fn current_library_items(&self) -> &[SharedItemEntry] {
        self.library_component.core().current_library_entries()
    }

    /// Mutable access to the subscription list's ratatui `ListState`.
    pub fn subscription_list_state_mut(&mut self) -> &mut ListState {
        self.library_component.subscription_list_state_mut()
    }

    /// Mutable access to the manage-libraries list's ratatui `ListState`.
    pub fn manage_list_state_mut(&mut self) -> &mut ListState {
        self.library_component.manage_list_state_mut()
    }

    /// Mutable access to the browse-items list's ratatui `ListState`.
    pub fn browse_list_state_mut(&mut self) -> &mut ListState {
        self.library_component.browse_list_state_mut()
    }

    /// Open the library browse view and sync its selection into
    /// rendering state.
    pub fn open_library_browse(&mut self) {
        self.library_component.core_mut().open_browse();
        self.sync_browse_selection();
        self.mode = Mode::LibraryBrowse;
    }

    /// Open the manage-libraries view and sync its selection into
    /// rendering state.
    pub fn open_library_manage(&mut self) {
        self.library_component.core_mut().open_manage();
        self.sync_manage_selection();
        self.mode = Mode::LibraryManage;
    }

    /// Push the browse state's current indices into the `ListState`s that
    /// actually drive the highlighted row — `browse_select_next`/`prev`
    /// only update the core index, they don't touch rendering state.
    pub fn sync_browse_selection(&mut self) {
        let (sub_index, item_index) = match self.library_component.core().get_browse_state() {
            Some(b) => (b.selected_subscriptions_index(), b.selected_item_index()),
            None => (None, None),
        };
        self.library_component
            .subscription_list_state_mut()
            .select(sub_index);
        self.library_component
            .browse_list_state_mut()
            .select(item_index);
    }

    /// Same as `sync_browse_selection`, for the manage-libraries list.
    pub fn sync_manage_selection(&mut self) {
        let index = self
            .library_component
            .core()
            .get_manage_state()
            .as_ref()
            .and_then(|m| m.selected_library_index());
        self.library_component.manage_list_state_mut().select(index);
    }

    /// Open the metadata form pre-filled from a downloaded shared item,
    /// ready to be saved as a new local item.
    pub fn handle_item_download_ready(&mut self, entry: &SharedItemEntry, local_path: PathBuf) {
        let form = TuiMetadataForm::from_candidate(entry);
        self.metadata_component
            .core_mut()
            .set_edit(form, EditContext::NewItem { path: local_path });
        self.mode = Mode::MetadataEdit;
    }

    /// Open the publish dialog for the currently selected collection.
    /// No-op if no collection (or the trivial "All" collection) is
    /// selected.
    pub fn open_library_publish_for_selected(&mut self) {
        let Some(id) = self.collection_component.selected().map(|c| c.id) else {
            return;
        };
        if Collection::is_trivial_collection(id) {
            return;
        }
        let Some(name) = self.collection_component.items().iter().find_map(|c| {
            if c.id == id {
                Some(c.name.clone())
            } else {
                None
            }
        }) else {
            return;
        };
        self.library_component.core_mut().open_publish(id, name);
        self.mode = Mode::LibraryPublish;
    }

    /// Unsubscribe from the subscription currently highlighted in the
    /// browse view. No-op unless the subscriptions pane is focused and a
    /// subscription is highlighted.
    /// # Errors
    /// Returns an error if the underlying unsubscribe fails.
    pub async fn unsubscribe_selected_library(&mut self) -> Result<()> {
        let Some(browse) = &self.library_component.core().get_browse_state() else {
            return Ok(());
        };
        if browse.current_focus() != BrowseFocus::Subscriptions {
            return Ok(());
        }
        let Some(i) = browse.selected_subscriptions_index() else {
            return Ok(());
        };
        let Some(namespace_id) = self
            .library_component
            .core()
            .subscriptions()
            .get(i)
            .map(|s| s.namespace_id.clone())
        else {
            return Ok(());
        };

        self.library_component
            .core_mut()
            .unsubscribe(&namespace_id)
            .await?;

        let len = self.library_component.core().subscriptions().len();
        if let Some(browse) = self.library_component.core_mut().get_browse_state_mut() {
            *browse.selected_subscriptions_index_mut() =
                if len == 0 { None } else { Some(i.min(len - 1)) };
            *browse.selected_item_index_mut() = None;
        }
        self.sync_browse_selection();

        self.notify(
            "Subscription removed",
            " Shared libraries ".to_string(),
            Level::Info,
        );
        Ok(())
    }

    /// Subscribe using the subscribe dialog's current ticket/nickname
    /// input, switching to the browse view on success.
    /// # Errors
    /// Returns an error if refreshing the newly subscribed library's
    /// items fails; subscribe failures themselves are shown as a
    /// notification.
    pub async fn confirm_library_subscribe(&mut self) -> Result<()> {
        let ticket = self
            .library_component
            .subscribe_ticket()
            .value()
            .trim()
            .to_string();
        let nickname = self
            .library_component
            .subscribe_nickname()
            .value()
            .trim()
            .to_string();
        let ok = self
            .library_component
            .core_mut()
            .confirm_subscribe(ticket, nickname)
            .await?;
        if ok {
            self.mode = Mode::LibraryBrowse;
            self.notify(
                "Subscribed to library",
                " Shared libraries ".to_string(),
                Level::Info,
            );
        }
        self.library_component.core_mut().refresh_current().await?;
        Ok(())
    }

    /// Publish the currently selected collection as a shared library,
    /// using the publish dialog's current name/description input.
    /// # Errors
    /// Returns an error if fetching the collection's items fails.
    pub async fn publish_collection_as_library(&mut self) -> Result<()> {
        let Some(collection) = self.collection_component.selected() else {
            return Ok(());
        };

        let name = self
            .library_component
            .collection_name()
            .value()
            .trim()
            .to_string();
        let description = self
            .library_component
            .collection_description()
            .value()
            .trim()
            .to_string();
        let description = if description.is_empty() {
            None
        } else {
            Some(description)
        };
        let filter = (!Collection::is_trivial_collection(collection.id)).then(|| Expr::Compare {
            field: Field::Collection,
            op: Op::Eq,
            value: Value::Single(collection.name.clone()),
        });
        let items_in_collection = self.archive.get_items(filter.as_ref()).await?;

        if let Some(name) = self
            .library_component
            .core_mut()
            .confirm_publish(name, description, items_in_collection.as_slice())
            .await?
        {
            self.notify(
                format!("Librery '{name}' published"),
                " Sharing ".to_string(),
                Level::Info,
            );
        }
        // TODO: handle error

        Ok(())
    }
}
