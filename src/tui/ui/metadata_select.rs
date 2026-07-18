use ratatui::{
    Frame,
    layout::Constraint,
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Padding},
};

use crate::{
    schema::graphics::Spannable,
    tui::{app::App, ui::source_tag::SourceTag},
};

pub fn draw_metadata_select_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(70), Constraint::Percentage(55));
    f.render_widget(Clear, center);

    let items: Vec<ListItem> = app
        .metadata
        .candidates
        .iter()
        .map(|c| {
            let authors = if c.authors.is_empty() {
                "Unknown Author".to_string()
            } else {
                c.authors.join(", ")
            };
            let date = c
                .publication_date
                .clone()
                .unwrap_or_else(|| "—".to_string());

            let source_spans: Vec<Span> = c
                .sources
                .iter()
                .flat_map(|s| vec![SourceTag(s).to_span(), Span::raw(" ")])
                .collect();

            ListItem::new(vec![
                Line::from(vec![
                    format!("[{}] ", c.item_type).dim(),
                    c.title.clone().bold(),
                ]),
                Line::from(vec![authors.into(), "  •  ".dim(), date.into()]),
                Line::from(source_spans),
                Line::raw(""),
            ])
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .border_style(Style::default().blue())
                .padding(Padding::uniform(1))
                .title(Line::from("Select metadata".bold()).yellow().italic())
                .title_bottom(
                    Line::from(" <Enter> Confirm  <Esc> Cancel ")
                        .right_aligned()
                        .yellow(),
                ),
        )
        .highlight_style(Style::default().green());

    f.render_stateful_widget(list, center, &mut app.metadata.list_state);
}
