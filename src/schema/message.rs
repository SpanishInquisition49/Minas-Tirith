use std::path::PathBuf;

use ratatui_image::protocol::StatefulProtocol;
use uuid::Uuid;

use crate::{
    metadata::dedup::MergedCandidate,
    peer2peer::{discovery::PeerInfo, library::SharedItemEntry},
};

#[derive(Debug)]
pub enum SaveOutcome {
    Saved { was_update: bool },
    Failed { reason: String, was_update: bool },
}

pub struct CoverImageData {
    pub item_id: i32,
    pub protocol: StatefulProtocol,
    pub url: Option<String>,
}

pub struct AbstractData {
    pub item_id: i32,
    pub abstract_text: Option<String>,
    pub success: bool,
}

pub struct LibraryItemsDiscovered {
    pub namespace_id: String,
    pub items: Vec<SharedItemEntry>,
}

pub struct LibraryDownloadReady {
    pub entry: SharedItemEntry,
    pub local_path: PathBuf,
}

pub struct LibraryDownloadFailed {
    pub item_id: Uuid,
    pub reason: String,
}

pub enum Message {
    Save(SaveOutcome),
    Metadata(Vec<MergedCandidate>),
    ImageCover(Box<CoverImageData>),
    ImageCoverFailed { item_id: i32 },
    Abstract(AbstractData),
    PeerDiscovered(PeerInfo),
    PeerExpired(PeerInfo),
    LibraryItemsDiscovered(Box<LibraryItemsDiscovered>),
    ItemDownloadReady(Box<LibraryDownloadReady>),
    ItemDownloadFailed(Box<LibraryDownloadFailed>),
}
