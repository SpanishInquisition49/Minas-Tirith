use std::time::Duration;

use color_eyre::eyre::eyre;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use futures::StreamExt;
use ratatui::{Terminal, backend::Backend};
use tokio::time::interval;

use crate::tui::{app::App, ui::draw};

pub async fn run<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> color_eyre::Result<()> {
    let mut events = EventStream::new();
    let mut tick = interval(Duration::from_millis(100));

    loop {
        // HACK: couldn't hoist the error with the '?' operator
        if let Err(e) = terminal.draw(|f| draw(f, app)) {
            return Err(eyre!("{e}"));
        }

        tokio::select! {
            maybe_event = events.next() => {
                if let Some(Ok(Event::Key(key))) = maybe_event {
                    handle_key(app, key).await?;
                }
            }
            _ = tick.tick() => {
                app.poll_covers();
            }
        }

        if app.quit {
            break;
        } else {
            app.request_cover_for_selected();
        }
    }
    Ok(())
}

async fn handle_key(app: &mut App, key: KeyEvent) -> color_eyre::Result<()> {
    match key.code {
        KeyCode::Char('q') => app.quit = true,
        KeyCode::Char('j') => app.select_next(),
        KeyCode::Char('k') => app.select_prev(),
        _ => {}
    }
    Ok(())
}
