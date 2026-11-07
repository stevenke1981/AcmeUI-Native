//! Accessors, readonly, undo/redo, mouse selection, extended shortcuts, placeholder.

use super::*;

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
fn readonly_history_operations_preserve_state_and_history() {
    let mut s = TextInputState::new();
    handle_text(&mut s, "hello");
    s.set_selection_range(1, 4);
    handle_text(&mut s, "i");
    s.set_preedit("pending", None);
    s.readonly = true;
    for _ in 0..2 {
        assert!(!s.undo());
        assert_eq!(s.text, "hio");
        assert_eq!(s.cursor(), 2);
        assert_eq!(s.selection(), None);
        assert_eq!(s.preedit, "pending");
    }
    s.readonly = false;
    assert!(s.undo());
    assert_eq!(s.text, "hello");
    assert_eq!(s.selection(), Some((1, 4)));
    s.set_preedit("pending", None);
    s.readonly = true;
    for _ in 0..2 {
        assert!(!s.redo());
        assert_eq!(s.text, "hello");
        assert_eq!(s.cursor(), 4);
        assert_eq!(s.selection(), Some((1, 4)));
        assert_eq!(s.preedit, "pending");
    }
    s.readonly = false;
    assert!(s.redo());
    assert_eq!(s.text, "hio");
    assert_eq!(s.cursor(), 2);
}

#[test]
fn ctrl_delete_selection_is_one_undoable_edit() {
    for key in [PlatformKey::Backspace, PlatformKey::Delete] {
        let mut s = TextInputState::new();
        s.focused = true;
        handle_text(&mut s, "hello world");
        s.set_selection_range(0, 5);
        assert!(handle_key(&mut s, &key, true, None, true, false));
        assert_eq!(s.text, " world");
        assert!(s.undo());
        assert_eq!(s.text, "hello world");
        assert_eq!(s.selection(), Some((0, 5)));
        assert_eq!(s.cursor(), 5);
        assert!(s.redo());
        assert_eq!(s.text, " world");
        assert_eq!(s.selection(), None);
        assert_eq!(s.cursor(), 0);
        assert!(s.undo());
        assert!(s.undo());
        assert_eq!(s.text, "");
    }
}

#[test]
fn ctrl_delete_selection_discards_old_redo_branch() {
    for key in [PlatformKey::Backspace, PlatformKey::Delete] {
        let mut s = TextInputState::new();
        s.focused = true;
        handle_text(&mut s, "hello");
        handle_text(&mut s, " world");
        assert!(s.undo());
        s.select_all();
        assert!(handle_key(&mut s, &key, true, None, true, false));
        assert_eq!(s.text, "");
        assert!(!s.redo());
        assert!(s.undo());
        assert_eq!(s.text, "hello");
    }
}

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
    assert_eq!(s.text, "", "undo after redo should restore pre-redo state");
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
