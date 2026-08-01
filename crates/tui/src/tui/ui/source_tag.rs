use minastirith_core::traits::Colorable;

use crate::traits::Spannable;

pub struct SourceTag {
    pub source: String,
}

impl SourceTag {
    pub fn new(source: String) -> Self {
        Self { source }
    }
}

impl Colorable for SourceTag {}

impl Spannable for SourceTag {
    fn span_text(&self) -> &str {
        &self.source
    }
}
