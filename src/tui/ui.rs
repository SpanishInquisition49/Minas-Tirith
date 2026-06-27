use ratatui::{
    Frame,
    layout::{
        Constraint,
        Direction::{self, Horizontal},
        Layout, Rect,
    },
    style::{Modifier, Style, Styled, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Clear, FrameExt, List, ListItem, Padding, Paragraph},
};
use ratatui_image::StatefulImage;

use crate::tui::app::{App, Mode};

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
    }
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
                .padding(Padding::uniform(1))
                .title(title)
                .title_bottom(bottom_line.right_aligned()),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED).yellow());

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_details(f: &mut Frame, app: &mut App, area: Rect) {
    let title = Line::from("Details".bold());
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .padding(Padding::uniform(1))
        .title(title);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(item) = app.selected_item() else {
        f.render_widget(Paragraph::new("No selected item"), inner);
        return;
    };

    let has_cover_url = item.fields.cover_image_url.is_some();
    let titles_style = Style::new().bold().dark_gray();
    let mut card = vec![
        Line::from(vec![
            "Title: ".bold().style(titles_style),
            item.fields.title.to_string().into(),
        ]),
        Line::from(vec![
            "Type: ".bold().style(titles_style),
            item.fields.r#type.to_string().into(),
        ]),
    ];
    if let Some(date) = item.fields.publication_date.clone() {
        card.push(Line::from(vec![
            "Publication Date: ".bold().style(titles_style),
            date.into(),
        ]));
    }
    if let Some(doi) = item.fields.doi.clone() {
        card.push(Line::from(vec![
            "DOI: ".bold().style(titles_style),
            doi.into(),
        ]));
    }
    if let Some(isbn) = item.fields.isbn.clone() {
        card.push(Line::from(vec![
            "ISBN: ".bold().style(titles_style),
            isbn.into(),
        ]));
    }

    let cols = Layout::default()
        .direction(Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(0)])
        .split(inner);

    draw_cover_slot(f, app, cols[0], has_cover_url);
    f.render_widget(Paragraph::new(card), cols[1]);
}

fn draw_cover_slot(f: &mut Frame, app: &mut App, area: Rect, has_cover_url: bool) {
    let placeholder_block = Block::default()
        .borders(Borders::ALL)
        .padding(Padding::uniform(1))
        .border_style(Style::default().dim());

    if !has_cover_url {
        let placeholder = Paragraph::new("📚\nNo cover")
            .alignment(ratatui::layout::Alignment::Center)
            .block(placeholder_block);
        f.render_widget(placeholder, area);
        return;
    }

    match app.selected_cover() {
        Some(protocol) => {
            f.render_stateful_widget(StatefulImage::default(), area, protocol);
        }
        None => {
            let placeholder = Paragraph::new("Loading…")
                .alignment(ratatui::layout::Alignment::Center)
                .block(placeholder_block);
            f.render_widget(placeholder, area);
        }
    }
}

fn draw_status(f: &mut Frame, area: Rect) {
    let instructions = Line::from(vec![
        " Navigate Up ".into(),
        "<K>".blue().bold(),
        " Navigate Down ".into(),
        "<J>".blue().bold(),
        " Search ".into(),
        "</>".blue().bold(),
        " Add Item ".into(),
        "<A>".blue().bold(),
        " Quit ".into(),
        "<Q> ".blue().bold(),
    ]);
    f.render_widget(Paragraph::new(instructions), area);
}
