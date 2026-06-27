use std::collections::{HashMap, HashSet};

use color_eyre::eyre::Context;
use ratatui::widgets::ListState;
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    database::archive::Archive, metadata::image_cache::ImageCache, schema::item::DatabaseItem,
};

pub enum Mode {
    Normal,
    Search,
}

pub struct App {
    pub archive: Archive,
    pub mode: Mode,
    pub items: Vec<DatabaseItem>,
    pub list_state: ListState,
    pub search_query: String,
    pub quit: bool,

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
        let items = archive.get_all_items().await?;
        let mut list = ListState::default();
        if !items.is_empty() {
            list.select_first();
        }

        let (image_tx, image_rx) = mpsc::unbounded_channel();

        let mut app = Self {
            archive,
            picker,
            cache,
            mode: Mode::Normal,
            items,
            list_state: list,
            search_query: String::new(),
            quit: false,
            covers: HashMap::new(),
            pending_covers: HashSet::new(),
            image_tx,
            image_rx,
        };
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
}
