use std::path::PathBuf;

use ratatui_image::protocol::StatefulProtocol;
use uuid::Uuid;

use crate::{
    metadata::dedup::MergedCandidate,
    peer2peer::{discovery::PeerInfo, library::SharedPaperEntry},
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

pub enum Message {
    Save(SaveOutcome),
    Metadata(Vec<MergedCandidate>),
    ImageCover(Box<CoverImageData>),
    Abstract(AbstractData),
    PeerDiscovered(PeerInfo),
    PeerExpired(PeerInfo),
    LibraryPapersDiscovered {
        namespace_id: String,
        papers: Vec<SharedPaperEntry>,
    },
    PaperDownloadReady {
        entry: SharedPaperEntry,
        local_path: PathBuf,
    },
    PaperDownloadFailed {
        paper_id: Uuid,
        reason: String,
    },
}
