//! Nerd Font glyph constants used across the TUI.
//!
//! These are Private Use Area codepoints from the Font Awesome set patched in
//! by Nerd Fonts. They render as tofu/blank boxes in a non-patched font.

use minastirith_core::metadata::common_metadata::ItemType;

// Item types
pub const BOOK: &str = "\u{f02d}";
pub const ARTICLE: &str = "\u{f1ea}";
pub const THESIS: &str = "\u{f19d}";
pub const REPORT: &str = "\u{f0f6}";
pub const MISC: &str = "\u{f016}";

// Metadata fields
pub const AUTHOR: &str = "\u{f007}";
pub const BUILDING: &str = "\u{f1ad}";
pub const CALENDAR: &str = "\u{f073}";
pub const HASHTAG: &str = "\u{f292}";
pub const BARCODE: &str = "\u{f02a}";
pub const FOLDER: &str = "\u{f07b}";
pub const TAG: &str = "\u{f02b}";
pub const ALIGN_LEFT: &str = "\u{f036}";
pub const INFO: &str = "\u{f05a}";

// Sections / panels
pub const SEARCH: &str = "\u{f002}";
pub const GLOBE: &str = "\u{f0ac}";
pub const LIST: &str = "\u{f00b}";
pub const KEYBOARD: &str = "\u{f11c}";
pub const RSS: &str = "\u{f09e}";

// Actions / keybinding hints
pub const PLUS: &str = "\u{f067}";
pub const PENCIL: &str = "\u{f040}";
pub const DOWNLOAD: &str = "\u{f019}";
pub const UPLOAD: &str = "\u{f0ee}";
pub const EXCHANGE: &str = "\u{f0ec}";
pub const POWER: &str = "\u{f011}";
pub const CHECK: &str = "\u{f00c}";
pub const CLOSE: &str = "\u{f00d}";
pub const TRASH: &str = "\u{f1f8}";
pub const TICKET: &str = "\u{f145}";
pub const COPY: &str = "\u{f0c5}";
pub const ARROWS_V: &str = "\u{f07d}";
pub const CHECKBOX_CHECKED: &str = "\u{f046}";
pub const CHECKBOX_UNCHECKED: &str = "\u{f096}";

/// Icon representing an item's [`ItemType`].
pub fn item_type_icon(item_type: ItemType) -> &'static str {
    match item_type {
        ItemType::Book => BOOK,
        ItemType::Article => ARTICLE,
        ItemType::Report => REPORT,
        ItemType::Thesis => THESIS,
        ItemType::Misc => MISC,
    }
}
