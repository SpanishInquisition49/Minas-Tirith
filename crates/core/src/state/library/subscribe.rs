use color_eyre::Result;
use std::borrow::Cow;

use minastirith_core_derive::{Cyclable, Focusable};

use crate::{state::library::LibraryState, traits::Focusable};

#[derive(PartialEq, Eq, Clone, Copy, Default, Cyclable)]
pub enum SubscribeField {
    #[default]
    Ticket,
    Nickname,
}

#[derive(Default, Focusable)]
#[focus(field)]
pub struct LibrarySubscribeState {
    field: SubscribeField,
    subscribing: bool,
    last_error: Option<String>,
}

impl LibrarySubscribeState {
    /// Whether the subscribe request is currently in flight.
    #[must_use]
    pub fn is_subscribing(&self) -> bool {
        self.subscribing
    }

    /// The last error reported by a subscribe attempt, if any.
    pub fn last_error(&self) -> Option<Cow<'_, str>> {
        self.last_error.as_deref().map(Cow::Borrowed)
    }
}

// NOTE: method related to the subscription of collections from other peers
impl LibraryState {
    /// The active subscribe state, if the subscribe dialog is open.
    #[must_use]
    pub fn get_subscribe_state(&self) -> Option<&LibrarySubscribeState> {
        self.subscribe.as_ref()
    }

    /// Mutable access to the active subscribe state, if the subscribe
    /// dialog is open.
    pub fn get_subscribe_state_mut(&mut self) -> Option<&mut LibrarySubscribeState> {
        self.subscribe.as_mut()
    }

    /// Open the subscribe dialog.
    pub fn open_subscribe(&mut self) {
        self.subscribe = Some(LibrarySubscribeState::default());
    }

    /// Cycle focus to the next field in the subscribe dialog.
    pub fn subscribe_cycle_focus(&mut self) {
        if let Some(s) = &mut self.subscribe {
            s.focus_next();
        }
    }

    /// Subscribe to the shared library referenced by `ticket` under
    /// `nickname`. Returns `false` if there's no active subscribe session,
    /// either field is empty, or the attempt failed.
    /// # Errors
    /// Failures from the underlying subscribe are recorded on the
    /// subscribe state rather than propagated; this function itself
    /// always returns `Ok`.
    pub async fn confirm_subscribe(&mut self, ticket: String, nickname: String) -> Result<bool> {
        let Some(s) = &mut self.subscribe else {
            return Ok(false);
        };
        if ticket.is_empty() || nickname.is_empty() {
            return Ok(false);
        }
        s.subscribing = true;
        match self.subscribe(ticket, nickname).await {
            Ok(()) => {
                self.subscribe = None;
                Ok(true)
            }
            Err(e) => {
                if let Some(s) = &mut self.subscribe {
                    s.subscribing = false;
                    s.last_error = Some(e.to_string());
                }
                Ok(false)
            }
        }
    }

    /// Close the subscribe dialog, discarding the session.
    pub fn cancel_subscribe(&mut self) {
        self.subscribe = None;
    }
}
