use color_eyre::Result;
use ratatui_notifications::Level;

use crate::{
    metadata::dedup::MergedCandidate,
    schema::message::{AbstractData, Message, SaveOutcome},
    tui::app::{App, Mode},
};

impl App {
    fn handle_metadata_search_message(&mut self, candidates: Vec<MergedCandidate>) {
        let path = self.file_explorer.current().path.clone();
        let has_candidates = self.metadata.on_search_results(candidates, path);
        if has_candidates {
            self.mode = Mode::MetadataSelect;
        } else {
            self.notify(
                "Couldn't find metadata",
                "  Warning ".to_string(),
                Level::Warn,
            );
            self.mode = Mode::MetadataEdit;
        }
    }

    async fn handle_save_message(&mut self, outcome: SaveOutcome) -> Result<()> {
        let (saved, was_update) = self.metadata.on_save_result(outcome);
        if saved {
            self.request_refresh_item_list().await?;
        } else {
            let title = match was_update {
                true => " Update tome ",
                false => " Insert new tome ",
            };
            let reason = self.metadata.last_error.clone().unwrap_or_default();
            self.notify(reason, title.to_string(), Level::Error);
        }
        self.mode = Mode::Normal;
        Ok(())
    }

    fn handle_abstract_message(&mut self, data: AbstractData) {
        let id = data.item_id;
        let applied = self.metadata.on_abstract_result(data);
        let Some(item) = self.items.iter_mut().find(|i| i.id == id) else {
            return;
        };
        let title = item.fields.title.clone();
        if applied {
            self.notify(
                format!("Found abstact for '{title}'"),
                " Fetching Metadata ".to_string(),
                Level::Info,
            );
        } else {
            // NOTE: on failure we log instead of pushing a notification, is less annoying
            // moreover, the fetching is fired without the user consent
            tracing::warn!(item_id = id, "Could not find an abstract text")
        }
    }

    pub async fn poll_messages(&mut self) -> Result<()> {
        while let Ok(message) = self.task_channel_rx.try_recv() {
            match message {
                Message::Save(outcome) => self.handle_save_message(outcome).await?,
                Message::Metadata(candidates) => self.handle_metadata_search_message(candidates),
                Message::ImageCover(cover_data) => {
                    self.covers.handle_ready(*cover_data, &mut self.items)
                }
                Message::ImageCoverFailed { item_id } => self.covers.handle_failed(item_id),
                Message::Abstract(abstract_data) => self.handle_abstract_message(abstract_data),
                Message::PeerDiscovered(peer_info) => self.peers.on_peer_discover(peer_info),
                Message::PeerExpired(peer_info) => self.peers.on_peer_expiration(peer_info),
                Message::LibraryItemsDiscovered(items_data) => {
                    self.library
                        .handle_items_discovered(items_data.namespace_id, items_data.items);
                }
                Message::ItemDownloadReady(download_ready_data) => {
                    self.handle_item_download_ready(
                        download_ready_data.entry,
                        download_ready_data.local_path,
                    );
                }
                Message::ItemDownloadFailed(download_failed_data) => {
                    self.notify(
                        download_failed_data.reason,
                        " Download tome ".to_string(),
                        Level::Error,
                    );
                }
            }
        }
        Ok(())
    }
}
