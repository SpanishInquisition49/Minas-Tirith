use crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::{App, Mode};

impl App {
    pub fn handle_help_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.help_scroll = self.help_scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.help_scroll = self.help_scroll.saturating_sub(1);
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = Mode::Normal;
                self.help_scroll = 0;
            }
            _ => {}
        }
    }
}
