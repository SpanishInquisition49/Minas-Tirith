use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph},
};

use crate::{
    schema::collection::Collection,
    tui::app::{App, Focus, collection::AssignMode},
};

pub fn draw_collection_sidebar(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = matches!(app.focus, Focus::Collections);
    let border_style = if focused {
        Style::default().green()
    } else {
        Style::default().blue()
    };

    let items: Vec<ListItem> = app
        .collections
        .items
        .iter()
        .map(|c| ListItem::new(c.name.clone()))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .border_style(border_style)
                .padding(Padding::uniform(1))
                .title(Line::from(" Collections ".bold().italic().yellow()))
                .title_bottom(
                    Line::from(vec![
                        " New: ".yellow(),
                        "<n> ".green(),
                        "Delete: ".yellow(),
                        "<d> ".green(),
                        "Export Bibtex: ".yellow(),
                        "<b> ".green(),
                    ])
                    .right_aligned(),
                ),
        )
        .highlight_style(Style::default().green());

    f.render_stateful_widget(list, area, &mut app.collections.list_state);
}

pub fn draw_collection_create_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(40), Constraint::Length(3));
    f.render_widget(Clear, center);
    let input = Paragraph::new(app.collections.name_input.value()).block(
        Block::bordered()
            .border_set(border::THICK)
            .border_style(Style::default().blue())
            .title(" New collection name ".yellow().italic())
            .title_bottom(
                Line::from(" <Enter> Confirm  <Esc> Cancel ")
                    .right_aligned()
                    .yellow(),
            ),
    );
    f.render_widget(input, center);
    let x = app.collections.name_input.visual_cursor() as u16 + 1;
    f.set_cursor_position((center.x + x, center.y + 1));
}

pub fn draw_collection_assign_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(50), Constraint::Percentage(50));
    f.render_widget(Clear, center);

    let Some(state) = &mut app.collections.assign else {
        return;
    };

    let (items, title) = match state.mode {
        AssignMode::Items => {
            let items = app
                .items
                .iter()
                .map(|i| {
                    let mark = if state.selected.contains(&i.id) {
                        "[x]"
                    } else {
                        "[ ]"
                    };
                    ListItem::new(format!("{mark} {}", i.fields.title))
                })
                .collect::<Vec<_>>();
            let collection_name = app.collections.items.iter().find_map(|c| {
                if c.id == state.id {
                    Some(c.name.as_str())
                } else {
                    None
                }
            });
            (
                items,
                format!(" Edit items for '{}' ", collection_name.unwrap_or_default()),
            )
        }
        AssignMode::Collections => {
            let items = app
                .collections
                .items
                .iter()
                // NOTE: exclude the trivial collection "All"
                .filter(|c| !Collection::is_trivial_collection(c.id))
                .map(|c| {
                    let mark = if state.selected.contains(&c.id) {
                        "[x]"
                    } else {
                        "[ ]"
                    };
                    ListItem::new(format!("{mark} {}", c.name))
                })
                .collect::<Vec<_>>();
            let item_title = app.items.iter().find_map(|i| {
                if i.id == state.id {
                    Some(i.fields.title.as_str())
                } else {
                    None
                }
            });
            (
                items,
                format!(
                    " Edit collections for '{}' ",
                    item_title.unwrap_or_default()
                ),
            )
        }
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .border_style(Style::default().blue())
                .padding(Padding::uniform(1))
                .title(Line::from(title.bold().italic().yellow()))
                .title_bottom(
                    Line::from(vec![
                        " Toggle: ".yellow(),
                        "<Space>".green(),
                        " Save: ".yellow(),
                        "<Ctrl+S>".green(),
                        " Cancel: ".yellow(),
                        "<Esc> ".green(),
                    ])
                    .right_aligned(),
                ),
        )
        .highlight_style(Style::default().green());

    f.render_stateful_widget(list, center, &mut state.list_state);
}
