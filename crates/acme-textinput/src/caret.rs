//! Caret geometry: byte offset to x, caret scrolling and preedit extent.

use super::*;

/// Helper to compute `byte_offset_to_x` without creating `TextConstraints` each time.
pub(crate) fn byte_offset_to_x_inner(
    text: &str,
    offset: usize,
    fonts: &mut FontSystem,
    style: &TextStyle,
    scale: f32,
) -> f32 {
    if text.is_empty() || offset == 0 {
        return 0.0;
    }
    // Never slice inside a character, whatever offset a caller passes.
    let clamped = floor_char_boundary(text, offset.min(text.len()));
    if clamped == 0 {
        return 0.0;
    }
    if clamped >= text.len() {
        // Full width
        let layout = fonts.shape(text, style, TextConstraints::default(), scale);
        return layout.width;
    }
    // Shape prefix up to (but not including) the byte at `clamped`.
    // `clamped` is guaranteed < text.len() here, so the split is safe.
    let (prefix, _) = text.split_at(clamped);
    let layout = fonts.shape(prefix, style, TextConstraints::default(), scale);
    layout.width
}

/// Width of the drawn caret and of the IME caret area, in logical pixels.
pub(crate) const CARET_WIDTH: f32 = 1.5;

/// Largest char boundary of `text` that is `<= offset` (clamped to the length).
pub(crate) fn floor_char_boundary(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

/// Convert a byte offset into a text string to its pixel x-coordinate.
///
/// Shapes the substring `text[..offset]` and returns its width.
/// Returns `0.0` if `offset` is `0` or `text` is empty.
/// Returns the full text width if `offset` ≥ `text.len()`.
pub fn byte_offset_to_x(
    text: &str,
    offset: usize,
    fonts: &mut FontSystem,
    style: &TextStyle,
    scale: f32,
) -> f32 {
    byte_offset_to_x_inner(text, offset, fonts, style, scale)
}

/// Adjust [`TextInputState::scroll_offset`] so the caret stays inside the
/// visible text area.
///
/// `style` supplies the font size used to measure text, the same input as
/// [`TextInputState::ime_caret_area`]. `content_width` is the visible text width
/// in logical pixels (field width minus horizontal padding). A negative or
/// non-finite `content_width` is treated as `0`.
///
/// While an IME preedit is active, the focus point is the composition cursor
/// (the preedit cursor end, or the preedit end when unset), and the scroll
/// range includes the whole preedit width. Without a preedit both extras are
/// zero, so the result is the committed-caret rule.
///
/// After the call, the content-local caret x (`ime_caret_area(..)[0]`, which is
/// `caret_x - scroll_offset`) lies in `[0, content_width]`. The scroll stays in
/// `[0, text_width - content_width]`, so it shrinks when text is removed and
/// returns to `0` when the whole text fits. Text is not modified, and a
/// focused or readonly flag is not checked, so the caller decides when to call.
pub fn scroll_caret_into_view(
    state: &mut TextInputState,
    fonts: &mut FontSystem,
    style: &TextStyle,
    scale: f32,
    content_width: f32,
) {
    let content_width = if content_width.is_finite() {
        content_width.max(0.0)
    } else {
        0.0
    };
    let display = state.display_text();
    let (preedit_cursor_x, preedit_w) = preedit_extent(state, fonts, style, scale);
    let caret_x =
        byte_offset_to_x(&display, state.display_cursor(), fonts, style, scale) + preedit_cursor_x;
    let text_width = byte_offset_to_x(&display, display.len(), fonts, style, scale) + preedit_w;
    // Reserve the caret's own width so a caret at the end of the text is drawn
    // inside the content area (render_text_input skips one at the right edge).
    let visible = (content_width - CARET_WIDTH).max(0.0);
    let max_scroll = (text_width - visible).max(0.0);
    let current = if state.scroll_offset.is_finite() {
        state.scroll_offset
    } else {
        0.0
    };
    // Clamp first so text removal shrinks the scroll before the caret check.
    let mut scroll = current.clamp(0.0, max_scroll);
    if caret_x - scroll < 0.0 {
        scroll = caret_x;
    } else if caret_x - scroll > visible {
        scroll = caret_x - visible;
    }
    state.scroll_offset = scroll.clamp(0.0, max_scroll);
}

/// Preedit geometry for scrolling, as `(cursor_x, width)` in logical pixels.
/// `cursor_x` is the composition cursor measured from the committed caret and
/// `width` is the whole drawn preedit. Both are `0.0` when no preedit is active.
/// The widths come from the same string `render_text_input` draws (the mask for
/// password fields), so the scroll matches what is on screen.
pub(crate) fn preedit_extent(
    state: &TextInputState,
    fonts: &mut FontSystem,
    style: &TextStyle,
    scale: f32,
) -> (f32, f32) {
    if state.preedit.is_empty() {
        return (0.0, 0.0);
    }
    let drawn = preedit::preedit_text(state);
    let cursor_end = state
        .preedit_cursor
        .map_or(state.preedit.len(), |(_, end)| end);
    let cursor_x = byte_offset_to_x_inner(
        &drawn,
        preedit::drawn_offset(state, cursor_end),
        fonts,
        style,
        scale,
    );
    let width = byte_offset_to_x_inner(&drawn, drawn.len(), fonts, style, scale);
    (cursor_x, width)
}
