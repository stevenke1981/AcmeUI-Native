//! ROADMAP 005 E1: `scroll_caret_into_view` keeps the caret inside the visible
//! text area. Uses only the public API of `acme-textinput` and `acme-text`.
//! Widths are logical pixels; the scale argument only affects raster placement.

use acme_text::{FontSystem, TextStyle};
use acme_textinput::{TextInputState, handle_text, scroll_caret_into_view};

const CONTENT_W: f32 = 120.0;
const SCALES: [f32; 4] = [1.0, 1.25, 1.5, 2.0];

fn style() -> TextStyle {
    TextStyle {
        font_size: 16.0,
        line_height: 24.0,
        ..TextStyle::default()
    }
}

/// Content-local caret x, as the Gallery reports it (`caret_x - scroll_offset`).
fn local_caret_x(state: &TextInputState, fonts: &mut FontSystem, scale: f32) -> f32 {
    state.ime_caret_area(fonts, &style(), scale)[0]
}

fn follow(state: &mut TextInputState, fonts: &mut FontSystem, scale: f32) {
    scroll_caret_into_view(state, fonts, &style(), scale, CONTENT_W);
}

fn assert_caret_visible(state: &TextInputState, fonts: &mut FontSystem, scale: f32, ctx: &str) {
    let x = local_caret_x(state, fonts, scale);
    assert!(x.is_finite(), "caret x not finite {x} {ctx}");
    assert!(
        (-0.001..=CONTENT_W + 0.001).contains(&x),
        "caret x {x} outside [0, {CONTENT_W}] {ctx}"
    );
    assert!(
        state.scroll_offset >= 0.0,
        "scroll_offset negative {} {ctx}",
        state.scroll_offset
    );
}

#[test]
fn long_text_keeps_caret_visible_at_the_end() {
    let mut fonts = FontSystem::new();
    for scale in SCALES {
        let mut state = TextInputState::new();
        state.focused = true;
        handle_text(&mut state, &"w".repeat(80));
        follow(&mut state, &mut fonts, scale);
        assert!(
            state.scroll_offset > 0.0,
            "overflowing text must scroll at scale {scale}"
        );
        assert_caret_visible(&state, &mut fonts, scale, &format!("scale {scale}"));
    }
}

#[test]
fn short_text_does_not_scroll() {
    let mut fonts = FontSystem::new();
    let mut state = TextInputState::new();
    handle_text(&mut state, "abc");
    follow(&mut state, &mut fonts, 1.0);
    assert_eq!(state.scroll_offset, 0.0);
}

#[test]
fn moving_home_scrolls_back_to_zero() {
    let mut fonts = FontSystem::new();
    let mut state = TextInputState::new();
    handle_text(&mut state, &"w".repeat(80));
    follow(&mut state, &mut fonts, 1.0);
    assert!(state.scroll_offset > 0.0);
    state.cursor_home();
    follow(&mut state, &mut fonts, 1.0);
    assert_eq!(state.scroll_offset, 0.0, "Home must reveal the start");
    assert_caret_visible(&state, &mut fonts, 1.0, "after Home");
}

#[test]
fn moving_left_across_the_field_follows_the_caret() {
    let mut fonts = FontSystem::new();
    let mut state = TextInputState::new();
    handle_text(&mut state, &"w".repeat(80));
    follow(&mut state, &mut fonts, 1.0);
    let start_scroll = state.scroll_offset;
    for step in 0..60 {
        state.cursor_prev();
        follow(&mut state, &mut fonts, 1.0);
        assert_caret_visible(&state, &mut fonts, 1.0, &format!("left step {step}"));
    }
    assert!(
        state.scroll_offset < start_scroll,
        "moving left must scroll back"
    );
}

#[test]
fn deleting_text_reduces_scroll() {
    let mut fonts = FontSystem::new();
    let mut state = TextInputState::new();
    handle_text(&mut state, &"w".repeat(80));
    follow(&mut state, &mut fonts, 1.0);
    let before = state.scroll_offset;
    for _ in 0..20 {
        assert!(state.delete_before_cursor());
    }
    follow(&mut state, &mut fonts, 1.0);
    assert!(
        state.scroll_offset < before,
        "scroll must shrink after deleting: before {before}, after {}",
        state.scroll_offset
    );
    assert_caret_visible(&state, &mut fonts, 1.0, "after partial delete");
}

#[test]
fn deleting_everything_returns_scroll_to_zero() {
    let mut fonts = FontSystem::new();
    let mut state = TextInputState::new();
    handle_text(&mut state, &"w".repeat(80));
    follow(&mut state, &mut fonts, 1.0);
    assert!(state.scroll_offset > 0.0);
    state.select_all();
    assert!(state.delete_before_cursor());
    follow(&mut state, &mut fonts, 1.0);
    assert_eq!(state.scroll_offset, 0.0);
}

#[test]
fn cjk_text_keeps_caret_visible_while_typing_and_moving() {
    let mut fonts = FontSystem::new();
    for scale in SCALES {
        let mut state = TextInputState::new();
        state.focused = true;
        for _ in 0..12 {
            handle_text(&mut state, "注音輸入");
            follow(&mut state, &mut fonts, scale);
            assert_caret_visible(&state, &mut fonts, scale, &format!("typing at {scale}"));
        }
        assert!(state.scroll_offset > 0.0, "CJK overflow must scroll");
        for step in 0..30 {
            state.cursor_prev();
            follow(&mut state, &mut fonts, scale);
            assert_caret_visible(
                &state,
                &mut fonts,
                scale,
                &format!("moving at {scale} step {step}"),
            );
        }
    }
}

