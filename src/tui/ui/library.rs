use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph},
};

use crate::tui::app::{App, library::BrowseFocus};

pub fn draw_library_publish_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(60), Constraint::Length(25));
    f.render_widget(Clear, center);

    let Some(s) = &app.library.publish else {
        return;
    };

    let title = if s.publishing {
        " Publishing... "
    } else {
        " Published as Library "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .border_style(Style::default().blue())
        .padding(Padding::uniform(1))
        .title(title.bold().italic().yellow())
        .title_bottom(
            Line::from(vec![
                " Field: ".yellow(),
                "<Tab> ".green(),
                "Confirm: ".yellow(),
                "<Ctrl+S> ".green(),
                "Cancel: ".yellow(),
                "<Esc> ".green(),
            ])
            .right_aligned(),
        );
    let inner = block.inner(center);
    f.render_widget(block, center);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(format!("Collection: {}", s.collection_name)).dim(),
        rows[0],
    );

    if let Some(ticket) = &s.result_ticket {
        let msg = Paragraph::new(vec![
            Line::from("Published! Invite Ticket:".green()),
            Line::from(ticket.clone()),
        ]);
        f.render_widget(msg, inner);
        return;
    }

    let name_style = if s.field == super::super::app::library::PublishField::Name {
        Style::default().green()
    } else {
        Style::default().yellow()
    };
    let desc_style = if s.field == super::super::app::library::PublishField::Description {
        Style::default().green()
    } else {
        Style::default().yellow()
    };

    f.render_widget(
        Paragraph::new(s.name_input.value())
            .block(Block::bordered().title(" Name ").border_style(name_style)),
        rows[1],
    );
    f.render_widget(
        Paragraph::new(s.description_input.value()).block(
            Block::bordered()
                .title(" Description ")
                .border_style(desc_style),
        ),
        rows[2],
    );

    if let Some(err) = &s.last_error {
        f.render_widget(Paragraph::new(format!("Error: {err}")).red(), rows[2]);
    }
}

pub fn draw_library_subscribe_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(60), Constraint::Length(25));
    f.render_widget(Clear, center);

    let Some(s) = &app.library.subscribe else {
        return;
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .border_style(Style::default().blue())
        .padding(Padding::uniform(1))
        .title(" Subscribe to library ".bold().italic().yellow())
        .title_bottom(
            Line::from(vec![
                " Field: ".yellow(),
                "<Tab> ".green(),
                "Confirm: ".yellow(),
                "<Ctrl+S> ".green(),
                "Cancel: ".yellow(),
                "<Esc> ".green(),
            ])
            .right_aligned(),
        );
    let inner = block.inner(center);
    f.render_widget(block, center);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Length(25)])
        .split(inner);

    use crate::tui::app::library::SubscribeField;
    let ticket_style = if s.field == SubscribeField::Ticket {
        Style::default().green()
    } else {
        Style::default().yellow()
    };
    let nick_style = if s.field == SubscribeField::Nickname {
        Style::default().green()
    } else {
        Style::default().yellow()
    };

    f.render_widget(
        Paragraph::new(s.ticket_input.value()).block(
            Block::bordered()
                .title(" Ticket ")
                .border_style(ticket_style),
        ),
        rows[0],
    );
    f.render_widget(
        Paragraph::new(s.nickname_input.value()).block(
            Block::bordered()
                .title(" Nickname ")
                .border_style(nick_style),
        ),
        rows[1],
    );

    if let Some(err) = &s.last_error {
        f.render_widget(Paragraph::new(format!("Error: {err}")).red(), rows[1]);
    }
}

pub fn draw_library_browse_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(80), Constraint::Percentage(70));
    f.render_widget(Clear, center);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(center);

    let Some(browse) = &app.library.browse else {
        return;
    };

    let sub_focused = browse.focus == BrowseFocus::Subscriptions;
    let subs: Vec<ListItem> = app
        .library
        .subscriptions
        .iter()
        .map(|s| ListItem::new(s.nickname.clone()))
        .collect();
    let subs_list = List::new(subs)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .border_style(if sub_focused {
                    Style::default().green()
                } else {
                    Style::default().blue()
                })
                .padding(Padding::uniform(1))
                .title(" Subscribed Libraries ".bold().yellow())
                .title_bottom(Line::from(vec![" Add: ".yellow(), "<a> ".green()]).right_aligned()),
        )
        .highlight_style(Style::default().green());

    let papers: Vec<ListItem> = app
        .library
        .current_papers()
        .iter()
        .map(|p| {
            let authors = if p.authors.is_empty() {
                "Uknown Autor".to_string()
            } else {
                p.authors.join(", ")
            };
            ListItem::new(vec![
                Line::from(p.title.clone().bold()),
                Line::from(authors.dim()),
            ])
        })
        .collect();
    let papers_focused = browse.focus == BrowseFocus::Papers;
    let papers_list = List::new(papers)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .border_style(if papers_focused {
                    Style::default().green()
                } else {
                    Style::default().blue()
                })
                .padding(Padding::uniform(1))
                .title(" Available tomes ".bold().yellow())
                .title_bottom(
                    Line::from(vec![
                        " Import: ".yellow(),
                        "<Enter> ".green(),
                        " Focus: ".yellow(),
                        "<Tab> ".green(),
                        " Close: ".yellow(),
                        "<Esc> ".green(),
                    ])
                    .right_aligned(),
                ),
        )
        .highlight_style(Style::default().green());
    let Some(browse) = &mut app.library.browse else {
        return;
    };
    f.render_stateful_widget(subs_list, cols[0], &mut browse.subscription_list_state);
    f.render_stateful_widget(papers_list, cols[1], &mut browse.paper_list_state);
}
