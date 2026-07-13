use chrono::{DateTime, Utc};
use ratatui::{style::Style, text::Span};

use crate::schema::graphics::Spannable;

#[derive(Clone, sqlx::FromRow, Debug)]
pub struct Collection {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Spannable for Collection {
    fn to_span(&self) -> ratatui::prelude::Span<'static> {
        let (bg, fg) = Self::tag_colors(&self.slug);
        Span::styled(format!(" {} ", self.name), Style::default().fg(fg).bg(bg))
    }

    fn span_len(&self) -> usize {
        self.name.len() + 2
    }
}
