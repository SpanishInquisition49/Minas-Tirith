use std::path::PathBuf;

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
    Abstract(AbstractData),
    PeerDiscovered(PeerInfo),
    PeerExpired(PeerInfo),
    LibraryItemsDiscovered(Box<LibraryItemsDiscovered>),
    ItemDownloadReady(Box<LibraryDownloadReady>),
    ItemDownloadFailed(Box<LibraryDownloadFailed>),
}
