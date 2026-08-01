use minastirith_core::schema::collection::Collection;

use crate::traits::Spannable;

impl Spannable for Collection {
    fn span_text(&self) -> &str {
        &self.name
    }
}
