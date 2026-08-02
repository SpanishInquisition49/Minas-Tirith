use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use minastirith_core::traits::MetadataForm;

use crate::tui::app::App;

impl App {
    pub(in crate::tui::app::modes) fn handle_metadata_editing_mode(&mut self, key: KeyEvent) {
        let editing = self
            .metadata_component
            .core_mut()
            .form_mut()
            .as_ref()
            .is_some_and(|f| f.editing);
        if editing {
            let Some(form) = self.metadata_component.core_mut().form_mut() else {
                return;
            };
            match key.code {
                KeyCode::Enter | KeyCode::Esc => form.editing = false,
                _ => {}
            }
        } else {
            match (key.modifiers, key.code) {
                (KeyModifiers::CONTROL, KeyCode::Char('s')) => self.confirm_metadata_form(),

                (_, KeyCode::Char('j') | KeyCode::Down) => {
                    if let Some(f) = self.metadata_component.core_mut().form_mut() {
                        f.next_field();
                    }
                }
                (_, KeyCode::Char('k') | KeyCode::Up) => {
                    if let Some(f) = self.metadata_component.core_mut().form_mut() {
                        f.prev_field();
                    }
                }
                (_, KeyCode::Char('t')) => {
                    if let Some(f) = self.metadata_component.core_mut().form_mut() {
                        f.cycle_item_type();
                    }
                }
                (_, KeyCode::Enter) => {
                    if let Some(f) = self.metadata_component.core_mut().form_mut() {
                        f.editing = true;
                    }
                }
                (_, KeyCode::Esc | KeyCode::Char('q')) => {
                    self.cancel_metadata_form();
                }
                _ => {}
            }
        }
    }

    pub(in crate::tui::app::modes) fn handle_metadata_select_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.select_metadata_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_metadata_prev(),
            KeyCode::Enter => self.open_metadata_edit_for_candidate(),
            KeyCode::Esc | KeyCode::Char('q') => self.cancel_metadata_selection(),
            _ => {}
        }
    }
}
