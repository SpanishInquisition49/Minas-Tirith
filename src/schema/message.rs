use ratatui_image::protocol::StatefulProtocol;

use crate::metadata::dedup::MergedCandidate;

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
}
