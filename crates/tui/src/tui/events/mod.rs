use std::time::Duration;

use color_eyre::{Result, eyre::bail};
use crossterm::event::EventStream;
use futures::StreamExt;
use ratatui::{Terminal, backend::Backend};
use tokio::time::interval;

use crate::tui::{app::App, ui::draw};

pub async fn run<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let mut events = EventStream::new();
    let mut tick = interval(Duration::from_millis(16));

    loop {
        // HACK: couldn't hoist the error with the '?' operator
        if let Err(e) = terminal.draw(|f| draw(f, app)) {
            bail!("{e}");
        }

        app.notification_tick();
        tokio::select! {
            maybe_event = events.next() => {
                if let Some(event) = maybe_event && let Ok(event) = event {
                    app.handle(event).await?;
                }
            }
            _ = tick.tick() => {
                app.poll_messages().await?;
            }
        }

        if app.quit() {
            break;
        } else {
            app.request_cover_for_selected();
            app.request_abstract_for_selected();
        }
    }
    Ok(())
}
