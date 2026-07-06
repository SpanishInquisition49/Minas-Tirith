use ratatui::{
    Frame,
    layout::{
        Constraint,
        Direction::{self, Horizontal},
        Layout, Rect,
    },
    style::{Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, FrameExt, List, ListItem, Padding, Paragraph},
};

use crate::tui::{
    app::{App, Mode},
    ui::{details::draw_details, form::draw_metadata_edit_popup},
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let main = Layout::default()
        .direction(Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(outer[0]);

    draw_list(f, app, main[0]);
    draw_details(f, app, main[1]);
    draw_status(f, outer[1]);
    match app.mode {
        Mode::Normal => {}                      // NO additional rendering
        Mode::Insert => draw_add_popup(f, app), // Add item popup
        Mode::Search => todo!(),
        Mode::MetadataSelect => draw_metadata_select_popup(f, app),
        Mode::MetadataEdit => draw_metadata_edit_popup(f, app),
    }
}

fn draw_metadata_select_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(60), Constraint::Percentage(40));
    f.render_widget(Clear, center);

    let items: Vec<ListItem> = app
        .metadata_candidates
        .iter()
        .map(|c| {
            let authors = c.authors().join(", ");
            let date = c.publication_date().unwrap_or_default();
            ListItem::new(format!(
                "[{}] {}  —  {}  ({})",
                c.source(),
                c.title(),
                authors,
                date,
            ))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .border_style(Style::default().blue())
                .padding(Padding::uniform(1))
                .title(Line::from("Select metadata".bold()))
                .title_bottom(Line::from(" <Enter> Confirm  <Esc> Cancel ").right_aligned()),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED).yellow());

    f.render_stateful_widget(list, center, &mut app.metadata_list_state);
}

fn draw_add_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(60), Constraint::Percentage(20));
    f.render_widget(Clear, center);
    f.render_widget_ref(app.file_explorer.widget(), center);
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .items
        .iter()
        .map(|i| ListItem::new(i.fields.title.clone()))
        .collect();

    let index = app.items_list_state.selected().unwrap_or_default() + 1;
    let total = app.items.len();
    let bottom_line = Line::from(format!(" {index} of {total} "));
    let title = Line::from("Tomes".bold());
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .border_style(Style::default().blue())
                .padding(Padding::uniform(1))
                .title(title)
                .title_bottom(bottom_line.right_aligned()),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED).yellow());

    f.render_stateful_widget(list, area, &mut app.items_list_state);
}

fn draw_status(f: &mut Frame, area: Rect) {
    let instructions = Line::from(vec![
        " Navigate Up ".into(),
        "<K>".blue().bold(),
        " Navigate Down ".into(),
        "<J>".blue().bold(),
        " Search ".into(),
        "</>".blue().bold(),
        " Add Tome ".into(),
        "<A>".blue().bold(),
        " Edit Tome ".into(),
        "<E>".blue().bold(),
        " Quit ".into(),
        "<Q> ".blue().bold(),
    ]);
    f.render_widget(Paragraph::new(instructions), area);
}
