use color_eyre::eyre::eyre;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent};
use futures::StreamExt;
use ratatui::{Terminal, backend::Backend};

use crate::tui::{app::App, ui::draw};

pub async fn run<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> color_eyre::Result<()> {
    let mut events = EventStream::new();

    loop {
        // HACK: couldn't hoist the error with the '?' operator
        if let Err(e) = terminal.draw(|f| draw(f, app)) {
            return Err(eyre!("{e}"));
        }

        if let Some(Ok(Event::Key(key))) = events.next().await {
            handle_key(app, key).await?;
        }

        if app.quit {
            break;
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
