use minastirith_core::{
    schema::collection::Collection, state::collection::collection_assign::AssignMode,
};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Padding, Paragraph},
};

use crate::tui::{
    app::{App, Focus},
    ui::icons,
};

/// Render the collection sidebar.
pub fn draw_collection_sidebar(f: &mut Frame, app: &mut App, area: Rect) {
    let focused = matches!(app.focus(), Focus::Collections);
    let border_style = if focused {
        Style::default().green()
    } else {
        Style::default().blue()
    };

    let items: Vec<ListItem> = app
        .collections()
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
                .title(Line::from(
                    format!(" {} Collections ", icons::FOLDER).bold().italic().yellow(),
                ))
                .title_bottom(
                    Line::from(vec![
                        format!(" {} New: ", icons::PLUS).yellow(),
                        "<n> ".green(),
                        format!("{} Delete: ", icons::TRASH).yellow(),
                        "<d> ".green(),
                        format!("{} Export Bibtex: ", icons::DOWNLOAD).yellow(),
                        "<b> ".green(),
                    ])
                    .right_aligned(),
                ),
        )
        .highlight_style(Style::default().green());

    f.render_stateful_widget(list, area, app.collection_list_state_mut());
}

/// Render the collection-creation popup.
pub fn draw_collection_create_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(40), Constraint::Length(3));
    f.render_widget(Clear, center);
    let input = Paragraph::new(app.collection_input_field().value()).block(
        Block::bordered()
            .border_set(border::THICK)
            .border_style(Style::default().blue())
            .title(format!(" {} New collection name ", icons::FOLDER).yellow().italic())
            .title_bottom(
                Line::from(vec![
                    format!(" {} <Enter> Confirm  ", icons::CHECK).yellow(),
                    format!("{} <Esc> Cancel ", icons::CLOSE).yellow(),
                ])
                .right_aligned(),
            ),
    );
    f.render_widget(input, center);
    let x = u16::try_from(app.collection_input_field().visual_cursor()).unwrap_or_default() + 1;
    f.set_cursor_position((center.x + x, center.y + 1));
}

/// Render the item/collection assignment popup.
pub fn draw_collection_assign_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(50), Constraint::Percentage(50));
    f.render_widget(Clear, center);

    let Some(state) = app.collection_assign_state() else {
        return;
    };

    let (items, title) = match state.mode() {
        AssignMode::Items => {
            let items = app
                .items()
                .iter()
                .map(|i| {
                    let mark = if state.selected().contains(&i.id) {
                        icons::CHECKBOX_CHECKED
                    } else {
                        icons::CHECKBOX_UNCHECKED
                    };
                    ListItem::new(format!("{mark} {}", i.fields.title))
                })
                .collect::<Vec<_>>();
            let collection_name = app.collections().iter().find_map(|c| {
                if c.id == state.id() {
                    Some(c.name.as_str())
                } else {
                    None
                }
            });
            (
                items,
                format!(
                    " {} Edit items for '{}' ",
                    icons::FOLDER,
                    collection_name.unwrap_or_default()
                ),
            )
        }
        AssignMode::Collections => {
            let items = app
                .collections()
                .iter()
                // NOTE: exclude the trivial collection "All"
                .filter(|c| !Collection::is_trivial_collection(c.id))
                .map(|c| {
                    let mark = if state.selected().contains(&c.id) {
                        icons::CHECKBOX_CHECKED
                    } else {
                        icons::CHECKBOX_UNCHECKED
                    };
                    ListItem::new(format!("{mark} {}", c.name))
                })
                .collect::<Vec<_>>();
            let item_title = app.items().iter().find_map(|i| {
                if i.id == state.id() {
                    Some(i.fields.title.as_str())
                } else {
                    None
                }
            });
            (
                items,
                format!(
                    " {} Edit collections for '{}' ",
                    icons::FOLDER,
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
                        format!(" {} Toggle: ", icons::CHECK).yellow(),
                        "<Space>".green(),
                        format!(" {} Save: ", icons::UPLOAD).yellow(),
                        "<Ctrl+S>".green(),
                        format!(" {} Cancel: ", icons::CLOSE).yellow(),
                        "<Esc> ".green(),
                    ])
                    .right_aligned(),
                ),
        )
        .highlight_style(Style::default().green());

    let Some(state) = app.collection_assign_state_mut() else {
        return;
    };
    let mut list_state = ListState::default();
    list_state.select(state.selected_index());
    f.render_stateful_widget(list, center, &mut list_state);
}
