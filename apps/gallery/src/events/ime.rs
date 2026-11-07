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
    // A composition wider than the field scrolls its start (the anchor) left of
    // the field; keep the candidate window on the visible content instead.
    let content_w = (fw - 2.0 * padding).max(0.0);
    let cx = cx.clamp(0.0, content_w);
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

#[cfg(test)]
mod tests {
    use super::*;

    const FIELD: [f32; 4] = [40.0, 10.0, 120.0, 36.0];

    #[test]
    fn wide_composition_keeps_the_candidate_anchor_inside_the_field() {
        let mut fonts = FontSystem::new();
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let mut state = TextInputState::new();
            state.focused = true;
            state.set_preedit("ㄓㄨˋㄧㄣ注音輸入法測試字串很長很長", None);
            let rect = refresh_ime_caret_rect(&mut state, FIELD, false, &mut fonts, scale)
                .expect("focused field has a caret rect");
            assert!(
                state.scroll_offset > 0.0,
                "scale {scale}: composition must scroll"
            );
            let left = FIELD[0];
            let right = FIELD[0] + FIELD[2];
            assert!(
                (left..=right).contains(&rect[0]),
                "scale {scale}: anchor x {} outside field [{left}, {right}]",
                rect[0]
            );
        }
    }

    #[test]
    fn unscrolled_anchor_is_unchanged_by_the_clamp() {
        let mut fonts = FontSystem::new();
        let mut state = TextInputState::new();
        state.focused = true;
        state.set_preedit("ㄓ", None);
        let theme = theme_for(false);
        let style = field_text_style(&theme);
        let raw = state.ime_caret_area(&mut fonts, &style, 1.0)[0];
        let rect = compute_ime_caret_rect(&state, FIELD, false, &mut fonts, 1.0).unwrap();
        assert_eq!(rect[0], FIELD[0] + theme.spacing.px2 + raw);
    }
}
