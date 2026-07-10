use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols::{self, border},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
};
use ratatui_image::StatefulImage;

use crate::{metadata::common_metadata::ItemType, schema::graphics::Spannable, tui::app::App};

pub fn draw_details(f: &mut Frame, app: &mut App, area: Rect) {
    let title = Line::from(" Details ".yellow().bold().italic());
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .border_style(Style::default().blue())
        .padding(Padding::uniform(1))
        .title(title);
    let inner = block.inner(area);

    let Some(item) = app.selected_item() else {
        f.render_widget(block, area);
        f.render_widget(Paragraph::new("No selected item".yellow()), inner);
        return;
    };
    block = block.title(format!(" {} ", item.fields.title).yellow().italic().bold());
    f.render_widget(block, area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(0)])
        .spacing(2)
        .split(inner);

    let item_type = ItemType::try_from(item.fields.r#type.as_str()).unwrap_or(ItemType::Misc);
    let has_cover_url = item.fields.cover_image_url.is_some();
    let titles_style = Style::new().bold().dark_gray();
    let text_width = cols[1].width;
    let mut card = vec![];
    card.push(Line::from("Authors:\n".bold().style(titles_style)));
    let authors_pills: Vec<(usize, Span<'static>)> = item
        .authors
        .iter()
        .map(|a| (a.span_len(), a.to_span()))
        .collect();
    card.extend(wrap_pills(authors_pills, text_width));
    card.push(Line::from(Span::styled(
        symbols::DOT.repeat(text_width as usize),
        Style::default().dark_gray(),
    )));
    card.push(Line::from(vec![
        "Type: ".bold().style(titles_style),
        item.fields.r#type.to_string().into(),
    ]));
    if let Some(container) = &item.fields.container {
        let name = match item_type {
            ItemType::Book => "Publisher: ",
            ItemType::Article => "Journal: ",
            ItemType::Report => "Institution: ",
            ItemType::Thesis => "University: ",
            ItemType::Misc => "How Published: ",
        };
        card.extend(wrap_labeled_field(
            name,
            container,
            text_width,
            Style::default().bold().dark_gray(),
        ));
    }

    if let Some(date) = &item.fields.publication_date {
        card.push(Line::from(vec![
            "Publication Date: ".bold().style(titles_style),
            date.into(),
        ]));
    }
    if let Some(doi) = &item.fields.doi {
        card.push(Line::from(vec![
            "DOI: ".bold().style(titles_style),
            doi.into(),
        ]));
    }
    if let Some(isbn) = &item.fields.isbn {
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

    f.render_widget(Paragraph::new(card), cols[1]);
    draw_cover_slot(f, app, cols[0], has_cover_url);
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

fn wrap_labeled_field(
    label: &str,
    value: &str,
    width: u16,
    label_style: Style,
) -> Vec<Line<'static>> {
    let width = width as usize;
    let indent_width = label.chars().count();
    let content_width = width.saturating_sub(indent_width).max(1);

    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();

    for word in value.split_whitespace() {
        let candidate_len = if current.is_empty() {
            word.chars().count()
        } else {
            current.chars().count() + 1 + word.chars().count()
        };

        if candidate_len > content_width && !current.is_empty() {
            chunks.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    if chunks.is_empty() {
        chunks.push(String::new());
    }

    let indent = " ".repeat(indent_width);
    chunks
        .into_iter()
        .enumerate()
        .map(|(i, chunk)| {
            if i == 0 {
                Line::from(vec![
                    Span::styled(label.to_string(), label_style),
                    chunk.into(),
                ])
            } else {
                Line::from(vec![Span::raw(indent.clone()), chunk.into()])
            }
        })
        .collect()
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
