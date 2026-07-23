use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use color_eyre::{Result, eyre::bail};
use directories::ProjectDirs;
use ratatui::{
    style::{Color, Style},
    widgets::ListState,
};
use ratatui_explorer::{FileExplorer, FileExplorerBuilder};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use ratatui_notifications::{
    Anchor, Animation, AutoDismiss, Level, Notification, Notifications, SizeConstraint,
    SlideDirection, Timing,
};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::{
    database::archive::Archive,
    metadata::{
        common_metadata::ItemType, dedup::MergedCandidate, facade::MetadataProvider,
        image_cache::ImageCache,
    },
    peer2peer::library::SharedPaperEntry,
    schema::{
        collection::Collection,
        form::MetadataForm,
        item::DatabaseItem,
        message::{AbstractData, Message, SaveOutcome},
    },
    tui::app::{
        collection::CollectionState, library::LibraryState, peer2peer::PeerState,
        traits::ListWidget,
    },
};

pub mod collection;
mod cover;
pub mod library;
mod metadata_edit;
pub mod peer2peer;
pub mod traits;

pub use cover::CoverState;
pub use metadata_edit::{EditContext, MetadataEditState};

pub enum Focus {
    Items,
    Collections,
}

pub enum Mode {
    Normal,
    Insert,
    Search,
    MetadataSelect,
    MetadataEdit,
    CollectionCreate,
    CollectionAssign,
    LibraryPublish,
    LibrarySubscribe,
    LibraryBrowse,
    LibraryManage,
    Help,
}

pub const TABS_LABELS: [&str; 6] = ["All", "Book", "Article", "Thesis", "Report", "Misc"];

pub struct App {
    // TODO: make all this fields private for other crates
    pub notifications: Notifications,
    pub archive: Arc<Archive>,
    pub mode: Mode,
    pub items: Vec<DatabaseItem>,
    pub items_list_state: ListState,
    pub selectd_tab: usize,
    pub quit: bool,
    pub file_explorer: FileExplorer,
    pub tick_counter: usize,

    pub metadata: MetadataEditState,
    pub covers: CoverState,
    pub collections: CollectionState,
    pub peers: PeerState,
    pub focus: Focus,

    pub library: LibraryState,
    pub help_scroll: u16,

    task_channel_rx: UnboundedReceiver<Message>,
}

impl App {
    pub async fn new(
        archive: Archive,
        picker: Picker,
        cache: ImageCache,
        proj_dirs: &ProjectDirs,
    ) -> Result<Self> {
        let (task_channel_tx, task_channel_rx) = mpsc::unbounded_channel();
        let task_channel_tx = Arc::new(task_channel_tx);
        let import_dir = proj_dirs.data_dir().join("tomes");
        std::fs::create_dir_all(&import_dir)?;
        let archive = Arc::new(archive);
        let provider = MetadataProvider::new();
        let peers = PeerState::new(proj_dirs, task_channel_tx.clone()).await?;
        let library = LibraryState::new(
            archive.clone(),
            peers.share_node.clone(),
            task_channel_tx.clone(),
            import_dir,
        );

        let mut app = Self {
            notifications: Notifications::new(),
            archive: archive.clone(),
            mode: Mode::Normal,
            items: Vec::new(),
            items_list_state: ListState::default(),
            selectd_tab: 0,
            quit: false,
            file_explorer: Self::build_explorer(None)?,
            tick_counter: 0,
            metadata: MetadataEditState::new(archive.clone(), provider, task_channel_tx.clone()),
            covers: CoverState::new(archive.clone(), picker, cache, task_channel_tx.clone()),
            collections: CollectionState::new(archive.clone()),
            focus: Focus::Items,
            peers,
            task_channel_rx,
            library,
            help_scroll: 0,
        };
        app.request_refresh_item_list().await?;
        app.request_refresh_collections().await?;
        app.request_cover_for_selected();
        app.library.refresh().await?;
        app.library.reopen_known_namespaces().await?;
        Ok(app)
    }

