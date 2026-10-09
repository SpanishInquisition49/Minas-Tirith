use std::time::Duration;

use color_eyre::{Result, eyre::bail};
use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::{Terminal, backend::Backend};

use crate::tui::{app::App, ui::draw};

/// Run the main TUI event loop: render each frame, then either handle the
/// next input event or poll background-task messages on a ~16ms tick,
/// until the app requests to quit.
/// # Errors
/// Returns an error if drawing the frame or handling an event/message
/// fails.
pub async fn run<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let mut events = EventStream::new();

    loop {
        let animating = app.notifications().has_notification() || app.is_searching_metadata();
        let timeout = if animating {
            Duration::from_millis(16)
        } else {
            Duration::from_millis(500)
        };
        let mut redraw = false;

        tokio::select! {
            maybe_event = events.next() => {
                if let Some(Ok(event)) = maybe_event {app.handle(event).await?; redraw = true}
            }
            Some(msg) = app.task_channel_rx.recv() => { app.handle_message(msg).await?; redraw = true; }
            Some(msg) = app.cover_channel_rx.recv() => { app.handle_cover_message(msg); redraw = true; }
            () = tokio::time::sleep(timeout) => {redraw = animating;}
        }

        if animating {
            app.notification_tick();
        }

        // HACK: couldn't hoist the error with the '?' operator
        if redraw && let Err(e) = terminal.draw(|f| draw(f, app)) {
            bail!("{e}");
        }

        if app.quit() {
            break;
        }
        app.request_cover_for_selected();
        app.request_abstract_for_selected();
    }
    Ok(())
}
