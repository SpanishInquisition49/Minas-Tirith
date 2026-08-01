use minastirith_core::schema::author::Author;

use crate::traits::Spannable;

impl Spannable for Author {
    fn span_text(&self) -> &str {
        &self.name
    }
}
