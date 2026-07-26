use std::collections::HashMap;

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use ratatui_notifications::Level;

use crate::{
    schema::{collection::Collection, item::DatabaseItem},
    tui::app::{App, traits::ListWidget},
};

impl App {
    pub fn bulk_bibtex_to_system_clipboard(&mut self) {
        let Some(collection) = self.collections.selected() else {
            self.notify(
                "No selected collection",
                " Bulk Export citation ".to_string(),
                Level::Error,
            );
            return;
        };
        let filtered_items: Vec<&DatabaseItem> = if Collection::is_trivial_collection(collection.id)
        {
            self.items.iter().collect()
        } else {
            self.items
                .iter()
                .filter(|i| i.collections.iter().any(|c| c.id == collection.id))
                .collect::<Vec<_>>()
        };

        // NOTE: to handle possible cite keys overlap we keep track of the used keys
        // and append to conflicting keys their version (an incremental counter)
        let mut bibtex = String::default();
        let mut key_version_map: HashMap<String, usize> = HashMap::default();
        for item in filtered_items {
            let cite_key = item.cite_key();
            let key = if key_version_map.contains_key(&cite_key) {
                let version = key_version_map.get(&cite_key).copied().unwrap_or(1);
                let key = format!("{}-{}", cite_key, version);
                key_version_map.insert(cite_key, version + 1);
                Some(key)
            } else {
                key_version_map.insert(cite_key, 1);
                None
            };
            bibtex.push_str(&format!("{}\n", item.to_bibtex(key)));
        }
        self.send_to_sys_clipboard(bibtex);
    }

    pub fn send_bibtex_to_system_clipboard(&mut self) {
        let Some(item) = self.selected_item() else {
            self.notify(
                "No selected item",
                " Export citation ".to_string(),
                Level::Error,
            );
            return;
        };
        let bibtex = item.to_bibtex(None);

        self.send_to_sys_clipboard(bibtex);
    }

    pub fn copy_current_ticket_to_clipboard(&mut self) {
        let Some(ticket) = self
            .library
            .manage
            .as_ref()
            .and_then(|s| s.current_ticket.clone())
        else {
            self.notify(
                "No ticket generated",
                " Copy ticket ".to_string(),
                Level::Error,
            );
            return;
        };
        self.send_to_sys_clipboard(ticket);
    }

    // TODO: change the signature of this method to enable notification title and body customization
    // to account for both bibtex and ticket copy
    fn send_to_sys_clipboard(&mut self, content: String) {
        match ClipboardContext::new() {
            Ok(mut ctx) => match ctx.set_contents(content) {
                Ok(()) => {
                    self.notify(
                        "Copied Bibtex",
                        "  Export collection ".to_string(),
                        Level::Info,
                    );
                }
                Err(e) => {
                    tracing::error!(error = %e, "Could not set contents of the system clipboard")
                }
            },
            Err(e) => {
                tracing::error!(error = %e, "Could not get system clipboard");
                self.notify(
                    "Clipboard unavailable",
                    " Export citation ".to_string(),
                    Level::Error,
                )
            }
        }
    }
}
