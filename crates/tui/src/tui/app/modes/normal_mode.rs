use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use minastirith_core::traits::Focusable;

use crate::tui::app::{App, Focus, Mode};

impl App {
    pub(in crate::tui::app::modes) async fn handle_normal_mode(
        &mut self,
        key: KeyEvent,
    ) -> Result<()> {
        if let KeyCode::Tab = key.code {
            self.focus_next();
            return Ok(());
        }
        if let KeyCode::Char('?') = key.code {
            self.open_help();
            return Ok(());
        }
        match self.focus {
            Focus::Items => match key.code {
                KeyCode::Char('[') => self.prev_tab().await?,
                KeyCode::Char(']') => self.next_tab().await?,
                KeyCode::Char('q') => self.quit = true,
                KeyCode::Char('j') | KeyCode::Down => self.next_item(),
                KeyCode::Char('k') | KeyCode::Up => self.prev_item(),
                KeyCode::Enter => self.items_component.core().open_item()?,
                KeyCode::Char('a') => self.request_open_file_picker()?,
                KeyCode::Char('e') => {
                    self.open_metadata_edit_for_selected_item();
                }
                KeyCode::Char('b') => self.send_bibtex_to_system_clipboard(),
                KeyCode::Char('c') => self.open_collection_assign_for_selected(),
                KeyCode::Char('L') => self.open_library_browse(),
                KeyCode::Char('/') => self.mode = Mode::Search,
                _ => {}
            },
            Focus::Collections => match key.code {
                KeyCode::Char('j') | KeyCode::Down => self.select_collection_next(),
                KeyCode::Char('k') | KeyCode::Up => self.select_collection_prev(),
                KeyCode::Char('n') => self.open_collection_create(),
                KeyCode::Char('c') => self.open_item_assign_for_selected_collection(),
                KeyCode::Char('b') => {
                    self.bulk_bibtex_to_system_clipboard().await?;
                }
                KeyCode::Char('d') => self.delete_collection().await?,
                KeyCode::Enter => self.confirm_collection_selection().await?,
                KeyCode::Char('q') => self.quit = true,
                KeyCode::Char('p') => self.open_library_publish_for_selected(),
                KeyCode::Char('L') => self.open_library_manage(),
                _ => {}
            },
        }
        Ok(())
    }
}
