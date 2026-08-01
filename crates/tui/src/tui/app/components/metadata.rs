use std::sync::Arc;

use minastirith_core::{
    database::Archive, metadata::facade::MetadataProvider, schema::message::Message,
    state::metadata::MetadataState,
};
use ratatui::widgets::ListState;
use tokio::sync::mpsc::UnboundedSender;

use crate::schema::tui_metadata_form::TuiMetadataForm;

pub struct MetadataComponent {
    core: MetadataState<TuiMetadataForm>,
    list_state: ListState,
}

impl MetadataComponent {
    pub fn new(
        archive: Arc<Archive>,
        provider: Arc<MetadataProvider>,
        tx: Arc<UnboundedSender<Message>>,
    ) -> Self {
        Self {
            core: MetadataState::new(archive, provider, tx),
            list_state: ListState::default(),
        }
    }

    pub fn core(&self) -> &MetadataState<TuiMetadataForm> {
        &self.core
    }

    pub fn core_mut(&mut self) -> &mut MetadataState<TuiMetadataForm> {
        &mut self.core
    }

    pub fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }
}
