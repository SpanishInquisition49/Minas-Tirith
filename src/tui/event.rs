use std::time::Duration;

use color_eyre::eyre::bail;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyModifiers};
use futures::StreamExt;
use ratatui::{Terminal, backend::Backend};
use tokio::time::interval;

use crate::tui::app::{App, Focus, Mode};
use crate::tui::ui::base::draw;

pub async fn run<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> color_eyre::Result<()> {
    let mut events = EventStream::new();
    let mut tick = interval(Duration::from_millis(16));

    loop {
        // HACK: couldn't hoist the error with the '?' operator
        if let Err(e) = terminal.draw(|f| draw(f, app)) {
            bail!("{e}");
        }

        app.notifications.tick(Duration::from_millis(16));

        tokio::select! {
            maybe_event = events.next() => {
                if let Some(Ok(event)) = maybe_event {
                    if let Mode::Insert = app.mode {
                        app.file_explorer.handle(&event)?;
                    };
                    if let Mode::MetadataEdit = app.mode && let Some(form) = &mut app.metadata.form && form.editing {
                        form.handle_event(&event);
                    }
                    if let Mode::CollectionCreate = app.mode {
                        app.collections.handle_event(&event);
                    }
                    if let Event::Key(key) = event {
                        handle_key(app, key).await?;
                    }
                    if let Mode::LibraryPublish = app.mode {
                        app.library.handle_publish_event(&event);
                    }
                    if let Mode::LibrarySubscribe = app.mode {
                        app.library.handle_subscribe_event(&event);
                    }
                }
            }
            _ = tick.tick() => {
                app.poll_messages().await?;
            }
        }

        if app.quit {
            break;
        } else {
            app.request_cover_for_selected();
            app.request_abstract_for_selected();
        }
    }
    Ok(())
}

