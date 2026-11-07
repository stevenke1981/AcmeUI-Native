//! handle_text, handle_key, shortcuts and clipboard.

use super::*;

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
    assert!(!handle_keyboard_shortcut(&mut s, "q", None));
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
