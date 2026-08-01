use color_eyre::Result;
use std::borrow::Cow;

use crate::{metadata::shared_library::SharedLibrary, state::library::LibraryState};

#[derive(Default, Debug)]
pub struct LibraryManageState {
    selected_library_index: Option<usize>,
    current_ticket: Option<String>,
    generating: bool,
    last_error: Option<String>,
}

impl LibraryManageState {
    pub fn selected_library_index(&self) -> Option<usize> {
        self.selected_library_index
    }

    pub fn selected_library_index_mut(&mut self) -> &mut Option<usize> {
        &mut self.selected_library_index
    }

    pub fn current_ticket(&self) -> Option<Cow<'_, str>> {
        self.current_ticket.as_deref().map(Cow::Borrowed)
    }

    pub fn is_generating(&self) -> bool {
        self.generating
    }

    pub fn get_last_error(&self) -> Option<Cow<'_, str>> {
        self.last_error.as_deref().map(Cow::Borrowed)
    }
}

impl LibraryState {
    pub fn get_manage_state(&self) -> &Option<LibraryManageState> {
        &self.manage
    }

    pub fn get_manage_state_mut(&mut self) -> &mut Option<LibraryManageState> {
        &mut self.manage
    }

    pub fn open_manage(&mut self) {
        let mut manage = LibraryManageState::default();
        if !self.shared_libraries.is_empty() {
            manage.selected_library_index_mut().replace(0);
        }
        self.manage = Some(manage);
    }

    pub fn close_manage(&mut self) {
        self.manage = None
    }

    /// Cycle forward the library list, returning the new index
    pub fn manage_select_next(&mut self) -> Option<usize> {
        let manage = self.manage.as_mut()?;
        let len = self.shared_libraries.len();
        if len == 0 {
            return None;
        }
        let i = match manage.selected_library_index() {
            Some(i) if i + 1 < len => i + 1,
            _ => 0,
        };
        manage.selected_library_index_mut().replace(i);
        Some(i)
    }

    /// Cycle backward the library list, returning the new index
    pub fn manage_select_prev(&mut self) -> Option<usize> {
        let manage = self.manage.as_mut()?;
        let len = self.shared_libraries.len();
        if len == 0 {
            return None;
        }
        let i = match manage.selected_library_index() {
            Some(i) if i > 0 => i - 1,
            _ => len - 1,
        };
        manage.selected_library_index_mut().replace(i);
        Some(i)
    }

    pub fn manage_selected(&self) -> Option<&SharedLibrary> {
        let manage = self.manage.as_ref()?;
        self.shared_libraries.get(manage.selected_library_index()?)
    }

    pub async fn manage_generate_ticket(&mut self) -> Result<()> {
        let Some(namespace_id) = self.manage_selected().map(|l| l.namespace_id.clone()) else {
            return Ok(());
        };
        match self.manage.as_mut() {
            Some(manage) => manage.generating = true,
            None => return Ok(()),
        };
        let result = self.get_or_refresh_ticket(&namespace_id).await;
        if let Some(manage) = self.manage.as_mut() {
            manage.generating = false;
            match result {
                Ok(ticket) => manage.current_ticket = Some(ticket),
                Err(e) => manage.last_error = Some(e.to_string()),
            }
        }
        Ok(())
    }

    pub async fn manage_delete_selected(&mut self) -> Result<()> {
        let Some(id) = self.manage_selected().map(|l| l.id) else {
            return Ok(());
        };
        self.unpublish(id).await?;
        if let Some(manage) = self.manage.as_mut() {
            if !self.shared_libraries.is_empty() {
                manage.selected_library_index_mut().replace(0);
            }
            manage.current_ticket = None;
        }
        Ok(())
    }
}
