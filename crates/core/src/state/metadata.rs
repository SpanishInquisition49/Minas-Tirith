use std::{borrow::Cow, collections::HashSet, path::PathBuf, sync::Arc};

use minastirith_core_derive::Selectable;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    database::Archive,
    metadata::{dedup::MergedCandidate, facade::MetadataProvider},
    schema::message::{AbstractData, Message, SaveOutcome},
    traits::MetadataForm,
};

#[derive(PartialEq, Eq, Debug)]
pub enum EditContext {
    NewItem { path: PathBuf },
    ExistingItem { id: i32 },
}

/// `MetadataState` handle the fetching and editing metadata of items
#[derive(Selectable)]
#[select(candidates, selected_candidate_index)]
pub struct MetadataState<T: MetadataForm> {
    archive: Arc<Archive>,
    provider: Arc<MetadataProvider>,
    tx: Arc<UnboundedSender<Message>>,

    candidates: Vec<MergedCandidate>,
    selected_candidate_index: Option<usize>,

    form: Option<T>,
    edit_context: Option<EditContext>,
    candidate_path: Option<PathBuf>,

    is_saving: bool,
    is_searching: bool,
    last_error: Option<String>,

    pending_abstract: HashSet<i32>,
    failed_abstract: HashSet<i32>,
}

impl<T: MetadataForm> MetadataState<T> {
    pub fn new(
        archive: Arc<Archive>,
        provider: Arc<MetadataProvider>,
        tx: Arc<UnboundedSender<Message>>,
    ) -> Self {
        Self {
            archive,
            provider,
            tx,
            candidates: Vec::default(),
            selected_candidate_index: None,
            form: None,
            edit_context: None,
            candidate_path: None,
            is_saving: false,
            is_searching: false,
            last_error: None,
            pending_abstract: HashSet::default(),
            failed_abstract: HashSet::default(),
        }
    }

    pub fn is_saving(&self) -> bool {
        self.is_saving
    }

    pub fn is_searching(&self) -> bool {
        self.is_searching
    }

    pub fn last_error(&self) -> Option<Cow<'_, str>> {
        self.last_error.as_deref().map(Cow::Borrowed)
    }

    pub fn form_mut(&mut self) -> &mut Option<T> {
        &mut self.form
    }

    // NOTE: ================== METADATA FETCHING ==================

    /// Kicks off a metadata search task for the given filename
    pub fn request_fetch_candidates(&mut self, filename: String) {
        self.is_searching = true;
        let tx = self.tx.clone();
        let provider = self.provider.clone();
        tokio::spawn(async move {
            let candidates = provider.fetch(&filename).await;
            if let Err(e) = tx.send(Message::Metadata(candidates)) {
                tracing::error!(error = %e, "Failed to send metadata candidates to the main task");
            }
        });
    }

    /// Applies search results. Returns `true` if there are candidates to pick from;
    /// if `false`, a blank form for the new item is prepared as a fallback.
    pub fn on_search_results(
        &mut self,
        candidates: Vec<MergedCandidate>,
        fallback_path: PathBuf,
    ) -> bool {
        self.is_searching = false;
        self.candidates = candidates;
        self.candidate_path = Some(fallback_path.clone());
        if self.candidates.is_empty() {
            self.form = Some(T::default());
            self.edit_context = Some(EditContext::NewItem {
                path: fallback_path,
            });
            false
        } else {
            self.selected_candidate_index.replace(0);
            true
        }
    }

    // NOTE: ================== METADATA EDITING ==================

    pub fn open_edit_for_candidate(&mut self) -> bool {
        let Some(index) = self.selected_candidate_index else {
            return false;
        };
        let Some(candidate) = self.candidates.get(index) else {
            return false;
        };
        let Some(path) = self.candidate_path.clone() else {
            return false;
        };

        self.form = Some(T::from_candidate(candidate));
        self.edit_context = Some(EditContext::NewItem { path });
        true
    }

    pub fn clear_candidates(&mut self) {
        self.candidates.clear();
    }

    pub fn set_edit(&mut self, form: T, ctx: EditContext) {
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

        self.is_saving = true;
        self.last_error = None;
        let archive = self.archive.clone();
        let tx = self.tx.clone();
        let snapshot = form.snapshot();
        tokio::spawn(async move {
            let result = match &ctx {
                EditContext::NewItem { path } => archive.save_item_from_form(&snapshot, path).await,
                EditContext::ExistingItem { id } => {
                    archive.update_item_from_form(*id, &snapshot).await
                }
            };
            let was_update = matches!(&ctx, EditContext::ExistingItem { id: _ });
            let outcome = match result {
                Ok(_) => SaveOutcome::Saved { was_update },
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to write to database");
                    SaveOutcome::Failed {
                        reason: e.to_string(),
                        was_update,
                    }
                }
            };

            if let Err(e) = tx.send(Message::Save(outcome)) {
                tracing::error!(error = %e, "Failed to send the saving outcome to the the main task")
            }
        });
    }

    pub fn on_save_result(&mut self, outcome: SaveOutcome) -> (bool, bool) {
        self.is_saving = false;
        match outcome {
            SaveOutcome::Saved { was_update } => {
                self.candidates.clear();
                (true, was_update)
            }
            SaveOutcome::Failed { reason, was_update } => {
                self.last_error = Some(reason.to_string());
                (false, was_update)
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
            let msg = if let Some(a) = &abstract_text {
                match archive.set_item_description(id, a).await {
                    Ok(_) => AbstractData {
                        item_id: id,
                        abstract_text,
                        success: true,
                    },
                    Err(e) => {
                        tracing::warn!(error = %e, item_id = id, "Failed to save abstract to database");
                        AbstractData {
                            item_id: id,
                            abstract_text: Some(e.to_string()),
                            success: false,
                        }
                    }
                }
            } else {
                AbstractData {
                    item_id: id,
                    abstract_text: Some("Abstract not found".to_string()),
                    success: false,
                }
            };
            if let Err(e) = tx.send(Message::Abstract(msg)) {
                tracing::error!(error = %e, item_id = id, "Failed to send the abstact to the main task")
            }
        });
    }

    pub fn on_abstract_result(&mut self, data: AbstractData) -> bool {
        self.pending_abstract.remove(&data.item_id);
        if !data.success {
            self.failed_abstract.insert(data.item_id);
        }
        data.abstract_text.is_some() && data.success
    }
}