    /// Create a new file explorer, if specified open from the given working directory
    fn build_explorer(working_dir: Option<&PathBuf>) -> Result<FileExplorer> {
        let working_dir = match working_dir {
            Some(dir) => dir,
            None => match directories::BaseDirs::new() {
                Some(base_dirs) => &base_dirs.home_dir().to_path_buf(),
                None => bail!("Cannot get home directory"),
            },
        };
        Ok(FileExplorerBuilder::default()
            .working_dir(working_dir)
            .filter_map(|f| {
                if f.is_dir {
                    Some(f)
                } else {
                    match f.path.extension() {
                        Some(extension) => match extension.to_str() {
                            Some("pdf") | Some("epub") => Some(f),
                            _ => None,
                        },
                        None => None,
                    }
                }
            })
            .build()?)
    }

    fn notify(&mut self, message: impl Into<String>, title: String, level: Level) {
        match Notification::new(message.into())
            .title(title)
            .timing(
                Timing::Fixed(Duration::from_millis(500)),
                Timing::Fixed(Duration::from_secs(5)),
                Timing::Fixed(Duration::from_millis(500)),
            )
            .border_style(Style::default().fg(match level {
                Level::Error => Color::Red,
                Level::Warn => Color::Yellow,
                _ => Color::Green,
            }))
            .max_size(SizeConstraint::Percentage(0.6), SizeConstraint::Absolute(4))
            .anchor(Anchor::TopRight)
            .animation(Animation::Slide)
            .level(level)
            .slide_direction(SlideDirection::FromTop)
            .auto_dismiss(AutoDismiss::After(Duration::from_secs(2)))
            .build()
        {
            Ok(notif) => {
                let _ = self.notifications.add(notif);
            }
            Err(e) => {
                tracing::error!(error = %e, "Could not build notification")
            }
        }
    }

    pub fn tabs_prev(&mut self) {
        self.selectd_tab = if self.selectd_tab == 0 {
            TABS_LABELS.len() - 1
        } else {
            self.selectd_tab - 1
        };
    }

    pub fn tabs_next(&mut self) {
        self.selectd_tab = if self.selectd_tab + 1 == TABS_LABELS.len() {
            0
        } else {
            self.selectd_tab + 1
        };
    }

    pub fn select_prev(&mut self) {
        let len = self
            .items
            .iter()
            .fold(0, |acc, i| if self.keep_items(i) { acc + 1 } else { acc });
        let i = match self.items_list_state.selected() {
            Some(i) if i > 0 => i - 1,
            Some(_) => len - 1,
            None => 0,
        };
        self.items_list_state.select(Some(i));
    }