async fn handle_key(app: &mut App, key: KeyEvent) -> color_eyre::Result<()> {
    match app.mode {
        Mode::Normal => {
            if let KeyCode::Tab = key.code {
                app.toggle_focus();
                return Ok(());
            }
            if let KeyCode::Char('?') = key.code {
                app.open_help();
                return Ok(());
            }
            match app.focus {
                Focus::Items => match key.code {
                    KeyCode::Char('[') => app.tabs_prev(),
                    KeyCode::Char(']') => app.tabs_next(),
                    KeyCode::Char('q') => app.quit = true,
                    KeyCode::Char('j') | KeyCode::Down => app.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => app.select_prev(),
                    KeyCode::Enter => app.request_file_opening()?,
                    KeyCode::Char('a') => app.request_open_file_picker()?,
                    KeyCode::Char('e') => {
                        app.open_metadata_edit_for_selected_item();
                    }
                    KeyCode::Char('b') => app.send_bibtex_to_system_clipboard(),
                    KeyCode::Char('c') => app.open_collection_assign_for_selected(),
                    KeyCode::Char('L') => app.open_library_browse(),
                    KeyCode::Char('/') => app.mode = Mode::Search,
                    _ => {}
                },
                Focus::Collections => match key.code {
                    KeyCode::Char('j') | KeyCode::Down => app.select_collection_next(),
                    KeyCode::Char('k') | KeyCode::Up => app.select_collection_prev(),
                    KeyCode::Char('n') => app.open_collection_create(),
                    KeyCode::Char('c') => app.open_item_assign_for_selected_collection(),
                    KeyCode::Char('b') => app.bulk_bibtex_to_system_clipboard(),
                    KeyCode::Char('d') => app.delete_collection().await?,
                    KeyCode::Enter => app.confirm_collection_selection(),
                    KeyCode::Char('q') => app.quit = true,
                    KeyCode::Char('p') => app.open_library_publish_for_selected(),
                    KeyCode::Char('L') => app.open_library_manage(),
                    _ => {}
                },
            }
        }
        Mode::Insert => match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Esc)
            | (KeyModifiers::NONE, KeyCode::Backspace)
            | (KeyModifiers::NONE, KeyCode::Char('q')) => app.mode = Mode::Normal,
            (KeyModifiers::NONE, KeyCode::Char('a')) => {
                app.request_fetch_metadata_candidates();
            }
            _ => {}
        },
        Mode::MetadataSelect => match key.code {
            KeyCode::Char('j') | KeyCode::Down => app.select_metadata_next(),
            KeyCode::Char('k') | KeyCode::Up => app.select_metadata_prev(),
            KeyCode::Enter => app.open_metadata_edit_for_candidate(),
            KeyCode::Esc | KeyCode::Char('q') => app.cancel_metadata_selection(),
            _ => {}
        },
        Mode::Search => match key.code {
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q') => app.mode = Mode::Normal,
            _ => {}
        },
        Mode::MetadataEdit => {
            let editing = app
                .metadata
                .form
                .as_ref()
                .map(|f| f.editing)
                .unwrap_or(false);
            if editing {
                let Some(form) = app.metadata.form.as_mut() else {
                    return Ok(());
                };
                match key.code {
                    KeyCode::Enter | KeyCode::Esc => form.editing = false,
                    _ => {}
                }
            } else {
                match (key.modifiers, key.code) {
                    (KeyModifiers::CONTROL, KeyCode::Char('s')) => app.confirm_metadata_form(),

                    (_, KeyCode::Char('j')) | (_, KeyCode::Down) => {
                        if let Some(f) = app.metadata.form.as_mut() {
                            f.next_field();
                        }
                    }
                    (_, KeyCode::Char('k')) | (_, KeyCode::Up) => {
                        if let Some(f) = app.metadata.form.as_mut() {
                            f.prev_field();
                        }
                    }
                    (_, KeyCode::Char('t')) => {
                        if let Some(f) = app.metadata.form.as_mut() {
                            f.cycle_item_type();
                        }
                    }
                    (_, KeyCode::Enter) => {
                        if let Some(f) = app.metadata.form.as_mut() {
                            f.editing = true;
                        }
                    }
                    (_, KeyCode::Esc) | (_, KeyCode::Char('q')) => {
                        app.cancel_metadata_form();
                    }
                    _ => {}
                }
            }
        }
        Mode::CollectionCreate => match key.code {
            KeyCode::Enter => app.confirm_collection_create().await?,
            KeyCode::Esc => app.close_collection_create(),
            _ => {}
        },
        Mode::CollectionAssign => match (key.modifiers, key.code) {
            (_, KeyCode::Char('j')) | (_, KeyCode::Down) => app.collection_assign_next(),
            (_, KeyCode::Char('k')) | (_, KeyCode::Up) => app.collection_assign_prev(),
            (_, KeyCode::Char(' ')) | (_, KeyCode::Enter) => app.collection_assign_toggle_current(),
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => app.confirm_collection_assign().await?,
            (_, KeyCode::Esc) | (_, KeyCode::Char('q')) => app.cancel_collection_assign(),
            _ => {}
        },
        Mode::LibraryPublish => match (key.modifiers, key.code) {
            (_, KeyCode::Tab) => app.library.publish_next_field(),
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                app.publish_collection_as_library().await?
            }
            (_, KeyCode::Esc) | (_, KeyCode::Char('q')) => app.cancel_publish(),
            _ => {}
        },
        Mode::LibrarySubscribe => match (key.modifiers, key.code) {
            (_, KeyCode::Tab) => app.library.subscribe_next_field(),
            (KeyModifiers::CONTROL, KeyCode::Char('s')) => app.confirm_library_subscribe().await?,
            (_, KeyCode::Esc) => app.cancel_subscribe(),
            _ => {}
        },
        Mode::LibraryBrowse => match key.code {
            KeyCode::Tab => app.library.browse_toggle_focus(),
            KeyCode::Char('j') | KeyCode::Down => app.library.browse_select_next(),
            KeyCode::Char('k') | KeyCode::Up => app.library.browse_select_prev(),
            KeyCode::Char('r') => app.refresh_current_library().await?,
            KeyCode::Char('d') => app.unsubscribe_selected_library().await?,
            KeyCode::Char('a') => {
                app.library.open_subscribe();
                app.mode = Mode::LibrarySubscribe;
            }
            KeyCode::Enter => app.library.browse_confirm_import(),
            KeyCode::Esc => {
                app.library.close_browse();
                app.mode = Mode::Normal;
            }
            _ => {}
        },
        Mode::LibraryManage => match key.code {
            KeyCode::Char('j') | KeyCode::Down => app.library.manage_select_next(),
            KeyCode::Char('k') | KeyCode::Up => app.library.manage_select_prev(),
            KeyCode::Char('t') => app.library.manage_generate_ticket().await?,
            KeyCode::Char('c') => app.copy_current_ticket_to_clipboard(),
            KeyCode::Char('d') => app.library.manage_delete_selected().await?,
            KeyCode::Esc | KeyCode::Char('q') => {
                app.library.close_manage();
                app.mode = Mode::Normal;
            }
            _ => {}
        },
        Mode::Help => match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                app.help_scroll = app.help_scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.help_scroll = app.help_scroll.saturating_sub(1);
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = Mode::Normal;
                app.help_scroll = 0;
            }
            _ => {}
        },
    }
    Ok(())
}
