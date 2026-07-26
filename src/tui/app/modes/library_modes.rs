use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::{App, Mode};

impl App {
    pub(in crate::tui::app::modes) async fn handle_library_browse_mode(
        &mut self,
        key: KeyEvent,
    ) -> Result<()> {
        match key.code {
            KeyCode::Tab => self.library.browse_toggle_focus(),
            KeyCode::Char('j') | KeyCode::Down => self.library.browse_select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.library.browse_select_prev(),
            KeyCode::Char('r') => self.refresh_current_library().await?,
            KeyCode::Char('d') => self.unsubscribe_selected_library().await?,
            KeyCode::Char('a') => {
                self.library.open_subscribe();
                self.mode = Mode::LibrarySubscribe;
            }
            KeyCode::Enter => self.library.browse_confirm_import(),
            KeyCode::Esc => {
                self.library.close_browse();
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    pub(in crate::tui::app::modes) async fn handle_library_manage_mode(
        &mut self,
        key: KeyEvent,
    ) -> Result<()> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.library.manage_select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.library.manage_select_prev(),
            KeyCode::Char('t') => self.library.manage_generate_ticket().await?,
            KeyCode::Char('c') => self.copy_current_ticket_to_clipboard(),
            KeyCode::Char('d') => self.library.manage_delete_selected().await?,
            KeyCode::Esc | KeyCode::Char('q') => {
                self.library.close_manage();
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        Ok(())
    }

    pub(in crate::tui::app::modes) async fn handle_library_publish_mode(
        &mut self,
        key: KeyEvent,
    ) -> Result<()> {
        match (key.modifiers, key.code) {
            (_, KeyCode::Tab) => self.library.publish_next_field(),
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                self.publish_collection_as_library().await?
            }
            (_, KeyCode::Esc) | (_, KeyCode::Char('q')) => self.cancel_publish(),
            _ => {}
        }
        Ok(())
    }

    pub(in crate::tui::app::modes) async fn handle_library_subscribe_mode(
        self: &mut App,
        key: KeyEvent,
    ) -> Result<()> {
        match (key.modifiers, key.code) {
            (_, KeyCode::Tab) => self.library.subscribe_next_field(),
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => self.confirm_library_subscribe().await?,
            (_, KeyCode::Esc) => self.cancel_subscribe(),
            _ => {}
        }
        Ok(())
    }
}
