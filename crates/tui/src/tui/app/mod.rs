use std::{path::PathBuf, sync::Arc, time::Duration};

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use color_eyre::{Result, eyre::bail};
use directories::ProjectDirs;
use minastirith_core::{
    database::Archive,
    metadata::{facade::MetadataProvider, image_cache::ImageCache},
    peer2peer::node::ShareNode,
    schema::message::Message,
    traits::{Cyclable, Focusable},
};
use ratatui::style::{Color, Style};
use ratatui_explorer::{FileExplorer, FileExplorerBuilder, Theme};
use ratatui_image::picker::Picker;
use ratatui_notifications::{
    Anchor, Animation, AutoDismiss, Level, Notification, Notifications, SizeConstraint,
    SlideDirection, Timing,
};
use tokio::sync::mpsc::{self, UnboundedReceiver};

use crate::{
    traits::SelectableSync,
    tui::app::components::{
        collection::CollectionComponent,
        cover::{CoverComponent, CoverMessage},
        item::ItemComponent,
        library::LibraryComponent,
        metadata::MetadataComponent,
    },
};

pub mod actions;
pub mod components;
pub mod messages;
pub mod modes;

#[derive(Clone, Copy, Debug)]
pub enum Focus {
    Items,
    Collections,
}

impl Cyclable for Focus {
    fn next(&self) -> Self {
        match self {
            Focus::Items => Focus::Collections,
            Focus::Collections => Focus::Items,
        }
    }

