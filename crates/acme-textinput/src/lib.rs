//! TextInput widget for AcmeUI Native.
//!
//! Provides [`TextInputState`] for managing editable text state (cursor, selection,
//! IME preedit, password masking) and functions for rendering, keyboard handling,
//! and text insertion.
#![forbid(unsafe_op_in_unsafe_fn)]

use acme_platform::{Clipboard, PlatformKey};
use acme_render_wgpu::{Frame, Quad, TextRun};
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextStyle, TextWrap};
use acme_theme::{Theme, ThemeColor};
use unicode_segmentation::UnicodeSegmentation;

// ---------------------------------------------------------------------------
// TextInputState
// ---------------------------------------------------------------------------

/// State for a text input field.
///
/// Manages the text buffer, cursor position (as a byte offset), optional text
/// selection, IME preedit state, focus, and password masking.
#[derive(Clone, Debug)]
pub struct TextInputState {
    /// The underlying text buffer.
    pub text: String,
    /// Byte offset of the cursor into `text`.
    pub cursor: usize,
    /// Optional byte-range selection `(start, end)`.
    /// `start` is always ≤ `end`.
    pub selection: Option<(usize, usize)>,
    /// IME preedit text (displayed inline but not yet committed).
    pub preedit: String,
    /// Optional cursor position within the preedit text as byte offsets.
    pub preedit_cursor: Option<(usize, usize)>,
    /// Whether this input currently has keyboard focus.
    pub focused: bool,
    /// If `true`, the display text is masked with `password_char`.
    pub password: bool,
    /// The character used for password masking (default `'•'`).
    pub password_char: char,
}

impl Default for TextInputState {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            selection: None,
            preedit: String::new(),
            preedit_cursor: None,
            focused: false,
            password: false,
            password_char: '•',
        }
    }
}

impl TextInputState {
    /// Create a new empty `TextInputState`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a single character at the cursor, replacing any active selection.
    ///
    /// Returns `true` if the state was modified.
    pub fn insert_char(&mut self, c: char) -> bool {
        self.replace_selection();
        if self.cursor > self.text.len() {
            self.cursor = self.text.len();
        }
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
        self.selection = None;
        true
    }

    /// Delete the grapheme cluster before the cursor (Backspace).
    ///
    /// If a selection is active, deletes the selection instead.
    /// Returns `true` if the state was modified.
    pub fn delete_before_cursor(&mut self) -> bool {
        if self.selection.is_some() {
            return self.delete_selection();
        }
        if self.cursor == 0 || self.text.is_empty() {
            return false;
        }
        let offset = prev_grapheme_boundary(&self.text, self.cursor);
        self.text.drain(offset..self.cursor);
        self.cursor = offset;
        true
    }

    /// Delete the grapheme cluster after the cursor (Delete / Forward Delete).
    ///
    /// If a selection is active, deletes the selection instead.
    /// Returns `true` if the state was modified.
    pub fn delete_after_cursor(&mut self) -> bool {
        if self.selection.is_some() {
            return self.delete_selection();
        }
        if self.cursor >= self.text.len() || self.text.is_empty() {
            return false;
        }
        let end = next_grapheme_boundary(&self.text, self.cursor);
        self.text.drain(self.cursor..end);
        true
    }

    /// Move the cursor one grapheme cluster to the left (towards the start).
    pub fn cursor_prev(&mut self) {
        self.selection = None;
        if self.cursor == 0 || self.text.is_empty() {
            return;
        }
        self.cursor = prev_grapheme_boundary(&self.text, self.cursor);
    }

    /// Move the cursor one grapheme cluster to the right (towards the end).
    pub fn cursor_next(&mut self) {
        self.selection = None;
        if self.cursor >= self.text.len() || self.text.is_empty() {
            return;
        }
        self.cursor = next_grapheme_boundary(&self.text, self.cursor);
    }

    /// Move the cursor to the beginning of the text.
    pub fn cursor_home(&mut self) {
        self.selection = None;
        self.cursor = 0;
    }

    /// Move the cursor to the end of the text.
    pub fn cursor_end(&mut self) {
        self.selection = None;
        self.cursor = self.text.len();
    }

    /// Select all text.
    pub fn select_all(&mut self) {
        if self.text.is_empty() {
            self.selection = None;
            return;
        }
        self.selection = Some((0, self.text.len()));
    }

    // ---- IME support ----

    /// Set IME preedit text and optional cursor position within it.
    ///
    /// `cursor` is `(start, end)` byte offsets within the preedit string,
    /// or `None` if the cursor is at the end of the preedit.
    pub fn set_preedit(&mut self, text: &str, cursor: Option<(usize, usize)>) {
        self.preedit = text.to_string();
        self.preedit_cursor = cursor;
    }

