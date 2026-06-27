use ratatui::{
    Frame,
    layout::{
        Constraint,
        Direction::{self, Horizontal},
        Layout, Rect,
    },
    style::{Modifier, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::tui::app::App;

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
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .items
        .iter()
        .map(|i| ListItem::new(i.fields.title.clone()))
        .collect();

    let index = app.list_state.selected().unwrap_or_default() + 1;
    let total = app.items.len();
    let bottom_line = Line::from(format!(" {index} of {total} "));
    let title = Line::from("Items".bold());
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .border_style(Style::default().blue())
                .title(title)
                .title_bottom(bottom_line.right_aligned()),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED).yellow());

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_details(f: &mut Frame, app: &App, area: Rect) {
    let text = match app.selected_item() {
        Some(item) => item.fields.to_string(),
        None => "No selected item".to_string(),
    };
    let title = Line::from("Details".bold());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .title(title);
    f.render_widget(Paragraph::new(text).block(block), area);
}

fn draw_status(f: &mut Frame, area: Rect) {
    let instructions = Line::from(vec![
        " Naveigate Up ".into(),
        "<K>".blue().bold(),
        " Navigate Down ".into(),
        "<J>".blue().bold(),
        " Search ".into(),
        "</>".blue().bold(),
        " Quit ".into(),
        "<Q> ".blue().bold(),
    ]);
    f.render_widget(Paragraph::new(instructions), area);
}
