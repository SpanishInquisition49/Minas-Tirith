use std::{collections::HashSet, sync::Arc};

use color_eyre::eyre::Result;
use directories::ProjectDirs;
use ratatui::widgets::ListState;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    peer2peer::{
        discovery::{PeerInfo, spawn_discovery_listener},
        node::ShareNode,
    },
    schema::message::Message,
    tui::app::traits::ListWidget,
};

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct ShareRequest {
    pub peer: PeerInfo,
    pub item: bool,
}

pub struct PeerState {
    peers: Vec<PeerInfo>,
    pub share_node: Arc<ShareNode>,
    list_state: ListState,
    ignored: HashSet<PeerInfo>,
    pending_requests: HashSet<ShareRequest>,
    incoming_requests: HashSet<ShareRequest>,
}

impl PeerState {
    pub async fn new(proj_dirs: &ProjectDirs, tx: Arc<UnboundedSender<Message>>) -> Result<Self> {
        let share_node = Arc::new(ShareNode::bind(proj_dirs.data_dir()).await?);
        spawn_discovery_listener(share_node.clone(), tx);
        Ok(Self {
            peers: Vec::new(),
            share_node,
            list_state: ListState::default(),
            ignored: HashSet::new(),
            pending_requests: HashSet::new(),
            incoming_requests: HashSet::new(),
        })
    }

    pub fn request_share_item(&mut self) {
        let Some(index) = self.list_state.selected() else {
            return;
        };

        let Some(peer) = self.peers.get(index) else {
            return;
        };

        self.pending_requests.insert(ShareRequest {
            peer: peer.clone(),
            item: true,
        });
    }

    pub fn on_peer_discover(&mut self, peer_info: PeerInfo) {
        if !self.peers.contains(&peer_info) {
            self.peers.push(peer_info);
        }
        if self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
        }
    }

    pub fn on_peer_expiration(&mut self, peer_info: PeerInfo) {
        self.peers.retain(|p| *p != peer_info);
    }
}

impl ListWidget<PeerInfo> for PeerState {
    fn items(&self) -> &[PeerInfo] {
        &self.peers
    }

    fn items_mut(&mut self) -> &mut Vec<PeerInfo> {
        self.peers.as_mut()
    }

    fn list_state(&self) -> &ListState {
        &self.list_state
    }

    fn list_state_mut(&mut self) -> &mut ListState {
        &mut self.list_state
    }
}
