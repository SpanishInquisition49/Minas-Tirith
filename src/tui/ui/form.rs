use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    symbols::border,
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
        let frame = spinner[app.tick_counter % spinner.len()];
        app.tick_counter += 1;
        Line::from(format!(" {frame} Saving... ").bold().italic().yellow())
    } else {
        Line::from(" Edit metadata ".bold().italic().yellow())
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .border_style(Style::default().blue())
        .padding(Padding::uniform(1))
        .title(title)
        .title_bottom(
            Line::from(vec![
                " Move".yellow(),
                " <j/k>".green(),
                " Edit field".yellow(),
                " <Enter>".green(),
                " Save".yellow(),
                " <Ctrl+S>".green(),
                " Cancel".yellow(),
                " <Esc> ".green(),
            ])
            .yellow()
            .right_aligned(),
        );

    let inner = block.inner(center);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 7),
            Constraint::Ratio(1, 7),
            Constraint::Ratio(1, 7),
            Constraint::Ratio(1, 7),
            Constraint::Ratio(1, 7),
            Constraint::Ratio(1, 7),
            Constraint::Ratio(1, 7),
        ])
        .split(inner);
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
        let width = rows[i].width.max(3) - 3;
        let scroll = match i {
            0 => form.title.visual_scroll(width as usize),
            1 => form.description.visual_scroll(width as usize),
            2 => form.doi.visual_scroll(width as usize),
            3 => form.isbn.visual_scroll(width as usize),
            4 => form.publication_date.visual_scroll(width as usize),
            5 => form.tags.visual_scroll(width as usize),
            _ => unreachable!(),
        };
        let value = form.field_value(i);
        let is_selected = i == form.field_index;
        let style = if is_selected {
            Style::default().yellow()
        } else {
            Style::default().white()
        };

        let input = Paragraph::new(value)
            .style(style)
            .scroll((0, scroll as u16))
            .block(Block::bordered().title(format!(" {label} ")));
        f.render_widget(input, rows[i]);
        if form.editing && is_selected {
            let x = match i {
                0 => form.title.visual_cursor().max(scroll) - scroll + 1,
                1 => form.description.visual_cursor().max(scroll) - scroll + 1,
                2 => form.doi.visual_cursor().max(scroll) - scroll + 1,
                3 => form.isbn.visual_cursor().max(scroll) - scroll + 1,
                4 => form.publication_date.visual_cursor().max(scroll) - scroll + 1,
                5 => form.tags.visual_cursor().max(scroll) - scroll + 1,
                _ => unreachable!(),
            };
            f.set_cursor_position((rows[i].x + x as u16, rows[i].y + 1))
        }
    }
    f.render_widget(Paragraph::new(lines), rows[6]);
}
