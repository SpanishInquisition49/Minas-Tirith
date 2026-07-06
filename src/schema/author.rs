use core::fmt;

use chrono::{DateTime, Utc};
use ratatui::{style::Style, text::Span};

use crate::schema::graphics::Spannable;

#[derive(Clone, sqlx::FromRow, Debug)]
pub struct Author {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl fmt::Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = format!("Name: {}\n", self.name);
        if let Some(bio) = &self.bio {
            res.push_str(&format!("Bio:\n{}", bio));
        }
        write!(f, "{res}")
    }
}

impl Spannable for Author {
    fn to_span(&self) -> Span<'static> {
        let (bg, fg) = Self::tag_colors(&self.slug);
        Span::styled(format!(" {} ", self.name), Style::default().bg(bg).fg(fg))
    }

    fn span_len(&self) -> usize {
        self.name.len() + 2
    }
}
