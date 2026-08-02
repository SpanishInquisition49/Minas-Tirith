use color_eyre::Result;
use std::borrow::Cow;

use minastirith_core_derive::{Cyclable, Focusable};

use crate::{schema::item::DatabaseItem, state::library::LibraryState, traits::Focusable};

#[derive(PartialEq, Eq, Clone, Copy, Default, Cyclable)]
pub enum PublishField {
    #[default]
    Name,
    Description,
}

#[derive(Focusable)]
#[focus(field)]
pub struct LibraryPublishState {
    collection_id: i32,
    collection_name: String,
    field: PublishField,
    publishing: bool,
    result_ticket: Option<String>,
    last_error: Option<String>,
}

impl LibraryPublishState {
    /// Start a publish session for the collection identified by
    /// `collection_id` (with display name `collection_name`).
    #[must_use]
    pub fn new(collection_id: i32, collection_name: String) -> Self {
        LibraryPublishState {
            collection_id,
            collection_name,
            field: PublishField::default(),
            publishing: false,
            result_ticket: None,
            last_error: None,
        }
    }

    /// Id of the collection being published.
    #[must_use]
    pub fn collection_id(&self) -> i32 {
        self.collection_id
    }

    /// Display name of the collection being published.
    #[must_use]
    pub fn collection_name(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.collection_name)
    }

    /// Whether the publish request is currently in flight.
    #[must_use]
    pub fn is_publishing(&self) -> bool {
        self.publishing
    }

    /// The share ticket produced by a successful publish, if any.
    pub fn result_ticket(&self) -> Option<Cow<'_, str>> {
        self.result_ticket.as_deref().map(Cow::Borrowed)
    }

    /// The last error reported by a publish attempt, if any.
    pub fn last_error(&self) -> Option<Cow<'_, str>> {
        self.last_error.as_deref().map(Cow::Borrowed)
    }
}

// NOTE: methods related to the publish of collections
impl LibraryState {
    /// The active publish state, if the publish dialog is open.
    #[must_use]
    pub fn get_publish_state(&self) -> Option<&LibraryPublishState> {
        self.publish.as_ref()
    }

    /// Mutable access to the active publish state, if the publish dialog
    /// is open.
    pub fn get_publish_state_mut(&mut self) -> Option<&mut LibraryPublishState> {
        self.publish.as_mut()
    }

    /// Open the publish dialog for the collection identified by
    /// `collection_id`/`collection_name`.
    pub fn open_publish(&mut self, collection_id: i32, collection_name: String) {
        self.publish = Some(LibraryPublishState::new(collection_id, collection_name));
    }

    /// Cycle focus to the next field in the publish dialog.
    pub fn publish_cycle_focus(&mut self) {
        if let Some(s) = &mut self.publish {
            s.focus_next();
        }
    }

    /// Publish `items_in_collection` as a shared library named `name`
    /// (with optional `description`), returning the name on success or
    /// `None` if there's no active publish session or the attempt failed.
    /// # Errors
    /// Failures from the underlying publish are recorded on the publish
    /// state rather than propagated; this function itself always returns
    /// `Ok`.
    pub async fn confirm_publish(
        &mut self,
        name: String,
        description: Option<String>,
        items_in_collection: &[DatabaseItem],
    ) -> Result<Option<String>> {
        let Some(s) = &self.publish else {
            return Ok(None);
        };

        let collection_id = s.collection_id;

        if let Some(s) = &mut self.publish {
            s.publishing = true;
        }

        let result = self
            .publish_collection(
                collection_id,
                name.clone(),
                description,
                items_in_collection,
            )
            .await;

        if let Some(s) = &mut self.publish {
            s.publishing = false;
            match result {
                Ok(ticket) => {
                    s.result_ticket = Some(ticket);
                    Ok(Some(name))
                }
                Err(e) => {
                    s.last_error = Some(e.to_string());
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Close the publish dialog, discarding the session.
    pub fn cancel_publish(&mut self) {
        self.publish = None;
    }
}
