use std::collections::HashMap;

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use color_eyre::eyre::Result;
use ratatui_notifications::Level;

use crate::tui::app::App;

impl App {
    pub async fn bulk_bibtex_to_system_clipboard(&mut self) -> Result<()> {
        let Some(id) = self.collection_component.selected().map(|c| c.id) else {
            self.notify(
                "No selected collection",
                " Bulk Export citation ".to_string(),
                Level::Error,
            );
            return Ok(());
        };

        let items = self.archive.get_items(None, Some(id)).await?;

        // NOTE: to handle possible cite keys overlap we keep track of the used keys
        // and append to conflicting keys their version (an incremental counter)
        let mut bibtex = String::default();
        let mut key_version_map: HashMap<String, usize> = HashMap::default();
        for item in items {
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
        self.send_to_sys_clipboard(
            " Export collection ".to_string(),
            "Collection bibtex sent to clipboard".to_string(),
            bibtex,
        );
        Ok(())
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

        self.send_to_sys_clipboard(
            " Export tome ".to_string(),
            "Tome bibtex sent to clipboard".to_string(),
            bibtex,
        );
    }

    pub fn copy_current_ticket_to_clipboard(&mut self) {
        if let Some(manage) = self.library_component.core().get_manage_state()
            && let Some(ticket) = manage.current_ticket()
        {
            self.send_to_sys_clipboard(
                " Copy ticket ".to_string(),
                "Ticket sent to clipboard".to_string(),
                ticket.to_string(),
            );
        } else {
            self.notify(
                "No ticket generated",
                " Copy ticket ".to_string(),
                Level::Error,
            );
        }
    }

    fn send_to_sys_clipboard(&mut self, title: String, body: String, clipboard_content: String) {
        match ClipboardContext::new() {
            Ok(mut ctx) => match ctx.set_contents(clipboard_content) {
                Ok(()) => {
                    self.notify(body, title, Level::Info);
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
