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
    /// Construct a [`MetadataComponent`] backed by `archive`, fetching
    /// candidates via `provider` and reporting results via `tx`.
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

    /// The underlying [`MetadataState`].
    pub fn core(&self) -> &MetadataState<TuiMetadataForm> {
        &self.core
    }

    /// Mutable access to the underlying [`MetadataState`].
    pub fn core_mut(&mut self) -> &mut MetadataState<TuiMetadataForm> {
        &mut self.core
    }

    /// Mutable access to the metadata candidate list's ratatui
    /// `ListState`.
    pub fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }
}
