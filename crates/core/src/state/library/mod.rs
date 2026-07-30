pub mod browse;
pub mod manage;
pub mod publish;
pub mod subscribe;

use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::Arc};

use color_eyre::eyre::{Context, OptionExt, Result, bail, eyre};
use futures::StreamExt;
use iroh_docs::{DocTicket, NamespaceId, api::Doc, engine::LiveEvent};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

use crate::{
    database::Archive,
    metadata::shared_library::{LibrarySubscription, SharedLibrary},
    peer2peer::{PrettyDisplay, library::SharedItemEntry, node::ShareNode},
    schema::{
        item::DatabaseItem,
        message::{LibraryDownloadFailed, LibraryDownloadReady, LibraryItemsDiscovered, Message},
    },
    state::library::{
        browse::LibraryBrowseState, manage::LibraryManageState, publish::LibraryPublishState,
        subscribe::LibrarySubscribeState,
    },
};

pub struct LibraryState {
    pub(in crate::state::library) archive: Arc<Archive>,
    pub(in crate::state::library) share_node: Arc<ShareNode>,
    pub(in crate::state::library) tx: Arc<UnboundedSender<Message>>,
    pub(in crate::state::library) import_dir: PathBuf,
    pub(in crate::state::library) browse: Option<LibraryBrowseState>,
    pub(in crate::state::library) publish: Option<LibraryPublishState>,
    pub(in crate::state::library) subscribe: Option<LibrarySubscribeState>,
    pub(in crate::state::library) manage: Option<LibraryManageState>,
    pub(in crate::state::library) shared_libraries: Vec<SharedLibrary>,
    pub(in crate::state::library) subscriptions: Vec<LibrarySubscription>,
    pub(in crate::state::library) browsed_items: HashMap<String, Vec<SharedItemEntry>>,
    pub(in crate::state::library) open_docs: HashMap<String, Doc>,
}

impl LibraryState {
    pub fn new(
        archive: Arc<Archive>,
        share_node: Arc<ShareNode>,
        tx: Arc<UnboundedSender<Message>>,
        import_dir: PathBuf,
    ) -> Self {
        Self {
            archive,
            share_node,
            tx,
            import_dir,
            browse: None,
            publish: None,
            subscribe: None,
            manage: None,
            shared_libraries: Vec::default(),
            subscriptions: Vec::default(),
            browsed_items: HashMap::default(),
            open_docs: HashMap::default(),
        }
    }

    /// Fetch the shared libraries and the subscriptions from the database
    pub async fn refresh(&mut self) -> Result<()> {
        self.shared_libraries = self.archive.get_all_shared_libraries().await?;
        self.subscriptions = self.archive.get_all_subscriptions().await?;
        Ok(())
    }

    /// Publish the given collection and return the corresponding ticket
    pub(in crate::state::library) async fn publish_colletion(
        &mut self,
        collection_id: i32,
        name: String,
        description: Option<String>,
        items_in_collection: &[&DatabaseItem],
    ) -> Result<String> {
        let (doc, author) = self.share_node.create_library().await?;
        let namespace_id = doc.id().to_string();

        let record = self
            .archive
            .create_shared_library(collection_id, &namespace_id, &name, description.as_deref())
            .await?;

        for item in items_in_collection {
            let file_path = PathBuf::from(&item.path);
            let tag = self.share_node.share_file(&file_path).await?;
            let file_name = file_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let blob_size = match std::fs::metadata(&file_path) {
                Ok(m) => m.len(),
                Err(e) => bail!(
                    "Cannot get metadata for {}: {}",
                    file_path.display(),
                    e.to_string()
                ),
            };

            let item_id = match &item.fields.shared_paper_id {
                Some(id) => Uuid::parse_str(id).with_context(|| {
                    format!("Parsing stored shared_paper_id for item {}", item.id)
                })?,
                None => {
                    let id = Uuid::new_v4();
                    self.archive
                        .set_shared_paper_id(item.id, &id.to_string())
                        .await?;
                    id
                }
            };
            let entry = SharedItemEntry::new(
                item,
                item_id,
                file_name,
                blob_size,
                tag.hash(),
                self.share_node.node_id(),
            );
            self.share_node.publish_paper(&doc, author, &entry).await?;
        }

        let ticket = self.share_node.share_library(&doc).await?;
        self.shared_libraries.push(record);

        Ok(ticket.to_string())
    }

