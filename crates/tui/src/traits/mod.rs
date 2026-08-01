use minastirith_core::traits::{Colorable, Selectable};
use ratatui::{
    style::{Color, Style},
    text::Span,
};

pub trait Spannable: Colorable {
    fn span_text(&self) -> &str;

    fn to_span(&self) -> Span<'_> {
        let (bg, fg) = Self::get_colors(self.span_text());
        let bg = Color::Rgb(bg.0, bg.1, bg.2);
        let fg = Color::Rgb(fg.0, fg.1, fg.2);
        Span::styled(
            format!(" {} ", self.span_text()),
            Style::default().fg(fg).bg(bg),
        )
    }

    fn span_len(&self) -> usize {
        self.span_text().len() + 2
    }
}

pub trait SelectableSync {
    type Inner: Selectable;

    fn inner(&mut self) -> &mut Self::Inner;

    fn sync(&mut self);

    fn select_next(&mut self) {
        self.inner().select_next();
        self.sync();
    }

    fn select_prev(&mut self) {
        self.inner().select_prev();
        self.sync();
    }

    fn resync_after_refresh(&mut self) {
        let len = self.inner().items().len();

        let idx = self.inner().selected_index();
        match idx {
            Some(i) if i >= len && len > 0 => {
                *self.inner().selected_index_mut() = Some(0);
            }
            None if len > 0 => {
                *self.inner().selected_index_mut() = Some(0);
            }
            _ => {}
        }

        self.sync();
    }
}
