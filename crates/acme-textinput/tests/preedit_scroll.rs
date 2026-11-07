//! ROADMAP 012 D: an IME preedit wider than the remaining field must scroll into
//! view, and the composition cursor must stay visible. Uses only the public API
//! of `acme-textinput` and `acme-text`, with the scale matrix of `ime_dpi.rs`.
//! Widths are logical pixels; the scale argument only affects raster placement.

use acme_core::{DrawCommand, Scene};
use acme_text::{FontSystem, GlyphAtlas, TextStyle};
use acme_textinput::{
    TextInputState, byte_offset_to_x, handle_text, render_text_input, scroll_caret_into_view,
};
use acme_theme::Theme;

/// Visible text width used by the scroll tests (logical px).
const CONTENT_W: f32 = 120.0;
/// Caret width reserved at the right edge (matches the crate's CARET_WIDTH).
const CARET_W: f32 = 1.5;
/// Largest content-local x the composition cursor may take.
const VISIBLE: f32 = CONTENT_W - CARET_W;
const EPS: f32 = 0.001;
const SCALES: [f32; 4] = [1.0, 1.25, 1.5, 2.0];

/// A 注音 composition that is wider than the whole field.
const LONG_PREEDIT: &str = "ㄓㄨˋㄧㄣ注音輸入法測試字串";
/// A short composition that fits in the field.
const SHORT_PREEDIT: &str = "ㄓㄨˋ";
/// Committed text with CJK and ASCII; 8 chars, 16 UTF-8 bytes.
const MIXED: &str = "注音輸入 abc";

fn style() -> TextStyle {
    TextStyle {
        font_size: 16.0,
        line_height: 24.0,
        ..TextStyle::default()
    }
}

/// A focused state with `committed` text, the cursor at its end, and a preedit.
fn composing(
    committed: &str,
    preedit: &str,
    preedit_cursor: Option<(usize, usize)>,
) -> TextInputState {
    let mut state = TextInputState::new();
    state.focused = true;
    state.text = committed.to_string();
    state.move_cursor_to_offset(committed.len());
    state.set_preedit(preedit, preedit_cursor);
    state
}

fn follow(state: &mut TextInputState, fonts: &mut FontSystem, scale: f32) {
    scroll_caret_into_view(state, fonts, &style(), scale, CONTENT_W);
}

/// Content-local x of the composition cursor: the committed caret (what
/// `ime_caret_area` reports, already minus `scroll_offset`) plus the preedit
/// width up to the cursor end, or up to the preedit end when `None`.
fn composition_x(state: &TextInputState, fonts: &mut FontSystem, scale: f32) -> f32 {
    let base = state.ime_caret_area(fonts, &style(), scale)[0];
    let end = state
        .preedit_cursor
        .map_or(state.preedit.len(), |(_, end)| end);
    base + byte_offset_to_x(&state.preedit, end, fonts, &style(), scale)
}

fn assert_composition_visible(
    state: &TextInputState,
    fonts: &mut FontSystem,
    scale: f32,
    ctx: &str,
) {
    let x = composition_x(state, fonts, scale);
    assert!(x.is_finite(), "composition x not finite {x} {ctx}");
    assert!(
        (-EPS..=VISIBLE + EPS).contains(&x),
        "composition cursor x {x} outside [0, {VISIBLE}] {ctx}"
    );
    assert!(
        state.scroll_offset >= 0.0,
        "scroll_offset negative {} {ctx}",
        state.scroll_offset
    );
}

#[test]
fn long_preedit_in_empty_field_scrolls_so_composition_cursor_is_visible() {
    let mut fonts = FontSystem::new();
    for scale in SCALES {
        let mut state = composing("", LONG_PREEDIT, None);
        follow(&mut state, &mut fonts, scale);
        assert!(
            state.scroll_offset > 0.0,
            "wide preedit must scroll at scale {scale}"
        );
        assert_composition_visible(&state, &mut fonts, scale, &format!("scale {scale}"));
    }
}

#[test]
fn long_preedit_after_overflowing_text_uses_the_preedit_cursor_end() {
    // Preedit cursor `(0, mid)` ends after the bopomofo; the scroll must keep
    // that end visible, not the committed caret.
    let mid = LONG_PREEDIT
        .find('注')
        .expect("fixture has a CJK character");
    let committed = "w".repeat(40);
    let mut fonts = FontSystem::new();
    for scale in SCALES {
        let mut state = composing(&committed, LONG_PREEDIT, Some((0, mid)));
        follow(&mut state, &mut fonts, scale);
        assert_composition_visible(&state, &mut fonts, scale, &format!("scale {scale}"));
    }
}

#[test]
fn short_preedit_at_the_end_of_overflowing_text_stays_visible() {
    let committed = "w".repeat(40);
    let mut fonts = FontSystem::new();
    for scale in SCALES {
        let mut state = composing(&committed, SHORT_PREEDIT, None);
        follow(&mut state, &mut fonts, scale);
        assert!(state.scroll_offset > 0.0, "committed text must scroll");
        assert_composition_visible(&state, &mut fonts, scale, &format!("scale {scale}"));
    }
}

#[test]
fn mixed_text_with_long_preedit_keeps_composition_visible() {
    let mut fonts = FontSystem::new();
    for scale in SCALES {
        let mut state = composing(MIXED, LONG_PREEDIT, None);
        follow(&mut state, &mut fonts, scale);
        assert_composition_visible(&state, &mut fonts, scale, &format!("scale {scale}"));
    }
}

