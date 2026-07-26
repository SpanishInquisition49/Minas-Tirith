use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyEvent, KeyEventKind};

use crate::tui::app::{App, Mode};

pub mod collection_modes;
pub mod help_mode;
pub mod insert_mode;
pub mod library_modes;
pub mod metadata_modes;
pub mod normal_mode;
pub mod search_mode;

impl App {
    pub async fn handle(&mut self, event: Event) -> Result<()> {
        if let Mode::Insert = self.mode {
            self.file_explorer.handle(&event)?;
        };
        if let Mode::MetadataEdit = self.mode
            && let Some(form) = &mut self.metadata.form
            && form.editing
        {
            form.handle_event(&event);
        }
        if let Mode::CollectionCreate = self.mode {
            self.collections.handle_event(&event);
        }
        if let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
        {
            self.handle_key(key).await?;
        }
        if let Mode::LibraryPublish = self.mode {
            self.library.handle_publish_event(&event);
        }
        if let Mode::LibrarySubscribe = self.mode {
            self.library.handle_subscribe_event(&event);
        }
        Ok(())
    }

    async fn handle_key(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        match self.mode {
            Mode::Normal => self.handle_normal_mode(key).await?,
            Mode::Insert => self.handle_insert_mode(key),
            Mode::MetadataSelect => self.handle_metadata_select_mode(key),
            Mode::Search => self.handle_search_mode(key),
            Mode::MetadataEdit => self.handle_metadata_editing_mode(key).await?,
            Mode::CollectionCreate => self.handle_collection_create_mode(key).await?,
            Mode::CollectionAssign => self.handle_collection_assign_mode(key).await?,
            Mode::LibraryPublish => self.handle_library_publish_mode(key).await?,
            Mode::LibrarySubscribe => self.handle_library_subscribe_mode(key).await?,
            Mode::LibraryBrowse => self.handle_library_browse_mode(key).await?,
            Mode::LibraryManage => self.handle_library_manage_mode(key).await?,
            Mode::Help => self.handle_help_mode(key),
        }
        Ok(())
    }
}
