use std::{collections::HashSet, path::PathBuf, sync::Arc};

use ratatui::widgets::ListState;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    database::archive::Archive,
    metadata::{common_metadata::ItemMetadata, facade::MetadataProvider},
    schema::{
        form::MetadataForm,
        message::{AbstractData, Message, SaveOutcome},
    },
    tui::app::traits::ListWidget,
};

pub enum EditContext {
    NewItem { path: PathBuf },
    ExistingItem { id: i32 },
}

pub struct MetadataEditState {
    archive: Arc<Archive>,
    provider: Arc<MetadataProvider>,
    tx: Arc<UnboundedSender<Message>>,

    pub candidates: Vec<Box<dyn ItemMetadata>>,
    pub list_state: ListState,
    pub form: Option<MetadataForm>,
    pub edit_context: Option<EditContext>,
    pub saving: bool,
    pub is_searching: bool,
    pub last_error: Option<String>,
    pub candidate_path: Option<PathBuf>,
    pub pending_abstract: HashSet<i32>,
    pub failed_abstract: HashSet<i32>,
}

impl ListWidget<Box<dyn ItemMetadata>> for MetadataEditState {
    fn items(&self) -> &[Box<dyn ItemMetadata>] {
        &self.candidates
    }

    fn items_mut(&mut self) -> &mut Vec<Box<dyn ItemMetadata>> {
        &mut self.candidates
    }

    fn list_state(&self) -> &ListState {
        &self.list_state
    }

    fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }
}

impl MetadataEditState {
    pub fn new(
        archive: Arc<Archive>,
        provider: MetadataProvider,
        tx: Arc<UnboundedSender<Message>>,
    ) -> Self {
        Self {
            archive,
            provider: Arc::new(provider),
            tx,
            candidates: Vec::new(),
            list_state: ListState::default(),
            form: None,
            edit_context: None,
            saving: false,
            is_searching: false,
            last_error: None,
            candidate_path: None,
            pending_abstract: HashSet::default(),
            failed_abstract: HashSet::default(),
        }
    }

    // NOTE: ================== METADATA FETCHING ==================

    /// Kicks off a metadata search task for the given filename
    pub fn request_fetch_candidates(&mut self, filename: String) {
        self.is_searching = true;
        let tx = self.tx.clone();
        let provider = self.provider.clone();
        tokio::spawn(async move {
            let candidates = provider.fetch(&filename).await;
            let _ = tx.send(Message::Metadata(candidates));
        });
    }

    /// Applies search results. Returns `true` if there are candidates to pick from;
    /// if `false`, a black form for the new item has already be prepared as a fallback.
    pub fn on_search_results(
        &mut self,
        candidates: Vec<Box<dyn ItemMetadata>>,
        fallback_path: PathBuf,
    ) -> bool {
        self.is_searching = false;
        self.candidates = candidates;
        self.list_state = ListState::default();
        self.candidate_path = Some(fallback_path.clone());
        if !self.candidates.is_empty() {
            self.list_state.select(Some(0));
            true
        } else {
            self.form = Some(MetadataForm::new());
            self.edit_context = Some(EditContext::NewItem {
                path: fallback_path,
            });
            false
        }
    }

    // NOTE: ================== METADATA EDITING ==================

    pub fn open_edit_for_candidate(&mut self) -> bool {
        let Some(index) = self.list_state.selected() else {
            return false;
        };
        let Some(candidate) = self.candidates.get(index) else {
            return false;
        };
        let Some(path) = self.candidate_path.clone() else {
            return false;
        };

        self.form = Some(MetadataForm::from_candidate(candidate.as_ref()));
        self.edit_context = Some(EditContext::NewItem { path });
        true
    }

    pub fn clear_candidates(&mut self) {
        self.candidates.clear();
    }

    pub fn set_edit(&mut self, form: MetadataForm, ctx: EditContext) {
        self.form = Some(form);
        self.edit_context = Some(ctx);
    }

    pub fn cancel_form(&mut self) {
        self.form = None;
        self.edit_context = None;
    }

    /// Kicks off a saving task in the background
    pub fn confirm_save(&mut self) {
        let Some(form) = self.form.take() else {
            return;
        };
        let Some(ctx) = self.edit_context.take() else {
            return;
        };

        self.saving = true;
        self.last_error = None;
        let archive = self.archive.clone();
        let tx = self.tx.clone();
        let snapshot = form.snapshot();
        tokio::spawn(async move {
            let result = match ctx {
                EditContext::NewItem { path } => {
                    archive.save_item_from_form(&snapshot, &path).await
                }
                EditContext::ExistingItem { id } => {
                    archive.update_item_from_form(id, &snapshot).await
                }
            };
            let outcome = match result {
                Ok(_) => SaveOutcome::Saved,
                Err(e) => SaveOutcome::Failed(e.to_string()),
            };

            let _ = tx.send(Message::Save(outcome));
        });
    }

    pub fn on_save_result(&mut self, outcome: &SaveOutcome) -> bool {
        self.saving = false;
        match outcome {
            SaveOutcome::Saved => {
                self.candidates.clear();
                true
            }
            SaveOutcome::Failed(err) => {
                self.last_error = Some(err.to_string());
                false
            }
        }
    }

    // NOTE: ================== ABSTRACT FETCHING ==================

    /// Kicks off an abstract fetching task if the item doesn't already have it or if a task failed
    /// before or if a task is in flight
    pub fn request_abstract(
        &mut self,
        id: i32,
        has_description: bool,
        title: String,
        doi: Option<String>,
        isbn: Option<String>,
    ) {
        if has_description
            || self.pending_abstract.contains(&id)
            || self.failed_abstract.contains(&id)
        {
            return;
        }

        let archive = self.archive.clone();
        let provider = self.provider.clone();
        let tx = self.tx.clone();
        self.pending_abstract.insert(id);

        tokio::spawn(async move {
            let abstract_text = provider.fetch_abstract(&title, doi, isbn).await;
            if let Some(a) = &abstract_text {
                let _ = archive.set_item_description(id, a).await;
            }
            let _ = tx.send(Message::Abstract(AbstractData {
                item_id: id,
                abstract_text,
            }));
        });
    }

    pub fn on_abstract_result(&mut self, data: &AbstractData) -> bool {
        self.pending_abstract.remove(&data.item_id);
        if data.abstract_text.is_none() {
            self.failed_abstract.insert(data.item_id);
        }
        data.abstract_text.is_some()
    }
}
