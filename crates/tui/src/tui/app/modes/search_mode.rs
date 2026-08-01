use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::App;

impl App {
    pub(in crate::tui::app::modes) async fn handle_search_mode(
        &mut self,
        key: KeyEvent,
    ) -> Result<()> {
        match key.code {
            KeyCode::Esc => self.cancel_search().await?,
            KeyCode::Enter => self.confirm_search().await?,
            _ => {}
        }
        Ok(())
    }
}
