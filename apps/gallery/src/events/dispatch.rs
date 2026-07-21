//! Per-event-type dispatch functions (Layer 3).
//!
//! Each handler takes explicit mutable/immutable references to the
//! specific `Gallery` fields it needs — enabling Rust field-level borrow
//! checking instead of a monolithic `&mut self`.
//!
//! Architecture:
//!
//!   ┌────────────────────────────────────────────┐
//!   │  main.rs — event() match on PlatformEvent  │  Layer 3
//!   ├────────────────────────────────────────────┤     Event Dispatch
//!   │  dispatch.rs — per-event-type handlers      │
//!   ├────────────────────────────────────────────┤
//!   │  activate.rs — ActivationCtx + handle_msg  │  Layer 2
//!   │  ime.rs — compute_ime_caret_rect           │     State Transition
//!   ├────────────────────────────────────────────┤
//!   │  hit.rs — hit_test (pure query)            │  Layer 1
//!   └────────────────────────────────────────────┘     Query

use acme_platform::PlatformKey;
use acme_textinput::TextInputState;

use crate::events::{compute_ime_caret_rect, hit_test, ActivationCtx, handle_message};
use crate::render::{point_in_rect, scrolled_hit_rect};
use crate::types::*;

// ── Pointer ─────────────────────────────────────────────────────────────────

/// Handle `PointerMoved` — update cursor + hovered.
pub fn handle_pointer_moved(
    cursor: &mut (f32, f32),
    hovered: &mut Option<usize>,
    x: f32,
    y: f32,
    button_info: &[HitRegion],
    scroll: f32,
    scroll_clip_rect: [f32; 4],
) -> bool {
    *cursor = (x, y);
    let next = hit_test(button_info, *cursor, scroll, scroll_clip_rect);
    let changed = next != *hovered;
    *hovered = next;
    changed
}

/// Handle `PointerButton { pressed: true }` — text-input focus + hit test.
#[allow(clippy::too_many_arguments)]
pub fn handle_pointer_pressed(
    pressed: &mut Option<usize>,
    text_input: &mut TextInputState,
    text_input_rect: [f32; 4],
    cursor: (f32, f32),
    button_info: &[HitRegion],
    scroll: f32,
    scroll_clip_rect: [f32; 4],
    dark: bool,
    fonts: &mut acme_text::FontSystem,
    last_scale_factor: f32,
    ime_caret_window_rect: &mut Option<[f32; 4]>,
) -> bool {
    let in_text = {
        let [tx, ty, tw, th] = text_input_rect;
        cursor.0 >= tx && cursor.0 <= tx + tw && cursor.1 >= ty && cursor.1 <= ty + th
    };
    text_input.focused = in_text;
    *ime_caret_window_rect =
        compute_ime_caret_rect(text_input, text_input_rect, dark, fonts, last_scale_factor);
    *pressed = hit_test(button_info, cursor, scroll, scroll_clip_rect);
    true
}

/// Handle `PointerButton { pressed: false }` — activation dispatch.
pub fn handle_pointer_released(
    pressed: &mut Option<usize>,
    cursor: (f32, f32),
    button_info: &[HitRegion],
    scroll: f32,
    scroll_clip_rect: [f32; 4],
    mut act_ctx: ActivationCtx,
) -> bool {
    let pressed_was = pressed.take();
    let activated = pressed_was
        .filter(|&value| Some(value) == hit_test(button_info, cursor, scroll, scroll_clip_rect));
    activated.is_some_and(|index| {
        let message = button_info[index].message;
        handle_message(&mut act_ctx, message)
    })
}

// ── Scroll ───────────────────────────────────────────────────────────────────

/// Handle `Scroll` — virtual-list or content scroll.
#[allow(clippy::too_many_arguments)]
pub fn handle_scroll_event(
    selected_category: usize,
    selected_page: usize,
    cursor: (f32, f32),
    vlist_viewport_rect: [f32; 4],
    scroll: f32,
    max_scroll: f32,
    vlist_scroll: &mut f32,
    gallery_scroll: &mut f32,
    delta_y: f32,
) -> bool {
    let vlist_screen = scrolled_hit_rect(vlist_viewport_rect, scroll);
    if selected_category == 4
        && selected_page == 3
        && vlist_viewport_rect[2] > 0.0
        && point_in_rect(cursor.0, cursor.1, vlist_screen)
    {
        *vlist_scroll = (*vlist_scroll - delta_y).clamp(0.0, crate::helpers::vlist_max_scroll());
        return true;
    }
    *gallery_scroll = (*gallery_scroll - delta_y).clamp(0.0, max_scroll);
    true
}

// ── Keys ─────────────────────────────────────────────────────────────────────

