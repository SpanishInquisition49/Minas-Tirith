use std::hash::{DefaultHasher, Hash, Hasher};

use ratatui::{style::Color, text::Span};

pub trait Spannable {
    fn to_span(&self) -> Span<'static>;

    fn tag_colors(slug: &str) -> (Color, Color) {
        let mut hasher = DefaultHasher::new();
        slug.hash(&mut hasher);
        let hash = hasher.finish();

        let hue = (hash % 360) as f64;
        let (r, g, b) = Self::hsl_to_rgb(hue, 0.55, 0.45);

        let bg = Color::Rgb(r, g, b);
        let fg = if Self::relative_luminance(r, g, b) > 0.5 {
            Color::Black
        } else {
            Color::White
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
