use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use color_eyre::eyre::Context;
use ratatui::{
    style::{Color, Style},
    symbols::border,
    widgets::ListState,
};
use ratatui_explorer::{FileExplorer, FileExplorerBuilder};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use ratatui_notifications::{
    Anchor, Animation, AutoDismiss, Level, Notification, Notifications, SizeConstraint,
    SlideDirection, Timing,
};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    database::archive::Archive,
    metadata::{
        common_metadata::ItemMetadata, cover_generator::generate_cover, crosseref::CrossrefManager,
        image_cache::ImageCache, openlibrary::OpenLibraryManager, proxy::MetadataFetcher,
    },
    schema::{form::MetadataForm, item::DatabaseItem},
};

pub enum Mode {
    Normal,
    Insert,
    Search,
    MetadataSelect,
    MetadataEdit,
}

pub enum EditContext {
    NewItem { path: PathBuf },
    ExistingItem { id: i32 },
}

pub enum SaveOutcome {
    Saved,
    Failed(String),
}

pub struct App {
    pub notifications: Notifications,
    pub archive: Arc<Archive>,
    pub mode: Mode,
    pub items: Vec<DatabaseItem>,
    pub items_list_state: ListState,
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
    save_tx: Arc<UnboundedSender<SaveOutcome>>,
    save_rx: UnboundedReceiver<SaveOutcome>,

    pub is_searching: bool,
    metadata_search_tx: Arc<UnboundedSender<Vec<Box<dyn ItemMetadata>>>>,
    metadata_search_rx: UnboundedReceiver<Vec<Box<dyn ItemMetadata>>>,

    openlibrary: Arc<OpenLibraryManager>,
    crossref: Arc<CrossrefManager>,
    candidate_path: Option<PathBuf>,
    picker: Arc<Picker>,
    cache: Arc<ImageCache>,
    covers: HashMap<i32, StatefulProtocol>,
    pending_covers: HashSet<i32>,
    image_tx: Arc<UnboundedSender<(i32, StatefulProtocol, Option<String>)>>,
    image_rx: UnboundedReceiver<(i32, StatefulProtocol, Option<String>)>,
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
        let (image_tx, image_rx) = mpsc::unbounded_channel();
        let (save_tx, save_rx) = mpsc::unbounded_channel();
        let (metada_search_tx, metadata_search_rx) = mpsc::unbounded_channel();

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
            image_tx: Arc::new(image_tx),
            image_rx,
            crossref: Arc::new(CrossrefManager::new()),
            openlibrary: Arc::new(OpenLibraryManager::new()),
            metadata_candidates: Vec::new(),
            metadata_list_state: ListState::default(),
            candidate_path: None,
            metadata_form: None,
            edit_context: None,
            saving: false,
            last_error: None,
            save_tx: Arc::new(save_tx),
            save_rx,
            tick_counter: 0,
            is_searching: false,
            metadata_search_tx: Arc::new(metada_search_tx),
            metadata_search_rx,
        };
        app.request_refresh_item_list().await?;
        app.request_cover_for_selected();
        Ok(app)
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

    pub fn selected_item(&self) -> Option<&DatabaseItem> {
        self.items_list_state
            .selected()
            .and_then(|i| self.items.get(i))
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
        let tx = self.image_tx.clone();

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
                let _ = tx.send((id, protocol, None));
            }
        });
    }

    fn spawn_cover_generation(&mut self, id: i32, file_path: String) {
        self.pending_covers.insert(id);
        let cache = self.cache.clone();
        let picker = self.picker.clone();
        let archive = self.archive.clone();
        let tx = self.image_tx.clone();
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
                let _ = tx.send((id, protocol, Some(url)));
            }
        });
    }
    pub fn poll_covers(&mut self) {
        while let Ok((id, protocol, maybe_url)) = self.image_rx.try_recv() {
            self.pending_covers.remove(&id);
            self.covers.insert(id, protocol);
            if let Some(url) = maybe_url
                && let Some(item) = self.items.iter_mut().find(|i| i.id == id)
            {
                item.fields.cover_image_url = Some(url);
            }
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
        let tx = self.metadata_search_tx.clone();
        let openlibrary = self.openlibrary.clone();
        let crossref = self.crossref.clone();
        tokio::spawn(async move {
            let mut candidates: Vec<Box<dyn ItemMetadata>> = Vec::new();
            let books = openlibrary.fetch(&filename).await;
            if let Ok(books) = books {
                candidates.extend(
                    books
                        .into_iter()
                        .map(|b| Box::new(b) as Box<dyn ItemMetadata>),
                );
            }

            let articles = crossref.fetch(&filename).await;
            if let Ok(articles) = articles {
                candidates.extend(
                    articles
                        .into_iter()
                        .map(|a| Box::new(a) as Box<dyn ItemMetadata>),
                );
            }
            let _ = tx.send(candidates);
        });
    }

    pub fn poll_metadata_search(&mut self) {
        while let Ok(candidates) = self.metadata_search_rx.try_recv() {
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
        let tx = self.save_tx.clone();

        tokio::spawn(async move {
            let result = match ctx {
                EditContext::NewItem { path } => archive.save_item_from_form(&form, &path).await,
                EditContext::ExistingItem { id } => archive.update_item_from_form(id, &form).await,
            };
            let outcome = match result {
                Ok(()) => SaveOutcome::Saved,
                Err(e) => SaveOutcome::Failed(e.to_string()),
            };
            let _ = tx.send(outcome);
        });
    }

    pub async fn poll_save(&mut self) -> color_eyre::Result<()> {
        while let Ok(outcome) = self.save_rx.try_recv() {
            self.saving = false;
            match outcome {
                SaveOutcome::Saved => {
                    self.mode = Mode::Normal;
                    self.metadata_candidates.clear();
                    self.request_refresh_item_list().await?;
                }
                SaveOutcome::Failed(err) => self.last_error = Some(err),
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
}
