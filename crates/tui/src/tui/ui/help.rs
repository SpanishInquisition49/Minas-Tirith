use minastirith_core::schema::keybindings::KEYBINDINGS;
use ratatui::{
    Frame,
    layout::Constraint,
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap},
};

use crate::tui::{app::App, ui::icons};

fn section_icon(section: &str) -> &'static str {
    match section {
        "Items" => icons::BOOK,
        "Collections" | "Collection Create" | "Collection Assign" => icons::FOLDER,
        "Global" => icons::GLOBE,
        "File Explorer" => icons::SEARCH,
        "Metadata Selection" => icons::LIST,
        "Metadata Edit" => icons::PENCIL,
        "Library Publish" => icons::UPLOAD,
        "Library Subscribe" => icons::RSS,
        "Library Browse" => icons::BOOK,
        "Library Manage" => icons::TICKET,
        "Help" => icons::KEYBOARD,
        _ => icons::INFO,
    }
}

/// Render the help popup.
pub fn draw_help_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(70), Constraint::Percentage(80));
    f.render_widget(Clear, center);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .border_style(Style::default().blue())
        .padding(Padding::uniform(1))
        .title(format!(" {} Keybindings ", icons::KEYBOARD).bold().italic().yellow())
        .title_bottom(
            Line::from(vec![
                format!(" {} Scroll: ", icons::ARROWS_V).yellow(),
                "<j/k> ".green(),
                format!("{} Close: ", icons::CLOSE).yellow(),
                "<Esc> ".green(),
            ])
            .right_aligned(),
        );
    let inner = block.inner(center);
    f.render_widget(block, center);

    let mut lines: Vec<Line> = Vec::new();
    let mut current_section: Option<&str> = None;

    for entry in KEYBINDINGS {
        if current_section != Some(entry.section) {
            if current_section.is_some() {
                lines.push(Line::raw(""));
            }
            lines.push(Line::from(
                format!("{} {}", section_icon(entry.section), entry.section)
                    .bold()
                    .yellow(),
            ));
            current_section = Some(entry.section);
        }
        lines.push(Line::from(vec![
            format!("  {:<22}", entry.key).green(),
            entry.action.into(),
        ]));
    }

    let total_wrapped_lines: usize = lines
        .iter()
        .map(|line| {
            let width = line.width().max(1);
            width.div_ceil(inner.width.max(1) as usize)
        })
        .sum();

    let max_scroll = u16::try_from(total_wrapped_lines.saturating_sub(inner.height as usize))
        .unwrap_or(u16::MAX);
    app.set_help_scroll(max_scroll);

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((app.get_help_scroll(), 0));
    f.render_widget(paragraph, inner);
}
