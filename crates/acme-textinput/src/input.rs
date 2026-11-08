//! Keyboard, shortcut and text-commit handling.

use super::*;

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
/// | `Escape`                | Cancel preedit, else blur     |
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
            if state.focused && !state.preedit.is_empty() {
                // First Escape cancels the IME composition and keeps focus.
                state.cancel_preedit();
                true
            } else if state.focused {
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
        PlatformKey::Space
        | PlatformKey::ArrowUp
        | PlatformKey::ArrowDown
        | PlatformKey::PageUp
        | PlatformKey::PageDown
        | PlatformKey::Insert
        | PlatformKey::Function(_)
        | PlatformKey::Other => {
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

/// Handle clipboard and edit-history shortcuts for a text input.
///
/// Call this from the application event loop when `PlatformEvent::Key` has
/// `ctrl: true` and `text` is `Some("a")`, `Some("c")`, `Some("v")`, or
/// `Some("x")`, `Some("z")` (undo), or `Some("y")` (redo).
/// Callers should invoke this only on key press, and map Ctrl+Shift+Z to `"y"`.
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
        "z" => state.undo(),
        "y" => state.redo(),
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
    // Committed text ends any IME composition; never leave a stale preedit drawn.
    state.cancel_preedit();

    state.push_undo_with_state(old_text, old_cursor, old_selection);
    true
}
