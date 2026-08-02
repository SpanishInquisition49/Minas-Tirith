use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::App;

impl App {
    pub(in crate::tui::app::modes) async fn handle_collection_assign_mode(
        &mut self,
        key: KeyEvent,
    ) -> Result<()> {
        match (key.modifiers, key.code) {
            (_, KeyCode::Char('j') | KeyCode::Down) => self.collection_assign_next(),
            (_, KeyCode::Char('k') | KeyCode::Up) => self.collection_assign_prev(),
            (_, KeyCode::Char(' ') | KeyCode::Enter) => self.collection_assign_toggle_current(),
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => self.confirm_collection_assign().await?,
            (_, KeyCode::Esc | KeyCode::Char('q')) => self.cancel_collection_assign(),
            _ => {}
        }
        Ok(())
    }

    pub(in crate::tui::app::modes) async fn handle_collection_create_mode(
        &mut self,
        key: KeyEvent,
    ) -> Result<()> {
        match key.code {
            KeyCode::Enter => self.confirm_collection_create().await?,
            KeyCode::Esc => self.close_collection_create(),
            _ => {}
        }
        Ok(())
    }
}
