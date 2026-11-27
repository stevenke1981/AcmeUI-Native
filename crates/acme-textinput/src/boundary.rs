//! Grapheme and word boundary helpers.

use unicode_segmentation::{GraphemeCursor, UnicodeSegmentation};

/// Return the byte offset of the grapheme cluster boundary immediately before
/// `offset`.  If `offset` is already at a boundary, returns the previous one.
pub(crate) fn prev_grapheme_boundary(text: &str, offset: usize) -> usize {
    // Round up so the start of a scalar containing `offset` is still a candidate.
    let mut start = offset.min(text.len());
    while !text.is_char_boundary(start) {
        start += 1;
    }
    GraphemeCursor::new(start, text.len(), true)
        .prev_boundary(text, 0)
        .expect("the complete text provides all grapheme context")
        .unwrap_or(0)
}

/// Return the byte offset of the grapheme cluster boundary immediately after
/// `offset`.
pub(crate) fn next_grapheme_boundary(text: &str, offset: usize) -> usize {
    // Round down so the end of a scalar containing `offset` is still a candidate.
    let mut start = offset.min(text.len());
    while !text.is_char_boundary(start) {
        start -= 1;
    }
    GraphemeCursor::new(start, text.len(), true)
        .next_boundary(text, 0)
        .expect("the complete text provides all grapheme context")
        .unwrap_or(text.len())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grapheme_neighbors_match_full_segmentation_at_every_byte() {
        for text in [
            "",
            "ASCII\r\ntext\n",
            "繁體中文 a\u{0301}\u{0327} é",
            "\u{0600}a 한",
            "🇦🇧🇨🇩🇪 🇫🇬🇭🇮🇯🇰",
            "a👩🏽\u{200d}🚀👨\u{200d}👩\u{200d}👧z",
            "क्\u{200d}षि क्ष्क कष",
        ] {
            let boundaries: Vec<_> = text
                .grapheme_indices(true)
                .map(|(boundary, _)| boundary)
                .chain(std::iter::once(text.len()))
                .collect();
            for offset in (0..=text.len() + 1).chain(std::iter::once(usize::MAX)) {
                let clamped = offset.min(text.len());
                let previous = boundaries.iter().copied().rfind(|&b| b < clamped);
                let next = boundaries.iter().copied().find(|&b| b > clamped);
                assert_eq!(
                    prev_grapheme_boundary(text, offset),
                    previous.unwrap_or(0),
                    "previous boundary in {text:?} at {offset}"
                );
                assert_eq!(
                    next_grapheme_boundary(text, offset),
                    next.unwrap_or(text.len()),
                    "next boundary in {text:?} at {offset}"
                );
            }
        }
    }
}
