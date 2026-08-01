use minastirith_core::schema::tag::Tag;

use crate::traits::Spannable;

impl Spannable for Tag {
    fn span_text(&self) -> &str {
        &self.name
    }
}
