use std::hash::{DefaultHasher, Hash, Hasher};

use ratatui::{
    style::{Color, Style},
    text::Span,
};

use crate::schema::graphics::Spannable;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub slug: String,
}

impl Spannable for Tag {
    fn to_span(&self) -> Span<'static> {
        let (bg, fg) = Self::tag_colors(&self.slug);
        Span::styled(
            format!(" {} ", self.name.clone()),
            Style::default().bg(bg).fg(fg),
        )
    }
}
