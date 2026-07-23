// src/tui/ui/help.rs
use ratatui::{
    Frame,
    layout::Constraint,
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Clear, Padding, Paragraph, Wrap},
};

use crate::{schema::keybindings::KEYBINDINGS, tui::app::App};

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
        .title(" Keybindings ".bold().italic().yellow())
        .title_bottom(
            Line::from(vec![
                " Scroll: ".yellow(),
                "<j/k> ".green(),
                " Close: ".yellow(),
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
            lines.push(Line::from(entry.section.bold().yellow()));
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

    let max_scroll = total_wrapped_lines.saturating_sub(inner.height as usize) as u16;
    app.help_scroll = app.help_scroll.min(max_scroll);

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((app.help_scroll, 0));
    f.render_widget(paragraph, inner);
}
