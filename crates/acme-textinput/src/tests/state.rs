//! TextInputState editing, cursor, selection, preedit and masking.

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
