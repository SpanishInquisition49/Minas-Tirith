use std::collections::{HashMap, HashSet};

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
        crosseref::CrossrefManager, image_cache::ImageCache, openlibrary::OpenLibraryManager,
        proxy::MetadataFetcher,
    },
    schema::item::DatabaseItem,
};

pub enum Mode {
    Normal,
    Insert,
    Search,
}

pub struct App {
    pub archive: Archive,
    pub mode: Mode,
    pub items: Vec<DatabaseItem>,
    pub list_state: ListState,
    pub search_query: String,
    pub quit: bool,
    pub file_explorer: FileExplorer,

    openlibrary: OpenLibraryManager,
    crossref: CrossrefManager,
    picker: Picker,
    cache: ImageCache,
    covers: HashMap<i32, StatefulProtocol>,
    pending_covers: HashSet<i32>,
    image_tx: UnboundedSender<(i32, StatefulProtocol)>,
    image_rx: UnboundedReceiver<(i32, StatefulProtocol)>,
}

impl App {
    pub async fn new(
        archive: Archive,
        picker: Picker,
        cache: ImageCache,
    ) -> color_eyre::Result<Self> {
        let mut list = ListState::default();
        let theme = Theme::default()
            .with_title_top(|_| Line::from(" Add new item ".bold()))
            .with_title_bottom(|_| {
                Line::from(vec![" Select: ".into(), "<C-S> ".blue()]).right_aligned()
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
            list_state: list,
            search_query: String::new(),
            quit: false,
            covers: HashMap::new(),
            pending_covers: HashSet::new(),
            image_tx,
            image_rx,
            crossref: CrossrefManager::new(),
            openlibrary: OpenLibraryManager::new(),
        };
        app.request_refresh_item_list().await?;
        app.request_cover_for_selected();
        Ok(app)
    }

    pub fn select_prev(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) if i > 0 => i - 1,
            Some(i) => i,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) if i + 1 < self.items.len() => i + 1,
            Some(i) => i,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn selected_item(&self) -> Option<&DatabaseItem> {
        self.list_state.selected().and_then(|i| self.items.get(i))
    }

    pub fn request_cover_for_selected(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let Some(url) = item.fields.cover_image_url.clone() else {
            return;
        };
        let id = item.id;

        if self.covers.contains_key(&id) || self.pending_covers.contains(&id) {
            return;
        }
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
                let _ = tx.send((id, protocol));
            }
        });
    }

    pub fn poll_covers(&mut self) {
        while let Ok((id, protocol)) = self.image_rx.try_recv() {
            self.pending_covers.remove(&id);
            self.covers.insert(id, protocol);
        }
    }

    pub fn selected_cover(&mut self) -> Option<&mut StatefulProtocol> {
        let id = self.selected_item()?.id;
        self.covers.get_mut(&id)
    }

    pub async fn request_refresh_item_list(&mut self) -> color_eyre::Result<()> {
        self.items = self.archive.get_all_items().await?;
        Ok(())
    }

    pub async fn request_add_file(&mut self) -> color_eyre::Result<()> {
        let file = self.file_explorer.current();
        if !file.is_file() {
            return Ok(());
        }
        let metadata = self.openlibrary.fetch(&file.name).await?;
        if let Some(item) = metadata.first() {
            self.archive.add_item(item).await?;
            return Ok(());
        }

        let metadata = self.crossref.fetch(&file.name).await?;
        if let Some(item) = metadata.first() {
            self.archive.add_item(item).await?;
        }

        Ok(())
    }
}
