use std::str::FromStr;

use iroh::PublicKey;
use minastirith_core::{
    peer2peer::{NodeIdDisplay, PrettyDisplay},
    state::library::{browse::BrowseFocus, publish::PublishField, subscribe::SubscribeField},
    traits::Focusable,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph},
};

use crate::tui::app::App;

pub fn draw_library_publish_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(60), Constraint::Length(25));
    f.render_widget(Clear, center);

    let Some(library_publish_state) = app.get_publish_state() else {
        return;
    };

    let title = if library_publish_state.is_publishing() {
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
        Paragraph::new(format!(
            "Collection: {}",
            library_publish_state.collection_name()
        ))
        .dim(),
        rows[0],
    );

    if let Some(ticket) = &library_publish_state.result_ticket() {
        let msg = Paragraph::new(vec![
            Line::from("Published! Invite Ticket:".green()),
            Line::from(ticket.clone()),
        ]);
        f.render_widget(msg, inner);
        return;
    }

    let name_style = if library_publish_state.current_focus() == PublishField::Name {
        Style::default().green()
    } else {
        Style::default().yellow()
    };
    let desc_style = if library_publish_state.current_focus() == PublishField::Description {
        Style::default().green()
    } else {
        Style::default().yellow()
    };

    f.render_widget(
        Paragraph::new(app.collection_name_input().value())
            .block(Block::bordered().title(" Name ").border_style(name_style)),
        rows[1],
    );
    f.render_widget(
        Paragraph::new(app.collection_description_input().value()).block(
            Block::bordered()
                .title(" Description ")
                .border_style(desc_style),
        ),
        rows[2],
    );

    if let Some(err) = library_publish_state.last_error() {
        f.render_widget(Paragraph::new(format!("Error: {err}")).red(), rows[2]);
    }
}

pub fn draw_library_subscribe_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(60), Constraint::Length(25));
    f.render_widget(Clear, center);

    let Some(library_subscribe_state) = app.get_subscribe_state() else {
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

    let ticket_style = if library_subscribe_state.current_focus() == SubscribeField::Ticket {
        Style::default().green()
    } else {
        Style::default().yellow()
    };
    let nick_style = if library_subscribe_state.current_focus() == SubscribeField::Nickname {
        Style::default().green()
    } else {
        Style::default().yellow()
    };

    f.render_widget(
        Paragraph::new(app.subscribe_ticket().value()).block(
            Block::bordered()
                .title(" Ticket ")
                .border_style(ticket_style),
        ),
        rows[0],
    );
    f.render_widget(
        Paragraph::new(app.subscribe_nickname().value()).block(
            Block::bordered()
                .title(" Nickname ")
                .border_style(nick_style),
        ),
        rows[1],
    );

    if let Some(err) = library_subscribe_state.last_error() {
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

    let Some(browse) = app.get_browse_state() else {
        return;
    };

    let sub_focused = browse.current_focus() == BrowseFocus::Subscriptions;
    let subs: Vec<ListItem> = app
        .get_subscriptions()
        .iter()
        .map(|s| {
            let pretty_owner = owner_pretty_name(&s.owner_node_id);
            ListItem::new(vec![
                Line::from(s.nickname.clone().bold()),
                Line::from(pretty_owner.dim()),
            ])
        })
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

    let items: Vec<ListItem> = app
        .current_library_items()
        .iter()
        .map(|p| {
            let authors = if p.authors.is_empty() {
                "Uknown Autor".to_string()
            } else {
                p.authors.join(", ")
            };
            let pretty_owner = NodeIdDisplay(p.owner).pretty_name();

            ListItem::new(vec![
                Line::from(p.title.clone().bold()),
                Line::from(authors.dim()),
                Line::from(pretty_owner.dim()),
            ])
        })
        .collect();
    let papers_focused = browse.current_focus() == BrowseFocus::Items;
    let papers_list = List::new(items)
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
    if app.get_library_browse_state_mut().is_none() {
        return;
    }
    f.render_stateful_widget(subs_list, cols[0], app.subscription_list_state_mut());
    f.render_stateful_widget(papers_list, cols[1], app.browse_list_state_mut());
}

pub fn draw_library_manage_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(75), Constraint::Percentage(65));
    f.render_widget(Clear, center);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(center);

    let items: Vec<ListItem> = app
        .get_shared_libraries()
        .iter()
        .map(|l| ListItem::new(l.name.clone()))
        .collect();

    let Some(manage) = app.get_library_manage_state_mut() else {
        return;
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .border_style(Style::default().green())
                .padding(Padding::uniform(1))
                .title(" My Libraries ".bold().italic().yellow())
                .title_bottom(
                    Line::from(vec![
                        " Ticket: ".yellow(),
                        "<t> ".green(),
                        " Delete: ".yellow(),
                        "<d> ".green(),
                        " Copy: ".yellow(),
                        "<c> ".green(),
                        " Close: ".yellow(),
                        "<Esc> ".green(),
                    ])
                    .right_aligned(),
                ),
        )
        .highlight_style(Style::default().green());

    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .border_style(Style::default().blue())
        .padding(Padding::uniform(1))
        .title(" Details ".bold().italic().yellow());
    let inner = detail_block.inner(cols[1]);
    f.render_widget(detail_block, cols[1]);

    let mut lines = Vec::new();
    if manage.is_generating() {
        lines.push(Line::from("Generating Ticket...".yellow()));
    } else if let Some(ticket) = manage.current_ticket() {
        lines.push(Line::from("Invite Ticket:".green().bold()));
        lines.push(Line::raw(""));
        lines.push(Line::from(ticket.clone()));
    } else if let Some(err) = manage.get_last_error() {
        lines.push(Line::from(format!("Error: {err}").red()));
    } else {
        lines.push(Line::from(
            "Press <t> to generate the invite ticket for the library.".dim(),
        ));
    }
    f.render_widget(
        Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: false }),
        inner,
    );
    f.render_stateful_widget(list, cols[0], app.manage_list_state_mut());
}

fn owner_pretty_name(owner_node_id: &str) -> String {
    match PublicKey::from_str(owner_node_id) {
        Ok(pk) => NodeIdDisplay(pk).pretty_name(),
        Err(_) => owner_node_id.to_string(), // fallback: show raw id
    }
}