/// Handle `Tab` key — blur text-input / cycle focused button.
#[allow(clippy::too_many_arguments)]
pub fn handle_tab_key(
    focused: &mut usize,
    text_input: &mut TextInputState,
    text_input_rect: [f32; 4],
    dark: bool,
    fonts: &mut acme_text::FontSystem,
    last_scale_factor: f32,
    ime_caret_window_rect: &mut Option<[f32; 4]>,
    button_info: &[HitRegion],
    shift: bool,
) -> bool {
    if text_input.focused {
        text_input.focused = false;
        *ime_caret_window_rect =
            compute_ime_caret_rect(text_input, text_input_rect, dark, fonts, last_scale_factor);
    }
    let count = button_info.len();
    if count > 0 {
        *focused = if shift {
            (*focused + count - 1) % count
        } else {
            (*focused + 1) % count
        };
    }
    true
}

/// Handle `Enter` / `Space` — activate focused button.
pub fn handle_enter_space_key(
    text_input: &TextInputState,
    button_info: &[HitRegion],
    focused: usize,
    mut act_ctx: ActivationCtx,
) -> bool {
    if text_input.focused {
        false
    } else {
        let message = button_info[focused].message;
        handle_message(&mut act_ctx, message)
    }
}

/// Handle tree-navigation arrow keys (Data > Tree page).
pub fn handle_tree_arrow_key(
    key: &PlatformKey,
    tree_selected: &mut Option<&'static str>,
    tree_expanded: &mut u32,
) -> bool {
    match key {
        PlatformKey::ArrowRight => {
            if let Some(sel) = *tree_selected {
                *tree_expanded = crate::helpers::tree_set_expanded(*tree_expanded, sel, true);
            }
            true
        }
        PlatformKey::ArrowLeft => {
            if let Some(sel) = *tree_selected {
                *tree_expanded = crate::helpers::tree_set_expanded(*tree_expanded, sel, false);
            }
            true
        }
        PlatformKey::Home => {
            *tree_selected = Some("docs");
            true
        }
        PlatformKey::End => {
            *tree_selected = if crate::helpers::tree_is_expanded(*tree_expanded, "code") {
                if crate::helpers::tree_is_expanded(*tree_expanded, "code_src") {
                    Some("code_lib")
                } else {
                    Some("code_toml")
                }
            } else {
                Some("code")
            };
            true
        }
        _ => false,
    }
}

/// Handle general key events for text input (Ctrl+C/V, navigation, etc.).
#[allow(clippy::too_many_arguments)]
pub fn handle_general_key(
    text_input: &mut TextInputState,
    key: &PlatformKey,
    pressed: bool,
    ctrl: bool,
    shift: bool,
    text: Option<&str>,
    clipboard: &Option<acme_platform::Clipboard>,
    text_input_rect: [f32; 4],
    dark: bool,
    fonts: &mut acme_text::FontSystem,
    last_scale_factor: f32,
    ime_caret_window_rect: &mut Option<[f32; 4]>,
) -> bool {
    if !text_input.focused {
        return false;
    }
    let changed = if ctrl && let Some(t) = text
        && matches!(t, "a" | "c" | "v" | "x")
    {
        acme_textinput::handle_keyboard_shortcut(text_input, t, clipboard.as_ref())
    } else {
        acme_textinput::handle_key(text_input, key, pressed, clipboard.as_ref(), ctrl, shift)
    };
    if changed {
        *ime_caret_window_rect =
            compute_ime_caret_rect(text_input, text_input_rect, dark, fonts, last_scale_factor);
    }
    changed
}

// ── IME ──────────────────────────────────────────────────────────────────────

/// Handle `ImePreedit` — set preedit string and refresh caret.
pub fn handle_ime_preedit(
    text: &str,
    text_input: &mut TextInputState,
    text_input_rect: [f32; 4],
    dark: bool,
    fonts: &mut acme_text::FontSystem,
    last_scale_factor: f32,
    ime_caret_window_rect: &mut Option<[f32; 4]>,
) -> bool {
    text_input.set_preedit(text, None);
    *ime_caret_window_rect =
        compute_ime_caret_rect(text_input, text_input_rect, dark, fonts, last_scale_factor);
    true
}

/// Handle `ImeCommit` — commit text and refresh caret.
#[allow(clippy::too_many_arguments)]
pub fn handle_ime_commit(
    text: &str,
    text_input: &mut TextInputState,
    ime_text: &mut String,
    text_input_rect: [f32; 4],
    dark: bool,
    fonts: &mut acme_text::FontSystem,
    last_scale_factor: f32,
    ime_caret_window_rect: &mut Option<[f32; 4]>,
) -> bool {
    acme_textinput::handle_text(text_input, text);
    *ime_text = text_input.text.clone();
    *ime_caret_window_rect =
        compute_ime_caret_rect(text_input, text_input_rect, dark, fonts, last_scale_factor);
    true
}
