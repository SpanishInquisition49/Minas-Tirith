use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use minastirith_core::traits::Focusable;

use crate::tui::app::{App, Mode};

impl App {
    pub(in crate::tui::app::modes) async fn handle_library_browse_mode(
        &mut self,
        key: KeyEvent,
    ) -> Result<()> {
        match key.code {
            KeyCode::Tab => {
                if let Some(b) = self.library_component.core_mut().get_browse_state_mut() {
                    b.focus_next();
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.library_component.core_mut().browse_select_next();
                self.sync_browse_selection();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.library_component.core_mut().browse_select_prev();
                self.sync_browse_selection();
            }
            KeyCode::Char('r') => self.library_component.core_mut().refresh_current().await?,
            KeyCode::Char('d') => self.unsubscribe_selected_library().await?,
            KeyCode::Char('a') => {
                self.library_component.core_mut().open_subscribe();
                self.mode = Mode::LibrarySubscribe;
            }
            KeyCode::Enter => self.library_component.core_mut().browse_confirm_import(),
            KeyCode::Esc => {
                self.library_component.core_mut().close_browse();
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
            KeyCode::Char('j') | KeyCode::Down => {
                self.library_component.core_mut().manage_select_next();
                self.sync_manage_selection();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.library_component.core_mut().manage_select_prev();
                self.sync_manage_selection();
            }
            KeyCode::Char('t') => {
                self.library_component
                    .core_mut()
                    .manage_generate_ticket()
                    .await?;
            }
            KeyCode::Char('c') => self.copy_current_ticket_to_clipboard(),
            KeyCode::Char('d') => {
                self.library_component
                    .core_mut()
                    .manage_delete_selected()
                    .await?;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.library_component.core_mut().close_manage();
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
            (_, KeyCode::Tab) => {
                if let Some(p) = self.library_component.core_mut().get_publish_state_mut() {
                    p.focus_next();
                }
            }
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                self.publish_collection_as_library().await?;
            }
            (_, KeyCode::Esc | KeyCode::Char('q')) => {
                self.library_component.core_mut().cancel_publish();
            }
            _ => {}
        }
        Ok(())
    }

    pub(in crate::tui::app::modes) async fn handle_library_subscribe_mode(
        self: &mut App,
        key: KeyEvent,
    ) -> Result<()> {
        match (key.modifiers, key.code) {
            (_, KeyCode::Tab) => {
                if let Some(s) = self.library_component.core_mut().get_subscribe_state_mut() {
                    s.focus_next();
                }
            }
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => self.confirm_library_subscribe().await?,
            (_, KeyCode::Esc) => self.library_component.core_mut().cancel_subscribe(),
            _ => {}
        }
        Ok(())
    }
}
