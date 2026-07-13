use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use color_eyre::eyre::Context;
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
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tui_input::Input;

use crate::{
    database::archive::Archive,
    metadata::{
        common_metadata::{ItemMetadata, ItemType},
        cover_generator::generate_cover,
        facade::MetadataProvider,
        image_cache::ImageCache,
    },
    schema::{
        collection::Collection,
        form::MetadataForm,
        item::DatabaseItem,
        message::{AbstractData, CoverImageData, Message, SaveOutcome},
    },
};

pub enum Focus {
    Items,
    Collections,
}

pub enum AssignMode {
    /// Assign items to the selected collection
    Items,
    /// Assign collections to the selected item
    Collections,
}

pub struct CollectionAssignState {
    pub mode: AssignMode,
    pub id: i32,
    pub original: HashSet<i32>,
    pub selected: HashSet<i32>,
    pub list_state: ListState,
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

pub enum EditContext {
    NewItem { path: PathBuf },
    ExistingItem { id: i32 },
}

pub const TABS_LABELS: [&str; 6] = ["All", "Book", "Article", "Thesis", "Report", "Misc"];

pub struct App {
    pub notifications: Notifications,
    pub archive: Arc<Archive>,
    pub mode: Mode,
    pub items: Vec<DatabaseItem>,
    pub items_list_state: ListState,
    pub selectd_tab: usize,
    //pub search_query: String,
    pub quit: bool,
    pub file_explorer: FileExplorer,
    // Metadata for new items
    pub metadata_candidates: Vec<Box<dyn ItemMetadata>>,
    pub metadata_list_state: ListState,
    pub metadata_form: Option<MetadataForm>,
    pub edit_context: Option<EditContext>,
    pub saving: bool,
    pub last_error: Option<String>,
    pub tick_counter: usize,
    pub is_searching: bool,

    // Collections
    pub collections: Vec<Collection>,
    pub collections_list_state: ListState,
    pub selected_collection: Option<i32>,
    pub focus: Focus,
    pub collection_name_input: Input,
    pub collection_assign: Option<CollectionAssignState>,

    metadata_provider: Arc<MetadataProvider>,
    candidate_path: Option<PathBuf>,
    picker: Arc<Picker>,
    cache: Arc<ImageCache>,
    covers: HashMap<i32, StatefulProtocol>,
    pending_covers: HashSet<i32>,
    pending_abstract: HashSet<i32>,
    failed_abstract: HashSet<i32>,

    task_channel_tx: Arc<UnboundedSender<Message>>,
    task_channel_rx: UnboundedReceiver<Message>,
}

impl App {
    pub async fn new(
        archive: Archive,
        picker: Picker,
        cache: ImageCache,
    ) -> color_eyre::Result<Self> {
        let list = ListState::default();

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

        let mut app = Self {
            notifications: Notifications::new(),
            archive: Arc::new(archive),
            picker: Arc::new(picker),
            cache: Arc::new(cache),
            file_explorer: explorer,
            mode: Mode::Normal,
            items: Vec::new(),
            items_list_state: list,
            //search_query: String::new(),
            quit: false,
            covers: HashMap::new(),
            pending_covers: HashSet::new(),
            metadata_provider: Arc::new(MetadataProvider::new()),
            metadata_candidates: Vec::new(),
            metadata_list_state: ListState::default(),
            candidate_path: None,
            metadata_form: None,
            edit_context: None,
            saving: false,
            last_error: None,
            tick_counter: 0,
            is_searching: false,
            task_channel_tx: Arc::new(task_channel_tx),
            task_channel_rx,
            selectd_tab: 0,
            pending_abstract: HashSet::new(),
            failed_abstract: HashSet::new(),
            collections: Vec::new(),
            collections_list_state: ListState::default(),
            selected_collection: None,
            focus: Focus::Items,
            collection_name_input: Input::default(),
            collection_assign: None,
        };
        app.request_refresh_item_list().await?;
        app.request_refresh_collections().await?;
        app.request_cover_for_selected();
        Ok(app)
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
        let i = match self.items_list_state.selected() {
            Some(i) if i > 0 => i - 1,
            Some(_) => self.items.len() - 1,
            None => 0,
        };
        self.items_list_state.select(Some(i));
    }