    /// Commit the current IME preedit text, inserting it into the text buffer
    /// at the cursor position (replacing any active selection).
    pub fn commit_preedit(&mut self) {
        if self.preedit.is_empty() {
            return;
        }
        self.replace_selection();
        let text = std::mem::take(&mut self.preedit);
        if self.cursor > self.text.len() {
            self.cursor = self.text.len();
        }
        self.text.insert_str(self.cursor, &text);
        self.cursor += text.len();
        self.preedit_cursor = None;
        self.selection = None;
    }

    /// Cancel the current IME preedit without committing.
    pub fn cancel_preedit(&mut self) {
        self.preedit.clear();
        self.preedit_cursor = None;
    }

    // ---- helpers ----

    /// Return the display text, masked with `password_char` if `password` is true.
    ///
    /// Each grapheme cluster in the original text is replaced with one
    /// `password_char`. This preserves the visual width approximately.
    pub fn display_text(&self) -> String {
        if self.password {
            let count = self.text.graphemes(true).count();
            // Pre-allocate: grapheme count * password_char UTF-8 len
            let cap = count * self.password_char.len_utf8();
            let mut out = String::with_capacity(cap);
            for _ in 0..count {
                out.push(self.password_char);
            }
            out
        } else {
            self.text.clone()
        }
    }

    /// Compute the pixel x-coordinate of the cursor for rendering.
    ///
    /// Shapes the text up to the cursor byte offset and returns its width.
    pub fn cursor_x(&self, fonts: &mut FontSystem, style: &TextStyle, scale: f32) -> f32 {
        let display = self.display_text();
        byte_offset_to_x(&display, self.cursor, fonts, style, scale)
    }

    // ---- internal helpers ----

    /// If a selection exists, delete it and place the cursor at the start.
    /// Returns `true` if a selection was deleted.
    fn delete_selection(&mut self) -> bool {
        let (start, end) = match self.selection {
            Some(s) => s,
            None => return false,
        };
        if start >= self.text.len() || end > self.text.len() || start >= end {
            self.selection = None;
            return false;
        }
        self.text.drain(start..end);
        self.cursor = start;
        self.selection = None;
        true
    }

