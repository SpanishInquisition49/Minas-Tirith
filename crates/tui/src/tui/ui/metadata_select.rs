use ratatui::{
    Frame,
    layout::Constraint,
    style::{Style, Stylize},
    symbols::{self, border},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Padding},
};

use crate::{
    traits::Spannable,
    tui::{
        app::App,
        ui::{icons, source_tag::SourceTag},
    },
};

/// Render the metadata candidate selection popup.
pub fn draw_metadata_select_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(70), Constraint::Percentage(55));
    f.render_widget(Clear, center);
    let mut items = Vec::new();
    let sources: Vec<Vec<SourceTag>> = app
        .get_metadata_candidates()
        .iter()
        .map(|c| {
            c.sources
                .iter()
                .map(|s| SourceTag::new(s.clone()))
                .collect()
        })
        .collect();
    for (index, candidate) in app.get_metadata_candidates().iter().enumerate() {
        let authors = if candidate.authors.is_empty() {
            "Unknown Author".to_string()
        } else {
            candidate.authors.join(", ")
        };
        let date = candidate
            .publication_date
            .clone()
            .unwrap_or_else(|| "—".to_string());

        let spans: Vec<Span<'_>> = sources[index]
            .iter()
            .flat_map(|s| vec![s.to_span(), Span::raw(" ")])
            .collect();

        let lines = ListItem::new(vec![
            Line::from(vec![
                format!(
                    "{} [{}] ",
                    icons::item_type_icon(candidate.item_type),
                    candidate.item_type
                )
                .dim(),
                candidate.title.clone().bold(),
            ]),
            Line::from(vec![
                authors.into(),
                format!(" {} ", symbols::DOT).dim(),
                date.into(),
            ]),
            Line::from(spans),
            Line::raw(""),
        ]);
        items.push(lines);
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .border_style(Style::default().blue())
                .padding(Padding::uniform(1))
                .title(
                    Line::from(format!("{} Select metadata", icons::LIST).bold())
                        .yellow()
                        .italic(),
                )
                .title_bottom(
                    Line::from(vec![
                        Span::from(format!(" {} <Enter> Confirm  ", icons::CHECK)),
                        Span::from(format!("{} <Esc> Cancel ", icons::CLOSE)),
                    ])
                    .right_aligned()
                    .yellow(),
                ),
        )
        .highlight_style(Style::default().green());

    f.render_stateful_widget(list, center, app.get_metadata_list_state_mut());
}
