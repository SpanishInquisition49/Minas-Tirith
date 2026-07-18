use ratatui::{style::Style, text::Span};

use crate::schema::graphics::Spannable;

pub struct SourceTag<'a>(pub &'a str);

impl Spannable for SourceTag<'_> {
    fn to_span(&self) -> ratatui::prelude::Span<'static> {
        let (bg, fg) = Self::tag_colors(&self.0);
        Span::styled(format!(" {} ", self.0), Style::default().fg(fg).bg(bg))
    }

    fn span_len(&self) -> usize {
        self.0.len() + 2
    }
}