    fn prev(&self) -> Self {
        match self {
            Focus::Items => Focus::Collections,
            Focus::Collections => Focus::Items,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Mode {
    Normal,
    Insert,
    Search,
    MetadataSelect,
    MetadataEdit,
    CollectionCreate,
    CollectionAssign,
    LibraryPublish,
    LibrarySubscribe,
    LibraryBrowse,
    LibraryManage,
    Help,
}

pub struct App {
    pub(in crate::tui::app) notifications: Notifications,
    pub(in crate::tui::app) archive: Arc<Archive>,
    pub(in crate::tui::app) mode: Mode,
    pub(in crate::tui::app) quit: bool,
    pub(in crate::tui::app) file_explorer: FileExplorer,
    pub(in crate::tui::app) tick_counter: usize,
    // NOTE: must stay alive for the lifetime of the app — on X11 the
    // clipboard is only served to other apps while this context's
    // background thread is running; a short-lived context loses ownership
    // of the selection the instant it's dropped, so pasted content is gone.
    pub(in crate::tui::app) clipboard: Option<ClipboardContext>,

    pub(in crate::tui::app) metadata_component: MetadataComponent,
    pub(in crate::tui::app) items_component: ItemComponent,
    pub(in crate::tui::app) covers: CoverComponent,
    pub(in crate::tui::app) collection_component: CollectionComponent,
    pub(in crate::tui::app) focus: Focus,

    pub(in crate::tui::app) library_component: LibraryComponent,
    pub(in crate::tui::app) help_scroll: u16,

    pub(in crate::tui::app) task_channel_rx: UnboundedReceiver<Message>,
    pub(in crate::tui::app) cover_channel_rx: UnboundedReceiver<CoverMessage>,
}

impl Focusable for App {
    type Focus = Focus;

    fn current_focus(&self) -> Self::Focus {
        self.focus
    }

    fn current_focus_mut(&mut self) -> &mut Self::Focus {
        &mut self.focus
    }
}

impl App {
    pub async fn new(
        archive: Archive,
        picker: Picker,
        cache: ImageCache,
        proj_dirs: &ProjectDirs,
    ) -> Result<Self> {
        let (task_channel_tx, task_channel_rx) = mpsc::unbounded_channel();
        let (cover_channel_tx, cover_channel_rx) = mpsc::unbounded_channel();
        let task_channel_tx = Arc::new(task_channel_tx);
        let cover_channel_tx = Arc::new(cover_channel_tx);
        let import_dir = proj_dirs.data_dir().join("tomes");
        std::fs::create_dir_all(&import_dir)?;
        let archive = Arc::new(archive);
        let provider = Arc::new(MetadataProvider::new());
        let library = LibraryComponent::new(
            archive.clone(),
            ShareNode::bind(proj_dirs.data_dir()).await?,
            task_channel_tx.clone(),
            import_dir,
        );

        let mut app = Self {
            notifications: Notifications::new(),
            archive: archive.clone(),
            mode: Mode::Normal,
            quit: false,
            file_explorer: Self::build_explorer(None)?,
            tick_counter: 0,
            clipboard: ClipboardContext::new().ok(),
            items_component: ItemComponent::new(archive.clone()),
            metadata_component: MetadataComponent::new(
                archive.clone(),
                provider,
                task_channel_tx.clone(),
            ),
            covers: CoverComponent::new(archive.clone(), picker, cache, cover_channel_tx.clone()),
            collection_component: CollectionComponent::new(archive.clone()),
            focus: Focus::Items,
            task_channel_rx,
            cover_channel_rx,
            library_component: library,
            help_scroll: 0,
        };
        // NOTE: fetch from SQLite items, collections and kicks off cover fetching
        app.items_component.refresh(None).await?;
        app.request_cover_for_selected();
        app.library_component.core_mut().refresh().await?;
        app.library_component
            .core_mut()
            .reopen_known_namespaces()
            .await?;
        app.collection_component.core_mut().refresh().await?;
        app.collection_component.resync_after_refresh();
        Ok(app)
    }

    /// Create a new file explorer, if specified open from the given working directory
    fn build_explorer(working_dir: Option<&PathBuf>) -> Result<FileExplorer> {
        let working_dir = match working_dir {
            Some(dir) => dir,
            None => match directories::BaseDirs::new() {
                Some(base_dirs) => &base_dirs.home_dir().to_path_buf(),
                None => bail!("Cannot get home directory"),
            },
        };
        Ok(FileExplorerBuilder::default()
            .working_dir(working_dir)
            .filter_map(|f| {
                if f.is_dir {
                    Some(f)
                } else {
                    match f.path.extension() {
                        Some(extension) => match extension.to_str() {
                            Some("pdf") | Some("epub") => Some(f),
                            _ => None,
                        },
                        None => None,
                    }
                }
            })
            .build()?)
    }

    pub fn set_explorer_theme(&mut self, theme: Theme) {
        self.file_explorer.set_theme(theme);
    }

    pub fn file_explorer(&self) -> &FileExplorer {
        &self.file_explorer
    }

    fn notify(&mut self, message: impl Into<String>, title: String, level: Level) {
        match Notification::new(message.into())
            .title(title)
            .timing(
                Timing::Fixed(Duration::from_millis(500)),
                Timing::Fixed(Duration::from_secs(5)),
                Timing::Fixed(Duration::from_millis(500)),
            )
            .border_style(Style::default().fg(match level {
                Level::Error => Color::Red,
                Level::Warn => Color::Yellow,
                _ => Color::Green,
            }))
            .max_size(SizeConstraint::Percentage(0.6), SizeConstraint::Absolute(4))
            .anchor(Anchor::TopRight)
            .animation(Animation::Slide)
            .level(level)
            .slide_direction(SlideDirection::FromTop)
            .auto_dismiss(AutoDismiss::After(Duration::from_secs(2)))
            .build()
        {
            Ok(notif) => {
                let _ = self.notifications.add(notif);
            }
            Err(e) => {
                tracing::error!(error = %e, "Could not build notification")
            }
        }
    }

    pub fn notification_tick(&mut self) {
        self.notifications.tick(Duration::from_millis(16));
    }

    pub fn notifications(&mut self) -> &mut Notifications {
        &mut self.notifications
    }

    pub fn open_help(&mut self) {
        self.help_scroll = 0;
        self.mode = Mode::Help;
    }

    pub fn set_help_scroll(&mut self, max_scroll: u16) {
        self.help_scroll = self.help_scroll.min(max_scroll);
    }

    pub fn get_help_scroll(&self) -> u16 {
        self.help_scroll
    }

    pub fn quit(&self) -> bool {
        self.quit
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn focus(&self) -> Focus {
        self.focus
    }

    pub fn tick(&mut self) -> &str {
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame = spinner[self.tick_counter % spinner.len()];
        self.tick_counter += 1;
        frame
    }
}
