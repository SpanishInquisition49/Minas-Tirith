use std::{borrow::Cow, collections::HashMap, path::PathBuf, str::FromStr, sync::Arc};

use chrono::Utc;
use color_eyre::eyre::{Context, Result, bail, eyre};
use crossterm::event::Event;
use futures::StreamExt;
use iroh_docs::{DocTicket, NamespaceId, api::Doc, engine::LiveEvent};
use ratatui::widgets::ListState;
use tokio::sync::mpsc::UnboundedSender;
use tui_input::{Input, backend::crossterm::EventHandler};
use uuid::Uuid;

use crate::{
    database::archive::Archive,
    metadata::shared_library::{LibrarySubscription, SharedLibrary},
    peer2peer::{PrettyDisplay, library::SharedItemEntry, node::ShareNode},
    schema::{
        item::DatabaseItem,
        message::{LibraryDownloadFailed, LibraryDownloadReady, LibraryItemsDiscovered, Message},
    },
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PublishField {
    Name,
    Description,
}

pub struct LibraryPublishState {
    pub(in crate::tui::app) collection_id: i32,
    pub(in crate::tui::app) collection_name: String,
    pub(in crate::tui::app) name_input: Input,
    pub(in crate::tui::app) description_input: Input,
    pub(in crate::tui::app) field: PublishField,
    pub(in crate::tui::app) publishing: bool,
    pub(in crate::tui::app) result_ticket: Option<String>,
    pub(in crate::tui::app) last_error: Option<String>,
}

impl LibraryPublishState {
    pub fn collection_name(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.collection_name)
    }

    pub fn collection_name_input(&self) -> &Input {
        &self.name_input
    }

    pub fn collection_description_input(&self) -> &Input {
        &self.description_input
    }

    pub fn publish_field(&self) -> PublishField {
        self.field
    }

    pub fn is_publishing(&self) -> bool {
        self.publishing
    }

    pub fn result_ticket(&self) -> Option<Cow<'_, str>> {
        self.result_ticket.as_deref().map(Cow::Borrowed)
    }

    pub fn get_last_error(&self) -> Option<Cow<'_, str>> {
        self.last_error.as_deref().map(Cow::Borrowed)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SubscribeField {
    Ticket,
    Nickname,
}

pub struct LibrarySubscribeState {
    pub(in crate::tui::app) ticket_input: Input,
    pub(in crate::tui::app) nickname_input: Input,
    pub(in crate::tui::app) field: SubscribeField,
    pub(in crate::tui::app) subscribing: bool,
    pub(in crate::tui::app) last_error: Option<String>,
}

impl LibrarySubscribeState {
    pub fn ticket_input(&self) -> &Input {
        &self.ticket_input
    }

    pub fn nickname_input(&self) -> &Input {
        &self.nickname_input
    }

    pub fn field(&self) -> SubscribeField {
        self.field
    }

    pub fn get_last_error(&self) -> Option<Cow<'_, str>> {
        self.last_error.as_deref().map(Cow::Borrowed)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum BrowseFocus {
    Subscriptions,
    Papers,
}

pub struct LibraryBrowseState {
    pub(in crate::tui::app) subscription_list_state: ListState,
    pub(in crate::tui::app) item_list_state: ListState,
    pub(in crate::tui::app) focus: BrowseFocus,
}

impl LibraryBrowseState {
    pub fn subscription_list_state_mut(&mut self) -> &mut ListState {
        &mut self.subscription_list_state
    }

    pub fn item_list_state_mut(&mut self) -> &mut ListState {
        &mut self.item_list_state
    }

    pub fn focus(&self) -> BrowseFocus {
        self.focus
    }
}

pub struct LibraryManageState {
    pub(in crate::tui::app) list_state: ListState,
    pub(in crate::tui::app) current_ticket: Option<String>,
    pub(in crate::tui::app) generating: bool,
    pub(in crate::tui::app) last_error: Option<String>,
}

impl LibraryManageState {
    pub fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }

    pub fn current_ticket(&self) -> Option<Cow<'_, str>> {
        self.current_ticket.as_deref().map(Cow::Borrowed)
    }

    pub fn is_generating(&self) -> bool {
        self.generating
    }

    pub fn get_last_error(&self) -> Option<Cow<'_, str>> {
        self.last_error.as_deref().map(Cow::Borrowed)
    }
}

