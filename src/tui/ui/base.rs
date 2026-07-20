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
    widgets::{Block, Borders, Clear, FrameExt, List, ListItem, Padding, Tabs},
};
use ratatui_explorer::Theme;

use crate::{
    peer2peer::PrettyDisplay,
    schema::graphics::Spannable,
    tui::{
        app::{App, Mode, TABS_LABELS},
        ui::{
            collection::{
                draw_collection_assign_popup, draw_collection_create_popup, draw_collection_sidebar,
            },
            details::draw_details,
            form::draw_metadata_edit_popup,
            metadata_select::draw_metadata_select_popup,
        },
    },
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let main = Layout::default()
        .direction(Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(f.area());

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(main[0]);

    let title = if app.metadata.is_searching {
        let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame = spinner[app.tick_counter % spinner.len()];
        app.tick_counter += 1;
        format!(" {frame} Searching for metadata... ")
            .bold()
            .yellow()
            .italic()
    } else {
        " Pick a tome ".bold().yellow().italic()
    };
    let theme = Theme::default()
        .with_title_top(move |_| Line::from(title.clone()))
        .with_title_bottom(|_| {
            Line::from(vec![" Select: ".yellow(), "<A> ".green()]).right_aligned()
        })
        .with_block(Block::bordered().border_set(border::THICK).blue())
        .with_highlight_item_style(Style::default().add_modifier(Modifier::REVERSED).yellow())
        .with_highlight_dir_style(Style::default().add_modifier(Modifier::REVERSED).yellow());

    app.file_explorer.set_theme(theme);

    draw_collection_sidebar(f, app, rows[0]);
    draw_list(f, app, rows[1]);
    draw_details(f, app, main[1]);
    match app.mode {
        Mode::Normal => {}                      // NO additional rendering
        Mode::Insert => draw_add_popup(f, app), // Add item popup
        Mode::Search => {}                      // TODO: add the search feature
        Mode::MetadataSelect => draw_metadata_select_popup(f, app),
        Mode::MetadataEdit => draw_metadata_edit_popup(f, app),
        Mode::CollectionCreate => draw_collection_create_popup(f, app),
        Mode::CollectionAssign => draw_collection_assign_popup(f, app),
    }
    app.notifications.render(f, f.area());
}

fn draw_add_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(60), Constraint::Percentage(60));
    f.render_widget(Clear, center);
    f.render_widget_ref(app.file_explorer.widget(), center);
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Max(1), Constraint::Fill(1), Constraint::Fill(1)])
        .split(area);
    let tabs = Tabs::new(TABS_LABELS)
        .style(Style::default().italic())
        .highlight_style(Style::default().yellow().underlined().italic().bold())
        .select(app.selectd_tab)
        .divider(symbols::DOT)
        .padding(" ", " ");
    let items: Vec<ListItem> = app
        .items
        .iter()
        .filter(|i| app.keep_items(i))
        .map(|i| ListItem::new(i.fields.title.clone()))
        .collect();
    let total = items.len();
    let index = app.items_list_state.selected().unwrap_or_default();
    let bottom_line = Line::from(format!(" {} of {total} ", index + 1).yellow());

    // NOTE: when switching fro tabs to tabs if the index overflow or nothing was selected, select
    // the first item
    match app.items_list_state.selected() {
        Some(i) if i > total && total > 0 => app.items_list_state.select(Some(0)),
        None if total > 0 => app.items_list_state.select(Some(0)),
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

    // TODO: change the position of this code, this is temporary for debug purpose
    let peers = app.peers.iter().map(|i| i.to_span()).collect::<Vec<_>>();
    let peer_list = List::new(peers).block(
        Block::default()
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .border_style(Style::default().blue())
            .title(
                format!(" {} ", app.share_node.pretty_name())
                    .yellow()
                    .italic()
                    .bold(),
            ),
    );

    f.render_stateful_widget(
        list,
        rows[1] + Offset::new(0, -1),
        &mut app.items_list_state,
    );
    f.render_widget(tabs, rows[0] + Offset::new(1, 0));
    f.render_widget(peer_list, rows[2]);
}
