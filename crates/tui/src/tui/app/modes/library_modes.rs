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
                self.library_component
                    .core_mut()
                    .get_browse_state_mut()
                    .as_mut()
                    .and_then(|b| Some(b.focus_next()));
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.library_component.core_mut().browse_select_next()
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.library_component.core_mut().browse_select_prev()
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
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.library_component.core_mut().manage_select_prev();
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
                self.library_component
                    .core_mut()
                    .get_publish_state_mut()
                    .as_mut()
                    .and_then(|p| Some(p.focus_next()));
            }
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                self.publish_collection_as_library().await?
            }
            (_, KeyCode::Esc) | (_, KeyCode::Char('q')) => {
                self.library_component.core_mut().cancel_publish()
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
                self.library_component
                    .core_mut()
                    .get_subscribe_state_mut()
                    .as_mut()
                    .and_then(|s| Some(s.focus_next()));
            }
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => self.confirm_library_subscribe().await?,
            (_, KeyCode::Esc) => self.library_component.core_mut().cancel_subscribe(),
            _ => {}
        }
        Ok(())
    }
}
