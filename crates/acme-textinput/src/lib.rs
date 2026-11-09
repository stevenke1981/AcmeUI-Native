//! TextInput widget for AcmeUI Native.
//!
//! Provides [`TextInputState`] for managing editable text state (cursor, selection,
//! IME preedit, password masking) and functions for rendering, keyboard handling,
//! and text insertion.
#![forbid(unsafe_op_in_unsafe_fn)]

use acme_core::{
    AtlasUpload as SceneAtlasUpload, Color, DrawCommand, GlyphDraw, GlyphFormat, Point,
    QuadPrimitive, Rect, Scene, TextPrimitive,
};
use acme_platform::{Clipboard, PlatformKey};
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextLayout, TextStyle, TextWrap};
use acme_theme::{Theme, ThemeColor};
use unicode_segmentation::UnicodeSegmentation;

mod boundary;
mod caret;
mod input;
mod preedit;
mod render;

use boundary::*;
pub use caret::*;
pub use input::*;
pub use render::*;

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
    /// Cached shaped text layout, invalidated when shaping inputs change.
    /// The key includes display/placeholder text, typography, constraints and DPI.
    /// Set to `None` initially; populated on first render.
    pub(crate) cached_layout: Option<(InputLayoutKey, TextLayout)>,
}

/// All inputs that affect cached shaping, including physical glyph scale.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct InputLayoutKey {
    text: String,
    style: TextStyle,
    constraints: TextConstraints,
    scale: f32,
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
    /// Readonly inputs leave both state and history unchanged.
    pub fn undo(&mut self) -> bool {
        if self.readonly {
            return false;
        }
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
    /// Readonly inputs leave both state and history unchanged.
    pub fn redo(&mut self) -> bool {
        if self.readonly {
            return false;
        }
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
        let clamped = byte_offset.min(self.text.len());
        // Snap down to a grapheme boundary: a caret inside a character (or a
        // combining sequence) would make later slicing panic.
        self.cursor = self
            .text
            .grapheme_indices(true)
            .map(|(start, _)| start)
            .chain(std::iter::once(self.text.len()))
            .take_while(|&start| start <= clamped)
            .last()
            .unwrap_or(0);
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
        let raw_x = byte_offset_to_x(&display, self.display_cursor(), fonts, style, scale);
        // Match render_text_input: text_origin_x = content_x - scroll_offset.
        let x = raw_x - self.scroll_offset;
        let h = style.line_height;
        [x, 0.0, CARET_WIDTH, h]
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
            return self.delete_before_cursor();
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
            return self.delete_after_cursor();
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
        byte_offset_to_x(&display, self.display_cursor(), fonts, style, scale)
    }

    /// The cursor as a byte offset into [`display_text`](Self::display_text).
    ///
    /// Equal to the cursor for plain text. For password inputs every grapheme
    /// is shown as one `password_char`, so the offset is re-counted in mask
    /// characters; using the raw cursor there sliced inside a mask character.
    fn display_cursor(&self) -> usize {
        let cursor = floor_char_boundary(&self.text, self.cursor);
        if self.password {
            self.text[..cursor].graphemes(true).count() * self.password_char.len_utf8()
        } else {
            cursor
        }
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

#[cfg(test)]
mod tests;

#[cfg(test)]
mod render_acceptance_tests;
