use ratatui::widgets::ListState;
use ratatui_image::protocol::StatefulProtocol;

use crate::{
    metadata::dedup::MergedCandidate,
    schema::form::MetadataForm,
    tui::app::{App, Mode, components::metadata_edit::EditContext, traits::ListWidget},
};

impl App {
    pub fn get_metadata_candidates(&self) -> &[MergedCandidate] {
        &self.metadata.candidates
    }

    pub fn get_metadata_list_state_mut(&mut self) -> &mut ListState {
        &mut self.metadata.list_state
    }

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
        self.metadata.request_fetch_candidates(filename);
    }

    pub fn select_metadata_prev(&mut self) {
        self.metadata.select_prev();
    }

    pub fn select_metadata_next(&mut self) {
        self.metadata.select_next();
    }

    pub fn open_metadata_edit_for_candidate(&mut self) {
        if self.metadata.open_edit_for_candidate() {
            self.mode = Mode::MetadataEdit;
        }
    }

    pub fn cancel_metadata_selection(&mut self) {
        self.metadata.clear_candidates();
        self.mode = Mode::Insert;
    }

    pub fn open_metadata_edit_for_selected_item(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let form = MetadataForm::from_item(item);
        let id = item.id;
        self.metadata
            .set_edit(form, EditContext::ExistingItem { id });
        self.mode = Mode::MetadataEdit;
    }

    pub fn confirm_metadata_form(&mut self) {
        self.metadata.confirm_save();
    }

    pub fn cancel_metadata_form(&mut self) {
        self.metadata.cancel_form();
        self.mode = Mode::Normal;
    }

    pub fn request_abstract_for_selected(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let id = item.id;
        let has_description = item.fields.description.is_some();
        let title = item.fields.title.clone();
        let doi = item.fields.doi.clone();
        let isbn = item.fields.isbn.clone();
        self.metadata
            .request_abstract(id, has_description, title, doi, isbn);
    }

    pub fn request_cover_for_selected(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let id = item.id;
        let cover_url = item.fields.cover_image_url.clone();
        let path = item.path.clone();
        self.covers.request(id, cover_url, path);
    }

    pub fn selected_cover(&mut self) -> Option<&mut StatefulProtocol> {
        let id = self.selected_item()?.id;
        self.covers.get_mut(id)
    }

    pub fn is_searching_metadata(&self) -> bool {
        self.metadata.is_searching
    }

    pub fn is_saving_metadata(&self) -> bool {
        self.metadata.saving
    }

    pub fn get_metadata_form_mut(&mut self) -> Option<&mut MetadataForm> {
        self.metadata.form.as_mut()
    }

    pub fn get_metadata_last_error(&self) -> Option<&String> {
        self.metadata.last_error.as_ref()
    }
}
