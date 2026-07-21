//! TextInput widget for AcmeUI Native.
//!
//! Provides [`TextInputState`] for managing editable text state (cursor, selection,
//! IME preedit, password masking) and functions for rendering, keyboard handling,
//! and text insertion.
#![forbid(unsafe_op_in_unsafe_fn)]

use acme_core::{Color, DrawCommand, GlyphDraw, GlyphFormat, Point, QuadPrimitive, Rect, Scene, TextPrimitive, AtlasUpload as SceneAtlasUpload};
use acme_platform::{Clipboard, PlatformKey};
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextLayout, TextStyle, TextWrap};
use acme_theme::{Theme, ThemeColor};
use unicode_segmentation::UnicodeSegmentation;

// ---------------------------------------------------------------------------
// EditTransaction — undo/redo record
// ---------------------------------------------------------------------------

/// A single undo/redo transaction capturing the state before and after a mutation.
#[derive(Clone, Debug)]
struct EditTransaction {
    old_text: String,
    new_text: String,
    old_cursor: usize,
    new_cursor: usize,
    old_selection: Option<(usize, usize)>,
    new_selection: Option<(usize, usize)>,
}

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
    cursor: usize,
    /// Optional byte-range selection `(start, end)`.
    /// `start` is always ≤ `end`.
    selection: Option<(usize, usize)>,
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
    /// Placeholder text shown when the text buffer is empty.
    pub placeholder: String,
    /// If `true`, all mutation methods return `false` without modifying state.
    pub readonly: bool,
    /// Visual flag — when `true` the border is rendered in the `danger` colour.
    pub invalid: bool,
    /// Horizontal scroll offset in logical pixels.
    pub scroll_offset: f32,
    /// Maximum undo history entries (default 50).
    max_undo: usize,
    /// Undo transaction stack.
    undo_stack: Vec<EditTransaction>,
    /// Redo transaction stack.
    redo_stack: Vec<EditTransaction>,
    /// Cached shaped text layout, auto-invalidated by text comparison.
    /// The `String` is the `render_text` that was shaped (display text or
    /// placeholder).  Set to `None` initially; populated on first render.
    pub(crate) cached_layout: Option<(String, TextLayout)>,
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
            placeholder: String::new(),
            readonly: false,
            invalid: false,
            scroll_offset: 0.0,
            max_undo: 50,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            cached_layout: None,
        }
    }
}

impl TextInputState {
    /// Create a new empty `TextInputState`.
    pub fn new() -> Self {
        Self::default()
    }

    // ---- Accessors ----

    /// Return the current cursor byte offset.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Return the current selection range, if any.
    pub fn selection(&self) -> Option<(usize, usize)> {
        self.selection
    }

    /// Returns `true` when a selection is active.
    pub fn has_selection(&self) -> bool {
        self.selection.is_some()
    }

    // ---- Undo / Redo ----

    /// Undo the last mutation.
    ///
    /// Returns `true` if a transaction was available and was reverted.
    pub fn undo(&mut self) -> bool {
        let tx = match self.undo_stack.pop() {
            Some(t) => t,
            None => return false,
        };
        // tx: old → new  (state before mutation → state after mutation)
        // Current state = tx.new. We restore to tx.old.
        // Push reverse (old → new) onto redo stack so redo can go back.
        self.redo_stack.push(EditTransaction {
            old_text: tx.old_text.clone(),
            new_text: tx.new_text.clone(),
            old_cursor: tx.old_cursor,
            new_cursor: tx.new_cursor,
            old_selection: tx.old_selection,
            new_selection: tx.new_selection,
        });
        // Restore old state (state before the mutation)
        self.text = tx.old_text;
        self.cursor = tx.old_cursor;
        self.selection = tx.old_selection;
        self.preedit.clear();
        self.preedit_cursor = None;
        true
    }

    /// Redo the last undone mutation.
    ///
    /// Returns `true` if a transaction was available and was re-applied.
    pub fn redo(&mut self) -> bool {
        let tx = match self.redo_stack.pop() {
            Some(t) => t,
            None => return false,
        };
        // tx: old → new  where old = restored state, new = mutation state
        // Current state = tx.old (the restored state).
        // We restore to tx.new (the mutation state).
        // Push the transaction back onto undo_stack so undo can revert this redo.
        // NOTE: We do NOT swap old_* ↔ new_* here — undo() must see the same
        // (old, new) pair it originally pushed onto the redo stack.  If we
        // swapped, then undo after redo would restore new (instead of old),
        // making it a no-op.
        self.undo_stack.push(EditTransaction {
            old_text: tx.old_text.clone(),
            new_text: tx.new_text.clone(),
            old_cursor: tx.old_cursor,
            new_cursor: tx.new_cursor,
            old_selection: tx.old_selection,
            new_selection: tx.new_selection,
        });
        // Restore new state (state before the undo that created this redo entry)
        self.text = tx.new_text;
        self.cursor = tx.new_cursor;
        self.selection = tx.new_selection;
        self.preedit.clear();
        self.preedit_cursor = None;
        true
    }