    /// Use the given ticket to subscribe to a library from another peer.
    /// *Note:* this method kicks off a long lived background task after every subscription
    /// for handling sync events.
    pub(in crate::state::library) async fn subscribe(
        &mut self,
        ticket_str: String,
        nickname: String,
    ) -> Result<()> {
        let ticket = DocTicket::from_str(&ticket_str).context("Parsing doc ticket")?;
        let owner_node_id = ticket
            .nodes
            .first()
            .map(|addr| addr.id.to_string())
            .ok_or_eyre("Ticket has no nodes")?;

        let namespace_id = ticket.capability.id();
        let namespace_str = namespace_id.to_string();
        let (doc, mut events) = self
            .share_node
            .docs
            .import_and_subscribe(ticket)
            .await
            .map_err(|e| eyre!("Importing and subscribing to doc: {}", e.to_string()))?;

        let record = self
            .archive
            .create_subscription(&namespace_str, &owner_node_id, &nickname)
            .await?;
        self.subscriptions.push(record);
        self.open_docs.insert(namespace_str.clone(), doc.clone());

        let share_node = self.share_node.clone();
        let tx = self.tx.clone();
        let namespace_for_task = namespace_str.clone();
        let doc_for_task = doc.clone();

        // NOTE: spawn a long lived task to refresh subscription
        // TODO: avoid spawning a new task for each subscription, use a background worker
        tokio::spawn(async move {
            loop {
                let event = match events.next().await {
                    Some(Ok(event)) => event,
                    Some(Err(e)) => {
                        tracing::warn!(error = %e, "Error in doc event stream");
                        continue;
                    }
                    // NOTE: the stream is closed, because the node is down or removed
                    None => break,
                };
                let should_refresh = matches!(
                    event,
                    LiveEvent::SyncFinished(_) | LiveEvent::PendingContentReady
                );

                if should_refresh {
                    match share_node.list_items(&doc_for_task).await {
                        Ok(items) => {
                            if let Err(e) = tx.send(Message::LibraryItemsDiscovered(Box::new(
                                LibraryItemsDiscovered {
                                    namespace_id: namespace_for_task.clone(),
                                    items,
                                },
                            ))) {
                                tracing::error!(error = %e, "Failed to send discovered items");
                            }
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "Failed to list items after sync event")
                        }
                    }
                }
            }
        });
        Ok(())
    }

    /// Add the discovered items to the browsed list
    pub fn handle_items_discovered(&mut self, namespace_id: String, items: Vec<SharedItemEntry>) {
        self.browsed_items.insert(namespace_id, items);
    }

    /// Kicks off a download request for the given item
    pub(in crate::state::library) fn request_import(&mut self, entry: SharedItemEntry) {
        let share_node = self.share_node.clone();
        let tx = self.tx.clone();
        let dest_dir = self.import_dir.clone();

        tokio::spawn(async move {
            let dest_path = dest_dir.join(&entry.file_name);
            let message = match share_node
                .download_blob_by_hash(entry.blob_hash, entry.owner, &dest_path)
                .await
            {
                Ok(local_path) => {
                    Message::ItemDownloadReady(Box::new(LibraryDownloadReady { entry, local_path }))
                }
                Err(e) => Message::ItemDownloadFailed(Box::new(LibraryDownloadFailed {
                    item_id: entry.item_id,
                    reason: e.to_string(),
                })),
            };

            if let Err(e) = tx.send(message) {
                tracing::error!(error = %e, "Failed to send the download item status to main task")
            }
        });
    }

    pub async fn reopen_known_namespaces(&mut self) -> Result<()> {
        for library in &self.shared_libraries {
            let namespace_id = NamespaceId::from_str(&library.namespace_id).map_err(|e| {
                eyre!(
                    "Parsing stored namespace_id (published librart): {}",
                    e.to_string()
                )
            })?;
            match self.share_node.docs.open(namespace_id).await {
                Ok(Some(doc)) => {
                    self.open_docs.insert(library.namespace_id.clone(), doc);
                }
                Ok(None) => tracing::warn!(
                    namespace_id = %library.namespace_id,
                    "Published library namespace not found in local docs store"
                ),
                Err(e) => tracing::warn!(
                    error = %e,
                    namespace_id = %library.namespace_id,
                    "Failed to reopen published library"
                ),
            }
        }

        for subscription in self.subscriptions.clone() {
            if let Err(e) = self.reopen_subscription(&subscription.namespace_id).await {
                tracing::warn!(error = %e, namespace_id = %subscription.namespace_id, "Failed to reopen subscription");
            }
        }
        Ok(())
    }

    async fn reopen_subscription(&mut self, namespace_str: &str) -> Result<()> {
        let namespace_id = NamespaceId::from_str(namespace_str).map_err(|e| {
            eyre!(
                "Parsing stored namespace_id (subscription): {}",
                e.to_string()
            )
        })?;

        let Some(doc) = self
            .share_node
            .docs
            .open(namespace_id)
            .await
            .map_err(|e| eyre!("Reopening subscribed doc: {}", e.to_string()))?
        else {
            tracing::warn!(namespace_id = %namespace_str, "Subscribed namespace not found in local docs store");
            return Ok(());
        };

        self.open_docs
            .insert(namespace_str.to_string(), doc.clone());

        match self.share_node.list_items(&doc).await {
            Ok(papers) => {
                self.browsed_items.insert(namespace_str.to_string(), papers);
            }
            Err(e) => tracing::warn!(error = %e, "Failed to list papers on reopen"),
        }

        let events = doc
            .subscribe()
            .await
            .map_err(|e| eyre!("Subscribing to reopened doc events: {}", e.to_string()))?;
        let share_node = self.share_node.clone();
        let tx = self.tx.clone();
        let namespace_for_task = namespace_str.to_string();
        let doc_for_task = doc.clone();

        tokio::spawn(async move {
            tokio::pin!(events);
            loop {
                let event = match events.next().await {
                    Some(Ok(event)) => event,
                    Some(Err(e)) => {
                        tracing::warn!(error = %e, "Error in doc event stream (reopened)");
                        continue;
                    }
                    None => break,
                };
                let should_refresh = matches!(
                    event,
                    LiveEvent::SyncFinished(_) | LiveEvent::PendingContentReady
                );
                if should_refresh {
                    match share_node.list_items(&doc_for_task).await {
                        Ok(items) => {
                            if tx
                                .send(Message::LibraryItemsDiscovered(Box::new(
                                    LibraryItemsDiscovered {
                                        namespace_id: namespace_for_task.clone(),
                                        items,
                                    },
                                )))
                                .is_err()
                            {
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "Failed to list papers after sync event (reopened)")
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Get the ticket for the given namespace
    /// *Note:* it  open the namespace if wasn't already open
    pub(in crate::state::library) async fn get_or_refresh_ticket(
        &mut self,
        namespace_str: &str,
    ) -> Result<String> {
        if !self.open_docs.contains_key(namespace_str) {
            let ns = NamespaceId::from_str(namespace_str)
                .map_err(|e| eyre!("Parsing namespace_id: {}", e.to_string()))?;
            let doc = self
                .share_node
                .docs
                .open(ns)
                .await
                .map_err(|e| eyre!("Reopening doc for ticket generation: {}", e.to_string()))?
                .ok_or_eyre("Namespace not found locally")?;
            self.open_docs.insert(namespace_str.to_string(), doc);
        }

        let doc = self
            .open_docs
            .get(namespace_str)
            .ok_or_eyre("Namespace not found in open_docs")?;
        let ticket = self.share_node.share_library(doc).await?;

        Ok(ticket.to_string())
    }

    /// Delete a previously published library
    pub async fn unpublish(&mut self, id: i32) -> Result<()> {
        self.archive.delete_shared_library(id).await?;
        self.shared_libraries.retain(|l| l.id != id);
        Ok(())
    }

    /// Cancel a subscription to a shared library
    pub async fn unsubscribe(&mut self, namespace_str: &str) -> Result<()> {
        self.archive.delete_subscription(namespace_str).await?;
        self.subscriptions
            .retain(|s| s.namespace_id != namespace_str);
        self.open_docs.remove(namespace_str);
        self.browsed_items.remove(namespace_str);
        Ok(())
    }
}
