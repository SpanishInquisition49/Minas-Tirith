mod collection;
mod details;
mod form;
mod help;
pub(crate) mod icons;
mod library;
mod metadata_select;
pub mod source_tag;

use minastirith_core::{metadata::common_metadata::ItemType, state::item::TABS_LABELS};
use ratatui::{
    Frame,
    layout::{
        Constraint,
        Direction::{self, Horizontal},
        Layout, Offset, Rect,
    },
    style::{Modifier, Style, Stylize},
    symbols::{self, border},
    text::Line,
    widgets::{Block, Borders, Clear, FrameExt, List, ListItem, Padding, Paragraph, Tabs},
};
use ratatui_explorer::Theme;

use crate::tui::{
    app::{App, Mode},
    ui::{
        collection::{
            draw_collection_assign_popup, draw_collection_create_popup, draw_collection_sidebar,
        },
        details::draw_details,
        form::draw_metadata_edit_popup,
        help::draw_help_popup,
        library::{
            draw_library_browse_popup, draw_library_manage_popup, draw_library_publish_popup,
            draw_library_subscribe_popup,
        },
        metadata_select::draw_metadata_select_popup,
    },
};

/// Render the whole application for the current frame, dispatching to the
/// active mode's popups on top of the base layout.
pub fn draw(f: &mut Frame, app: &mut App) {
    let screen = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Max(3), Constraint::Fill(3)])
        .split(f.area());

    let main = Layout::default()
        .direction(Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(screen[1]);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(main[0]);

    let title = if app.is_searching_metadata() {
        format!(" {} Searching for metadata... ", app.tick())
            .bold()
            .yellow()
            .italic()
    } else {
        format!(" {} Pick a tome ", icons::BOOK).bold().yellow().italic()
    };
    let theme = Theme::default()
        .with_title_top(move |_| Line::from(title.clone()))
        .with_title_bottom(|_| {
            Line::from(vec![
                format!(" {} Select: ", icons::CHECK).yellow(),
                "<a> ".green(),
            ])
            .right_aligned()
        })
        .with_block(Block::bordered().border_set(border::THICK).blue())
        .with_highlight_item_style(Style::default().add_modifier(Modifier::REVERSED).yellow())
        .with_highlight_dir_style(Style::default().add_modifier(Modifier::REVERSED).yellow());

    app.set_explorer_theme(theme);

    draw_collection_sidebar(f, app, rows[0]);
    draw_list(f, app, rows[1]);
    draw_details(f, app, main[1]);
    draw_search_bar(f, app, screen[0]);
    match app.mode() {
        Mode::Normal | Mode::Search => {}       // No additional rendering
        Mode::Insert => draw_add_popup(f, app), // Add item pop-up
        Mode::MetadataSelect => draw_metadata_select_popup(f, app),
        Mode::MetadataEdit => draw_metadata_edit_popup(f, app),
        Mode::CollectionCreate => draw_collection_create_popup(f, app),
        Mode::CollectionAssign => draw_collection_assign_popup(f, app),
        Mode::LibraryPublish => draw_library_publish_popup(f, app),
        Mode::LibrarySubscribe => draw_library_subscribe_popup(f, app),
        Mode::LibraryBrowse => draw_library_browse_popup(f, app),
        Mode::LibraryManage => draw_library_manage_popup(f, app),
        Mode::Help => draw_help_popup(f, app),
    }
    app.notifications().render(f, f.area());
}

fn draw_add_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(60), Constraint::Percentage(60));
    f.render_widget(Clear, center);
    f.render_widget_ref(app.file_explorer().widget(), center);
}

fn draw_search_bar(f: &mut Frame, app: &mut App, area: Rect) {
    let searching = matches!(app.mode(), Mode::Search);
    let (text_style, border_style) = if searching {
        (Style::default().white(), Style::default().green())
    } else {
        (Style::default().dim(), Style::default().blue())
    };

    let search_input = app.item_search_input_mut();

    if searching {
        let width = area.width.max(3) - 3;
        let scroll = search_input.visual_scroll(width as usize);
        let x = search_input.visual_cursor().max(scroll) - scroll + 1;
        f.set_cursor_position((area.x + u16::try_from(x).unwrap_or_default(), area.y + 1));
    }

    let search_text = if app.item_search_input().value().is_empty() && !searching {
        "/ search..."
    } else {
        app.item_search_input().value()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .border_style(border_style)
        .title(format!(" {} Search ", icons::SEARCH).italic().bold().yellow());
    let value = Paragraph::new(search_text).block(block).style(text_style);
    f.render_widget(value, area);
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Max(1), Constraint::Fill(1)])
        .split(area);
    let tabs = Tabs::new(TABS_LABELS)
        .style(Style::default().italic())
        .highlight_style(Style::default().yellow().underlined().italic().bold())
        .select(app.selected_tab())
        .divider(symbols::DOT)
        .padding(" ", " ");

    let items: Vec<ListItem> = app
        .items()
        .iter()
        .map(|i| {
            let item_type = ItemType::try_from(i.fields.r#type.as_str()).unwrap_or_default();
            ListItem::new(format!(
                "{} {}",
                icons::item_type_icon(item_type),
                i.fields.title
            ))
        })
        .collect();
    let total = items.len();
    let index = app.selected_item_index().unwrap_or_default();
    let bottom_line = Line::from(format!(" {} of {total} ", index + 1).yellow());

    // NOTE: when switching from tabs to tabs if the index overflow or nothing was selected, select
    // the first item
    match app.selected_item_index() {
        Some(i) if i > total && total > 0 => app.select_item(0),
        None if total > 0 => app.select_item(0),
        _ => {}
    }

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .border_style(Style::default().blue())
                .padding(Padding::uniform(1))
                .title_bottom(bottom_line.right_aligned()),
        )
        .highlight_style(Style::default().green());

    // NOTE: the list must be rendered first, and the tabs after
    f.render_stateful_widget(
        list,
        rows[1] + Offset::new(0, -1),
        app.item_list_state_mut(),
    );
    f.render_widget(tabs, rows[0] + Offset::new(1, 0));
}