#[test]
fn mixed_cjk_and_ascii_has_byte_and_char_counts_that_differ() {
    // Cursors move by grapheme and the helper measures by cursor offset, so a
    // string whose byte and char counts differ must still keep the caret in view.
    let mut fonts = FontSystem::new();
    let mut state = TextInputState::new();
    handle_text(&mut state, "注音輸入 abc 注音輸入 abc 注音輸入 abc");
    assert_ne!(state.text.len(), state.text.chars().count());
    follow(&mut state, &mut fonts, 1.5);
    assert_caret_visible(&state, &mut fonts, 1.5, "mixed text");
}

#[test]
fn zero_and_non_finite_content_width_do_not_panic_or_go_negative() {
    let mut fonts = FontSystem::new();
    let mut state = TextInputState::new();
    handle_text(&mut state, &"w".repeat(40));
    for width in [0.0, -5.0, f32::NAN, f32::INFINITY] {
        scroll_caret_into_view(&mut state, &mut fonts, &style(), 1.0, width);
        assert!(state.scroll_offset.is_finite(), "width {width}");
        assert!(state.scroll_offset >= 0.0, "width {width}");
    }
    // With zero content width the caret must sit at the content origin.
    scroll_caret_into_view(&mut state, &mut fonts, &style(), 1.0, 0.0);
    assert!(local_caret_x(&state, &mut fonts, 1.0).abs() < 0.001);
}

#[test]
fn empty_text_keeps_scroll_at_zero() {
    let mut fonts = FontSystem::new();
    let mut state = TextInputState::new();
    state.scroll_offset = 30.0;
    follow(&mut state, &mut fonts, 1.0);
    assert_eq!(state.scroll_offset, 0.0);
}

// ---- Review regressions (ROADMAP 005 re-review) ----

/// The drawn caret (a CARET-width quad) must exist and lie inside the field
/// after scrolling to the end of overflowing text. Before the fix the helper
/// parked the caret on the right edge and render_text_input skipped it.
#[test]
fn caret_is_drawn_inside_the_field_at_the_end_of_overflowing_text() {
    use acme_core::{DrawCommand, Scene};
    use acme_text::GlyphAtlas;
    use acme_theme::Theme;

    let theme = Theme::light();
    let field = [0.0_f32, 0.0, 200.0, 40.0];
    for scale in SCALES {
        let mut fonts = FontSystem::new();
        let mut atlas = GlyphAtlas::new(1024, 1024);
        let mut state = TextInputState::new();
        state.focused = true;
        handle_text(&mut state, &"w".repeat(60));
        for _ in 0..2 {
            let mut scene = Scene::default();
            acme_textinput::render_text_input(
                &mut scene, &mut state, &mut fonts, &mut atlas, field, &theme, scale, true, None,
            );
            let carets: Vec<_> = scene
                .commands()
                .iter()
                .filter_map(|command| match command {
                    DrawCommand::Quad(quad)
                        if (quad.rect.size.width.get() - 1.5).abs() < f32::EPSILON =>
                    {
                        Some(quad.rect)
                    }
                    _ => None,
                })
                .collect();
            if state.scroll_offset > 0.0 {
                let caret = carets
                    .first()
                    .unwrap_or_else(|| panic!("no caret drawn at scale {scale}"));
                let right = caret.origin.x.get() + caret.size.width.get();
                assert!(
                    right <= field[0] + field[2],
                    "caret right {right} outside field at scale {scale}"
                );
            }
            // Same style and content width render_text_input uses.
            let field_style = TextStyle {
                font_size: theme.typography.body,
                line_height: theme.typography.body * theme.typography.line_height,
                ..TextStyle::default()
            };
            let content_width = field[2] - 2.0 * theme.spacing.px2;
            scroll_caret_into_view(&mut state, &mut fonts, &field_style, scale, content_width);
        }
        assert!(
            state.scroll_offset > 0.0,
            "text should overflow at scale {scale}"
        );
    }
}

#[test]
fn mid_character_selection_cursor_does_not_panic() {
    let mut fonts = FontSystem::new();
    let mut state = TextInputState::new();
    handle_text(&mut state, "注音");
    state.set_selection_range(0, 1);
    scroll_caret_into_view(&mut state, &mut fonts, &style(), 1.0, CONTENT_W);
    let _ = state.ime_caret_area(&mut fonts, &style(), 1.0);
}

#[test]
fn password_caret_follows_masked_text_without_panicking() {
    let mut fonts = FontSystem::new();
    let mut state = TextInputState::new();
    state.password = true;
    handle_text(&mut state, "abc");
    state.cursor_prev();
    scroll_caret_into_view(&mut state, &mut fonts, &style(), 1.0, CONTENT_W);
    let two = state.ime_caret_area(&mut fonts, &style(), 1.0)[0];
    state.cursor_prev();
    let one = state.ime_caret_area(&mut fonts, &style(), 1.0)[0];
    assert!(
        one < two,
        "caret should move left over masked text ({one} !< {two})"
    );
}
