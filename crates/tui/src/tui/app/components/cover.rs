use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::Arc,
};

use color_eyre::eyre::Context;
use minastirith_core::{
    database::Archive,
    metadata::{cover_generator::generate_cover, image_cache::ImageCache},
    schema::item::DatabaseItem,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use tokio::sync::mpsc::UnboundedSender;

pub struct CoverImageData {
    item_id: i32,
    protocol: StatefulProtocol,
    url: Option<String>,
}

pub enum CoverMessage {
    Ready(Box<CoverImageData>),
    Failed { item_id: i32 },
}

pub struct CoverComponent {
    archive: Arc<Archive>,
    picker: Arc<Picker>,
    cache: Arc<ImageCache>,
    tx: Arc<UnboundedSender<CoverMessage>>,
    covers: HashMap<i32, StatefulProtocol>,
    pending: HashSet<i32>,
    failed: HashMap<i32, usize>,
}

impl CoverComponent {
    pub fn new(
        archive: Arc<Archive>,
        picker: Picker,
        cache: ImageCache,
        tx: Arc<UnboundedSender<CoverMessage>>,
    ) -> Self {
        Self {
            archive,
            picker: Arc::new(picker),
            cache: Arc::new(cache),
            tx,
            covers: HashMap::default(),
            pending: HashSet::default(),
            failed: HashMap::default(),
        }
    }

    /// Kicks off a download or generation task if not already in cache or pin flight
    pub fn request(&mut self, id: i32, cover_url: Option<String>, file_path: String) {
        if self.covers.contains_key(&id)
            || self.pending.contains(&id)
            || self.too_many_failures(&id)
        {
            return;
        }

        match cover_url {
            Some(url) => self.spawn_download(id, url),
            None => self.spawn_generation(id, file_path),
        }
    }

    /// Don't kicks off another request if too many have failed before
    fn too_many_failures(&self, item_id: &i32) -> bool {
        match self.failed.get(item_id) {
            Some(f) => *f >= 5,
            None => false,
        }
    }

    /// Applies an inbound 'ItemCover' message: stores the decoded protocol
    /// and, for the generated covers, persist the URL on the item
    pub fn handle_ready(&mut self, data: CoverImageData, items: &mut [DatabaseItem]) {
        self.pending.remove(&data.item_id);
        self.covers.insert(data.item_id, data.protocol);
        if let Some(url) = data.url
            && let Some(item) = items.iter_mut().find(|i| i.id == data.item_id)
        {
            item.fields.cover_image_url = Some(url);
        }
    }

    pub fn handle_failed(&mut self, item_id: i32) {
        self.pending.remove(&item_id);
        match self.failed.get(&item_id) {
            Some(f) => self.failed.insert(item_id, f + 1),
            None => self.failed.insert(item_id, 0),
        };
    }

    pub fn get_mut(&mut self, item_id: i32) -> Option<&mut StatefulProtocol> {
        self.covers.get_mut(&item_id)
    }

    fn spawn_download(&mut self, id: i32, url: String) {
        self.pending.insert(id);
        let cache = self.cache.clone();
        let picker = self.picker.clone();
        let tx = self.tx.clone();

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

            let message = match result {
                Ok(protocol) => CoverMessage::Ready(Box::new(CoverImageData {
                    item_id: id,
                    protocol,
                    url: None,
                })),
                Err(e) => {
                    tracing::warn!(error = %e, item_id = id, "Download cover failed");
                    CoverMessage::Failed { item_id: id }
                }
            };
            if let Err(e) = tx.send(message) {
                tracing::error!(error = %e, item_id = id, "Failed to send the cover download result to the main task")
            }
        });
    }

    fn spawn_generation(&mut self, id: i32, file_path: String) {
        self.pending.insert(id);
        let cache = self.cache.clone();
        let picker = self.picker.clone();
        let archive = self.archive.clone();
        let tx = self.tx.clone();
        let path = PathBuf::from(file_path);

        tokio::spawn(async move {
            let result: color_eyre::Result<(StatefulProtocol, String)> = async {
                let Some(cover_path) = generate_cover(&path, id, &cache).await? else {
                    return Err(color_eyre::eyre::eyre!("No cover could be generated"));
                };
                let url = format!("file://{}", cover_path.display());
                archive.set_cover_image_url(id, &url).await?;

                let dyn_image = tokio::task::spawn_blocking(move || image::open(&cover_path))
                    .await
                    .context("Joining image decode task")?
                    .context("Decode generated cover image")?;
                Ok((picker.new_resize_protocol(dyn_image), url))
            }
            .await;

            let message = match result {
                Ok((protocol, url)) => CoverMessage::Ready(Box::new(CoverImageData {
                    item_id: id,
                    protocol,
                    url: Some(url),
                })),
                Err(e) => {
                    tracing::warn!(error = %e, item_id = id, "Cover generation failed");
                    CoverMessage::Failed { item_id: id }
                }
            };
            if let Err(e) = tx.send(message) {
                tracing::error!(error = %e, item_id = id, "Failed to send the cover generation result to the main task")
            }
        });
    }
}
