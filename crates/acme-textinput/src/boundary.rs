//! Grapheme and word boundary helpers.

use super::*;

/// Return the byte offset of the grapheme cluster boundary immediately before
/// `offset`.  If `offset` is already at a boundary, returns the previous one.
pub(crate) fn prev_grapheme_boundary(text: &str, offset: usize) -> usize {
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
pub(crate) fn next_grapheme_boundary(text: &str, offset: usize) -> usize {
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
pub(crate) fn prev_word_boundary(text: &str, offset: usize) -> usize {
    let clamped = offset.min(text.len());
    text.split_word_bound_indices()
        .take_while(|(i, _)| *i < clamped)
        .last()
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Return the byte offset of the word boundary immediately after `offset`.
pub(crate) fn next_word_boundary(text: &str, offset: usize) -> usize {
    let clamped = offset.min(text.len());
    text.split_word_bound_indices()
        .find(|(i, _)| *i > clamped)
        .map(|(i, _)| i)
        .unwrap_or(text.len())
}
