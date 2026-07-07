use ratatui::{
    Frame,
    layout::{
        Constraint,
        Direction::{self, Horizontal},
        Layout, Rect,
    },
    style::{Modifier, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Clear, FrameExt, List, ListItem, Padding, Paragraph},
};
use ratatui_explorer::Theme;

use crate::tui::{
    app::{App, Mode},
    ui::{details::draw_details, form::draw_metadata_edit_popup},
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    let main = Layout::default()
        .direction(Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(outer[0]);

    let title = if app.is_searching {
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

    draw_list(f, app, main[0]);
    draw_details(f, app, main[1]);
    draw_status(f, outer[1]);
    match app.mode {
        Mode::Normal => {}                      // NO additional rendering
        Mode::Insert => draw_add_popup(f, app), // Add item popup
        Mode::Search => todo!(),
        Mode::MetadataSelect => draw_metadata_select_popup(f, app),
        Mode::MetadataEdit => draw_metadata_edit_popup(f, app),
    }
    app.notifications.render(f, f.area());
}

fn draw_metadata_select_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(60), Constraint::Percentage(40));
    f.render_widget(Clear, center);

    let items: Vec<ListItem> = app
        .metadata_candidates
        .iter()
        .map(|c| {
            let authors = c.authors().join(", ");
            let date = c.publication_date().unwrap_or_default();
            ListItem::new(format!(
                "[{}] {}  —  {}  ({})",
                c.source(),
                c.title(),
                authors,
                date,
            ))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .border_style(Style::default().blue())
                .padding(Padding::uniform(1))
                .title(Line::from("Select metadata".bold()).yellow().italic())
                .title_bottom(
                    Line::from(" <Enter> Confirm  <Esc> Cancel ")
                        .right_aligned()
                        .yellow(),
                ),
        )
        .highlight_style(Style::default().green());

    f.render_stateful_widget(list, center, &mut app.metadata_list_state);
}

fn draw_add_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(60), Constraint::Percentage(20));
    f.render_widget(Clear, center);
    f.render_widget_ref(app.file_explorer.widget(), center);
}

fn draw_list(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .items
        .iter()
        .map(|i| ListItem::new(i.fields.title.clone()))
        .collect();

    let index = app.items_list_state.selected().unwrap_or_default() + 1;
    let total = app.items.len();
    let bottom_line = Line::from(format!(" {index} of {total} ").yellow());
    let title = Line::from(" Tomes ".bold().yellow().italic());
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .border_style(Style::default().blue())
                .padding(Padding::uniform(1))
                .title(title)
                .title_bottom(bottom_line.right_aligned()),
        )
        .highlight_style(Style::default().green());

    f.render_stateful_widget(list, area, &mut app.items_list_state);
}

fn draw_status(f: &mut Frame, area: Rect) {
    let instructions = Line::from(vec![
        " Navigate: ".yellow(),
        "<K/J>".green().bold(),
        " Search: ".yellow(),
        "</>".green().bold(),
        " Add Tome: ".yellow(),
        "<A>".green().bold(),
        " Edit Tome: ".yellow(),
        "<E>".green().bold(),
        " Export Bibtex: ".yellow(),
        "<B>".green().bold(),
        " Quit: ".yellow(),
        "<Q> ".green().bold(),
    ]);
    f.render_widget(Paragraph::new(instructions), area);
}
