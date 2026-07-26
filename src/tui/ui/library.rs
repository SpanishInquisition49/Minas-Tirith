use std::str::FromStr;

use iroh::PublicKey;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph},
};

use crate::{
    peer2peer::{NodeIdDisplay, PrettyDisplay},
    schema::graphics::Spannable,
    tui::{
        app::{
            App,
            components::library::{BrowseFocus, PublishField, SubscribeField},
        },
        ui::source_tag::SourceTag,
    },
};

pub fn draw_library_publish_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(60), Constraint::Length(25));
    f.render_widget(Clear, center);

    let Some(s) = app.get_publish_state() else {
        return;
    };

    let title = if s.is_publishing() {
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
        Paragraph::new(format!("Collection: {}", s.collection_name())).dim(),
        rows[0],
    );

    if let Some(ticket) = &s.result_ticket() {
        let msg = Paragraph::new(vec![
            Line::from("Published! Invite Ticket:".green()),
            Line::from(ticket.clone()),
        ]);
        f.render_widget(msg, inner);
        return;
    }

    let name_style = if s.publish_field() == PublishField::Name {
        Style::default().green()
    } else {
        Style::default().yellow()
    };
    let desc_style = if s.publish_field() == PublishField::Description {
        Style::default().green()
    } else {
        Style::default().yellow()
    };

    f.render_widget(
        Paragraph::new(s.collection_name_input().value())
            .block(Block::bordered().title(" Name ").border_style(name_style)),
        rows[1],
    );
    f.render_widget(
        Paragraph::new(s.collection_description_input().value()).block(
            Block::bordered()
                .title(" Description ")
                .border_style(desc_style),
        ),
        rows[2],
    );

    if let Some(err) = s.get_last_error() {
        f.render_widget(Paragraph::new(format!("Error: {err}")).red(), rows[2]);
    }
}

pub fn draw_library_subscribe_popup(f: &mut Frame, app: &mut App) {
    let center = f
        .area()
        .centered(Constraint::Percentage(60), Constraint::Length(25));
    f.render_widget(Clear, center);

    let Some(s) = app.get_subscribe_state() else {
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

    let ticket_style = if s.field() == SubscribeField::Ticket {
        Style::default().green()
    } else {
        Style::default().yellow()
    };
    let nick_style = if s.field() == SubscribeField::Nickname {
        Style::default().green()
    } else {
        Style::default().yellow()
    };

    f.render_widget(
        Paragraph::new(s.ticket_input().value()).block(
            Block::bordered()
                .title(" Ticket ")
                .border_style(ticket_style),
        ),
        rows[0],
    );
    f.render_widget(
        Paragraph::new(s.nickname_input().value()).block(
            Block::bordered()
                .title(" Nickname ")
                .border_style(nick_style),
        ),
        rows[1],
    );

    if let Some(err) = s.get_last_error() {
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

    let Some(browse) = app.get_library_browse_state() else {
        return;
    };

    let sub_focused = browse.focus() == BrowseFocus::Subscriptions;
    let subs: Vec<ListItem> = app
        .get_subscriptions()
        .iter()
        .map(|s| {
            let pretty_owner = owner_pretty_name(&s.owner_node_id);
            let owner_span = SourceTag(&pretty_owner).to_span();
            ListItem::new(vec![
                Line::from(s.nickname.clone().bold()),
                Line::from(owner_span),
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

    let papers: Vec<ListItem> = app
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
                Line::from(SourceTag(&pretty_owner).to_span()),
            ])
        })
        .collect();
    let papers_focused = browse.focus() == BrowseFocus::Papers;
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
    let Some(browse) = app.get_library_browse_state_mut() else {
        return;
    };
    f.render_stateful_widget(subs_list, cols[0], browse.subscription_list_state_mut());
    f.render_stateful_widget(papers_list, cols[1], browse.item_list_state_mut());
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
    f.render_stateful_widget(list, cols[0], manage.list_state_mut());

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
}

fn owner_pretty_name(owner_node_id: &str) -> String {
    match PublicKey::from_str(owner_node_id) {
        Ok(pk) => NodeIdDisplay(pk).pretty_name(),
        Err(_) => owner_node_id.to_string(), // fallback: mostra il raw id
    }
}
