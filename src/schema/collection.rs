use ratatui::{style::Style, text::Span};
use serde::Deserialize;

use crate::schema::graphics::Spannable;

#[derive(Clone, sqlx::FromRow, Deserialize, Debug)]
pub struct Collection {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

impl Collection {
    /// Get the All collection
    pub fn trivial_collection() -> Self {
        Self {
            id: -1,
            name: "All".to_string(),
            slug: "all".to_string(),
        }
    }

    /// Check if the given id is from the trivial collection
    pub fn is_trivial_collection(id: i32) -> bool {
        id == -1
    }
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