#[test]
fn ime_caret_area_follows_the_scroll_while_composing() {
    let committed = "w".repeat(40);
    let mut fonts = FontSystem::new();
    for scale in SCALES {
        let mut state = composing(&committed, LONG_PREEDIT, None);
        follow(&mut state, &mut fonts, scale);
        let mut unscrolled = state.clone();
        unscrolled.scroll_offset = 0.0;
        let x0 = unscrolled.ime_caret_area(&mut fonts, &style(), scale)[0];
        let x = state.ime_caret_area(&mut fonts, &style(), scale)[0];
        assert!(
            (x - (x0 - state.scroll_offset)).abs() < EPS,
            "scale {scale}: anchor {x} must equal unscrolled {x0} minus scroll {}",
            state.scroll_offset
        );
    }
}

#[test]
fn empty_preedit_scroll_is_unchanged_and_matches_no_preedit() {
    // Guard: with no preedit, the committed caret rule stays as it is today.
    let committed = "w".repeat(80);
    let mut fonts = FontSystem::new();
    for scale in SCALES {
        let mut plain = TextInputState::new();
        plain.text = committed.clone();
        plain.move_cursor_to_offset(committed.len());
        let mut empty = plain.clone();
        empty.set_preedit("", None);

        follow(&mut plain, &mut fonts, scale);
        follow(&mut empty, &mut fonts, scale);

        let text_w = byte_offset_to_x(&committed, committed.len(), &mut fonts, &style(), scale);
        assert_eq!(
            plain.scroll_offset.to_bits(),
            empty.scroll_offset.to_bits(),
            "scale {scale}: empty preedit changed the scroll"
        );
        assert!(
            (plain.scroll_offset - (text_w - VISIBLE)).abs() < EPS,
            "scale {scale}: scroll {} != text width {text_w} - visible {VISIBLE}",
            plain.scroll_offset
        );
    }
}

fn rect_right<U>(rect: acme_core::Rect<U>) -> f32 {
    rect.origin.x.get() + rect.size.width.get()
}

#[test]
fn rendered_composition_segment_lies_inside_the_scrolled_field() {
    let theme = Theme::light();
    let padding = theme.spacing.px2;
    // Field whose content width is exactly CONTENT_W.
    let field = [0.0_f32, 0.0, CONTENT_W + 2.0 * padding, 40.0];
    let content_w = field[2] - 2.0 * padding;
    let style = TextStyle {
        font_size: theme.typography.body,
        line_height: theme.typography.body * theme.typography.line_height,
        ..TextStyle::default()
    };
    for scale in SCALES {
        let mut fonts = FontSystem::new();
        let mut atlas = GlyphAtlas::new(1024, 1024);
        let mut state = composing("", LONG_PREEDIT, Some((0, LONG_PREEDIT.len())));
        scroll_caret_into_view(&mut state, &mut fonts, &style, scale, content_w);
        let mut scene = Scene::default();
        render_text_input(
            &mut scene, &mut state, &mut fonts, &mut atlas, field, &theme, scale, true, None,
        );
        // The thick composition segment is the only quad 3 px tall.
        let segment = scene
            .commands()
            .iter()
            .find_map(|command| match command {
                DrawCommand::Quad(quad) if (quad.rect.size.height.get() - 3.0).abs() < EPS => {
                    Some(quad.rect)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("no composition segment drawn at scale {scale}"));
        let right = rect_right(segment);
        assert!(
            right <= padding + VISIBLE + EPS,
            "segment right {right} past visible edge {} at scale {scale}",
            padding + VISIBLE
        );
        // The segment start may scroll out of the clip (the preedit is wider than
        // the field); the composition cursor at the right edge must stay visible.
        assert!(
            right >= padding - EPS,
            "composition cursor {right} left of content at scale {scale}"
        );
    }
}

#[test]
fn password_preedit_underline_starts_at_the_ime_caret_anchor() {
    // render_text_input must place the underline with the same display-space
    // offset that ime_caret_area uses. A password mask makes raw and display
    // offsets differ.
    let theme = Theme::light();
    let field = [0.0_f32, 0.0, 200.0, 40.0];
    let padding = theme.spacing.px2;
    let mut fonts = FontSystem::new();
    let mut atlas = GlyphAtlas::new(1024, 1024);
    let mut state = TextInputState::new();
    state.focused = true;
    state.password = true;
    handle_text(&mut state, "abc");
    state.set_preedit(SHORT_PREEDIT, None);
    let style = TextStyle {
        font_size: theme.typography.body,
        line_height: theme.typography.body * theme.typography.line_height,
        ..TextStyle::default()
    };
    let anchor = state.ime_caret_area(&mut fonts, &style, 1.0)[0];

    let mut scene = Scene::default();
    render_text_input(
        &mut scene, &mut state, &mut fonts, &mut atlas, field, &theme, 1.0, true, None,
    );
    let underline = scene
        .commands()
        .iter()
        .find_map(|command| match command {
            DrawCommand::Quad(quad) if (quad.rect.size.height.get() - 1.0).abs() < EPS => {
                Some(quad.rect)
            }
            _ => None,
        })
        .expect("preedit underline drawn");
    let expected = padding + anchor;
    assert!(
        (underline.origin.x.get() - expected).abs() < EPS,
        "underline x {} != ime anchor {expected}",
        underline.origin.x.get()
    );
}
