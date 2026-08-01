use std::hash::{DefaultHasher, Hash, Hasher};

use crate::{
    metadata::common_metadata::ItemMetadata,
    schema::form::{Field, FormSnapshot},
};

pub trait Cyclable: Sized {
    fn next(&self) -> Self;
    fn prev(&self) -> Self;
}

pub trait Focusable {
    type Focus: Cyclable;
    fn current_focus(&self) -> Self::Focus;
    fn current_focus_mut(&mut self) -> &mut Self::Focus;

    fn focus_next(&mut self) {
        let next = self.current_focus().next();
        *self.current_focus_mut() = next;
    }

    fn focus_prev(&mut self) {
        let prev = self.current_focus().prev();
        *self.current_focus_mut() = prev;
    }
}

pub trait Selectable {
    type Item;
    fn items(&self) -> &[Self::Item];
    fn selected_index(&self) -> Option<usize>;
    fn selected_index_mut(&mut self) -> &mut Option<usize>;

    fn select_next(&mut self) {
        let len = self.items().len();
        let next = match self.selected_index() {
            Some(i) if i + 1 < len => Some(i + 1),
            Some(_) if len != 0 => Some(0),
            None if len != 0 => Some(0),
            _ => None,
        };
        *self.selected_index_mut() = next;
    }

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

    fn relative_luminance(r: u8, g: u8, b: u8) -> f64 {
        (0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64) / 255.0
    }

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

pub trait MetadataForm: Default {
    fn snapshot(self) -> FormSnapshot;

    fn from_candidate(candidate: &dyn ItemMetadata) -> Self;

    fn field_value(&self, field: &Field) -> String;
    fn field_title(&self, field: &Field) -> String;

    fn next_field(&mut self);
    fn prev_field(&mut self);

    fn cycle_item_type(&mut self);
}
