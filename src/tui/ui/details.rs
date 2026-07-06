use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};
use ratatui_image::StatefulImage;

use crate::{schema::graphics::Spannable, tui::app::App};

pub fn draw_details(f: &mut Frame, app: &mut App, area: Rect) {
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

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(0)])
        .spacing(2)
        .split(inner);

    let has_cover_url = item.fields.cover_image_url.is_some();
    let titles_style = Style::new().bold().dark_gray();
    let text_width = cols[1].width;
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
    card.push(Line::from(Span::styled(
        "·".repeat(text_width as usize),
        Style::default().dim(),
    )));
    card.push(Line::from("Authors:\n".bold().style(titles_style)));
    let authors_pills: Vec<(usize, Span<'static>)> = item
        .authors
        .iter()
        .map(|a| (a.span_len(), a.to_span()))
        .collect();
    card.extend(wrap_pills(authors_pills, text_width));
    card.push(Line::from(Span::styled(
        "·".repeat(text_width as usize),
        Style::default().dim(),
    )));
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

    if !item.tags.is_empty() {
        let mut tags: Vec<Span> = Vec::new();
        card.push(Line::from("Tags: ".bold().style(titles_style)));
        for tag in &item.tags {
            tags.push(tag.to_span());
            tags.push(Span::raw(" "));
        }
        card.push(Line::from(tags));
    }

    draw_cover_slot(f, app, cols[0], has_cover_url);
    f.render_widget(Paragraph::new(card), cols[1]);
}

fn wrap_pills(pills: Vec<(usize, Span<'static>)>, max_width: u16) -> Vec<Line<'static>> {
    let max_width = max_width as usize;
    let mut lines = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();
    let mut used = 0usize;

    for (w, span) in pills {
        if used + w > max_width && !current.is_empty() {
            lines.push(Line::from(std::mem::take(&mut current)));
            lines.push(Line::raw(""));
            used = 0;
        }
        current.push(span);
        current.push(Span::raw(" "));
        used += w + 1;
    }
    if !current.is_empty() {
        lines.push(Line::from(current));
    }
    lines
}

fn draw_cover_slot(f: &mut Frame, app: &mut App, area: Rect, has_cover_url: bool) {
    let cover_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(22), Constraint::Min(0)])
        .split(area)[0];

    let placeholder_block = Block::default()
        .borders(Borders::ALL)
        .padding(Padding::uniform(1))
        .border_style(Style::default().dim());

    if !has_cover_url {
        let placeholder = Paragraph::new(" \nNo cover")
            .alignment(ratatui::layout::Alignment::Center)
            .block(placeholder_block);
        f.render_widget(placeholder, cover_area);
        return;
    }

    match app.selected_cover() {
        Some(protocol) => {
            // Resize::Fit(None) è già il default: preserva aspect ratio
            f.render_stateful_widget(StatefulImage::default(), cover_area, protocol);
        }
        None => {
            let placeholder = Paragraph::new("Loading…")
                .alignment(ratatui::layout::Alignment::Center)
                .block(placeholder_block);
            f.render_widget(placeholder, cover_area);
        }
    }
}
