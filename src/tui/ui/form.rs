use std::str::FromStr;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Padding, Paragraph},
};

use crate::{
    schema::form::{FIELD_LABELS, Field},
    tui::app::App,
};

pub fn draw_metadata_edit_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(80), Constraint::Percentage(80));
    f.render_widget(Clear, center);

    let Some(form) = &mut app.metadata.form else {
        return;
    };

    let title = if app.metadata.saving {
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
            Constraint::Ratio(1, 10),
            Constraint::Ratio(2, 10),
            Constraint::Ratio(1, 10),
            Constraint::Ratio(1, 10),
            Constraint::Ratio(1, 10),
            Constraint::Ratio(1, 10),
            Constraint::Ratio(1, 10),
            Constraint::Ratio(1, 10),
            Constraint::Ratio(1, 10),
        ])
        .split(inner);
    f.render_widget(block, center);

    let mut lines: Vec<Line> = Vec::new();
    if let Some(err) = &app.metadata.last_error {
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

    let editing = form.editing;

    for (i, label) in FIELD_LABELS.iter().enumerate() {
        let Ok(current_field) = Field::from_str(label) else {
            unreachable!() // NOTE: converting from the static array can't fail
        };
        let is_selected = form.field == current_field;
        let (style, title_style) = if is_selected {
            (Style::default().white(), Style::default().green())
        } else {
            (Style::default().dark_gray(), Style::default().yellow())
        };

        if current_field == Field::Description {
            // Campo multi-riga: il widget gestisce da solo il proprio cursore,
            // non passa mai per f.set_cursor_position.
            let cursor_style = if editing && is_selected {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };

            form.description.set_style(style);
            form.description.set_cursor_style(cursor_style);
            form.description.set_cursor_line_style(Style::default());
            form.description.set_block(
                Block::bordered()
                    .title(format!(" {label} "))
                    .border_style(Style::default().blue())
                    .title_style(title_style),
            );

            f.render_widget(&form.description, rows[i]);
            continue;
        }

        let width = rows[i].width.max(3) - 3;
        let scroll = match current_field {
            Field::Title => form.title.visual_scroll(width as usize),
            Field::Description => unreachable!(),
            Field::Doi => form.doi.visual_scroll(width as usize),
            Field::Isbn => form.isbn.visual_scroll(width as usize),
            Field::PublicationDate => form.publication_date.visual_scroll(width as usize),
            Field::Tags => form.tags.visual_scroll(width as usize),
            Field::CoverUrl => form.cover_image_url.visual_scroll(width as usize),
            Field::Container => form.container.visual_scroll(width as usize),
        };
        let value = form.field_value(&current_field);
        let title = form.field_title(&current_field);

        let input = Paragraph::new(value)
            .style(style)
            .scroll((0, scroll as u16))
            .block(
                Block::bordered()
                    .title(format!(" {title} "))
                    .border_style(Style::default().blue())
                    .title_style(title_style),
            );
        f.render_widget(input, rows[i]);

        if editing && is_selected {
            let x = match current_field {
                Field::Title => form.title.visual_cursor().max(scroll) - scroll + 1,
                Field::Description => unreachable!(),
                Field::Doi => form.doi.visual_cursor().max(scroll) - scroll + 1,
                Field::Isbn => form.isbn.visual_cursor().max(scroll) - scroll + 1,
                Field::PublicationDate => {
                    form.publication_date.visual_cursor().max(scroll) - scroll + 1
                }
                Field::Tags => form.tags.visual_cursor().max(scroll) - scroll + 1,
                Field::CoverUrl => form.cover_image_url.visual_cursor().max(scroll) - scroll + 1,
                Field::Container => form.container.visual_cursor().max(scroll) - scroll + 1,
            };
            f.set_cursor_position((rows[i].x + x as u16, rows[i].y + 1))
        }
    }
    f.render_widget(Paragraph::new(lines), rows[8]);
}