    pub fn select_next(&mut self) {
        let i = match self.items_list_state.selected() {
            Some(i) if i + 1 < self.items.len() => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.items_list_state.select(Some(i));
    }

    pub fn keep_items(&self, item: &DatabaseItem) -> bool {
        let keep = match self.selected_collection {
            Some(_) if item.collections.is_empty() => false,
            Some(collection_id) => item.collections.iter().any(|c| c.id == collection_id),
            None => true,
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

    pub fn selected_item_mut(&mut self) -> Option<&mut DatabaseItem> {
        let slug = {
            let filtered_items = self
                .items
                .iter()
                .filter(|i| self.keep_items(i))
                .collect::<Vec<_>>();
            let index = self.items_list_state.selected()?;
            filtered_items.get(index)?.fields.slug.clone()
        };

        self.items.iter_mut().find(|i| i.fields.slug == slug)
    }
    pub fn request_cover_for_selected(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let id = item.id;

        if self.covers.contains_key(&id) || self.pending_covers.contains(&id) {
            return;
        }

        match item.fields.cover_image_url.clone() {
            Some(url) => self.spawn_cover_download(id, url),
            None => self.spawn_cover_generation(id, item.path.clone()),
        }
    }

    fn spawn_cover_download(&mut self, id: i32, url: String) {
        self.pending_covers.insert(id);
        let cache = self.cache.clone();
        let picker = self.picker.clone();
        let tx = self.task_channel_tx.clone();

        tokio::spawn(async move {
            let result: color_eyre::Result<StatefulProtocol> = async {
                let path = cache.get_or_download(&url).await?;
                let dyn_image = tokio::task::spawn_blocking(move || image::open(&path))
                    .await
                    .context("Joining image decode task")?
                    .context("Decode cover image")?;
                Ok(picker.new_resize_protocol(dyn_image))
            }
            .await;

            if let Ok(protocol) = result {
                let _ = tx.send(Message::ImageCover(Box::new(CoverImageData {
                    item_id: id,
                    protocol,
                    url: None,
                })));
            }
        });
    }

    fn spawn_cover_generation(&mut self, id: i32, file_path: String) {
        self.pending_covers.insert(id);
        let cache = self.cache.clone();
        let picker = self.picker.clone();
        let archive = self.archive.clone();
        let tx = self.task_channel_tx.clone();
        let path = PathBuf::from(file_path);

        tokio::spawn(async move {
            let result: color_eyre::Result<(StatefulProtocol, String)> = async {
                let Some(cover_path) = generate_cover(&path, id, &cache).await? else {
                    return Err(color_eyre::eyre::eyre!("No cover could be generated"));
                };
                let url = format!("file://{}", cover_path.display());
                archive.set_cover_image_url(id, &url).await?;

                let dyn_image = {
                    let cover_path = cover_path.clone();
                    tokio::task::spawn_blocking(move || image::open(&cover_path))
                        .await
                        .context("Joining image decode task")?
                        .context("Decode generated cover image")?
                };
                Ok((picker.new_resize_protocol(dyn_image), url))
            }
            .await;

            if let Ok((protocol, url)) = result {
                let _ = tx.send(Message::ImageCover(Box::new(CoverImageData {
                    item_id: id,
                    protocol,
                    url: Some(url),
                })));
            }
        });
    }
    pub fn handle_cover_message(&mut self, cover_data: CoverImageData) {
        self.pending_covers.remove(&cover_data.item_id);
        self.covers.insert(cover_data.item_id, cover_data.protocol);
        if let Some(url) = cover_data.url
            && let Some(item) = self.items.iter_mut().find(|i| i.id == cover_data.item_id)
        {
            item.fields.cover_image_url = Some(url);
        }
    }

    pub fn selected_cover(&mut self) -> Option<&mut StatefulProtocol> {
        let id = self.selected_item()?.id;
        self.covers.get_mut(&id)
    }

    pub async fn request_refresh_item_list(&mut self) -> color_eyre::Result<()> {
        self.items = self.archive.get_all_items().await?;
        if !self.items.is_empty() && self.items_list_state.selected().is_none() {
            self.items_list_state.select(Some(0));
        }
        Ok(())
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

        self.is_searching = true;
        let tx = self.task_channel_tx.clone();
        let provider = self.metadata_provider.clone();
        tokio::spawn(async move {
            let candidates = provider.fetch(&filename).await;
            let _ = tx.send(Message::Metadata(candidates));
        });
    }

    pub fn handle_metadata_search_message(&mut self, candidates: Vec<Box<dyn ItemMetadata>>) {
        let file = self.file_explorer.current();
        self.is_searching = false;
        self.metadata_candidates = candidates;
        self.metadata_list_state = ListState::default();
        self.candidate_path = Some(file.path.clone());

        if !self.metadata_candidates.is_empty() {
            self.metadata_list_state.select(Some(0));
            self.mode = Mode::MetadataSelect;
        } else {
            let file = self.file_explorer.current();
            if let Ok(notif) = Notification::new("Couldn't find metadata")
                .title("  Warning ")
                .timing(
                    Timing::Fixed(Duration::from_millis(500)),
                    Timing::Fixed(Duration::from_secs(3)),
                    Timing::Fixed(Duration::from_millis(500)),
                )
                .border_style(Style::default().fg(Color::Yellow))
                .title_style(Style::default().fg(Color::Yellow))
                .max_size(SizeConstraint::Percentage(0.6), SizeConstraint::Absolute(4))
                .anchor(Anchor::TopRight)
                .animation(Animation::Slide)
                .level(Level::Warn)
                .slide_direction(SlideDirection::FromTop)
                .auto_dismiss(AutoDismiss::After(Duration::from_secs(2)))
                .build()
            {
                let _ = self.notifications.add(notif);
            }
            self.metadata_form = Some(MetadataForm::new());
            self.edit_context = Some(EditContext::NewItem {
                path: file.path.clone(),
            });
            self.mode = Mode::MetadataEdit;
        }
    }

    pub fn select_metadata_prev(&mut self) {
        let i = match self.metadata_list_state.selected() {
            Some(i) if i > 0 => i - 1,
            Some(i) => i,
            None => 0,
        };
        self.metadata_list_state.select(Some(i));
    }

    pub fn select_metadata_next(&mut self) {
        let i = match self.metadata_list_state.selected() {
            Some(i) if i + 1 < self.metadata_candidates.len() => i + 1,
            Some(i) => i,
            None => 0,
        };
        self.metadata_list_state.select(Some(i));
    }

    pub fn open_metadata_edit_for_candidate(&mut self) {
        let Some(index) = self.metadata_list_state.selected() else {
            return;
        };
        let Some(candidate) = self.metadata_candidates.get(index) else {
            return;
        };
        let Some(path) = self.candidate_path.clone() else {
            return;
        };

        self.metadata_form = Some(MetadataForm::from_candidate(candidate.as_ref()));
        self.edit_context = Some(EditContext::NewItem { path });
        self.mode = Mode::MetadataEdit;
    }

    pub fn open_metadata_edit_for_selected_item(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let form = MetadataForm::from_item(item);
        let id = item.id;

        self.metadata_form = Some(form);
        self.edit_context = Some(EditContext::ExistingItem { id });
        self.mode = Mode::MetadataEdit;
    }

    pub fn confirm_metadata_form(&mut self) {
        let Some(form) = self.metadata_form.take() else {
            return;
        };
        let Some(ctx) = self.edit_context.take() else {
            return;
        };

        self.saving = true;
        self.last_error = None;
        let archive = self.archive.clone();
        let tx = self.task_channel_tx.clone();
        let snapshot = form.snapshot();

        tokio::spawn(async move {
            let result = match ctx {
                EditContext::NewItem { path } => {
                    archive.save_item_from_form(&snapshot, &path).await
                }
                EditContext::ExistingItem { id } => {
                    archive.update_item_from_form(id, &snapshot).await
                }
            };
            let outcome = match result {
                Ok(()) => SaveOutcome::Saved,
                Err(e) => SaveOutcome::Failed(e.to_string()),
            };
            let _ = tx.send(Message::Save(outcome));
        });
    }

    pub async fn handle_save_message(&mut self, outcome: &SaveOutcome) -> color_eyre::Result<()> {
        self.saving = false;
        match outcome {
            SaveOutcome::Saved => {
                self.mode = Mode::Normal;
                self.metadata_candidates.clear();
                self.request_refresh_item_list().await?;
            }
            SaveOutcome::Failed(err) => self.last_error = Some(err.to_string()),
        }
        Ok(())
    }

    pub fn handle_abstract_message(&mut self, data: &AbstractData) {
        self.pending_covers.remove(&data.item_id);
        let Some(item) = self.items.iter_mut().find(|i| i.id == data.item_id) else {
            return;
        };
        match &data.abstract_text {
            Some(text) => {
                item.fields.description = Some(text.to_owned());
            }
            None => {
                self.failed_abstract.insert(data.item_id);
                if let Ok(notif) = Notification::new(format!(
                    "Failed to find abstract for '{}'",
                    item.fields.title
                ))
                .title(" Fetching Metadata ")
                .timing(
                    Timing::Fixed(Duration::from_millis(500)),
                    Timing::Fixed(Duration::from_secs(3)),
                    Timing::Fixed(Duration::from_millis(500)),
                )
                .max_size(SizeConstraint::Percentage(0.6), SizeConstraint::Absolute(4))
                .anchor(Anchor::TopRight)
                .animation(Animation::Slide)
                .level(Level::Warn)
                .slide_direction(SlideDirection::FromTop)
                .auto_dismiss(AutoDismiss::After(Duration::from_secs(2)))
                .build()
                {
                    let _ = self.notifications.add(notif);
                }
            }
        }
    }

    pub async fn poll_messages(&mut self) -> color_eyre::Result<()> {
        while let Ok(message) = self.task_channel_rx.try_recv() {
            match message {
                Message::Save(save_outcome) => self.handle_save_message(&save_outcome).await?,
                Message::Metadata(item_metadatas) => {
                    self.handle_metadata_search_message(item_metadatas)
                }
                Message::ImageCover(cover_data) => self.handle_cover_message(*cover_data),
                Message::Abstract(abstract_data) => self.handle_abstract_message(&abstract_data),
            }
        }
        Ok(())
    }

    pub fn cancel_metadata_form(&mut self) {
        self.metadata_form = None;
        self.edit_context = None;
        self.mode = Mode::Normal;
    }

    pub fn cancel_metadata_selection(&mut self) {
        self.metadata_candidates.clear();
        self.mode = Mode::Insert; // torna al file explorer
    }

    pub fn request_file_opening(&self) -> color_eyre::Result<()> {
        let Some(index) = self.items_list_state.selected() else {
            return Ok(());
        };

        let Some(item) = self.items.get(index) else {
            return Ok(());
        };

        let path = PathBuf::from(&item.path);
        opener::open(path)?;
        Ok(())
    }

    pub fn send_bibtex_to_system_clipboard(&mut self) {
        let Some(item) = self.selected_item() else {
            if let Ok(notif) = Notification::new("No selected item")
                .title(" Export citation ")
                .timing(
                    Timing::Fixed(Duration::from_millis(500)),
                    Timing::Fixed(Duration::from_secs(3)),
                    Timing::Fixed(Duration::from_millis(500)),
                )
                .max_size(SizeConstraint::Percentage(0.6), SizeConstraint::Absolute(4))
                .anchor(Anchor::TopRight)
                .animation(Animation::Slide)
                .level(Level::Error)
                .slide_direction(SlideDirection::FromTop)
                .auto_dismiss(AutoDismiss::After(Duration::from_secs(2)))
                .build()
            {
                let _ = self.notifications.add(notif);
            }
            return;
        };
        let bibtex = item.to_bibtex();
        let mut ctx = ClipboardContext::new().unwrap();
        let _ = ctx.set_contents(bibtex.to_owned());
        if let Ok(notif) = Notification::new("Copied Bibtex")
            .title("  Export citation ")
            .timing(
                Timing::Fixed(Duration::from_millis(500)),
                Timing::Fixed(Duration::from_secs(3)),
                Timing::Fixed(Duration::from_millis(500)),
            )
            .max_size(SizeConstraint::Percentage(0.6), SizeConstraint::Absolute(4))
            .anchor(Anchor::TopRight)
            .animation(Animation::Slide)
            .level(Level::Info)
            .slide_direction(SlideDirection::FromTop)
            .auto_dismiss(AutoDismiss::After(Duration::from_secs(2)))
            .build()
        {
            let _ = self.notifications.add(notif);
        }
    }

    pub fn request_abstract_for_selected(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        if item.fields.description.is_some()
            || self.failed_abstract.contains(&item.id)
            || self.pending_abstract.contains(&item.id)
        {
            return;
        }

        let id = item.id;
        let title = item.fields.title.clone();
        let doi = item.fields.doi.clone();
        let isbn = item.fields.isbn.clone();

        let provider = self.metadata_provider.clone();
        let archive = self.archive.clone();
        let tx = self.task_channel_tx.clone();
        self.pending_abstract.insert(id);
        tokio::spawn(async move {
            let abstract_text = provider.fetch_abstract(&title, doi, isbn).await;
            if let Some(text) = &abstract_text {
                let _ = archive.set_item_description(id, text).await.is_ok();
                let _ = tx.send(Message::Abstract(AbstractData {
                    item_id: id,
                    abstract_text,
                }));
            }
        });
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Items => Focus::Collections,
            Focus::Collections => Focus::Items,
        }
    }

    pub async fn request_refresh_collections(&mut self) -> color_eyre::Result<()> {
        self.collections = self.archive.get_all_collections().await?;
        Ok(())
    }

    pub fn select_collection_prev(&mut self) {
        let len = self.collections.len() + 1;
        let i = match self.collections_list_state.selected() {
            Some(i) if i > 0 => i - 1,
            Some(_) => len - 1,
            None => 0,
        };
        self.collections_list_state.select(Some(i));
    }

    pub fn select_collection_next(&mut self) {
        let len = self.collections.len() + 1;
        let i = match self.collections_list_state.selected() {
            Some(i) if i + 1 < len => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.collections_list_state.select(Some(i));
    }

    pub fn confirm_collection_selection(&mut self) {
        let Some(i) = self.collections_list_state.selected() else {
            return;
        };
        self.selected_collection = if i == 0 {
            None
        } else {
            self.collections.get(i - 1).map(|c| c.id)
        };
        self.items_list_state = ListState::default();
        if !self.items.is_empty() {
            self.items_list_state.select(Some(0));
        }
    }

    pub async fn delete_collection(&mut self) -> color_eyre::Result<()> {
        let Some(i) = self.collections_list_state.selected() else {
            return Ok(());
        };
        if i == 0 {
            return Ok(());
        }
        if let Some(collection_id) = self.collections.get(i - 1).map(|c| c.id) {
            self.archive.delete_collecton(collection_id).await?;
            self.items
                .iter_mut()
                .for_each(|i| i.collections.retain(|c| c.id != collection_id));
        }
        self.request_refresh_collections().await?;
        Ok(())
    }

    pub fn open_collection_create(&mut self) {
        self.collection_name_input = Input::default();
        self.mode = Mode::CollectionCreate;
    }

    pub fn close_collection_create(&mut self) {
        self.mode = Mode::Normal;
    }

    pub async fn confirm_collection_create(&mut self) -> color_eyre::Result<()> {
        let name = self.collection_name_input.to_string();
        if !name.trim().is_empty() {
            self.archive.create_collection(name.trim()).await?;
            self.request_refresh_collections().await?;
        }
        self.mode = Mode::Normal;
        Ok(())
    }

    pub fn open_item_assign_for_selected_collection(&mut self) {
        let Some(i) = self.collections_list_state.selected() else {
            return;
        };
        if i == 0 {
            return;
        }
        let Some(collection) = self.collections.get(i - 1) else {
            return;
        };
        let collection_id = collection.id;
        let original: HashSet<i32> = self
            .items
            .iter()
            .filter(|i| i.collections.iter().any(|c| c.id == collection_id))
            .map(|i| i.id)
            .collect();
        let selected = original.clone();
        let mut list_state = ListState::default();
        if !self.items.is_empty() {
            list_state.select(Some(0));
        }
        self.collection_assign = Some(CollectionAssignState {
            mode: AssignMode::Items,
            id: collection_id,
            original,
            selected,
            list_state,
        });
        self.mode = Mode::CollectionAssign;
    }

    pub fn open_collection_assign_for_selected(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let item_id = item.id;
        let original: HashSet<i32> = item.collections.iter().map(|c| c.id).collect();
        let selected = original.clone();

        let mut list_state = ListState::default();
        if !self.collections.is_empty() {
            list_state.select(Some(0));
        }

        self.collection_assign = Some(CollectionAssignState {
            mode: AssignMode::Collections,
            id: item_id,
            original,
            selected,
            list_state,
        });
        self.mode = Mode::CollectionAssign;
    }

    pub fn collection_assign_next(&mut self) {
        let Some(state) = &mut self.collection_assign else {
            return;
        };
        let len = match state.mode {
            AssignMode::Items => self.items.len(),
            AssignMode::Collections => self.collections.len(),
        };
        if len == 0 {
            return;
        }
        let i = match state.list_state.selected() {
            Some(i) if i + 1 < len => i + 1,
            _ => 0,
        };
        state.list_state.select(Some(i));
    }

    pub fn collection_assign_prev(&mut self) {
        let Some(state) = &mut self.collection_assign else {
            return;
        };
        let len = match state.mode {
            AssignMode::Items => self.items.len(),
            AssignMode::Collections => self.collections.len(),
        };
        if len == 0 {
            return;
        }
        let i = match state.list_state.selected() {
            Some(i) if i > 0 => i - 1,
            _ => len - 1,
        };
        state.list_state.select(Some(i));
    }

    pub fn collection_assign_toggle_current(&mut self) {
        let Some(state) = &mut self.collection_assign else {
            return;
        };

        let Some(index) = state.list_state.selected() else {
            return;
        };
        let id = match state.mode {
            AssignMode::Items => {
                let Some(item_id) = self.items.get(index).map(|i| i.id) else {
                    return;
                };
                item_id
            }
            AssignMode::Collections => {
                let Some(collection_id) = self.collections.get(index).map(|c| c.id) else {
                    return;
                };
                collection_id
            }
        };
        if !state.selected.remove(&id) {
            state.selected.insert(id);
        }
    }

    pub fn cancel_collection_assign(&mut self) {
        self.collection_assign = None;
        self.mode = Mode::Normal;
    }

    pub async fn confirm_collection_assign(&mut self) -> color_eyre::Result<()> {
        let Some(state) = self.collection_assign.take() else {
            self.mode = Mode::Normal;
            return Ok(());
        };

        match state.mode {
            AssignMode::Items => {
                for &item_id in state.selected.difference(&state.original) {
                    self.archive
                        .add_item_to_collection(item_id, state.id)
                        .await?;
                }
                for &item_id in state.original.difference(&state.selected) {
                    self.archive
                        .remove_item_from_collection(item_id, state.id)
                        .await?;
                }
            }
            AssignMode::Collections => {
                for &collection_id in state.selected.difference(&state.original) {
                    self.archive
                        .add_item_to_collection(state.id, collection_id)
                        .await?;
                }
                for &collection_id in state.original.difference(&state.selected) {
                    self.archive
                        .remove_item_from_collection(state.id, collection_id)
                        .await?;
                }
            }
        }

        self.mode = Mode::Normal;
        self.request_refresh_item_list().await?;
        Ok(())
    }
}
