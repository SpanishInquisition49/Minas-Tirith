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

    pub fn collection_id(&self) -> i32 {
        self.collection_id
    }

    pub fn collection_name(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.collection_name)
    }

    pub fn is_publishing(&self) -> bool {
        self.publishing
    }

    pub fn result_ticket(&self) -> Option<Cow<'_, str>> {
        self.result_ticket.as_deref().map(Cow::Borrowed)
    }

    pub fn last_error(&self) -> Option<Cow<'_, str>> {
        self.last_error.as_deref().map(Cow::Borrowed)
    }
}

// NOTE: methods related to the publish of collections
impl LibraryState {
    pub fn open_publish(&mut self, collection_id: i32, collection_name: String) {
        self.publish = Some(LibraryPublishState::new(collection_id, collection_name))
    }

    pub fn publish_cycle_focus(&mut self) {
        if let Some(s) = &mut self.publish {
            s.focus_next();
        }
    }

    pub async fn confirm_publish(
        &mut self,
        name: String,
        description: Option<String>,
        items_in_collection: &[&DatabaseItem],
    ) -> Result<Option<String>> {
        let Some(s) = &self.publish else {
            return Ok(None);
        };

        let collection_id = s.collection_id;

        if let Some(s) = &mut self.publish {
            s.publishing = true;
        }

        let result = self
            .publish_colletion(
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

    pub fn cancel_publish(&mut self) {
        self.publish = None
    }
}
