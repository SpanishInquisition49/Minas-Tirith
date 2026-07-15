use std::{collections::HashSet, path::PathBuf, sync::Arc, time::Duration};

use cli_clipboard::{ClipboardContext, ClipboardProvider};
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
        common_metadata::{ItemMetadata, ItemType},
        facade::MetadataProvider,
        image_cache::ImageCache,
    },
    schema::{
        collection::Collection,
        item::DatabaseItem,
        message::{AbstractData, Message, SaveOutcome},
    },
    tui::app::{collection::CollectionState, traits::ListWidget},
};

pub mod collection;
mod cover;
mod metadata_edit;
mod traits;

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
}

pub const TABS_LABELS: [&str; 6] = ["All", "Book", "Article", "Thesis", "Report", "Misc"];

pub struct App {
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
    pub focus: Focus,

    task_channel_rx: UnboundedReceiver<Message>,
}

impl App {
    pub async fn new(
        archive: Archive,
        picker: Picker,
        cache: ImageCache,
    ) -> color_eyre::Result<Self> {
        let explorer = FileExplorerBuilder::default()
            .working_dir(std::env::home_dir().unwrap())
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
            .build()?;

        let (task_channel_tx, task_channel_rx) = mpsc::unbounded_channel();
        let task_channel_tx = Arc::new(task_channel_tx);
        let archive = Arc::new(archive);
        let provider = MetadataProvider::new();

        let mut app = Self {
            notifications: Notifications::new(),
            archive: archive.clone(),
            mode: Mode::Normal,
            items: Vec::new(),
            items_list_state: ListState::default(),
            selectd_tab: 0,
            quit: false,
            file_explorer: explorer,
            tick_counter: 0,
            metadata: MetadataEditState::new(archive.clone(), provider, task_channel_tx.clone()),
            covers: CoverState::new(archive.clone(), picker, cache, task_channel_tx.clone()),
            collections: CollectionState::new(archive.clone()),
            focus: Focus::Items,
            task_channel_rx,
        };
        app.request_refresh_item_list().await?;
        app.request_refresh_collections().await?;
        app.request_cover_for_selected();
        Ok(app)
    }

    fn notify(&mut self, message: impl Into<String>, title: String, level: Level) {
        if let Ok(notif) = Notification::new(message.into())
            .title(title)
            .timing(
                Timing::Fixed(Duration::from_millis(500)),
                Timing::Fixed(Duration::from_secs(3)),
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
            let _ = self.notifications.add(notif);
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

    pub async fn request_refresh_item_list(&mut self) -> color_eyre::Result<()> {
        self.items = self.archive.get_all_items().await?;
        if !self.items.is_empty() && self.items_list_state.selected().is_none() {
            self.items_list_state.select(Some(0));
        }
        Ok(())
    }

    pub fn request_file_opening(&self) -> color_eyre::Result<()> {
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

        let mut bibtex = String::default();
        for item in filtered_items {
            bibtex.push_str(&format!("{}\n", item.to_bibtex()));
        }

        match ClipboardContext::new() {
            Ok(mut ctx) => {
                let _ = ctx.set_contents(bibtex);
                self.notify(
                    "Copied Bibtex",
                    "  Export collection ".to_string(),
                    Level::Info,
                );
            }
            Err(_) => self.notify(
                "Clipboard unavailable",
                " Export citation ".to_string(),
                Level::Error,
            ),
        }
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
        let bibtex = item.to_bibtex();

        match ClipboardContext::new() {
            Ok(mut ctx) => {
                let _ = ctx.set_contents(bibtex);
                self.notify(
                    "Copied Bibtex",
                    "  Export citation ".to_string(),
                    Level::Info,
                );
            }
            Err(_) => self.notify(
                "Clipboard unavailable",
                " Export citation ".to_string(),
                Level::Error,
            ),
        }
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

    fn handle_metadata_search_message(&mut self, candidates: Vec<Box<dyn ItemMetadata>>) {
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
        let form = crate::schema::form::MetadataForm::from_item(item);
        let id = item.id;
        self.metadata
            .set_edit(form, EditContext::ExistingItem { id });
        self.mode = Mode::MetadataEdit;
    }

    pub fn confirm_metadata_form(&mut self) {
        self.metadata.confirm_save();
    }

    async fn handle_save_message(&mut self, outcome: SaveOutcome) -> color_eyre::Result<()> {
        let saved = self.metadata.on_save_result(outcome);
        if saved {
            self.mode = Mode::Normal;
            self.request_refresh_item_list().await?;
        }
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
        if applied {
            let title = item.fields.title.clone();
            // NOTE: we could also notify on fail, but i think it's just annoying
            self.notify(
                format!("Found abstact for '{title}'"),
                " Fetching Metadata ".to_string(),
                Level::Info,
            );
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Items => Focus::Collections,
            Focus::Collections => Focus::Items,
        }
    }

    pub async fn request_refresh_collections(&mut self) -> color_eyre::Result<()> {
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

    pub async fn delete_collection(&mut self) -> color_eyre::Result<()> {
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

    pub async fn confirm_collection_create(&mut self) -> color_eyre::Result<()> {
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

    pub async fn confirm_collection_assign(&mut self) -> color_eyre::Result<()> {
        self.collections.confirm_assign().await?;
        self.mode = Mode::Normal;
        self.request_refresh_item_list().await?;
        Ok(())
    }

    pub async fn poll_messages(&mut self) -> color_eyre::Result<()> {
        while let Ok(message) = self.task_channel_rx.try_recv() {
            match message {
                Message::Save(outcome) => self.handle_save_message(outcome).await?,
                Message::Metadata(candidates) => self.handle_metadata_search_message(candidates),
                Message::ImageCover(cover_data) => {
                    self.covers.handle_message(*cover_data, &mut self.items)
                }
                Message::Abstract(abstract_data) => self.handle_abstract_message(abstract_data),
            }
        }
        Ok(())
    }
}