pub struct LibraryState {
    archive: Arc<Archive>,
    share_node: Arc<ShareNode>,
    tx: Arc<UnboundedSender<Message>>,
    import_dir: PathBuf,

    pub(in crate::tui::app) browse: Option<LibraryBrowseState>,
    pub(in crate::tui::app) publish: Option<LibraryPublishState>,
    pub(in crate::tui::app) subscribe: Option<LibrarySubscribeState>,
    pub(in crate::tui::app) manage: Option<LibraryManageState>,
    pub(in crate::tui::app) shared_libraries: Vec<SharedLibrary>,
    pub(in crate::tui::app) subscriptions: Vec<LibrarySubscription>,
    pub(in crate::tui::app) browsed_papers: HashMap<String, Vec<SharedItemEntry>>,
    open_docs: HashMap<String, Doc>,
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
            shared_libraries: Vec::new(),
            subscriptions: Vec::new(),
            browsed_papers: HashMap::new(),
            open_docs: HashMap::new(),
            browse: None,
            publish: None,
            subscribe: None,
            manage: None,
        }
    }

    pub async fn refresh(&mut self) -> Result<()> {
        self.shared_libraries = self.archive.get_all_shared_libraries().await?;
        self.subscriptions = self.archive.get_all_subscriptions().await?;
        Ok(())
    }

    pub async fn publish_collection(
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
                Ok(metadata) => metadata.len(),
                Err(_) => bail!("Cannot get metadata for {}", file_path.display()),
            };

            let entry = SharedItemEntry {
                paper_id: Uuid::new_v4(),
                blob_hash: tag.hash(),
                blob_size,
                file_name,
                title: item.fields.title.clone(),
                description: item.fields.description.clone(),
                r#type: item.fields.r#type.clone(),
                doi: item.fields.doi.clone(),
                isbn: item.fields.isbn.clone(),
                publication_date: item.fields.publication_date.clone(),
                container: item.fields.container.clone(),
                authors: item.authors.iter().map(|a| a.name.clone()).collect(),
                tags: item.tags.iter().map(|t| t.name.clone()).collect(),
                cover_image_url: item.fields.cover_image_url.clone(),
                owner: self.share_node.node_id(),
                published_at: Utc::now(),
                updated_at: Utc::now(),
            };

            self.share_node.publish_paper(&doc, author, &entry).await?;
        }

        let ticket = self.share_node.share_library(&doc).await?;
        self.shared_libraries.push(record);

        Ok(ticket.to_string())
    }

    pub async fn subscribe(&mut self, ticket_str: String, nickname: String) -> Result<()> {
        let ticket = DocTicket::from_str(&ticket_str).context("Parsing doc ticket")?;
        let owner_node_id = ticket
            .nodes
            .first()
            .map(|addr| addr.id.to_string())
            .ok_or_else(|| eyre!("Ticket has no nodes"))?;

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

        tokio::spawn(async move {
            loop {
                let event = match events.next().await {
                    Some(Ok(event)) => event,
                    Some(Err(e)) => {
                        tracing::warn!(error = %e, "Error in doc event stream");
                        continue;
                    }
                    None => break, // stream chiuso, doc rimosso o nodo spento
                };

                let should_refresh = matches!(
                    event,
                    LiveEvent::SyncFinished(_) | LiveEvent::PendingContentReady
                );

                if should_refresh {
                    match share_node.list_papers(&doc_for_task).await {
                        Ok(items) => {
                            if let Err(e) = tx.send(Message::LibraryItemsDiscovered(Box::new(
                                LibraryItemsDiscovered {
                                    namespace_id: namespace_for_task.clone(),
                                    items,
                                },
                            ))) {
                                tracing::error!(error = %e, "Failed to send discovered papers");
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "Failed to list papers after sync event")
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub fn handle_items_discovered(&mut self, namespace_id: String, papers: Vec<SharedItemEntry>) {
        self.browsed_papers.insert(namespace_id, papers);
    }

    pub fn request_import(&mut self, entry: SharedItemEntry) {
        let share_node = self.share_node.clone();
        let tx = self.tx.clone();
        let dest_dir = self.import_dir.clone();
        let entry_for_task = entry.clone();

        tokio::spawn(async move {
            let dest_path = dest_dir.join(&entry_for_task.file_name);
            match share_node
                .download_blob_by_hash(entry_for_task.blob_hash, entry_for_task.owner, &dest_path)
                .await
            {
                Ok(local_path) => {
                    if let Err(e) =
                        tx.send(Message::ItemDownloadReady(Box::new(LibraryDownloadReady {
                            entry: entry_for_task,
                            local_path,
                        })))
                    {
                        tracing::error!(error = %e, "Failed to send paper download ready");
                    }
                }
                Err(e) => {
                    let _ = tx.send(Message::ItemDownloadFailed(Box::new(
                        LibraryDownloadFailed {
                            item_id: entry_for_task.paper_id,
                            reason: e.to_string(),
                        },
                    )));
                }
            }
        });
    }

    pub async fn refresh_current(&mut self) -> Result<()> {
        let Some(namespace) = self.current_namespace().map(str::to_string) else {
            return Ok(());
        };
        let Some(doc) = self.open_docs.get(&namespace) else {
            return Ok(());
        };
        let papers = self.share_node.list_papers(doc).await?;
        self.browsed_papers.insert(namespace, papers);
        Ok(())
    }

    // NOTE: PUBLISH POPfUP

    pub fn open_publish(&mut self, collection_id: i32, collection_name: String) {
        self.publish = Some(LibraryPublishState {
            collection_id,
            collection_name,
            name_input: Input::default(),
            description_input: Input::default(),
            field: PublishField::Name,
            result_ticket: None,
            publishing: false,
            last_error: None,
        });
    }

    pub fn publish_next_field(&mut self) {
        if let Some(s) = &mut self.publish {
            s.field = match s.field {
                PublishField::Name => PublishField::Description,
                PublishField::Description => PublishField::Name,
            }
        }
    }

    pub fn handle_publish_event(&mut self, event: &Event) {
        let Some(s) = &mut self.publish else { return };
        match s.field {
            PublishField::Name => s.name_input.handle_event(event),
            PublishField::Description => s.description_input.handle_event(event),
        };
    }

    pub async fn confirm_publish(
        &mut self,
        items_in_collection: &[&DatabaseItem],
    ) -> Result<Option<String>> {
        let Some(s) = &self.publish else {
            return Ok(None);
        };

        let name = s.name_input.value().trim().to_string();
        if name.is_empty() {
            return Ok(None);
        }

        let description = {
            let d = s.description_input.value().trim();
            if d.is_empty() {
                None
            } else {
                Some(d.to_string())
            }
        };

        let collection_id = s.collection_id;

        if let Some(s) = &mut self.publish {
            s.publishing = true;
        }

        let result = self
            .publish_collection(
                collection_id,
                name.clone(),
                description,
                items_in_collection,
            )
            .await;

        if let Some(s) = &mut self.publish {
            s.publishing = false;
            match result {
                Ok(ticket) => {
                    s.result_ticket = Some(ticket);
                    Ok(Some(name))
                }
                Err(e) => {
                    s.last_error = Some(e.to_string());
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    pub fn cancel_publish(&mut self) {
        self.publish = None
    }

    // NOTE: SUBSCRIBE POPUP

    pub fn open_subscribe(&mut self) {
        self.subscribe = Some(LibrarySubscribeState {
            ticket_input: Input::default(),
            nickname_input: Input::default(),
            field: SubscribeField::Ticket,
            subscribing: false,
            last_error: None,
        });
    }

    pub fn subscribe_next_field(&mut self) {
        if let Some(s) = &mut self.subscribe {
            s.field = match s.field {
                SubscribeField::Ticket => SubscribeField::Nickname,
                SubscribeField::Nickname => SubscribeField::Ticket,
            };
        }
    }

    pub fn handle_subscribe_event(&mut self, event: &Event) {
        let Some(s) = &mut self.subscribe else { return };
        match s.field {
            SubscribeField::Ticket => s.ticket_input.handle_event(event),
            SubscribeField::Nickname => s.nickname_input.handle_event(event),
        };
    }

    pub async fn confirm_subscribe(&mut self) -> color_eyre::Result<bool> {
        let Some(s) = &self.subscribe else {
            return Ok(false);
        };
        let ticket = s.ticket_input.value().trim().to_string();
        let nickname = s.nickname_input.value().trim().to_string();
        if ticket.is_empty() || nickname.is_empty() {
            return Ok(false);
        }

        if let Some(s) = &mut self.subscribe {
            s.subscribing = true;
        }

        let result = self.subscribe(ticket, nickname).await;

        match result {
            Ok(()) => {
                self.subscribe = None;
                Ok(true)
            }
            Err(e) => {
                if let Some(s) = &mut self.subscribe {
                    s.subscribing = false;
                    s.last_error = Some(e.to_string());
                }
                Ok(false)
            }
        }
    }

    pub fn cancel_subscribe(&mut self) {
        self.subscribe = None;
    }

    // NOTE: BROWSE POPUP

    pub fn open_browse(&mut self) {
        let mut subscription_list_state = ListState::default();
        if !self.subscriptions.is_empty() {
            subscription_list_state.select(Some(0));
        }
        self.browse = Some(LibraryBrowseState {
            subscription_list_state,
            item_list_state: ListState::default(),
            focus: BrowseFocus::Subscriptions,
        });
    }

    pub fn close_browse(&mut self) {
        self.browse = None;
    }

    pub fn browse_toggle_focus(&mut self) {
        if let Some(s) = &mut self.browse {
            s.focus = match s.focus {
                BrowseFocus::Subscriptions => BrowseFocus::Papers,
                BrowseFocus::Papers => BrowseFocus::Subscriptions,
            };
        }
    }

    fn current_namespace(&self) -> Option<&str> {
        let s = self.browse.as_ref()?;
        let i = s.subscription_list_state.selected()?;
        self.subscriptions
            .get(i)
            .map(|sub| sub.namespace_id.as_str())
    }

    pub fn current_items(&self) -> &[SharedItemEntry] {
        self.current_namespace()
            .and_then(|ns| self.browsed_papers.get(ns))
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn browse_select_next(&mut self) {
        let Some(s) = &self.browse else { return };

        let len = match s.focus {
            BrowseFocus::Subscriptions => self.subscriptions.len(),
            BrowseFocus::Papers => self.current_items().len(),
        };

        let Some(s) = &mut self.browse else { return };
        match s.focus {
            BrowseFocus::Subscriptions => {
                if len == 0 {
                    return;
                }
                let i = match s.subscription_list_state.selected() {
                    Some(i) if i + 1 < len => i + 1,
                    _ => 0,
                };
                s.subscription_list_state.select(Some(i));
                s.item_list_state = ListState::default();
            }
            BrowseFocus::Papers => {
                let index = s.item_list_state.selected();
                if len == 0 {
                    return;
                }
                let i = match index {
                    Some(i) if i + 1 < len => i + 1,
                    _ => 0,
                };
                s.item_list_state.select(Some(i));
            }
        }
    }

    pub fn browse_select_prev(&mut self) {
        let Some(s) = &self.browse else { return };

        let len = match s.focus {
            BrowseFocus::Subscriptions => self.subscriptions.len(),
            BrowseFocus::Papers => self.current_items().len(),
        };
        let Some(s) = &mut self.browse else { return };
        match s.focus {
            BrowseFocus::Subscriptions => {
                if len == 0 {
                    return;
                }
                let i = match s.subscription_list_state.selected() {
                    Some(i) if i > 0 => i - 1,
                    _ => len - 1,
                };
                s.subscription_list_state.select(Some(i));
                s.item_list_state = ListState::default();
            }
            BrowseFocus::Papers => {
                if len == 0 {
                    return;
                }
                let i = match s.item_list_state.selected() {
                    Some(i) if i > 0 => i - 1,
                    _ => len - 1,
                };
                s.item_list_state.select(Some(i));
            }
        }
    }

    pub fn browse_confirm_import(&mut self) {
        let Some(s) = &self.browse else { return };
        if s.focus != BrowseFocus::Papers {
            return;
        }
        let Some(i) = s.item_list_state.selected() else {
            return;
        };
        let Some(entry) = self.current_items().get(i).cloned() else {
            return;
        };
        self.request_import(entry);
    }

    pub async fn reopen_known_namespaces(&mut self) -> color_eyre::Result<()> {
        // NOTE: reopen published libraries, to keep them live
        for library in self.shared_libraries.clone() {
            let namespace_id = NamespaceId::from_str(&library.namespace_id).map_err(|e| {
                eyre!(
                    "Parsing stored namespace_id (published library): {}",
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

        // NOTE: reopen subscriptions
        for subscription in self.subscriptions.clone() {
            if let Err(e) = self.reopen_subscription(&subscription.namespace_id).await {
                tracing::warn!(error = %e, namespace_id = %subscription.namespace_id, "Failed to reopen subscription");
            }
        }

        Ok(())
    }

    async fn reopen_subscription(&mut self, namespace_str: &str) -> color_eyre::Result<()> {
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

        match self.share_node.list_papers(&doc).await {
            Ok(papers) => {
                self.browsed_papers
                    .insert(namespace_str.to_string(), papers);
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
                    match share_node.list_papers(&doc_for_task).await {
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

    pub async fn get_or_refresh_ticket(
        &mut self,
        namespace_id: &str,
    ) -> color_eyre::Result<String> {
        let doc = if let Some(doc) = self.open_docs.get(namespace_id) {
            doc.clone()
        } else {
            let ns = NamespaceId::from_str(namespace_id)
                .map_err(|e| eyre!("Parsing namespace_id: {}", e.to_string()))?;
            let doc = self
                .share_node
                .docs
                .open(ns)
                .await
                .map_err(|e| eyre!("Reopening doc for ticket generation: {}", e.to_string()))?
                .ok_or_else(|| eyre!("Namespace not found locally"))?;
            self.open_docs.insert(namespace_id.to_string(), doc.clone());
            doc
        };

        let ticket = self.share_node.share_library(&doc).await?;
        Ok(ticket.to_string())
    }

    pub async fn unpublish(&mut self, id: i32) -> color_eyre::Result<()> {
        self.archive.delete_shared_library(id).await?;
        self.shared_libraries.retain(|l| l.id != id);
        Ok(())
    }

    pub fn open_manage(&mut self) {
        let mut list_state = ListState::default();
        if !self.shared_libraries.is_empty() {
            list_state.select(Some(0));
        }
        self.manage = Some(LibraryManageState {
            list_state,
            current_ticket: None,
            generating: false,
            last_error: None,
        });
    }

    pub fn close_manage(&mut self) {
        self.manage = None;
    }

    pub fn manage_select_next(&mut self) {
        let Some(s) = &mut self.manage else { return };
        let len = self.shared_libraries.len();
        if len == 0 {
            return;
        }
        let i = match s.list_state.selected() {
            Some(i) if i + 1 < len => i + 1,
            _ => 0,
        };
        s.list_state.select(Some(i));
        s.current_ticket = None;
    }

    pub fn manage_select_prev(&mut self) {
        let Some(s) = &mut self.manage else { return };
        let len = self.shared_libraries.len();
        if len == 0 {
            return;
        }
        let i = match s.list_state.selected() {
            Some(i) if i > 0 => i - 1,
            _ => len - 1,
        };
        s.list_state.select(Some(i));
        s.current_ticket = None;
    }

    pub fn manage_selected(&self) -> Option<&SharedLibrary> {
        let s = self.manage.as_ref()?;
        let i = s.list_state.selected()?;
        self.shared_libraries.get(i)
    }

    pub async fn manage_generate_ticket(&mut self) -> color_eyre::Result<()> {
        let Some(namespace_id) = self.manage_selected().map(|l| l.namespace_id.clone()) else {
            return Ok(());
        };

        if let Some(s) = &mut self.manage {
            s.generating = true;
        }

        let result = self.get_or_refresh_ticket(&namespace_id).await;

        if let Some(s) = &mut self.manage {
            s.generating = false;
            match result {
                Ok(ticket) => s.current_ticket = Some(ticket),
                Err(e) => s.last_error = Some(e.to_string()),
            }
        }
        Ok(())
    }

    pub async fn manage_delete_selected(&mut self) -> color_eyre::Result<()> {
        let Some(id) = self.manage_selected().map(|l| l.id) else {
            return Ok(());
        };
        self.unpublish(id).await?;
        if let Some(s) = &mut self.manage {
            s.list_state = ListState::default();
            if !self.shared_libraries.is_empty() {
                s.list_state.select(Some(0));
            }
            s.current_ticket = None;
        }
        Ok(())
    }

    pub async fn unsubscribe(&mut self, namespace_id: &str) -> color_eyre::Result<()> {
        self.archive.delete_subscription(namespace_id).await?;
        self.subscriptions
            .retain(|s| s.namespace_id != namespace_id);
        self.open_docs.remove(namespace_id);
        self.browsed_papers.remove(namespace_id);
        Ok(())
    }
}
