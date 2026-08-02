use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{
    metadata::common_metadata::ItemMetadata,
    schema::form::{Field, FormSnapshot},
};

/// Types with a natural cyclic ordering of variants, used to bind arrow-key
/// navigation.
pub trait Cyclable: Sized {
    /// The next value in the cycle, wrapping around after the last one.
    #[must_use]
    fn next(&self) -> Self;
    /// The previous value in the cycle, wrapping around before the first
    /// one.
    #[must_use]
    fn prev(&self) -> Self;
}

/// Types that hold a single [`Cyclable`] focus value and can cycle it.
pub trait Focusable {
    type Focus: Cyclable;
    /// The currently focused value.
    fn current_focus(&self) -> Self::Focus;
    /// Mutable access to the currently focused value.
    fn current_focus_mut(&mut self) -> &mut Self::Focus;

    /// Advance focus to the next value.
    fn focus_next(&mut self) {
        let next = self.current_focus().next();
        *self.current_focus_mut() = next;
    }

    /// Move focus back to the previous value.
    fn focus_prev(&mut self) {
        let prev = self.current_focus().prev();
        *self.current_focus_mut() = prev;
    }
}

/// Types that hold a list and a single selected index into it, used to
/// drive list navigation.
pub trait Selectable {
    type Item;
    /// The items available for selection.
    fn items(&self) -> &[Self::Item];
    /// Index of the currently selected item, if any.
    fn selected_index(&self) -> Option<usize>;
    /// Mutable access to the index of the currently selected item.
    fn selected_index_mut(&mut self) -> &mut Option<usize>;

    /// Select the next item, wrapping around to the first one.
    fn select_next(&mut self) {
        let len = self.items().len();
        let next = match self.selected_index() {
            Some(i) if i + 1 < len => Some(i + 1),
            Some(_) | None if len != 0 => Some(0),
            _ => None,
        };
        *self.selected_index_mut() = next;
    }

    /// Select the previous item, wrapping around to the last one.
    fn select_prev(&mut self) {
        let len = self.items().len();
        let prev = match self.selected_index() {
            Some(i) if i > 0 => Some(i - 1),
            Some(_) if len != 0 => Some(len - 1),
            None if len != 0 => Some(0),
            _ => None,
        };
        *self.selected_index_mut() = prev;
    }
}

/// Generic implementation to get foreground and background colors based on a slug
/// for UI element like pills
pub trait Colorable {
    /// Get the background and foreground based on the given text
    /// return a pair of RGB colors
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    fn get_colors(slug: &str) -> ((u8, u8, u8), (u8, u8, u8)) {
        let mut hasher = DefaultHasher::new();
        slug.hash(&mut hasher);
        let hash = hasher.finish();

        let hue = (hash % 360) as f64;
        let (r, g, b) = Self::hsl_to_rgb(hue, 0.55, 0.45);

        let bg = (r, g, b);
        let fg = if Self::relative_luminance(r, g, b) > 0.5 {
            (0, 0, 0)
        } else {
            (255, 255, 255)
        };

        (bg, fg)
    }

    /// Relative luminance of an sRGB color, in `[0, 1]`, used to decide
    /// whether black or white foreground text is more legible.
    #[must_use]
    fn relative_luminance(r: u8, g: u8, b: u8) -> f64 {
        (0.299 * f64::from(r) + 0.587 * f64::from(g) + 0.114 * f64::from(b)) / 255.0
    }

    /// Convert an HSL color (`h` in degrees, `s`/`l` in `[0, 1]`) to sRGB.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r1, g1, b1) = match h as u32 {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        (
            ((r1 + m) * 255.0).round() as u8,
            ((g1 + m) * 255.0).round() as u8,
            ((b1 + m) * 255.0).round() as u8,
        )
    }
}

/// Types backing an editable metadata form, convertible to/from a fetched
/// [`ItemMetadata`] candidate.
pub trait MetadataForm: Default {
    /// Consume the form into a [`FormSnapshot`] ready for persistence.
    fn snapshot(self) -> FormSnapshot;

    /// Build a form pre-filled from a fetched metadata `candidate`.
    fn from_candidate(candidate: &dyn ItemMetadata) -> Self;

    /// Current display value of `field`.
    fn field_value(&self, field: &Field) -> String;
    /// Display title of `field`.
    fn field_title(&self, field: &Field) -> String;

    /// Move editing focus to the next field.
    fn next_field(&mut self);
    /// Move editing focus to the previous field.
    fn prev_field(&mut self);

    /// Cycle the item type to the next variant.
    fn cycle_item_type(&mut self);
}
