use std::path::PathBuf;

use color_eyre::Result;
use ratatui::widgets::ListState;
use ratatui_notifications::Level;

use crate::{
    metadata::shared_library::{LibrarySubscription, SharedLibrary},
    peer2peer::library::SharedItemEntry,
    schema::{collection::Collection, form::MetadataForm, item::DatabaseItem},
    tui::app::{
        App, Mode,
        components::{
            library::{
                BrowseFocus, LibraryBrowseState, LibraryManageState, LibraryPublishState,
                LibrarySubscribeState,
            },
            metadata_edit::EditContext,
        },
        traits::ListWidget,
    },
};

impl App {
    pub async fn publish_collection_as_library(&mut self) -> Result<()> {
        let Some(collection) = self.collections.selected() else {
            return Ok(());
        };

        let items_in_collection: Vec<&DatabaseItem> = self
            .items
            .iter()
            .filter(|i| i.collections.iter().any(|c| c.id == collection.id))
            .collect();

        match self.library.confirm_publish(&items_in_collection).await? {
            Some(name) => {
                self.notify(
                    format!("Librery '{name}' published"),
                    " Sharing ".to_string(),
                    Level::Info,
                );
            }
            // TODO: handle error
            None => {}
        }

        Ok(())
    }

    pub fn current_library_items(&self) -> &[SharedItemEntry] {
        self.library.current_items()
    }

    pub fn get_shared_libraries(&self) -> &[SharedLibrary] {
        &self.library.shared_libraries
    }

    pub async fn subscribe_to_library(
        &mut self,
        ticket_str: String,
        nickname: String,
    ) -> Result<()> {
        self.library.subscribe(ticket_str, nickname).await
    }

    pub fn request_import_paper(&mut self, entry: SharedItemEntry) {
        self.library.request_import(entry);
    }

    pub fn handle_item_download_ready(&mut self, entry: SharedItemEntry, local_path: PathBuf) {
        let form = MetadataForm::from_candidate(&entry);
        self.metadata
            .set_edit(form, EditContext::NewItem { path: local_path });
        self.mode = Mode::MetadataEdit;
    }

    pub fn open_library_publish_for_selected(&mut self) {
        let Some(id) = self.collections.highlighted_id() else {
            return;
        };
        if Collection::is_trivial_collection(id) {
            return;
        }
        let Some(name) = self.collections.items.iter().find_map(|c| {
            if c.id == id {
                Some(c.name.clone())
            } else {
                None
            }
        }) else {
            return;
        };
        self.library.open_publish(id, name);
        self.mode = Mode::LibraryPublish;
    }

    pub fn get_publish_state(&self) -> Option<&LibraryPublishState> {
        self.library.publish.as_ref()
    }
    pub fn get_publish_state_mut(&mut self) -> Option<&mut LibraryPublishState> {
        self.library.publish.as_mut()
    }

    pub async fn confirm_library_publish(&mut self) -> Result<()> {
        let Some(s) = &self.library.publish else {
            return Ok(());
        };
        let collection_id = s.collection_id;
        let items_in_collection: Vec<&DatabaseItem> = self
            .items
            .iter()
            .filter(|i| i.collections.iter().any(|c| c.id == collection_id))
            .collect();

        self.library.confirm_publish(&items_in_collection).await?;
        Ok(())
    }

    pub fn get_library_browse_state(&self) -> Option<&LibraryBrowseState> {
        self.library.browse.as_ref()
    }

    pub fn get_library_browse_state_mut(&mut self) -> Option<&mut LibraryBrowseState> {
        self.library.browse.as_mut()
    }

    pub fn open_library_browse(&mut self) {
        self.library.open_browse();
        self.mode = Mode::LibraryBrowse;
    }

    pub fn get_subscribe_state(&self) -> Option<&LibrarySubscribeState> {
        self.library.subscribe.as_ref()
    }

    pub async fn confirm_library_subscribe(&mut self) -> Result<()> {
        let ok = self.library.confirm_subscribe().await?;
        if ok {
            self.mode = Mode::LibraryBrowse;
            self.notify(
                "Libreria sottoscritta",
                " Condivisione ".to_string(),
                Level::Info,
            );
        }
        self.library.refresh_current().await?;
        Ok(())
    }

    pub fn cancel_publish(&mut self) {
        self.library.cancel_publish();
        self.mode = Mode::Normal;
    }

    pub fn cancel_subscribe(&mut self) {
        self.library.cancel_subscribe();
        self.mode = Mode::Normal;
    }

    pub fn get_subscriptions(&self) -> &[LibrarySubscription] {
        self.library.subscriptions.as_slice()
    }

    pub async fn refresh_current_library(&mut self) -> Result<()> {
        self.library.refresh_current().await
    }

    pub fn open_library_manage(&mut self) {
        self.library.open_manage();
        self.mode = Mode::LibraryManage;
    }

    pub fn get_library_manage_state(&self) -> Option<&LibraryManageState> {
        self.library.manage.as_ref()
    }

    pub fn get_library_manage_state_mut(&mut self) -> Option<&mut LibraryManageState> {
        self.library.manage.as_mut()
    }

    pub async fn unsubscribe_selected_library(&mut self) -> color_eyre::Result<()> {
        let Some(browse) = &self.library.browse else {
            return Ok(());
        };
        if browse.focus != BrowseFocus::Subscriptions {
            return Ok(());
        }
        let Some(i) = browse.subscription_list_state.selected() else {
            return Ok(());
        };
        let Some(namespace_id) = self
            .library
            .subscriptions
            .get(i)
            .map(|s| s.namespace_id.clone())
        else {
            return Ok(());
        };

        self.library.unsubscribe(&namespace_id).await?;

        if let Some(browse) = &mut self.library.browse {
            let len = self.library.subscriptions.len();
            browse.subscription_list_state.select(if len == 0 {
                None
            } else {
                Some(i.min(len - 1))
            });
            browse.item_list_state = ListState::default();
        }

        self.notify(
            "Subscription removed",
            " Shared libraries ".to_string(),
            Level::Info,
        );
        Ok(())
    }
}