    /// Replace the selected range with the preedit or prepare for insertion.
    /// Common pre-step for `insert_char`, `commit_preedit`, and `handle_text`.
    fn replace_selection(&mut self) {
        if self.selection.is_some() {
            self.delete_selection();
        }
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// Helper: convert a [`ThemeColor`] to an RGBA `[f32; 4]` array.
fn theme_color_to_array(c: &ThemeColor) -> [f32; 4] {
    [c.red, c.green, c.blue, c.alpha]
}

/// Helper: intersect two `[x, y, w, h]` rectangles.
fn intersect_rect(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let x = a[0].max(b[0]);
    let y = a[1].max(b[1]);
    let right = (a[0] + a[2]).min(b[0] + b[2]);
    let bottom = (a[1] + a[3]).min(b[1] + b[3]);
    if right > x && bottom > y {
        [x, y, right - x, bottom - y]
    } else {
        [0.0, 0.0, 0.0, 0.0]
    }
}

/// Renders a TextInput cursor and selection into a [`Frame`].
///
/// `rect` is `[x, y, width, height]` in logical pixels.
/// `clip` is an optional outer clip rectangle; the content area is further
/// clipped to the inner padding region.
#[allow(clippy::too_many_arguments)]
pub fn render_text_input(
    frame: &mut Frame,
    state: &TextInputState,
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
    rect: [f32; 4], // [x, y, width, height] in logical pixels
    theme: &Theme,
    scale: f32,
    focused: bool,
    clip: Option<[f32; 4]>,
) {
    let [x, y, w, h] = rect;
    if w <= 0.0 || h <= 0.0 {
        return;
    }

    let padding = theme.spacing.sm; // 8px vertical/horizontal inner padding
    let border_width = 1.0;

    let text_color = theme_color_to_array(&theme.colors.text);
    let border_color = if focused {
        theme_color_to_array(&theme.colors.focus)
    } else {
        theme_color_to_array(&theme.colors.border)
    };
    let bg_color = theme_color_to_array(&theme.colors.surface);

    // 1. Background
    frame.quads.push(Quad::solid(rect, bg_color));

    // 2. Border
    frame.quads.push(Quad {
        rect,
        color: [0.0, 0.0, 0.0, 0.0], // fully transparent fill
        radius: theme.radii.sm,
        border_width,
        border_color,
    });

    // 3. Text content
    let display = state.display_text();

    // Build style from theme typography tokens
    let font_size = theme.typography.body_size;
    let style = TextStyle {
        font_size,
        line_height: font_size * theme.typography.line_height,
        ..TextStyle::default()
    };

    // Compute content area (inside padding)
    let content_x = x + padding;
    let content_y = y + padding;
    let content_w = (w - 2.0 * padding).max(0.0);
    let content_h = (h - 2.0 * padding).max(0.0);

    // Build the clip rectangle for the content area
    let content_clip = [content_x, content_y, content_w, content_h];
    let effective_clip = match clip {
        Some(c) => intersect_rect(c, content_clip),
        None => content_clip,
    };

    // We want the text to be vertically centered in the content area.
    // Shape to get the text height, then compute y offset.
    let constraints = TextConstraints {
        max_width: Some(content_w),
        wrap: TextWrap::None,
    };
    let layout = fonts.shape(&display, &style, constraints, scale);
    let text_height = layout.height;
    // Vertically center
    let text_y = content_y + (content_h - text_height).max(0.0) / 2.0;

    // Prepare glyphs
    let prepared = fonts.prepare(&layout, atlas);

    frame.text.push(TextRun {
        origin: [content_x, text_y],
        color: text_color,
        clip: Some(effective_clip),
        prepared,
    });

    // 4. Selection highlight
    if let Some((sel_start, sel_end)) = state.selection {
        let clamped_start = sel_start.min(display.len());
        let clamped_end = sel_end.min(display.len());
        if clamped_start < clamped_end {
            let start_x = byte_offset_to_x_inner(&display, clamped_start, fonts, &style, scale);
            let end_x = byte_offset_to_x_inner(&display, clamped_end, fonts, &style, scale);
            let sel_w = end_x - start_x;
            if sel_w > 0.0 {
                frame.clipped_quads.push(
                    Quad::solid(
                        [content_x + start_x, content_y, sel_w, content_h],
                        [0.3, 0.5, 0.9, 0.25], // light blue selection tint
                    )
                    .with_clip(effective_clip),
                );
            }
        }
    }

    // 5. Cursor blink (simple cursor: always visible when focused)
    if focused {
        let cx = byte_offset_to_x_inner(&display, state.cursor, fonts, &style, scale);
        let cursor_x_pos = content_x + cx;
        // Only draw cursor if it's within the visible content area
        if cursor_x_pos < content_x + content_w {
            frame.clipped_quads.push(
                Quad::solid([cursor_x_pos, content_y, 1.5, content_h], text_color)
                    .with_clip(effective_clip),
            );
        }
    }

    // 6. IME preedit underline
    if !state.preedit.is_empty() {
        let preedit_x = byte_offset_to_x_inner(&display, state.cursor, fonts, &style, scale);
        let preedit_layout = fonts.shape(&state.preedit, &style, TextConstraints::default(), scale);
        let preedit_w = preedit_layout.width;
        if preedit_w > 0.0 {
            let underline_y = content_y + content_h - 1.5;
            frame.clipped_quads.push(
                Quad::solid(
                    [content_x + preedit_x, underline_y, preedit_w, 1.5],
                    text_color,
                )
                .with_clip(effective_clip),
            );
        }
    }
}

/// Helper to compute `byte_offset_to_x` without creating `TextConstraints` each time.
fn byte_offset_to_x_inner(
    text: &str,
    offset: usize,
    fonts: &mut FontSystem,
    style: &TextStyle,
    scale: f32,
) -> f32 {
    if text.is_empty() || offset == 0 {
        return 0.0;
    }
    let clamped = offset.min(text.len());
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

// ---------------------------------------------------------------------------
// Keyboard handling
// ---------------------------------------------------------------------------

/// Process a keyboard event into TextInputState changes.
///
/// # Keyboard map
///
/// | Key               | Action                 |
/// |-------------------|------------------------|
/// | `ArrowLeft`       | `cursor_prev()`        |
/// | `ArrowRight`      | `cursor_next()`        |
/// | `Backspace`       | `delete_before_cursor()` |
/// | `Delete`          | `delete_after_cursor()`  |
/// | `Home`            | `cursor_home()`        |
/// | `End`             | `cursor_end()`         |
/// | `Escape`          | Blur (clear focus)     |
/// | `Tab`             | (handled by focus manager — no-op here) |
/// | `Enter`           | (submitted — no-op here) |
///
/// When `ctrl` is `true` and the state is focused, the following text-based
/// shortcuts are available (via `key_text`):
///
/// | Shortcut | Action                    |
/// |----------|---------------------------|
/// | Ctrl+A   | `select_all()`            |
/// | Ctrl+C   | Copy selection to clipboard |
/// | Ctrl+V   | Paste from clipboard      |
/// | Ctrl+X   | Cut selection to clipboard |
///
/// Returns `true` if the state was modified.
pub fn handle_key(
    state: &mut TextInputState,
    key: &PlatformKey,
    pressed: bool,
    _clipboard: Option<&Clipboard>,
    ctrl: bool,
) -> bool {
    if !pressed {
        return false;
    }
    match key {
        PlatformKey::Tab => {
            // Handled by the focus manager — no state change here.
            false
        }
        PlatformKey::Enter => {
            // Typically signals submission — no state change here.
            false
        }
        PlatformKey::Escape => {
            if state.focused {
                state.focused = false;
                true
            } else {
                false
            }
        }
        PlatformKey::ArrowLeft => {
            if !state.focused {
                return false;
            }
            state.cursor_prev();
            true
        }
        PlatformKey::ArrowRight => {
            if !state.focused {
                return false;
            }
            state.cursor_next();
            true
        }
        PlatformKey::Backspace => {
            if !state.focused {
                return false;
            }
            state.delete_before_cursor()
        }
        PlatformKey::Delete => {
            if !state.focused {
                return false;
            }
            state.delete_after_cursor()
        }
        PlatformKey::Home => {
            if !state.focused {
                return false;
            }
            state.cursor_home();
            true
        }
        PlatformKey::End => {
            if !state.focused {
                return false;
            }
            state.cursor_end();
            true
        }
        PlatformKey::Space | PlatformKey::Other => {
            if !state.focused {
                return false;
            }
            if ctrl {
                // Cannot distinguish Ctrl+A/C/V/X without key_text;
                // the caller should route those through clipboard API before
                // calling handle_key.  For now this arm returns false so
                // ctrl+letter events don't accidentally match Space/Other.
                false
            } else {
                false
            }
        }
    }
}

/// Handle clipboard shortcuts for a text input.
///
/// Call this from the application event loop when `PlatformEvent::Key` has
/// `ctrl: true` and `text` is `Some("a")`, `Some("c")`, `Some("v")`, or
/// `Some("x")`.
///
/// Returns `true` if the state or clipboard was modified.
pub fn handle_keyboard_shortcut(
    state: &mut TextInputState,
    key_text: &str,
    clipboard: Option<&Clipboard>,
) -> bool {
    if !state.focused {
        return false;
    }
    match key_text {
        "a" => {
            state.select_all();
            true
        }
        "c" => {
            if let Some((start, end)) = state.selection
                && start < state.text.len()
                && end <= state.text.len()
                && start < end
            {
                let copied = state.text[start..end].to_string();
                if let Some(clip) = clipboard {
                    let _ = clip.set_text(&copied);
                }
            }
            true
        }
        "x" => {
            if let Some((start, end)) = state.selection
                && start < state.text.len()
                && end <= state.text.len()
                && start < end
            {
                let cut = state.text[start..end].to_string();
                if let Some(clip) = clipboard {
                    let _ = clip.set_text(&cut);
                }
                state.text.drain(start..end);
                state.cursor = start;
                state.selection = None;
            }
            true
        }
        "v" => {
            if let Some(clip) = clipboard
                && let Ok(text) = clip.get_text()
            {
                handle_text(state, &text);
            }
            true
        }
        _ => false,
    }
}

/// Handle text input (characters and IME committed text).
///
/// Inserts `text` at the cursor position, replacing any active selection.
/// Returns `true` if the state was modified.
pub fn handle_text(state: &mut TextInputState, text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    state.replace_selection();
    if state.cursor > state.text.len() {
        state.cursor = state.text.len();
    }
    state.text.insert_str(state.cursor, text);
    state.cursor += text.len();
    state.selection = None;
    true
}

// ---------------------------------------------------------------------------
// Grapheme-boundary helpers
// ---------------------------------------------------------------------------

/// Return the byte offset of the grapheme cluster boundary immediately before
/// `offset`.  If `offset` is already at a boundary, returns the previous one.
fn prev_grapheme_boundary(text: &str, offset: usize) -> usize {
    // Collect all grapheme cluster boundaries up to `offset`.
    let clamped = offset.min(text.len());
    let mut prev = 0;
    for (boundary, _grapheme) in text.grapheme_indices(true) {
        if boundary >= clamped {
            break;
        }
        prev = boundary;
    }
    prev
}

/// Return the byte offset of the grapheme cluster boundary immediately after
/// `offset`.
fn next_grapheme_boundary(text: &str, offset: usize) -> usize {
    let clamped = offset.min(text.len());
    for (boundary, grapheme) in text.grapheme_indices(true) {
        if boundary > clamped {
            return boundary;
        }
        if boundary == clamped {
            // This grapheme starts at `clamped` — return the next one.
            return boundary + grapheme.len();
        }
    }
    text.len()
}

// ---------------------------------------------------------------------------
// Quad extension helper for clipped quads
// ---------------------------------------------------------------------------

trait IntoClippedQuad {
    fn with_clip(self, clip: [f32; 4]) -> acme_render_wgpu::ClippedQuad;
}

impl IntoClippedQuad for Quad {
    fn with_clip(self, clip: [f32; 4]) -> acme_render_wgpu::ClippedQuad {
        acme_render_wgpu::ClippedQuad { quad: self, clip }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- TextInputState: insert ----

    #[test]
    fn insert_into_empty() {
        let mut s = TextInputState::new();
        assert!(s.insert_char('a'));
        assert_eq!(s.text, "a");
        assert_eq!(s.cursor, 1);
        assert_eq!(s.selection, None);
    }

    #[test]
    fn insert_at_end() {
        let mut s = TextInputState::new();
        s.text = "hi".to_string();
        s.cursor = 2;
        assert!(s.insert_char('!'));
        assert_eq!(s.text, "hi!");
        assert_eq!(s.cursor, 3);
    }

    #[test]
    fn insert_in_middle() {
        let mut s = TextInputState::new();
        s.text = "helo".to_string();
        s.cursor = 3;
        assert!(s.insert_char('l'));
        assert_eq!(s.text, "hello");
        assert_eq!(s.cursor, 4);
    }

    #[test]
    fn insert_replaces_selection() {
        let mut s = TextInputState::new();
        s.text = "hello world".to_string();
        s.cursor = 5;
        s.selection = Some((5, 11)); // select " world"
        assert!(s.insert_char('!'));
        assert_eq!(s.text, "hello!");
        assert_eq!(s.cursor, 6);
        assert_eq!(s.selection, None);
    }

    // ---- TextInputState: delete ----

    #[test]
    fn delete_before_empty() {
        let mut s = TextInputState::new();
        assert!(!s.delete_before_cursor());
    }

    #[test]
    fn delete_before_at_start() {
        let mut s = TextInputState::new();
        s.text = "abc".to_string();
        s.cursor = 0;
        assert!(!s.delete_before_cursor());
        assert_eq!(s.text, "abc");
    }

    #[test]
    fn delete_before_single_ascii() {
        let mut s = TextInputState::new();
        s.text = "abc".to_string();
        s.cursor = 1;
        assert!(s.delete_before_cursor());
        assert_eq!(s.text, "bc");
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn delete_before_grapheme() {
        let mut s = TextInputState::new();
        s.text = "a😊b".to_string();
        s.cursor = 5; // after the emoji (byte offset: 'a'=1, 😊=4, so after 😊 is at 5)
        assert!(s.delete_before_cursor());
        assert_eq!(s.text, "ab");
        assert_eq!(s.cursor, 1);
    }

    #[test]
    fn delete_before_with_selection() {
        let mut s = TextInputState::new();
        s.text = "hello world".to_string();
        s.cursor = 11;
        s.selection = Some((6, 11)); // select "world"
        assert!(s.delete_before_cursor());
        assert_eq!(s.text, "hello ");
        assert_eq!(s.cursor, 6);
        assert_eq!(s.selection, None);
    }

    #[test]
    fn delete_after_at_end() {
        let mut s = TextInputState::new();
        s.text = "abc".to_string();
        s.cursor = 3;
        assert!(!s.delete_after_cursor());
    }

    #[test]
    fn delete_after_single() {
        let mut s = TextInputState::new();
        s.text = "abc".to_string();
        s.cursor = 0;
        assert!(s.delete_after_cursor());
        assert_eq!(s.text, "bc");
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn delete_after_grapheme() {
        let mut s = TextInputState::new();
        s.text = "a😊b".to_string();
        s.cursor = 0;
        // The 'a' is a single ASCII grapheme (1 byte)
        assert!(s.delete_after_cursor());
        assert_eq!(s.text, "😊b");
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn delete_after_with_selection() {
        let mut s = TextInputState::new();
        s.text = "hello world".to_string();
        s.cursor = 0;
        s.selection = Some((0, 6)); // select "hello "
        assert!(s.delete_after_cursor());
        assert_eq!(s.text, "world");
        assert_eq!(s.cursor, 0);
        assert_eq!(s.selection, None);
    }

    // ---- TextInputState: cursor movement ----

    #[test]
    fn cursor_prev_empty() {
        let mut s = TextInputState::new();
        s.cursor_prev();
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn cursor_prev_ascii() {
        let mut s = TextInputState::new();
        s.text = "abc".to_string();
        s.cursor = 2;
        s.cursor_prev();
        assert_eq!(s.cursor, 1);
    }

    #[test]
    fn cursor_prev_grapheme() {
        let mut s = TextInputState::new();
        s.text = "a😊b".to_string();
        s.cursor = 5; // after 'a' (1) + 😊 (4) = 5 bytes
        s.cursor_prev();
        assert_eq!(s.cursor, 1); // Before the emoji
        s.cursor_prev();
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn cursor_next_empty() {
        let mut s = TextInputState::new();
        s.cursor_next();
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn cursor_next_ascii() {
        let mut s = TextInputState::new();
        s.text = "abc".to_string();
        s.cursor = 0;
        s.cursor_next();
        assert_eq!(s.cursor, 1);
        s.cursor_next();
        assert_eq!(s.cursor, 2);
        s.cursor_next();
        assert_eq!(s.cursor, 3);
        // at end
        s.cursor_next();
        assert_eq!(s.cursor, 3);
    }

    #[test]
    fn cursor_next_grapheme() {
        let mut s = TextInputState::new();
        s.text = "a😊b".to_string();
        s.cursor = 0;
        s.cursor_next();
        assert_eq!(s.cursor, 1); // after 'a'
        s.cursor_next();
        assert_eq!(s.cursor, 5); // after '😊' (1 + 4)
        s.cursor_next();
        assert_eq!(s.cursor, 6); // after 'b'
    }

    #[test]
    fn cursor_home_end() {
        let mut s = TextInputState::new();
        s.text = "hello world".to_string();
        s.cursor = 5;
        s.cursor_home();
        assert_eq!(s.cursor, 0);
        s.cursor_end();
        assert_eq!(s.cursor, 11);
    }

    #[test]
    fn cursor_movement_clears_selection() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 4;
        s.selection = Some((1, 4));
        s.cursor_prev();
        assert_eq!(s.selection, None);
        // Cursor moves to previous grapheme boundary from 4
        assert_eq!(s.cursor, 3);

        s.text = "hello".to_string();
        s.cursor = 1;
        s.selection = Some((1, 4));
        s.cursor_next();
        assert_eq!(s.selection, None);
        assert_eq!(s.cursor, 2);

        s.text = "hello".to_string();
        s.selection = Some((1, 4));
        s.cursor_home();
        assert_eq!(s.selection, None);

        s.text = "hello".to_string();
        s.selection = Some((1, 4));
        s.cursor_end();
        assert_eq!(s.selection, None);
    }

    // ---- TextInputState: selection ----

    #[test]
    fn select_all_empty() {
        let mut s = TextInputState::new();
        s.select_all();
        assert_eq!(s.selection, None);
    }

    #[test]
    fn select_all_nonempty() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.select_all();
        assert_eq!(s.selection, Some((0, 5)));
    }

    // ---- TextInputState: IME preedit/commit/cancel ----

    #[test]
    fn preedit_set_and_cancel() {
        let mut s = TextInputState::new();
        s.set_preedit("hello", Some((2, 2)));
        assert_eq!(s.preedit, "hello");
        assert_eq!(s.preedit_cursor, Some((2, 2)));

        s.cancel_preedit();
        assert!(s.preedit.is_empty());
        assert_eq!(s.preedit_cursor, None);
    }

    #[test]
    fn preedit_commit_into_empty() {
        let mut s = TextInputState::new();
        s.set_preedit("hello", None);
        s.commit_preedit();
        assert_eq!(s.text, "hello");
        assert_eq!(s.cursor, 5);
        assert!(s.preedit.is_empty());
    }

    #[test]
    fn preedit_commit_in_middle() {
        let mut s = TextInputState::new();
        s.text = "ab".to_string();
        s.cursor = 1;
        s.set_preedit("123", None);
        s.commit_preedit();
        assert_eq!(s.text, "a123b");
        assert_eq!(s.cursor, 4);
    }

    #[test]
    fn preedit_commit_replaces_selection() {
        let mut s = TextInputState::new();
        s.text = "hello world".to_string();
        s.cursor = 11;
        s.selection = Some((6, 11));
        s.set_preedit("there", None);
        s.commit_preedit();
        assert_eq!(s.text, "hello there");
        assert_eq!(s.cursor, 11);
    }

    #[test]
    fn preedit_commit_empty_is_noop() {
        let mut s = TextInputState::new();
        s.text = "hi".to_string();
        s.cursor = 2;
        s.set_preedit("", None);
        s.commit_preedit();
        assert_eq!(s.text, "hi");
        assert_eq!(s.cursor, 2);
    }

    // ---- TextInputState: password masking ----

    #[test]
    fn password_masking_empty() {
        let s = TextInputState::new();
        assert_eq!(s.display_text(), "");
    }

    #[test]
    fn password_masking_nonempty() {
        let mut s = TextInputState::new();
        s.password = true;
        s.password_char = '*';
        s.text = "hello".to_string();
        assert_eq!(s.display_text(), "*****");
    }

    #[test]
    fn password_masking_default_char() {
        let mut s = TextInputState::new();
        s.password = true;
        s.text = "secret".to_string();
        let masked = s.display_text();
        assert_eq!(masked.len(), 6 * '•'.len_utf8());
        assert!(masked.chars().all(|c| c == '•'));
    }

    #[test]
    fn password_masking_graphemes() {
        let mut s = TextInputState::new();
        s.password = true;
        s.password_char = '*';
        // Multi-byte characters — count grapheme clusters, not bytes
        s.text = "a😊b".to_string();
        assert_eq!(s.display_text(), "***");
    }

    #[test]
    fn display_text_normal() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        assert_eq!(s.display_text(), "hello");
    }

    // ---- handle_text ----

    #[test]
    fn handle_text_inserts_at_cursor() {
        let mut s = TextInputState::new();
        s.text = "ab".to_string();
        s.cursor = 1;
        assert!(handle_text(&mut s, "123"));
        assert_eq!(s.text, "a123b");
        assert_eq!(s.cursor, 4);
    }

    #[test]
    fn handle_text_replaces_selection() {
        let mut s = TextInputState::new();
        s.text = "hello world".to_string();
        s.cursor = 11;
        s.selection = Some((6, 11));
        assert!(handle_text(&mut s, "there"));
        assert_eq!(s.text, "hello there");
        assert_eq!(s.cursor, 11);
        assert_eq!(s.selection, None);
    }

    #[test]
    fn handle_text_empty_is_noop() {
        let mut s = TextInputState::new();
        s.text = "hi".to_string();
        s.cursor = 2;
        assert!(!handle_text(&mut s, ""));
        assert_eq!(s.text, "hi");
    }

    #[test]
    fn handle_text_commits_single_char() {
        let mut s = TextInputState::new();
        assert!(handle_text(&mut s, "x"));
        assert_eq!(s.text, "x");
        assert_eq!(s.cursor, 1);
    }

    // ---- handle_key ----

    #[test]
    fn handle_key_escape_blurs() {
        let mut s = TextInputState::new();
        s.focused = true;
        assert!(handle_key(&mut s, &PlatformKey::Escape, true, None, false));
        assert!(!s.focused);
    }

    #[test]
    fn handle_key_escape_noop_when_not_focused() {
        let mut s = TextInputState::new();
        s.focused = false;
        assert!(!handle_key(&mut s, &PlatformKey::Escape, true, None, false));
    }

    #[test]
    fn handle_key_tab_is_noop() {
        let mut s = TextInputState::new();
        assert!(!handle_key(&mut s, &PlatformKey::Tab, true, None, false));
    }

    #[test]
    fn handle_key_enter_is_noop() {
        let mut s = TextInputState::new();
        assert!(!handle_key(&mut s, &PlatformKey::Enter, true, None, false));
    }

    #[test]
    fn handle_key_noop_on_release() {
        let mut s = TextInputState::new();
        s.focused = true;
        assert!(!handle_key(
            &mut s,
            &PlatformKey::Escape,
            false,
            None,
            false
        ));
    }

    // ---- handle_key: new PlatformKey variants ----

    #[test]
    fn handle_key_arrow_left_moves_cursor() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 3;
        s.focused = true;
        assert!(handle_key(
            &mut s,
            &PlatformKey::ArrowLeft,
            true,
            None,
            false
        ));
        assert_eq!(s.cursor, 2);
    }

    #[test]
    fn handle_key_arrow_left_noop_when_not_focused() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 3;
        s.focused = false;
        assert!(!handle_key(
            &mut s,
            &PlatformKey::ArrowLeft,
            true,
            None,
            false
        ));
        assert_eq!(s.cursor, 3);
    }

    #[test]
    fn handle_key_arrow_right_moves_cursor() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 2;
        s.focused = true;
        assert!(handle_key(
            &mut s,
            &PlatformKey::ArrowRight,
            true,
            None,
            false
        ));
        assert_eq!(s.cursor, 3);
    }

    #[test]
    fn handle_key_backspace_deletes_before() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 5;
        s.focused = true;
        assert!(handle_key(
            &mut s,
            &PlatformKey::Backspace,
            true,
            None,
            false
        ));
        assert_eq!(s.text, "hell");
        assert_eq!(s.cursor, 4);
    }

    #[test]
    fn handle_key_backspace_noop_when_not_focused() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 5;
        s.focused = false;
        assert!(!handle_key(
            &mut s,
            &PlatformKey::Backspace,
            true,
            None,
            false
        ));
    }

    #[test]
    fn handle_key_delete_deletes_after() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 0;
        s.focused = true;
        assert!(handle_key(&mut s, &PlatformKey::Delete, true, None, false));
        assert_eq!(s.text, "ello");
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn handle_key_home_moves_to_start() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 3;
        s.focused = true;
        assert!(handle_key(&mut s, &PlatformKey::Home, true, None, false));
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn handle_key_end_moves_to_end() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 0;
        s.focused = true;
        assert!(handle_key(&mut s, &PlatformKey::End, true, None, false));
        assert_eq!(s.cursor, 5);
    }

    #[test]
    fn handle_key_navigation_clears_selection() {
        let mut s = TextInputState::new();
        s.text = "hello world".to_string();
        s.cursor = 5;
        s.selection = Some((2, 8));
        s.focused = true;
        handle_key(&mut s, &PlatformKey::ArrowLeft, true, None, false);
        assert_eq!(s.selection, None);
    }

    // ---- handle_keyboard_shortcut ----

    #[test]
    fn shortcut_ctrl_a_selects_all() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 2;
        s.focused = true;
        assert!(handle_keyboard_shortcut(&mut s, "a", None));
        assert_eq!(s.selection, Some((0, 5)));
    }

