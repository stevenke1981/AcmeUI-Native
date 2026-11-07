//! Word boundary helpers and IME caret area geometry.

use super::*;

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
