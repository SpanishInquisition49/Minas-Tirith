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
    tui::app::{App, Mode},
};

impl App {
    pub fn is_saving_metadata(&self) -> bool {
        self.metadata_component.core().is_saving()
    }

    pub fn is_searching_metadata(&self) -> bool {
        self.metadata_component.core().is_searching()
    }

    pub fn get_metadata_last_error(&self) -> Option<Cow<'_, str>> {
        self.metadata_component.core().last_error()
    }

    pub fn get_metadata_form_mut(&mut self) -> &mut Option<TuiMetadataForm> {
        self.metadata_component.core_mut().form_mut()
    }

    pub fn get_metadata_candidates(&self) -> &[MergedCandidate] {
        &self.metadata_component.core().items()
    }

    pub fn get_metadata_list_state_mut(&mut self) -> &mut ListState {
        self.metadata_component.list_state_mut()
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
        self.metadata_component
            .core_mut()
            .request_fetch_candidates(filename);
    }

    pub fn select_metadata_prev(&mut self) {
        self.metadata_component.core_mut().select_prev();
    }

    pub fn select_metadata_next(&mut self) {
        self.metadata_component.core_mut().select_next();
    }

    pub fn open_metadata_edit_for_candidate(&mut self) {
        if self.metadata_component.core_mut().open_edit_for_candidate() {
            self.mode = Mode::MetadataEdit;
        }
    }

    pub fn cancel_metadata_selection(&mut self) {
        self.metadata_component.core_mut().clear_candidates();
        self.mode = Mode::Insert;
    }

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

    pub fn confirm_metadata_form(&mut self) {
        self.metadata_component.core_mut().confirm_save();
    }

    pub fn cancel_metadata_form(&mut self) {
        self.metadata_component.core_mut().cancel_form();
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
        self.metadata_component
            .core_mut()
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
}