    #[test]
    fn shortcut_ctrl_c_does_not_panic() {
        let mut s = TextInputState::new();
        s.text = "select".to_string();
        s.selection = Some((0, 6));
        s.focused = true;
        // Without a real clipboard, copy should be a no-op that returns true
        assert!(handle_keyboard_shortcut(&mut s, "c", None));
    }

    #[test]
    fn shortcut_ctrl_x_cuts_selection() {
        let mut s = TextInputState::new();
        s.text = "cut me".to_string();
        s.cursor = 6;
        s.selection = Some((0, 6));
        s.focused = true;
        assert!(handle_keyboard_shortcut(&mut s, "x", None));
        assert!(s.text.is_empty());
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn shortcut_ctrl_v_noop_without_clipboard() {
        let mut s = TextInputState::new();
        s.text = "hello ".to_string();
        s.cursor = 6;
        s.focused = true;
        // Without clipboard, paste is a no-op that returns true
        assert!(handle_keyboard_shortcut(&mut s, "v", None));
    }

    #[test]
    fn shortcut_ctrl_a_noop_when_not_focused() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.focused = false;
        assert!(!handle_keyboard_shortcut(&mut s, "a", None));
    }

    #[test]
    fn shortcut_unknown_key_is_noop() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.focused = true;
        assert!(!handle_keyboard_shortcut(&mut s, "z", None));
    }

    // ---- Clipboard integration test ----
    // Tests clipboard read/write through state manipulation directly,
    // since the full clipboard keyboard shortcuts require PlatformKey
    // extension for Ctrl+C/V/X.

    #[test]
    fn copy_via_text_access() {
        // Simulate Ctrl+C: read from state.text using selection
        let mut s = TextInputState::new();
        s.text = "select me".to_string();
        s.selection = Some((0, 9));
        let copied = if let Some((start, end)) = s.selection {
            if start < s.text.len() && end <= s.text.len() {
                Some(s.text[start..end].to_string())
            } else {
                None
            }
        } else {
            None
        };
        assert_eq!(copied.as_deref(), Some("select me"));
    }

    #[test]
    fn cut_via_text_access() {
        // Simulate Ctrl+X: read selection, then delete it
        let mut s = TextInputState::new();
        s.text = "select me".to_string();
        s.cursor = 9;
        s.selection = Some((0, 9));
        let cut = if let Some((start, end)) = s.selection {
            if start < s.text.len() && end <= s.text.len() {
                let t = s.text[start..end].to_string();
                s.text.drain(start..end);
                s.cursor = start;
                s.selection = None;
                Some(t)
            } else {
                None
            }
        } else {
            None
        };
        assert_eq!(cut.as_deref(), Some("select me"));
        assert!(s.text.is_empty());
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn paste_via_text_handle() {
        // Simulate Ctrl+V: insert clipboard content via handle_text
        let mut s = TextInputState::new();
        s.text = "hello ".to_string();
        s.cursor = 6;
        let clipboard_content = "world";
        handle_text(&mut s, clipboard_content);
        assert_eq!(s.text, "hello world");
        assert_eq!(s.cursor, 11);
    }
}
