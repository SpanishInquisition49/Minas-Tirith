use ratatui_image::protocol::StatefulProtocol;

use crate::metadata::common_metadata::ItemMetadata;

#[derive(Debug)]
pub enum SaveOutcome {
    Saved,
    Failed(String),
}

pub struct CoverImageData {
    pub item_id: i32,
    pub protocol: StatefulProtocol,
    pub url: Option<String>,
}

pub struct AbstractData {
    pub item_id: i32,
    pub abstract_text: Option<String>,
}

pub enum Message {
    Save(SaveOutcome),
    Metadata(Vec<Box<dyn ItemMetadata>>),
    ImageCover(Box<CoverImageData>),
    Abstract(AbstractData),
}
