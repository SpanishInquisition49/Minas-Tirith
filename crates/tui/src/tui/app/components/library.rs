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
    /// Construct a [`LibraryComponent`] backed by `archive` and
    /// `share_node`, downloading imported items into `import_dir` and
    /// reporting async results via `tx`.
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

    /// The underlying [`LibraryState`].
    pub fn core(&self) -> &LibraryState {
        &self.core
    }

    /// Mutable access to the underlying [`LibraryState`].
    pub fn core_mut(&mut self) -> &mut LibraryState {
        &mut self.core
    }

    /// The input widget for the publish dialog's collection-name field.
    pub fn collection_name(&self) -> &Input {
        &self.collection_name
    }

    /// The input widget for the publish dialog's collection-description
    /// field.
    pub fn collection_description(&self) -> &Input {
        &self.collection_description
    }

    /// The input widget for the subscribe dialog's ticket field.
    pub fn subscribe_ticket(&self) -> &Input {
        &self.ticket
    }

    /// The input widget for the subscribe dialog's nickname field.
    pub fn subscribe_nickname(&self) -> &Input {
        &self.nickname
    }

    /// Mutable access to the subscription list's ratatui `ListState`.
    pub fn subscription_list_state_mut(&mut self) -> &mut ListState {
        &mut self.subscription_list_state
    }

    /// Mutable access to the browse-items list's ratatui `ListState`.
    pub fn browse_list_state_mut(&mut self) -> &mut ListState {
        &mut self.browse_list_state
    }

    /// Mutable access to the manage-libraries list's ratatui `ListState`.
    pub fn manage_list_state_mut(&mut self) -> &mut ListState {
        &mut self.manage_list_state
    }

    /// Route `event` to whichever publish-dialog input is currently
    /// focused. No-op if the publish dialog isn't open.
    pub fn handle_publish_event(&mut self, event: &Event) {
        let Some(s) = self.core.get_publish_state() else {
            return;
        };
        match s.current_focus() {
            PublishField::Name => self.collection_name.handle_event(event),
            PublishField::Description => self.collection_description.handle_event(event),
        };
    }

    /// Route `event` to whichever subscribe-dialog input is currently
    /// focused. No-op if the subscribe dialog isn't open.
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
