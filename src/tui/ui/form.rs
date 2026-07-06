use ratatui::{
    Frame,
    layout::Constraint,
    style::{Modifier, Style, Stylize},
    symbols::{self, border},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph},
};

use crate::{schema::form::FIELD_LABELS, tui::app::App};

pub fn draw_metadata_edit_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(60), Constraint::Percentage(60));
    f.render_widget(Clear, center);

    let Some(form) = &app.metadata_form else {
        return;
    };

    let title = if app.saving {
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame = spinner[(app.tick_counter as usize) % spinner.len()];
        Line::from(format!(" {frame} Saving... ").bold())
    } else {
        Line::from(" Edit metadata ".bold())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .border_style(Style::default().blue())
        .padding(Padding::uniform(1))
        .title(title)
        .title_bottom(
            Line::from(" Move <j/k> Edit field <Enter> Save <Ctrl+S> Cancel <Esc> ")
                .right_aligned(),
        );

    let inner = block.inner(center);
    f.render_widget(block, center);

    let mut lines: Vec<Line> = Vec::new();
    if let Some(err) = &app.last_error {
        lines.push(Line::from(Span::styled(
            format!("Error: {err}"),
            Style::default().red(),
        )));
    }
    lines.push(Line::from(vec![
        "Type: ".bold(),
        form.item_type.to_string().into(),
        " (press 't' to cylce)".dim(),
    ]));
    lines.push(Line::raw(""));
    for (i, label) in FIELD_LABELS.iter().enumerate() {
        let value = form.field_value(i);
        let is_selected = i == form.filed_index;
        let prefix = if is_selected { "> " } else { "  " };
        let style = if is_selected {
            Style::default().add_modifier(Modifier::REVERSED).yellow()
        } else {
            Style::default()
        };
        let suffix = if is_selected && form.editing { "|" } else { "" };
        lines.push(Line::from(Span::styled(
            format!("{prefix}{label}: {value}{suffix}"),
            style,
        )));
    }
    f.render_widget(Paragraph::new(lines), inner);
}
