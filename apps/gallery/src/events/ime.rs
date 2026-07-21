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
    let theme = if dark { Theme::dark() } else { Theme::light() };
    let font_size = theme.typography.body;
    let style = TextStyle {
        font_size,
        line_height: font_size * theme.typography.line_height,
        ..TextStyle::default()
    };
    let padding = theme.spacing.px2;
    let [cx, cy, cw, ch] = text_input.ime_caret_area(fonts, &style, last_scale_factor);
    Some([fx + padding + cx, fy + padding + cy, cw, ch.max(1.0)])
}