    pub fn select_next(&mut self) {
        let len = self
            .items
            .iter()
            .fold(0, |acc, i| if self.keep_items(i) { acc + 1 } else { acc });
        let i = match self.items_list_state.selected() {
            Some(i) if i + 1 < len => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.items_list_state.select(Some(i));
    }

    pub fn keep_items(&self, item: &DatabaseItem) -> bool {
        let keep = match self.collections.selected() {
            Some(c) if item.collections.is_empty() && !Collection::is_trivial_collection(c.id) => {
                false
            }
            Some(collection) if !Collection::is_trivial_collection(collection.id) => {
                item.collections.iter().any(|c| c.id == collection.id)
            }
            _ => true,
        };
        if self.selectd_tab == 0 {
            return keep;
        }
        let Ok(active_item_type) = ItemType::try_from(TABS_LABELS[self.selectd_tab]) else {
            return keep;
        };
        let item_type =
            ItemType::try_from(item.fields.r#type.as_str()).unwrap_or(ItemType::default());
        keep && active_item_type == item_type
    }

    pub fn selected_item(&self) -> Option<&DatabaseItem> {
        let slug = {
            let filtered_items = self
                .items
                .iter()
                .filter(|i| self.keep_items(i))
                .collect::<Vec<_>>();
            let index = self.items_list_state.selected()?;
            filtered_items.get(index)?.fields.slug.clone()
        };
        self.items.iter().find(|i| i.fields.slug == slug)
    }

    pub async fn request_refresh_item_list(&mut self) -> Result<()> {
        self.items = self.archive.get_all_items().await?;
        if !self.items.is_empty() && self.items_list_state.selected().is_none() {
            self.items_list_state.select(Some(0));
        }
        Ok(())
    }

    pub fn request_file_opening(&self) -> Result<()> {
        let Some(index) = self.items_list_state.selected() else {
            return Ok(());
        };
        let filtered = self
            .items
            .iter()
            .filter(|i| self.keep_items(i))
            .collect::<Vec<_>>();
        let Some(item) = filtered.get(index) else {
            return Ok(());
        };
        opener::open(PathBuf::from(&item.path))?;
        Ok(())
    }

    pub fn bulk_bibtex_to_system_clipboard(&mut self) {
        let Some(collection) = self.collections.selected() else {
            self.notify(
                "No selected collection",
                " Bulk Export citation ".to_string(),
                Level::Error,
            );
            return;
        };
        let filtered_items: Vec<&DatabaseItem> = if Collection::is_trivial_collection(collection.id)
        {
            self.items.iter().collect()
        } else {
            self.items
                .iter()
                .filter(|i| i.collections.iter().any(|c| c.id == collection.id))
                .collect::<Vec<_>>()
        };

        // NOTE: to handle possible cite keys overlap we keep track of the used keys
        // and append to conflicting keys their version (an incremental counter)
        let mut bibtex = String::default();
        let mut key_version_map: HashMap<String, usize> = HashMap::default();
        for item in filtered_items {
            let cite_key = item.cite_key();
            let key = if key_version_map.contains_key(&cite_key) {
                let version = key_version_map.get(&cite_key).copied().unwrap_or(1);
                let key = format!("{}-{}", cite_key, version);
                key_version_map.insert(cite_key, version + 1);
                Some(key)
            } else {
                key_version_map.insert(cite_key, 1);
                None
            };
            bibtex.push_str(&format!("{}\n", item.to_bibtex(key)));
        }
        self.send_to_sys_clipboard(bibtex);
    }

    pub fn send_bibtex_to_system_clipboard(&mut self) {
        let Some(item) = self.selected_item() else {
            self.notify(
                "No selected item",
                " Export citation ".to_string(),
                Level::Error,
            );
            return;
        };
        let bibtex = item.to_bibtex(None);

        self.send_to_sys_clipboard(bibtex);
    }

    fn send_to_sys_clipboard(&mut self, content: String) {
        match ClipboardContext::new() {
            Ok(mut ctx) => match ctx.set_contents(content) {
                Ok(()) => {
                    self.notify(
                        "Copied Bibtex",
                        "  Export collection ".to_string(),
                        Level::Info,
                    );
                }
                Err(e) => {
                    tracing::error!(error = %e, "Could not set contents of the system clipboard")
                }
            },
            Err(e) => {
                tracing::error!(error = %e, "Could not get system clipboard");
                self.notify(
                    "Clipboard unavailable",
                    " Export citation ".to_string(),
                    Level::Error,
                )
            }
        }
    }

    pub fn request_open_file_picker(&mut self) -> Result<()> {
        let cwd = self.file_explorer.cwd();
        self.file_explorer = Self::build_explorer(Some(cwd))?;
        self.mode = Mode::Insert;
        Ok(())
    }

    pub fn request_cover_for_selected(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let id = item.id;
        let cover_url = item.fields.cover_image_url.clone();
        let path = item.path.clone();
        self.covers.request(id, cover_url, path);
    }

    pub fn selected_cover(&mut self) -> Option<&mut StatefulProtocol> {
        let id = self.selected_item()?.id;
        self.covers.get_mut(id)
    }

    pub fn request_fetch_metadata_candidates(&mut self) {
        let file = self.file_explorer.current();
        if !file.is_file() {
            return;
        }
        let filename = file
            .path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        self.metadata.request_fetch_candidates(filename);
    }

    fn handle_metadata_search_message(&mut self, candidates: Vec<MergedCandidate>) {
        let path = self.file_explorer.current().path.clone();
        let has_candidates = self.metadata.on_search_results(candidates, path);
        if has_candidates {
            self.mode = Mode::MetadataSelect;
        } else {
            self.notify(
                "Couldn't find metadata",
                "  Warning ".to_string(),
                Level::Warn,
            );
            self.mode = Mode::MetadataEdit;
        }
    }

    pub fn select_metadata_prev(&mut self) {
        self.metadata.select_prev();
    }

    pub fn select_metadata_next(&mut self) {
        self.metadata.select_next();
    }

    pub fn open_metadata_edit_for_candidate(&mut self) {
        if self.metadata.open_edit_for_candidate() {
            self.mode = Mode::MetadataEdit;
        }
    }

    pub fn cancel_metadata_selection(&mut self) {
        self.metadata.clear_candidates();
        self.mode = Mode::Insert;
    }

    pub fn open_metadata_edit_for_selected_item(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let form = MetadataForm::from_item(item);
        let id = item.id;
        self.metadata
            .set_edit(form, EditContext::ExistingItem { id });
        self.mode = Mode::MetadataEdit;
    }

    pub fn confirm_metadata_form(&mut self) {
        self.metadata.confirm_save();
    }

    async fn handle_save_message(&mut self, outcome: SaveOutcome) -> Result<()> {
        let (saved, was_update) = self.metadata.on_save_result(outcome);
        if saved {
            self.request_refresh_item_list().await?;
        } else {
            let title = match was_update {
                true => " Update tome ",
                false => " Insert new tome ",
            };
            let reason = self.metadata.last_error.clone().unwrap_or_default();
            self.notify(reason, title.to_string(), Level::Error);
        }
        self.mode = Mode::Normal;
        Ok(())
    }

    pub fn cancel_metadata_form(&mut self) {
        self.metadata.cancel_form();
        self.mode = Mode::Normal;
    }

    pub fn request_abstract_for_selected(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let id = item.id;
        let has_description = item.fields.description.is_some();
        let title = item.fields.title.clone();
        let doi = item.fields.doi.clone();
        let isbn = item.fields.isbn.clone();
        self.metadata
            .request_abstract(id, has_description, title, doi, isbn);
    }

    fn handle_abstract_message(&mut self, data: AbstractData) {
        let id = data.item_id;
        let applied = self.metadata.on_abstract_result(data);
        let Some(item) = self.items.iter_mut().find(|i| i.id == id) else {
            return;
        };
        let title = item.fields.title.clone();
        if applied {
            self.notify(
                format!("Found abstact for '{title}'"),
                " Fetching Metadata ".to_string(),
                Level::Info,
            );
        } else {
            // NOTE: on failure we log instead of pushing a notification, is less annoying
            // moreover, the fetching is fired without the user consent
            tracing::warn!(item_id = id, "Could not find an abstract text")
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Items => Focus::Collections,
            Focus::Collections => Focus::Items,
        }
    }

    pub async fn request_refresh_collections(&mut self) -> Result<()> {
        self.collections.refresh().await
    }

    pub fn select_collection_prev(&mut self) {
        self.collections.select_prev();
    }

    pub fn select_collection_next(&mut self) {
        self.collections.select_next();
    }

    pub fn confirm_collection_selection(&mut self) {
        self.collections.confirm_selection();
        self.items_list_state = ListState::default();
        if !self.items.is_empty() {
            self.items_list_state.select(Some(0));
        }
    }

    pub async fn delete_collection(&mut self) -> Result<()> {
        if let Some(collection_id) = self.collections.delete_highlighted().await? {
            self.items
                .iter_mut()
                .for_each(|i| i.collections.retain(|c| c.id != collection_id));
        }
        Ok(())
    }

    pub fn open_collection_create(&mut self) {
        self.collections.open_create();
        self.mode = Mode::CollectionCreate;
    }

    pub fn close_collection_create(&mut self) {
        self.mode = Mode::Normal;
    }

    pub async fn confirm_collection_create(&mut self) -> Result<()> {
        self.collections.confirm_create().await?;
        self.mode = Mode::Normal;
        Ok(())
    }

    pub fn open_item_assign_for_selected_collection(&mut self) {
        if self.collections.open_item_assign(&self.items) {
            self.mode = Mode::CollectionAssign;
        }
    }

    pub fn open_collection_assign_for_selected(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let item_id = item.id;
        let collection_ids: HashSet<i32> = item.collections.iter().map(|c| c.id).collect();
        self.collections
            .open_collection_assign(item_id, collection_ids);
        self.mode = Mode::CollectionAssign;
    }

    pub fn collection_assign_next(&mut self) {
        self.collections.assign_next(self.items.len());
    }

    pub fn collection_assign_prev(&mut self) {
        self.collections.assign_prev(self.items.len());
    }

    pub fn collection_assign_toggle_current(&mut self) {
        self.collections.assign_toggle_current(&self.items);
    }

    pub fn cancel_collection_assign(&mut self) {
        self.collections.cancel_assign();
        self.mode = Mode::Normal;
    }

    pub async fn confirm_collection_assign(&mut self) -> Result<()> {
        self.collections.confirm_assign().await?;
        self.mode = Mode::Normal;
        self.request_refresh_item_list().await?;
        Ok(())
    }

    pub async fn poll_messages(&mut self) -> Result<()> {
        while let Ok(message) = self.task_channel_rx.try_recv() {
            match message {
                Message::Save(outcome) => self.handle_save_message(outcome).await?,
                Message::Metadata(candidates) => self.handle_metadata_search_message(candidates),
                Message::ImageCover(cover_data) => {
                    self.covers.handle_message(*cover_data, &mut self.items)
                }
                Message::Abstract(abstract_data) => self.handle_abstract_message(abstract_data),
                Message::PeerDiscovered(peer_info) => self.peers.on_peer_discover(peer_info),
                Message::PeerExpired(peer_info) => self.peers.on_peer_expiration(peer_info),
                Message::LibraryPapersDiscovered {
                    namespace_id,
                    papers,
                } => {
                    self.library.handle_papers_discovered(namespace_id, papers);
                }
                Message::PaperDownloadReady { entry, local_path } => {
                    self.handle_paper_download_ready(entry, local_path);
                }
                Message::PaperDownloadFailed { reason, .. } => {
                    self.notify(reason, " Download tome ".to_string(), Level::Error);
                }
            }
        }
        Ok(())
    }

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

    pub async fn subscribe_to_library(
        &mut self,
        ticket_str: String,
        nickname: String,
    ) -> Result<()> {
        self.library.subscribe(ticket_str, nickname).await
    }

    pub fn request_import_paper(&mut self, entry: SharedPaperEntry) {
        self.library.request_import(entry);
    }

    pub fn handle_paper_download_ready(&mut self, entry: SharedPaperEntry, local_path: PathBuf) {
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

    pub fn open_library_browse(&mut self) {
        self.library.open_browse();
        self.mode = Mode::LibraryBrowse;
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

    pub async fn refresh_current_library(&mut self) -> Result<()> {
        self.library.refresh_current().await
    }

    pub fn open_library_manage(&mut self) {
        self.library.open_manage();
        self.mode = Mode::LibraryManage;
    }

    pub fn copy_current_ticket_to_clipboard(&mut self) {
        let Some(ticket) = self
            .library
            .manage
            .as_ref()
            .and_then(|s| s.current_ticket.clone())
        else {
            self.notify(
                "No ticket generated",
                " Copy ticket ".to_string(),
                Level::Error,
            );
            return;
        };
        self.send_to_sys_clipboard(ticket);
    }

    pub fn open_help(&mut self) {
        self.help_scroll = 0;
        self.mode = Mode::Help;
    }

    pub async fn unsubscribe_selected_library(&mut self) -> color_eyre::Result<()> {
        let Some(browse) = &self.library.browse else {
            return Ok(());
        };
        if browse.focus != crate::tui::app::library::BrowseFocus::Subscriptions {
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
            browse.paper_list_state = ListState::default();
        }

        self.notify(
            "Subscription removed",
            " Shared libraries ".to_string(),
            Level::Info,
        );
        Ok(())
    }
}
