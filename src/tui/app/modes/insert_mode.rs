use crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::{App, Mode};

impl App {
    pub(in crate::tui::app::modes) fn handle_insert_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q') => self.mode = Mode::Normal,
            KeyCode::Char('a') => {
                self.request_fetch_metadata_candidates();
            }
            _ => {}
        }
    }
}
