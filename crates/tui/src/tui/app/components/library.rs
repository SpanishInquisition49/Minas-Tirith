use std::{path::PathBuf, sync::Arc};

use crossterm::event::Event;
use minastirith_core::{
    database::Archive,
    peer2peer::node::ShareNode,
    schema::message::Message,
    state::library::{LibraryState, publish::PublishField, subscribe::SubscribeField},
    traits::Focusable,
};
use ratatui::widgets::ListState;
use tokio::sync::mpsc::UnboundedSender;
use tui_input::{Input, backend::crossterm::EventHandler};

pub struct LibraryComponent {
    core: LibraryState,
    collection_name: Input,
    collection_description: Input,
    ticket: Input,
    nickname: Input,
    subscription_list_state: ListState,
    browse_list_state: ListState,
    manage_list_state: ListState,
}

impl LibraryComponent {
    pub fn new(
        archive: Arc<Archive>,
        share_node: ShareNode,
        tx: Arc<UnboundedSender<Message>>,
        import_dir: PathBuf,
    ) -> Self {
        Self {
            core: LibraryState::new(archive, share_node, tx, import_dir),
            collection_name: Input::default(),
            collection_description: Input::default(),
            ticket: Input::default(),
            nickname: Input::default(),
            subscription_list_state: ListState::default(),
            browse_list_state: ListState::default(),
            manage_list_state: ListState::default(),
        }
    }

    pub fn core(&self) -> &LibraryState {
        &self.core
    }

    pub fn core_mut(&mut self) -> &mut LibraryState {
        &mut self.core
    }

    pub fn collection_name(&self) -> &Input {
        &self.collection_name
    }

    pub fn collection_description(&self) -> &Input {
        &self.collection_description
    }

    pub fn subscribe_ticket(&self) -> &Input {
        &self.ticket
    }

    pub fn subscribe_nickname(&self) -> &Input {
        &self.nickname
    }

    pub fn subscription_list_state_mut(&mut self) -> &mut ListState {
        &mut self.subscription_list_state
    }

    pub fn browse_list_state_mut(&mut self) -> &mut ListState {
        &mut self.browse_list_state
    }

    pub fn manage_list_state_mut(&mut self) -> &mut ListState {
        &mut self.manage_list_state
    }

    pub fn handle_publish_event(&mut self, event: &Event) {
        let Some(s) = self.core.get_publish_state() else {
            return;
        };
        match s.current_focus() {
            PublishField::Name => self.collection_name.handle_event(event),
            PublishField::Description => self.collection_description.handle_event(event),
        };
    }

    pub fn handle_subscribe_event(&mut self, event: &Event) {
        let Some(s) = self.core.get_subscribe_state() else {
            return;
        };
        match s.current_focus() {
            SubscribeField::Ticket => self.ticket.handle_event(event),
            SubscribeField::Nickname => self.nickname.handle_event(event),
        };
    }
}
