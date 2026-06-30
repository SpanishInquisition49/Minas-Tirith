use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use color_eyre::eyre::Context;
use ratatui::{
    style::{Modifier, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, ListState},
};
use ratatui_explorer::{FileExplorer, FileExplorerBuilder, Theme};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    database::archive::Archive,
    metadata::{
        common_metadata::ItemMetadata, cover_generator::generate_cover, crosseref::CrossrefManager,
        image_cache::ImageCache, openlibrary::OpenLibraryManager, proxy::MetadataFetcher,
    },
    schema::item::DatabaseItem,
};

pub enum Mode {
    Normal,
    Insert,
    Search,
    MetadataSelect,
}

pub struct App {
    pub archive: Archive,
    pub mode: Mode,
    pub items: Vec<DatabaseItem>,
    pub items_list_state: ListState,
    pub search_query: String,
    pub quit: bool,
    pub file_explorer: FileExplorer,
    // Metadata for new items
    pub metadata_candidates: Vec<Box<dyn ItemMetadata>>,
    pub metadata_list_state: ListState,

    openlibrary: OpenLibraryManager,
    crossref: CrossrefManager,
    candidate_path: Option<PathBuf>,
    picker: Picker,
    cache: ImageCache,
    covers: HashMap<i32, StatefulProtocol>,
    pending_covers: HashSet<i32>,
    image_tx: UnboundedSender<(i32, StatefulProtocol, Option<String>)>,
    image_rx: UnboundedReceiver<(i32, StatefulProtocol, Option<String>)>,
}

impl App {
    pub async fn new(
        archive: Archive,
        picker: Picker,
        cache: ImageCache,
    ) -> color_eyre::Result<Self> {
        let list = ListState::default();
        let theme = Theme::default()
            .with_title_top(|_| Line::from(" Add new item ".bold()))
            .with_title_bottom(|_| {
                Line::from(vec![" Select: ".into(), "<C-Enter> ".blue()]).right_aligned()
            })
            .with_block(Block::bordered().border_set(border::THICK).blue())
            .with_highlight_item_style(Style::default().add_modifier(Modifier::REVERSED).yellow())
            .with_highlight_dir_style(Style::default().add_modifier(Modifier::REVERSED).yellow());
        let explorer = FileExplorerBuilder::default()
            .theme(theme)
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

        let mut app = Self {
            archive,
            picker,
            cache,
            file_explorer: explorer,
            mode: Mode::Normal,
            items: Vec::new(),
            items_list_state: list,
            search_query: String::new(),
            quit: false,
            covers: HashMap::new(),
            pending_covers: HashSet::new(),
            image_tx,
            image_rx,
            crossref: CrossrefManager::new(),
            openlibrary: OpenLibraryManager::new(),
            metadata_candidates: Vec::new(),
            metadata_list_state: ListState::default(),
            candidate_path: None,
        };
        app.request_refresh_item_list().await?;
        app.request_cover_for_selected();
        Ok(app)
    }

    pub fn select_prev(&mut self) {
        let i = match self.items_list_state.selected() {
            Some(i) if i > 0 => i - 1,
            Some(i) => i,
            None => 0,
        };
        self.items_list_state.select(Some(i));
    }

    pub fn select_next(&mut self) {
        let i = match self.items_list_state.selected() {
            Some(i) if i + 1 < self.items.len() => i + 1,
            Some(i) => i,
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

    pub async fn request_fetch_metadata_candidates(&mut self) -> color_eyre::Result<()> {
        let file = self.file_explorer.current();
        if !file.is_file() {
            return Ok(());
        }

        let mut candidates: Vec<Box<dyn ItemMetadata>> = Vec::new();

        let filename = file
            .path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let books = self.openlibrary.fetch(&filename).await?;
        candidates.extend(
            books
                .into_iter()
                .map(|b| Box::new(b) as Box<dyn ItemMetadata>),
        );

        let articles = self.crossref.fetch(&filename).await?;
        candidates.extend(
            articles
                .into_iter()
                .map(|a| Box::new(a) as Box<dyn ItemMetadata>),
        );

        self.metadata_candidates = candidates;
        self.metadata_list_state = ListState::default();
        self.candidate_path = Some(file.path.clone());

        if !self.metadata_candidates.is_empty() {
            self.metadata_list_state.select(Some(0));
            self.mode = Mode::MetadataSelect;
        }

        Ok(())
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

    pub async fn confirm_metadata_selection(&mut self) -> color_eyre::Result<()> {
        let Some(index) = self.metadata_list_state.selected() else {
            return Ok(());
        };
        let Some(candidate) = self.metadata_candidates.get(index) else {
            return Ok(());
        };

        let Some(candidate_path) = &self.candidate_path else {
            return Ok(());
        };
        self.archive
            .add_item(candidate.as_ref(), candidate_path)
            .await?;
        self.metadata_candidates.clear();
        self.request_refresh_item_list().await?;
        self.mode = Mode::Normal;
        Ok(())
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