    /// Clear the entire undo/redo history.
    pub fn clear_history(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    // ---- Mouse / touch click & drag selection ----

    /// Move the cursor to a specific byte offset (clamped to `text.len()`).
    ///
    /// Any active selection is cleared.
    pub fn move_cursor_to_offset(&mut self, byte_offset: usize) {
        self.cursor = byte_offset.min(self.text.len());
        self.selection = None;
    }

    /// Set an explicit selection range.
    ///
    /// Both `start` and `end` are clamped to `text.len()` and `start ≤ end`
    /// is enforced. The cursor is placed at `end`.
    pub fn set_selection_range(&mut self, start: usize, end: usize) {
        let clamped_end = end.min(self.text.len());
        let clamped_start = start.min(clamped_end);
        if clamped_start < clamped_end {
            self.selection = Some((clamped_start, clamped_end));
        } else {
            self.selection = None;
        }
        self.cursor = clamped_end;
    }

    /// Select the word around the given byte offset (double-click).
    ///
    /// Uses `unicode_segmentation` word boundary indices.
    /// If no valid word is found, the selection is cleared and the cursor
    /// is placed at `offset`.
    pub fn select_word_at_offset(&mut self, offset: usize) {
        if self.text.is_empty() {
            self.cursor = 0;
            self.selection = None;
            return;
        }
        let clamped = offset.min(self.text.len());
        // Collect word-boundary indices
        let words: Vec<(usize, &str)> = self.text.split_word_bound_indices().collect();
        for &(start, word) in &words {
            let end = start + word.len();
            if (start..=end).contains(&clamped) {
                self.selection = Some((start, end));
                self.cursor = end;
                return;
            }
        }
        // Fallback: no word found → just position the cursor
        self.cursor = clamped;
        self.selection = None;
    }

    // ---- IME caret area ----

    /// Compute the caret rectangle `[x, y, width, height]` at the current
    /// cursor position in logical pixels.
    ///
    /// Coordinates are relative to the **content origin** (left edge of the
    /// visible text area, i.e. field origin + horizontal padding). The returned
    /// `x` already subtracts [`Self::scroll_offset`], matching
    /// `render_text_input` where glyphs are drawn at
    /// `content_x - scroll_offset`.
    ///
    /// `y` is `0.0` (top of the content line box); height comes from
    /// `style.line_height`. Width is a small constant (caret width).
    ///
    /// Owners convert to window client space by adding the content origin:
    /// `[content_x + rect[0], content_y + rect[1], rect[2], rect[3]]`.
    pub fn ime_caret_area(
        &self,
        fonts: &mut FontSystem,
        style: &TextStyle,
        scale: f32,
    ) -> [f32; 4] {
        let display = self.display_text();
        let raw_x = byte_offset_to_x(&display, self.cursor, fonts, style, scale);
        // Match render_text_input: text_origin_x = content_x - scroll_offset.
        let x = raw_x - self.scroll_offset;
        let h = style.line_height;
        [x, 0.0, 1.5, h]
    }

    /// Insert a single character at the cursor, replacing any active selection.
    ///
    /// Returns `true` if the state was modified.
    pub fn insert_char(&mut self, c: char) -> bool {
        if self.readonly {
            return false;
        }
        debug_assert!(
            self.cursor <= self.text.len(),
            "cursor {} exceeds text length {}",
            self.cursor,
            self.text.len()
        );
        let old_text = self.text.clone();
        let old_cursor = self.cursor;
        let old_selection = self.selection;

        self.replace_selection();
        if self.cursor > self.text.len() {
            self.cursor = self.text.len();
        }
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
        self.selection = None;

        self.push_undo_with_state(old_text, old_cursor, old_selection);
        true
    }

    /// Delete the grapheme cluster before the cursor (Backspace).
    ///
    /// If a selection is active, deletes the selection instead.
    /// Returns `true` if the state was modified.
    pub fn delete_before_cursor(&mut self) -> bool {
        if self.readonly {
            return false;
        }
        debug_assert!(
            self.cursor <= self.text.len(),
            "cursor {} exceeds text length {}",
            self.cursor,
            self.text.len()
        );
        let old_text = self.text.clone();
        let old_cursor = self.cursor;
        let old_selection = self.selection;

        if self.selection.is_some() {
            let result = self.delete_selection();
            if result {
                self.push_undo_with_state(old_text, old_cursor, old_selection);
            }
            return result;
        }
        if self.cursor == 0 || self.text.is_empty() {
            return false;
        }
        let offset = prev_grapheme_boundary(&self.text, self.cursor);
        self.text.drain(offset..self.cursor);
        self.cursor = offset;

        self.push_undo_with_state(old_text, old_cursor, old_selection);
        true
    }

    /// Delete the grapheme cluster after the cursor (Delete / Forward Delete).
    ///
    /// If a selection is active, deletes the selection instead.
    /// Returns `true` if the state was modified.
    pub fn delete_after_cursor(&mut self) -> bool {
        if self.readonly {
            return false;
        }
        debug_assert!(
            self.cursor <= self.text.len(),
            "cursor {} exceeds text length {}",
            self.cursor,
            self.text.len()
        );
        let old_text = self.text.clone();
        let old_cursor = self.cursor;
        let old_selection = self.selection;

        if self.selection.is_some() {
            let result = self.delete_selection();
            if result {
                self.push_undo_with_state(old_text, old_cursor, old_selection);
            }
            return result;
        }
        if self.cursor >= self.text.len() || self.text.is_empty() {
            return false;
        }
        let end = next_grapheme_boundary(&self.text, self.cursor);
        self.text.drain(self.cursor..end);

        self.push_undo_with_state(old_text, old_cursor, old_selection);
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

    // ---- Selection extension (Shift+arrow) ----

    /// Move cursor one grapheme left and extend the selection.
    fn extend_selection_left(&mut self) {
        if self.text.is_empty() || self.cursor == 0 {
            return;
        }
        let old = self.cursor;
        self.cursor = prev_grapheme_boundary(&self.text, self.cursor);
        if old == self.cursor {
            return;
        }
        let anchor = match self.selection {
            None => old,
            Some((start, end)) => {
                // The anchor is the end opposite to where the cursor was.
                if old == start { end } else { start }
            }
        };
        self.selection = Some(if self.cursor < anchor {
            (self.cursor, anchor)
        } else {
            (anchor, self.cursor)
        });
    }

    /// Move cursor one grapheme right and extend the selection.
    fn extend_selection_right(&mut self) {
        if self.text.is_empty() || self.cursor >= self.text.len() {
            return;
        }
        let old = self.cursor;
        self.cursor = next_grapheme_boundary(&self.text, self.cursor);
        if old == self.cursor {
            return;
        }
        let anchor = match self.selection {
            None => old,
            Some((start, end)) => {
                if old == start {
                    end
                } else {
                    start
                }
            }
        };
        self.selection = Some(if self.cursor < anchor {
            (self.cursor, anchor)
        } else {
            (anchor, self.cursor)
        });
    }

    /// Extend the selection to the start of the text.
    fn extend_selection_to_start(&mut self) {
        if self.text.is_empty() {
            return;
        }
        let anchor = match self.selection {
            None => self.cursor,
            Some((_start, end)) => end, // keep the right-most end as anchor
        };
        self.cursor = 0;
        self.selection = Some((0, anchor));
    }

    /// Extend the selection to the end of the text.
    fn extend_selection_to_end(&mut self) {
        if self.text.is_empty() {
            return;
        }
        let anchor = match self.selection {
            None => self.cursor,
            Some((start, _end)) => start, // keep the left-most start as anchor
        };
        self.cursor = self.text.len();
        self.selection = Some((anchor, self.text.len()));
    }

    // ---- Word-level navigation (Ctrl+arrow) ----

    /// Jump to the previous word boundary.
    fn jump_word_left(&mut self) {
        if self.text.is_empty() || self.cursor == 0 {
            return;
        }
        self.selection = None;
        self.cursor = prev_word_boundary(&self.text, self.cursor);
    }

    /// Jump to the next word boundary.
    fn jump_word_right(&mut self) {
        if self.text.is_empty() || self.cursor >= self.text.len() {
            return;
        }
        self.selection = None;
        self.cursor = next_word_boundary(&self.text, self.cursor);
    }

    // ---- Word-level deletion (Ctrl+Backspace / Ctrl+Delete) ----

    /// Delete the word immediately before the cursor.
    fn delete_prev_word(&mut self) -> bool {
        if self.readonly {
            return false;
        }
        if self.selection.is_some() {
            return self.delete_selection();
        }
        if self.cursor == 0 || self.text.is_empty() {
            return false;
        }
        let old_text = self.text.clone();
        let old_cursor = self.cursor;
        let old_selection = self.selection;

        let word_start = prev_word_boundary(&self.text, self.cursor);
        self.text.drain(word_start..self.cursor);
        self.cursor = word_start;

        self.push_undo_with_state(old_text, old_cursor, old_selection);
        true
    }

    /// Delete the word immediately after the cursor.
    fn delete_next_word(&mut self) -> bool {
        if self.readonly {
            return false;
        }
        if self.selection.is_some() {
            return self.delete_selection();
        }
        if self.cursor >= self.text.len() || self.text.is_empty() {
            return false;
        }
        let old_text = self.text.clone();
        let old_cursor = self.cursor;
        let old_selection = self.selection;

        let word_end = next_word_boundary(&self.text, self.cursor);
        self.text.drain(self.cursor..word_end);

        self.push_undo_with_state(old_text, old_cursor, old_selection);
        true
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
        if self.readonly || self.preedit.is_empty() {
            return;
        }
        debug_assert!(
            self.cursor <= self.text.len(),
            "cursor {} exceeds text length {}",
            self.cursor,
            self.text.len()
        );
        let old_text = self.text.clone();
        let old_cursor = self.cursor;
        let old_selection = self.selection;

        self.replace_selection();
        let text = std::mem::take(&mut self.preedit);
        if self.cursor > self.text.len() {
            self.cursor = self.text.len();
        }
        self.text.insert_str(self.cursor, &text);
        self.cursor += text.len();
        self.preedit_cursor = None;
        self.selection = None;

        self.push_undo_with_state(old_text, old_cursor, old_selection);
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

    /// Record an undo transaction for the state *before* a mutation.
    ///
    /// The old state is passed explicitly (`old_text`, `old_cursor`,
    /// `old_selection`); the *new* state is read from `self` after the
    /// mutation has finished.  The redo stack is cleared.
    fn push_undo_with_state(
        &mut self,
        old_text: String,
        old_cursor: usize,
        old_selection: Option<(usize, usize)>,
    ) {
        let tx = EditTransaction {
            old_text,
            new_text: self.text.clone(),
            old_cursor,
            new_cursor: self.cursor,
            old_selection,
            new_selection: self.selection,
        };
        self.undo_stack.push(tx);
        if self.undo_stack.len() > self.max_undo {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

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

// ---------------------------------------------------------------------------
// TextInput rendering helpers
// ---------------------------------------------------------------------------

/// Theme-derived colours for a text input.
struct TextInputColors {
    text_color: [f32; 4],
    border_color: [f32; 4],
    disabled_text: [f32; 4],
    text_muted: [f32; 4],
    bg_color: [f32; 4],
}

/// Compute all theme-derived colours for a text input in one call.
fn compute_text_input_colors(theme: &Theme, state: &TextInputState, focused: bool) -> TextInputColors {
    let text_color = theme_color_to_array(&theme.colors.foreground);
    let disabled_text = theme_color_to_array(&theme.colors.disabled_text);
    let text_muted = theme_color_to_array(&theme.colors.muted_foreground);
    let bg_color = theme_color_to_array(&theme.colors.surface);

    // Border colour: invalid > focus > default
    let border_color = if state.invalid {
        theme_color_to_array(&theme.colors.danger)
    } else if focused {
        theme_color_to_array(&theme.colors.ring)
    } else {
        theme_color_to_array(&theme.colors.border)
    };

    // Slightly dimmed border when readonly
    let border_color = if state.readonly {
        let [r, g, b, a] = border_color;
        [r * 0.6, g * 0.6, b * 0.6, a]
    } else {
        border_color
    };

    TextInputColors { text_color, border_color, disabled_text, text_muted, bg_color }
}

/// Push background fill and border quads for a text input.
fn render_text_input_background(
    scene: &mut Scene,
    rect: [f32; 4],
    border_color: [f32; 4],
    bg_color: [f32; 4],
    theme: &Theme,
) {
    let border_width = 1.0;

    // 1. Background quad
    scene.push(DrawCommand::Quad(QuadPrimitive {
        rect: Rect::new(rect[0], rect[1], rect[2], rect[3]),
        color: Color::rgba(bg_color[0], bg_color[1], bg_color[2], bg_color[3]),
        radius: 0.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    }));

    // 2. Border quad
    scene.push(DrawCommand::Quad(QuadPrimitive {
        rect: Rect::new(rect[0], rect[1], rect[2], rect[3]),
        color: Color::rgba(0.0, 0.0, 0.0, 0.0),
        radius: theme.radii.sm,
        border_width,
        border_color: Color::rgba(border_color[0], border_color[1], border_color[2], border_color[3]),
    }));
}

/// Prepare glyph draw-calls and atlas uploads from the cached text layout.
///
/// Returns `(glyphs, uploads)` suitable for a `DrawCommand::Text` primitive.
fn prepare_text_glyphs(
    state: &TextInputState,
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
) -> (Vec<GlyphDraw>, Vec<SceneAtlasUpload>) {
    let (_, cached) = state
        .cached_layout
        .as_ref()
        .expect("prepare_text_glyphs called before cached_layout was populated");

    // Prepare glyphs from cached layout
    let prepared = fonts.prepare(cached, atlas);

    // Convert prepared text to scene types
    let mut glyphs = Vec::new();
    let mut uploads = Vec::new();
    for region in &prepared.uploads {
        let pfmt = match region.format {
            acme_text::AtlasFormat::Alpha8 => GlyphFormat::Alpha8,
            acme_text::AtlasFormat::Rgba8 => GlyphFormat::Rgba8,
        };
        uploads.push(SceneAtlasUpload {
            page: 0,
            origin: [region.x, region.y],
            size: [region.width, region.height],
            format: pfmt,
            pixels: region.pixels.clone(),
        });
    }
    for glyph in &prepared.glyphs {
        let gfmt = match glyph.format {
            acme_text::AtlasFormat::Alpha8 => GlyphFormat::Alpha8,
            acme_text::AtlasFormat::Rgba8 => GlyphFormat::Rgba8,
        };
        glyphs.push(GlyphDraw {
            x: glyph.x as f32,
            y: glyph.y as f32,
            width: glyph.width as f32,
            height: glyph.height as f32,
            atlas_x: glyph.atlas_x,
            atlas_y: glyph.atlas_y,
            format: gfmt,
        });
    }
    (glyphs, uploads)
}

/// Renders a TextInput cursor and selection into a [`Scene`].
///
/// `rect` is `[x, y, width, height]` in logical pixels.
/// `clip` is an optional outer clip rectangle; the content area is further
/// clipped to the inner padding region.
#[allow(clippy::too_many_arguments)]
pub fn render_text_input(
    scene: &mut Scene,
    state: &mut TextInputState,
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

    let padding = theme.spacing.px2; // 8px vertical/horizontal inner padding
    let colors = compute_text_input_colors(theme, state, focused);
    render_text_input_background(scene, rect, colors.border_color, colors.bg_color, theme);

    // Build style from theme typography tokens
    let style = TextStyle {
        font_size: theme.typography.body,
        line_height: theme.typography.body * theme.typography.line_height,
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
    // 3. Text content (or placeholder)
    let display = state.display_text();
    let is_empty = display.is_empty();
    let has_placeholder = !state.placeholder.is_empty();

    // If text is empty and a placeholder is set, render that instead.
    let render_text = if is_empty && has_placeholder {
        &state.placeholder
    } else {
        &display
    };

    // Choose colour: placeholder gets muted; readonly gets disabled
    let render_color = if is_empty && has_placeholder {
        colors.text_muted
    } else if state.readonly {
        colors.disabled_text
    } else {
        colors.text_color
    };
    // We want the text to be vertically centered in the content area.
    // Shape to get the text height, then compute y offset.
    let constraints = TextConstraints {
        max_width: Some(content_w),
        wrap: TextWrap::None,
    };
    // Use cached layout if the render_text hasn't changed — avoids re-shaping
    // every frame for static or infrequently-changing text.
    if state
        .cached_layout
        .as_ref()
        .is_none_or(|(t, _)| t != render_text)
    {
        let new_layout = fonts.shape(render_text, &style, constraints, scale);
        state.cached_layout = Some((render_text.to_string(), new_layout));
    }
    let (_, cached) = state
        .cached_layout
        .as_ref()
        .expect("cached_layout just populated above");
    let text_height = cached.height;
    // Vertically center
    let text_y = content_y + (content_h - text_height).max(0.0) / 2.0;

    // Apply horizontal scroll offset (text scrolls left, so content_x shifts)
    let text_origin_x = content_x - state.scroll_offset;

    // Prepare glyphs from cached layout
    let (glyphs, uploads) = prepare_text_glyphs(state, fonts, atlas);

    // Push clip, then text, then pop clip
    scene.push(DrawCommand::PushClip(Rect::new(
        effective_clip[0], effective_clip[1], effective_clip[2], effective_clip[3],
    )));
    scene.push(DrawCommand::Text(TextPrimitive {
        origin: Point::new(text_origin_x, text_y),
        color: Color::rgba(render_color[0], render_color[1], render_color[2], render_color[3]),
        glyphs,
        uploads,
    }));
    scene.push(DrawCommand::PopClip);
    // 4. Selection highlight
    if let Some((sel_start, sel_end)) = state.selection {
        let clamped_start = sel_start.min(display.len());
        let clamped_end = sel_end.min(display.len());
        if clamped_start < clamped_end {
            let start_x = byte_offset_to_x_inner(&display, clamped_start, fonts, &style, scale);
            let end_x = byte_offset_to_x_inner(&display, clamped_end, fonts, &style, scale);
            let sel_w = end_x - start_x;
            if sel_w > 0.0 {
                scene.push(DrawCommand::PushClip(Rect::new(
                    effective_clip[0], effective_clip[1], effective_clip[2], effective_clip[3],
                )));
                scene.push(DrawCommand::Quad(QuadPrimitive {
                    rect: Rect::new(text_origin_x + start_x, content_y, sel_w, content_h),
                    color: Color::rgba(0.3, 0.5, 0.9, 0.25),
                    radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }));
                scene.push(DrawCommand::PopClip);
            }
        }
    }
    // 5. Cursor blink
    if focused {
        let cx = byte_offset_to_x_inner(&display, state.cursor, fonts, &style, scale);
        let cursor_x_pos = text_origin_x + cx;
        // Only draw cursor if it's within the visible content area
        if cursor_x_pos < content_x + content_w {
            scene.push(DrawCommand::PushClip(Rect::new(
                effective_clip[0], effective_clip[1], effective_clip[2], effective_clip[3],
            )));
            scene.push(DrawCommand::Quad(QuadPrimitive {
                rect: Rect::new(cursor_x_pos, content_y, 1.5, content_h),
                color: Color::rgba(colors.text_color[0], colors.text_color[1], colors.text_color[2], colors.text_color[3]),
                radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            }));
            scene.push(DrawCommand::PopClip);
        }
    }
    // 6. IME preedit underline with composition cursor
    if !state.preedit.is_empty() {
        let preedit_x = byte_offset_to_x_inner(&display, state.cursor, fonts, &style, scale);
        let preedit_layout = fonts.shape(&state.preedit, &style, TextConstraints::default(), scale);
        let preedit_w = preedit_layout.width;
        if preedit_w > 0.0 {
            let underline_y = content_y + content_h - 2.5;
            // Main underline
            scene.push(DrawCommand::PushClip(Rect::new(
                effective_clip[0], effective_clip[1], effective_clip[2], effective_clip[3],
            )));
            scene.push(DrawCommand::Quad(QuadPrimitive {
                rect: Rect::new(text_origin_x + preedit_x, underline_y, preedit_w, 1.0),
                color: Color::rgba(colors.text_color[0], colors.text_color[1], colors.text_color[2], colors.text_color[3]),
                radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            }));
            scene.push(DrawCommand::PopClip);
            // Thicker segment at the preedit cursor position, if known
            if let Some((pc_start, pc_end)) = state.preedit_cursor {
                let preedit_cx_start = byte_offset_to_x_inner(
                    &state.preedit,
                    pc_start.min(state.preedit.len()),
                    fonts,
                    &style,
                    scale,
                );
                let preedit_cx_end = byte_offset_to_x_inner(
                    &state.preedit,
                    pc_end.min(state.preedit.len()),
                    fonts,
                    &style,
                    scale,
                );
                let seg_w = (preedit_cx_end - preedit_cx_start).max(1.5);
                scene.push(DrawCommand::PushClip(Rect::new(
                    effective_clip[0], effective_clip[1], effective_clip[2], effective_clip[3],
                )));
                scene.push(DrawCommand::Quad(QuadPrimitive {
                    rect: Rect::new(
                        text_origin_x + preedit_x + preedit_cx_start,
                        underline_y - 1.0,
                        seg_w,
                        3.0,
                    ),
                    color: Color::rgba(colors.text_color[0], colors.text_color[1], colors.text_color[2], colors.text_color[3]),
                    radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }));
                scene.push(DrawCommand::PopClip);
            }
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
/// | Key                     | Action                        |
/// |-------------------------|-------------------------------|
/// | `ArrowLeft`             | `cursor_prev()`               |
/// | `ArrowRight`            | `cursor_next()`               |
/// | `Shift`+`ArrowLeft`     | Extend selection left         |
/// | `Shift`+`ArrowRight`    | Extend selection right        |
/// | `Shift`+`Home`          | Extend selection to start     |
/// | `Shift`+`End`           | Extend selection to end       |
/// | `Ctrl`+`ArrowLeft`      | Jump to previous word boundary |
/// | `Ctrl`+`ArrowRight`     | Jump to next word boundary    |
/// | `Backspace`             | `delete_before_cursor()`      |
/// | `Delete`                | `delete_after_cursor()`       |
/// | `Ctrl`+`Backspace`      | Delete previous word          |
/// | `Ctrl`+`Delete`         | Delete next word              |
/// | `Home`                  | `cursor_home()`               |
/// | `End`                   | `cursor_end()`                |
/// | `Escape`                | Blur (clear focus)            |
/// | `Tab`                   | (handled by focus manager — no-op here) |
/// | `Enter`                 | (submitted — no-op here)      |
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
    shift: bool,
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
            if ctrl {
                state.jump_word_left();
                true
            } else if shift {
                state.extend_selection_left();
                true
            } else {
                state.cursor_prev();
                true
            }
        }
        PlatformKey::ArrowRight => {
            if !state.focused {
                return false;
            }
            if ctrl {
                state.jump_word_right();
                true
            } else if shift {
                state.extend_selection_right();
                true
            } else {
                state.cursor_next();
                true
            }
        }
        PlatformKey::Backspace => {
            if !state.focused {
                return false;
            }
            if ctrl {
                state.delete_prev_word()
            } else {
                state.delete_before_cursor()
            }
        }
        PlatformKey::Delete => {
            if !state.focused {
                return false;
            }
            if ctrl {
                state.delete_next_word()
            } else {
                state.delete_after_cursor()
            }
        }
        PlatformKey::Home => {
            if !state.focused {
                return false;
            }
            if shift {
                state.extend_selection_to_start();
            } else {
                state.cursor_home();
            }
            true
        }
        PlatformKey::End => {
            if !state.focused {
                return false;
            }
            if shift {
                state.extend_selection_to_end();
            } else {
                state.cursor_end();
            }
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
            if state.readonly {
                return false;
            }
            if let Some((start, end)) = state.selection
                && start < state.text.len()
                && end <= state.text.len()
                && start < end
            {
                let old_text = state.text.clone();
                let old_cursor = state.cursor;
                let old_selection = state.selection;

                let cut = state.text[start..end].to_string();
                if let Some(clip) = clipboard {
                    let _ = clip.set_text(&cut);
                }
                state.text.drain(start..end);
                state.cursor = start;
                state.selection = None;

                state.push_undo_with_state(old_text, old_cursor, old_selection);
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
    if text.is_empty() || state.readonly {
        return false;
    }
    debug_assert!(
        state.cursor <= state.text.len(),
        "cursor {} exceeds text length {}",
        state.cursor,
        state.text.len()
    );
    let old_text = state.text.clone();
    let old_cursor = state.cursor;
    let old_selection = state.selection;

    state.replace_selection();
    if state.cursor > state.text.len() {
        state.cursor = state.text.len();
    }
    state.text.insert_str(state.cursor, text);
    state.cursor += text.len();
    state.selection = None;

    state.push_undo_with_state(old_text, old_cursor, old_selection);
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
// Word-boundary helpers (used by Ctrl+arrow and Ctrl+Backspace/Delete)
// ---------------------------------------------------------------------------

/// Return the byte offset of the word boundary immediately before `offset`.
///
/// Uses [`UnicodeSegmentation::split_word_bound_indices`].
fn prev_word_boundary(text: &str, offset: usize) -> usize {
    let clamped = offset.min(text.len());
    text.split_word_bound_indices()
        .take_while(|(i, _)| *i < clamped)
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Return the byte offset of the word boundary immediately after `offset`.
fn next_word_boundary(text: &str, offset: usize) -> usize {
    let clamped = offset.min(text.len());
    text.split_word_bound_indices()
        .find(|(i, _)| *i > clamped)
        .map(|(i, _)| i)
        .unwrap_or(text.len())
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
        assert!(handle_key(
            &mut s,
            &PlatformKey::Escape,
            true,
            None,
            false,
            false
        ));
        assert!(!s.focused);
    }

    #[test]
    fn handle_key_escape_noop_when_not_focused() {
        let mut s = TextInputState::new();
        s.focused = false;
        assert!(!handle_key(
            &mut s,
            &PlatformKey::Escape,
            true,
            None,
            false,
            false
        ));
    }

    #[test]
    fn handle_key_tab_is_noop() {
        let mut s = TextInputState::new();
        assert!(!handle_key(
            &mut s,
            &PlatformKey::Tab,
            true,
            None,
            false,
            false
        ));
    }

    #[test]
    fn handle_key_enter_is_noop() {
        let mut s = TextInputState::new();
        assert!(!handle_key(
            &mut s,
            &PlatformKey::Enter,
            true,
            None,
            false,
            false
        ));
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
            false,
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
            false,
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
            false,
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
            false,
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
            false,
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
            false,
            false
        ));
    }

    #[test]
    fn handle_key_delete_deletes_after() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 0;
        s.focused = true;
        assert!(handle_key(
            &mut s,
            &PlatformKey::Delete,
            true,
            None,
            false,
            false
        ));
        assert_eq!(s.text, "ello");
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn handle_key_home_moves_to_start() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 3;
        s.focused = true;
        assert!(handle_key(
            &mut s,
            &PlatformKey::Home,
            true,
            None,
            false,
            false
        ));
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn handle_key_end_moves_to_end() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 0;
        s.focused = true;
        assert!(handle_key(
            &mut s,
            &PlatformKey::End,
            true,
            None,
            false,
            false
        ));
        assert_eq!(s.cursor, 5);
    }

    #[test]
    fn handle_key_navigation_clears_selection() {
        let mut s = TextInputState::new();
        s.text = "hello world".to_string();
        s.cursor = 5;
        s.selection = Some((2, 8));
        s.focused = true;
        handle_key(&mut s, &PlatformKey::ArrowLeft, true, None, false, false);
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

    // ---- New: accessors ----

    #[test]
    fn accessors_return_expected_values() {
        let mut s = TextInputState::new();
        assert_eq!(s.cursor(), 0);
        assert_eq!(s.selection(), None);
        assert!(!s.has_selection());

        s.text = "hello".to_string();
        s.cursor = 3;
        assert_eq!(s.cursor(), 3);

        s.selection = Some((1, 4));
        assert_eq!(s.selection(), Some((1, 4)));
        assert!(s.has_selection());
    }

    // ---- New: readonly guard ----

    #[test]
    fn readonly_prevents_insert_char() {
        let mut s = TextInputState::new();
        s.text = "abc".to_string();
        s.cursor = 1;
        s.readonly = true;
        assert!(!s.insert_char('x'));
        assert_eq!(s.text, "abc");
    }

    #[test]
    fn readonly_prevents_delete_before() {
        let mut s = TextInputState::new();
        s.text = "abc".to_string();
        s.cursor = 2;
        s.readonly = true;
        assert!(!s.delete_before_cursor());
        assert_eq!(s.text, "abc");
    }

    #[test]
    fn readonly_prevents_delete_after() {
        let mut s = TextInputState::new();
        s.text = "abc".to_string();
        s.cursor = 1;
        s.readonly = true;
        assert!(!s.delete_after_cursor());
        assert_eq!(s.text, "abc");
    }

    #[test]
    fn readonly_prevents_handle_text() {
        let mut s = TextInputState::new();
        s.text = "abc".to_string();
        s.cursor = 1;
        s.readonly = true;
        assert!(!handle_text(&mut s, "xyz"));
        assert_eq!(s.text, "abc");
    }

    #[test]
    fn readonly_prevents_commit_preedit() {
        let mut s = TextInputState::new();
        s.text = "abc".to_string();
        s.cursor = 1;
        s.readonly = true;
        s.set_preedit("xyz", None);
        s.commit_preedit();
        assert_eq!(s.text, "abc");
    }

    #[test]
    fn readonly_prevents_keyboard_cut() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 5;
        s.selection = Some((0, 5));
        s.focused = true;
        s.readonly = true;
        // Cut returns false when readonly because the mutation is rejected
        assert!(!handle_keyboard_shortcut(&mut s, "x", None));
        assert_eq!(s.text, "hello");
    }

    // ---- New: undo / redo ----

    #[test]
    fn undo_redo_basic() {
        let mut s = TextInputState::new();
        s.insert_char('a');
        s.insert_char('b');
        s.insert_char('c');
        assert_eq!(s.text, "abc");
        assert_eq!(s.cursor, 3);

        assert!(s.undo());
        assert_eq!(s.text, "ab");
        assert_eq!(s.cursor, 2);

        assert!(s.undo());
        assert_eq!(s.text, "a");
        assert_eq!(s.cursor, 1);

        assert!(s.redo());
        assert_eq!(s.text, "ab");
        assert_eq!(s.cursor, 2);
    }

    #[test]
    fn undo_redo_empty_stacks() {
        let mut s = TextInputState::new();
        assert!(!s.undo());
        assert!(!s.redo());
    }

    #[test]
    fn undo_after_delete() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 5;
        s.delete_before_cursor();
        assert_eq!(s.text, "hell");
        s.undo();
        assert_eq!(s.text, "hello");
    }

    #[test]
    fn undo_redo_clear_history() {
        let mut s = TextInputState::new();
        s.insert_char('a');
        s.insert_char('b');
        s.clear_history();
        assert!(!s.undo());
        assert!(!s.redo());
    }

    #[test]
    fn redo_cleared_on_new_mutation() {
        let mut s = TextInputState::new();
        s.insert_char('a');
        s.undo();
        assert_eq!(s.text, "");
        // New mutation clears redo stack
        s.insert_char('b');
        assert!(!s.redo());
        assert_eq!(s.text, "b");
    }

    #[test]
    fn undo_after_redo_restores_previous_state() {
        let mut s = TextInputState::new();
        s.insert_char('a');
        assert_eq!(s.text, "a");
        assert!(s.undo());
        assert_eq!(s.text, "");
        assert!(s.redo());
        assert_eq!(s.text, "a");
        assert!(s.undo());
        assert_eq!(
            s.text,
            "",
            "undo after redo should restore pre-redo state"
        );
    }

    #[test]
    fn undo_after_redo_multiple_chars() {
        let mut s = TextInputState::new();
        handle_text(&mut s, "abc");
        assert_eq!(s.text, "abc");
        assert!(s.undo()); // abc → ""
        assert_eq!(s.text, "");
        assert!(s.redo()); // "" → abc
        assert_eq!(s.text, "abc");
        s.insert_char('d');
        assert_eq!(s.text, "abcd"); // new mutation clears redo
        assert!(!s.redo()); // redo stack empty
    }

    #[test]
    fn undo_after_handle_text() {
        let mut s = TextInputState::new();
        s.text = "ab".to_string();
        s.cursor = 1;
        handle_text(&mut s, "123");
        assert_eq!(s.text, "a123b");
        s.undo();
        assert_eq!(s.text, "ab");
        assert_eq!(s.cursor, 1);
    }

    // ---- New: mouse click/drag selection ----

    #[test]
    fn move_cursor_to_offset_basic() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 0;
        s.move_cursor_to_offset(3);
        assert_eq!(s.cursor, 3);
        assert_eq!(s.selection, None);
    }

    #[test]
    fn move_cursor_to_offset_clamps() {
        let mut s = TextInputState::new();
        s.text = "hi".to_string();
        s.move_cursor_to_offset(999);
        assert_eq!(s.cursor, 2);
    }

    #[test]
    fn move_cursor_to_offset_clears_selection() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.selection = Some((1, 4));
        s.move_cursor_to_offset(2);
        assert_eq!(s.selection, None);
    }

    #[test]
    fn set_selection_range_basic() {
        let mut s = TextInputState::new();
        s.text = "hello world".to_string();
        s.set_selection_range(2, 7);
        assert_eq!(s.selection, Some((2, 7)));
        assert_eq!(s.cursor, 7);
    }

    #[test]
    fn set_selection_range_clamps() {
        let mut s = TextInputState::new();
        s.text = "hi".to_string();
        s.set_selection_range(0, 999);
        assert_eq!(s.selection, Some((0, 2)));
        assert_eq!(s.cursor, 2);
    }

    #[test]
    fn set_selection_range_empty_when_start_equals_end() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.set_selection_range(3, 3);
        assert_eq!(s.selection, None);
    }

    #[test]
    fn select_word_at_offset_double_click() {
        let mut s = TextInputState::new();
        s.text = "hello world foo".to_string();
        s.select_word_at_offset(7); // inside "world"
        assert_eq!(s.selection, Some((6, 11)));
        assert_eq!(s.cursor, 11);
    }

    #[test]
    fn select_word_at_offset_empty_text() {
        let mut s = TextInputState::new();
        s.select_word_at_offset(0);
        assert_eq!(s.selection, None);
        assert_eq!(s.cursor, 0);
    }

    // ---- New: extended keyboard shortcuts ----

    #[test]
    fn shift_arrow_left_extends_selection() {
        let mut s = TextInputState::new();
        s.text = "hello world".to_string();
        s.cursor = 5;
        s.focused = true;
        handle_key(&mut s, &PlatformKey::ArrowLeft, true, None, false, true);
        assert_eq!(s.selection, Some((4, 5)));
    }

    #[test]
    fn shift_arrow_right_extends_selection() {
        let mut s = TextInputState::new();
        s.text = "hello world".to_string();
        s.cursor = 5;
        s.focused = true;
        handle_key(&mut s, &PlatformKey::ArrowRight, true, None, false, true);
        assert_eq!(s.selection, Some((5, 6)));
    }

    #[test]
    fn shift_home_extends_to_start() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 3;
        s.focused = true;
        handle_key(&mut s, &PlatformKey::Home, true, None, false, true);
        assert_eq!(s.selection, Some((0, 3)));
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn shift_end_extends_to_end() {
        let mut s = TextInputState::new();
        s.text = "hello".to_string();
        s.cursor = 2;
        s.focused = true;
        handle_key(&mut s, &PlatformKey::End, true, None, false, true);
        assert_eq!(s.selection, Some((2, 5)));
        assert_eq!(s.cursor, 5);
    }

    #[test]
    fn ctrl_arrow_left_jumps_word() {
        let mut s = TextInputState::new();
        s.text = "hello world foo".to_string();
        s.cursor = 15;
        s.focused = true;
        handle_key(&mut s, &PlatformKey::ArrowLeft, true, None, true, false);
        // Should jump from end to start of "foo"
        assert_eq!(s.cursor, 12); // "foo" starts at 12
    }

    #[test]
    fn ctrl_arrow_right_jumps_word() {
        let mut s = TextInputState::new();
        s.text = "hello world".to_string();
        s.cursor = 0;
        s.focused = true;
        handle_key(&mut s, &PlatformKey::ArrowRight, true, None, true, false);
        // Should jump from start to end of first word "hello"
        assert_eq!(s.cursor, 5);
    }

    #[test]
    fn ctrl_backspace_deletes_prev_word() {
        let mut s = TextInputState::new();
        s.text = "hello world".to_string();
        s.cursor = 11;
        s.focused = true;
        handle_key(&mut s, &PlatformKey::Backspace, true, None, true, false);
        assert_eq!(s.text, "hello ");
        assert_eq!(s.cursor, 6);
    }

    #[test]
    fn ctrl_delete_deletes_next_word() {
        let mut s = TextInputState::new();
        s.text = "hello world".to_string();
        s.cursor = 0;
        s.focused = true;
        handle_key(&mut s, &PlatformKey::Delete, true, None, true, false);
        assert_eq!(s.text, " world");
        assert_eq!(s.cursor, 0);
    }

    // ---- New: placeholder state ----

    #[test]
    fn placeholder_default_is_empty() {
        let s = TextInputState::new();
        assert_eq!(s.placeholder, "");
    }

    #[test]
    fn invalid_default_is_false() {
        let s = TextInputState::new();
        assert!(!s.invalid);
    }

    #[test]
    fn scroll_offset_default_is_zero() {
        let s = TextInputState::new();
        assert_eq!(s.scroll_offset, 0.0);
    }

    // ---- New: word boundary helpers ----

    #[test]
    fn prev_word_boundary_empty() {
        assert_eq!(prev_word_boundary("", 0), 0);
    }

    #[test]
    fn next_word_boundary_empty() {
        assert_eq!(next_word_boundary("", 0), 0);
    }

    #[test]
    fn prev_word_boundary_works() {
        // "hello world" — split_word_bound_indices gives:
        //   (0, "hello"), (5, " "), (6, "world")
        // So the previous word boundary from 11 is 6 (start of "world"),
        // from 6 it's 5 (the space boundary), from 0 it's 0.
        assert_eq!(prev_word_boundary("hello world", 11), 6);
        assert_eq!(prev_word_boundary("hello world", 6), 5);
        assert_eq!(prev_word_boundary("hello world", 0), 0);
    }

    #[test]
    fn next_word_boundary_works() {
        assert_eq!(next_word_boundary("hello world", 0), 5);
        assert_eq!(next_word_boundary("hello world", 6), 11);
        assert_eq!(next_word_boundary("hello world", 11), 11);
    }

    // ---- IME caret area geometry ----

    #[test]
    fn ime_caret_area_empty_near_origin() {
        let s = TextInputState::new();
        let mut fonts = FontSystem::new();
        let style = TextStyle::default();
        let [x, y, w, h] = s.ime_caret_area(&mut fonts, &style, 1.0);
        assert!(x.abs() < 0.5, "empty caret x should be near 0, got {x}");
        assert_eq!(y, 0.0);
        assert!((w - 1.5).abs() < f32::EPSILON);
        assert!((h - style.line_height).abs() < f32::EPSILON);
    }

    #[test]
    fn ime_caret_area_x_increases_after_ascii_insert() {
        let mut s = TextInputState::new();
        let mut fonts = FontSystem::new();
        let style = TextStyle::default();
        let [x0, _, _, _] = s.ime_caret_area(&mut fonts, &style, 1.0);
        for ch in ['a', 'b', 'c', 'd'] {
            assert!(s.insert_char(ch));
        }
        let [x1, _, _, h] = s.ime_caret_area(&mut fonts, &style, 1.0);
        assert!(
            x1 > x0 + 1.0,
            "caret x should advance after ASCII insert: x0={x0}, x1={x1}"
        );
        assert!((h - style.line_height).abs() < f32::EPSILON);
    }

    #[test]
    fn ime_caret_area_accounts_for_scroll_offset() {
        let mut s = TextInputState::new();
        let mut fonts = FontSystem::new();
        let style = TextStyle::default();
        for ch in "hello world".chars() {
            assert!(s.insert_char(ch));
        }
        let [x_unscrolled, _, _, _] = s.ime_caret_area(&mut fonts, &style, 1.0);
        s.scroll_offset = 12.5;
        let [x_scrolled, _, _, _] = s.ime_caret_area(&mut fonts, &style, 1.0);
        assert!(
            (x_unscrolled - x_scrolled - 12.5).abs() < 0.01,
            "scroll_offset must reduce caret x: unscrolled={x_unscrolled}, scrolled={x_scrolled}"
        );
    }

    #[test]
    fn ime_caret_area_height_from_style_line_height() {
        let s = TextInputState::new();
        let mut fonts = FontSystem::new();
        let style = TextStyle {
            font_size: 18.0,
            line_height: 27.0,
            ..TextStyle::default()
        };
        let [_, _, _, h] = s.ime_caret_area(&mut fonts, &style, 1.0);
        assert!((h - 27.0).abs() < f32::EPSILON);
    }
}
