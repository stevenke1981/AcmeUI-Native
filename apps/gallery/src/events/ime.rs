//! Layer 2 — IME caret rectangle computation.
//!
//! Computes the window-client [`ime_caret_window_rect`][..] from the text
//! input field origin and the text-input crate's content-local caret geometry.

use acme_text::{FontSystem, TextStyle};
use acme_textinput::TextInputState;
use acme_theme::Theme;

/// Compute the window-client IME caret rectangle.
///
/// Returns `None` when the text input is not focused or has no valid rect.
///
/// * `text_input` — the text input state (focused flag + caret position).
/// * `text_input_rect` — the field's window-space rect `[x, y, w, h]`.
/// * `dark` — current theme flag (affects font size).
/// * `fonts` — font system (mutated by `ime_caret_area`).
/// * `last_scale_factor` — scale factor from the last frame.
pub fn compute_ime_caret_rect(
    text_input: &TextInputState,
    text_input_rect: [f32; 4],
    dark: bool,
    fonts: &mut FontSystem,
    last_scale_factor: f32,
) -> Option<[f32; 4]> {
    if !text_input.focused {
        return None;
    }
    let [fx, fy, fw, fh] = text_input_rect;
    if fw <= 0.0 || fh <= 0.0 {
        return None;
    }
    let theme = theme_for(dark);
    let style = field_text_style(&theme);
    let padding = theme.spacing.px2;
    let [cx, cy, cw, ch] = text_input.ime_caret_area(fonts, &style, last_scale_factor);
    Some([fx + padding + cx, fy + padding + cy, cw, ch.max(1.0)])
}

/// Scroll the focused field so its caret stays inside the visible text area.
///
/// Call after any text or caret change and before the caret rect is computed.
/// An unfocused field, or one with no size, is left unchanged.
pub fn scroll_text_input_to_caret(
    text_input: &mut TextInputState,
    text_input_rect: [f32; 4],
    theme: &Theme,
    fonts: &mut FontSystem,
    scale: f32,
) {
    if !text_input.focused {
        return;
    }
    let [_, _, fw, fh] = text_input_rect;
    if fw <= 0.0 || fh <= 0.0 {
        return;
    }
    let content_width = fw - 2.0 * theme.spacing.px2;
    acme_textinput::scroll_caret_into_view(
        text_input,
        fonts,
        &field_text_style(theme),
        scale,
        content_width,
    );
}

/// Scroll the caret into view, then compute the window-client IME caret rect.
///
/// Event handlers use this so the IME rect they report already matches the
/// scroll position. [`compute_ime_caret_rect`] alone cannot change the scroll.
pub fn refresh_ime_caret_rect(
    text_input: &mut TextInputState,
    text_input_rect: [f32; 4],
    dark: bool,
    fonts: &mut FontSystem,
    last_scale_factor: f32,
) -> Option<[f32; 4]> {
    scroll_text_input_to_caret(
        text_input,
        text_input_rect,
        &theme_for(dark),
        fonts,
        last_scale_factor,
    );
    compute_ime_caret_rect(text_input, text_input_rect, dark, fonts, last_scale_factor)
}

fn theme_for(dark: bool) -> Theme {
    if dark { Theme::dark() } else { Theme::light() }
}

/// Text style of the field, the same one `render_text_input` uses.
fn field_text_style(theme: &Theme) -> TextStyle {
    let font_size = theme.typography.body;
    TextStyle {
        font_size,
        line_height: font_size * theme.typography.line_height,
        ..TextStyle::default()
    }
}
