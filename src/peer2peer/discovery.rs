use std::{hash::Hash, sync::Arc};

use futures::StreamExt;
use iroh::PublicKey;
use iroh_mdns_address_lookup::DiscoveryEvent;
use ratatui::{style::Style, text::Span};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    peer2peer::{PrettyDisplay, node::ShareNode},
    schema::{graphics::Spannable, message::Message},
};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PeerInfo {
    pub node_id: PublicKey,
}

impl PrettyDisplay for PeerInfo {
    fn node_id(&self) -> PublicKey {
        self.node_id
    }
}

impl Spannable for PeerInfo {
    fn to_span(&self) -> Span<'static> {
        let node_id = self.pretty_name();
        let (bg, fg) = Self::tag_colors(&node_id);
        Span::styled(format!(" {node_id} "), Style::default().bg(bg).fg(fg))
    }

    fn span_len(&self) -> usize {
        self.node_id.to_string().len() + 2
    }
}

// Kicks off a background task that forwards mDNS discovery events
// to the main task channel
pub fn spawn_discovery_listener(node: Arc<ShareNode>, tx: Arc<UnboundedSender<Message>>) {
    tokio::spawn(async move {
        let mut events = node.mdns.subscribe().await;
        while let Some(event) = events.next().await {
            let message = match event {
                DiscoveryEvent::Discovered { endpoint_info, .. } => {
                    Message::PeerDiscovered(PeerInfo {
                        node_id: endpoint_info.endpoint_id,
                    })
                }
                DiscoveryEvent::Expired { endpoint_id } => Message::PeerExpired(PeerInfo {
                    node_id: endpoint_id,
                }),
                _ => continue,
            };

            if let Err(e) = tx.send(message) {
                tracing::error!(error = %e, "Failed to forward peer discovery event");
            }
        }
    });
}
