use crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::{App, Mode};

impl App {
    pub(in crate::tui::app::modes) fn handle_search_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q') => self.mode = Mode::Normal,
            _ => {}
        }
    }
}
