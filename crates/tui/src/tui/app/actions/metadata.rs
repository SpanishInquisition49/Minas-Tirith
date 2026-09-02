use std::borrow::Cow;

use minastirith_core::{
    metadata::dedup::MergedCandidate,
    state::metadata::EditContext,
    traits::{MetadataForm, Selectable},
};
use ratatui::widgets::ListState;
use ratatui_image::protocol::StatefulProtocol;

use crate::{
    schema::tui_metadata_form::TuiMetadataForm,
    traits::SelectableSync,
    tui::app::{App, Mode},
};

impl App {
    /// Whether a metadata save is currently in flight.
    pub fn is_saving_metadata(&self) -> bool {
        self.metadata_component.core().is_saving()
    }

    /// Whether a metadata search is currently in flight.
    pub fn is_searching_metadata(&self) -> bool {
        self.metadata_component.core().is_searching()
    }

    /// The last error reported by metadata fetching/saving, if any.
    pub fn get_metadata_last_error(&self) -> Option<Cow<'_, str>> {
        self.metadata_component.core().last_error()
    }

    /// Mutable access to the form currently being edited, if any.
    pub fn get_metadata_form_mut(&mut self) -> &mut Option<TuiMetadataForm> {
        self.metadata_component.core_mut().form_mut()
    }

    /// The metadata candidates from the last search.
    pub fn get_metadata_candidates(&self) -> &[MergedCandidate] {
        self.metadata_component.core().items()
    }

    /// Mutable access to the metadata candidate list's ratatui
    /// `ListState`.
    pub fn get_metadata_list_state_mut(&mut self) -> &mut ListState {
        self.metadata_component.list_state_mut()
    }

    /// Kick off a metadata search for the file currently highlighted in
    /// the file picker. No-op if the highlighted entry isn't a file.
    pub fn request_fetch_metadata_candidates(&mut self) {
        let file = self.file_explorer.current();
        if !file.is_file() {
            return;
        }
        let filename = file
            .path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        self.metadata_component
            .core_mut()
            .request_fetch_candidates(filename);
    }

    /// Select the previous metadata candidate.
    pub fn select_metadata_prev(&mut self) {
        self.metadata_component.select_prev();
    }

    /// Select the next metadata candidate.
    pub fn select_metadata_next(&mut self) {
        self.metadata_component.select_next();
    }

    /// Open the edit form pre-filled from the currently selected metadata
    /// candidate, switching to metadata-edit mode.
    pub fn open_metadata_edit_for_candidate(&mut self) {
        if self.metadata_component.core_mut().open_edit_for_candidate() {
            self.mode = Mode::MetadataEdit;
        }
    }

    /// Discard the current metadata candidates and return to insert mode.
    pub fn cancel_metadata_selection(&mut self) {
        self.metadata_component.core_mut().clear_candidates();
        self.mode = Mode::Insert;
    }

    /// Open the edit form pre-filled from the currently selected item,
    /// switching to metadata-edit mode.
    pub fn open_metadata_edit_for_selected_item(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let form = TuiMetadataForm::from_candidate(item);
        let id = item.id;
        self.metadata_component
            .core_mut()
            .set_edit(form, EditContext::ExistingItem { id });
        self.mode = Mode::MetadataEdit;
    }

    /// Confirm the metadata form and kick off a save.
    pub fn confirm_metadata_form(&mut self) {
        self.metadata_component.core_mut().confirm_save();
    }

    /// Discard the metadata form being edited and return to normal mode.
    pub fn cancel_metadata_form(&mut self) {
        self.metadata_component.core_mut().cancel_form();
        self.mode = Mode::Normal;
    }

    /// Kick off an abstract-fetch task for the currently selected item.
    pub fn request_abstract_for_selected(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let id = item.id;
        let has_description = item.fields.description.is_some();
        let title = item.fields.title.clone();
        let doi = item.fields.doi.clone();
        let isbn = item.fields.isbn.clone();
        self.metadata_component
            .core_mut()
            .request_abstract(id, has_description, title, doi, isbn);
    }

    /// Kick off a cover-image request for the currently selected item.
    pub fn request_cover_for_selected(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let id = item.id;
        let cover_url = item.fields.cover_image_url.clone();
        let path = item.path.clone();
        self.covers.request(id, cover_url, path);
    }

    /// The decoded cover image for the currently selected item, if any.
    pub fn selected_cover(&mut self) -> Option<&mut StatefulProtocol> {
        let id = self.selected_item()?.id;
        self.covers.get_mut(id)
    }
}
