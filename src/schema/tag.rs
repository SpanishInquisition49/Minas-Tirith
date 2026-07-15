use ratatui::{style::Style, text::Span};
use serde::Deserialize;

use crate::schema::graphics::Spannable;

#[derive(Clone, Debug, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub slug: String,
}

impl Spannable for Tag {
    fn to_span(&self) -> Span<'static> {
        let (bg, fg) = Self::tag_colors(&self.slug);
        Span::styled(format!(" {} ", self.name), Style::default().bg(bg).fg(fg))
    }

    fn span_len(&self) -> usize {
        self.name.len() + 2
    }
}
