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
    pub fn is_subscribing(&self) -> bool {
        self.subscribing
    }

    pub fn last_error(&self) -> Option<Cow<'_, str>> {
        self.last_error.as_deref().map(Cow::Borrowed)
    }
}

// NOTE: method related to the subscription of collections from other peers
impl LibraryState {
    pub fn get_subscribe_state(&self) -> &Option<LibrarySubscribeState> {
        &self.subscribe
    }

    pub fn get_subscribe_state_mut(&mut self) -> &mut Option<LibrarySubscribeState> {
        &mut self.subscribe
    }

    pub fn open_subscribe(&mut self) {
        self.subscribe = Some(LibrarySubscribeState::default());
    }

    pub fn subscribe_cycle_focus(&mut self) {
        if let Some(s) = &mut self.subscribe {
            s.focus_next();
        }
    }

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

    pub fn cancel_subscribe(&mut self) {
        self.subscribe = None;
    }
}
