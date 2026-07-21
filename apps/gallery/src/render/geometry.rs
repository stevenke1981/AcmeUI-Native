//! Color and geometry helpers — pure functions for rendering primitives.

use acme_core::{Color, QuadPrimitive, Rect};
use acme_theme::ThemeColor;

/// Convert a [`ThemeColor`] to [`Color`].
#[allow(dead_code)]
pub fn rgba(color: ThemeColor) -> Color {
    Color::rgba(color.red, color.green, color.blue, color.alpha)
}

/// Convert a [`ThemeColor`] to [`Color`].
pub fn theme_color_to_core(tc: ThemeColor) -> Color {
    Color::rgba(tc.red, tc.green, tc.blue, tc.alpha)
}

/// Build a [`QuadPrimitive`] from its parts.
pub fn quad_rect(
    rect: [f32; 4],
    fill: ThemeColor,
    radius: f32,
    border_width: f32,
    border_color: ThemeColor,
) -> QuadPrimitive {
    QuadPrimitive {
        rect: Rect::new(rect[0], rect[1], rect[2], rect[3]),
        color: theme_color_to_core(fill),
        radius,
        border_width,
        border_color: theme_color_to_core(border_color),
    }
}

/// Point-in-axis-aligned-rect test (window space).
pub fn point_in_rect(x: f32, y: f32, rect: [f32; 4]) -> bool {
    x >= rect[0] && x <= rect[0] + rect[2] && y >= rect[1] && y <= rect[1] + rect[3]
}

/// Map a content-space layout rect into window space using page scroll.
pub fn scrolled_hit_rect(rect: [f32; 4], scroll_y: f32) -> [f32; 4] {
    [rect[0], rect[1] - scroll_y, rect[2], rect[3]]
}
