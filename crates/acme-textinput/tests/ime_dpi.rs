//! ROADMAP 005 / D3: TextInput caret, preedit and IME candidate geometry across
//! the DPI scale matrix. Uses only the public API of `acme-textinput`, plus the
//! public `acme-text`, `acme-theme` and `acme-platform` types it already depends on.
//!
//! Limits (see the batch report):
//! - Geometry checked here is logical. `FontSystem::shape` receives `scale`, but
//!   the reported widths and glyph `x` values are logical pixels.
//! - Physical placement, the Gallery's `compute_ime_caret_rect` (private to the
//!   `gallery` binary) and real 注音 composition under Windows IME are NOT covered.
//!   Manual IME validation remains NOT_RUN.

use acme_platform::PlatformKey;
use acme_text::{FontSystem, TextConstraints, TextStyle};
use acme_textinput::{TextInputState, byte_offset_to_x, handle_key, handle_text};
use acme_theme::Theme;
use unicode_segmentation::UnicodeSegmentation;

/// Scale matrix from BRIEF D3 (1.75 included because it is cheap).
const SCALES: [f32; 5] = [1.0, 1.25, 1.5, 1.75, 2.0];

/// Mixed Traditional Chinese + ASCII: 16 UTF-8 bytes, 8 chars, 8 graphemes.
const MIXED: &str = "注音輸入 abc";
/// Bopomofo preedit for 注: 8 UTF-8 bytes, 3 chars.
const PREEDIT: &str = "ㄓㄨˋ";

/// Field size used by the caret-bounds tests (logical px).
const FIELD_W: f32 = 240.0;
const FIELD_H: f32 = 40.0;

/// Tolerance for the same logical caret x at different scales. Measured value
/// is recorded in the batch report.
const LOGICAL_TOLERANCE_PX: f32 = 0.001;
/// Tolerance for `caret(after commit) ~= caret(before) + width(preedit)`.
/// Measured value is recorded in the batch report.
const COMPOSITION_SUM_TOLERANCE_PX: f32 = 0.01;

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

/// Style and padding derived exactly like the Gallery's TextInput rendering
/// (`theme.typography.body`, `line_height * body`, `spacing.px2` padding).
struct Harness {
    fonts: FontSystem,
    style: TextStyle,
    padding: f32,
}

fn harness() -> Harness {
    let theme = Theme::light();
    let body = theme.typography.body;
    Harness {
        fonts: FontSystem::new(),
        style: TextStyle {
            font_size: body,
            line_height: body * theme.typography.line_height,
            ..TextStyle::default()
        },
        padding: theme.spacing.px2,
    }
}

impl Harness {
    /// Caret rect `[x, y, w, h]` for `state` at its current cursor.
    fn caret(&mut self, state: &TextInputState, scale: f32) -> [f32; 4] {
        state.ime_caret_area(&mut self.fonts, &self.style, scale)
    }

    /// Caret rect for `state` with the cursor moved to byte offset `offset`.
    fn caret_at(&mut self, state: &TextInputState, offset: usize, scale: f32) -> [f32; 4] {
        let mut s = state.clone();
        s.move_cursor_to_offset(offset);
        self.caret(&s, scale)
    }

    /// Shaped width of `text` at `scale` (logical px).
    fn width_of(&mut self, text: &str, scale: f32) -> f32 {
        self.fonts
            .shape(text, &self.style, TextConstraints::default(), scale)
            .width
    }
}

/// Grapheme-cluster start offsets of `text`, plus `text.len()` at the end.
fn grapheme_boundaries(text: &str) -> Vec<usize> {
    let mut offsets: Vec<usize> = text.grapheme_indices(true).map(|(i, _)| i).collect();
    offsets.push(text.len());
    offsets
}

/// A state whose text is `text` and whose cursor sits at the end.
fn input_with(text: &str) -> TextInputState {
    let mut s = TextInputState::new();
    s.text = text.to_string();
    s.move_cursor_to_offset(text.len());
    s
}

fn focused(mut s: TextInputState) -> TextInputState {
    s.focused = true;
    s
}

// ---------------------------------------------------------------------------
// Preconditions and cursor stepping (grapheme indices, not bytes)
// ---------------------------------------------------------------------------

#[test]
fn mixed_string_byte_and_char_counts_differ() {
    assert_eq!(MIXED.len(), 16, "UTF-8 byte count");
    assert_eq!(MIXED.chars().count(), 8, "char count");
    assert_eq!(MIXED.graphemes(true).count(), 8, "grapheme count");
    assert_eq!(PREEDIT.len(), 8, "preedit UTF-8 byte count");
    assert_eq!(PREEDIT.chars().count(), 3, "preedit char count");
}

#[test]
fn cursor_steps_land_on_grapheme_boundaries_only() {
    // Decomposed "é" (e + U+0301) is one grapheme spanning bytes 16..19.
    const TEXT: &str = "注音輸入 abc e\u{301}";
    assert_eq!(
        TEXT.len(),
        20,
        "4x3 CJK + space + abc + space + e + 2-byte combining mark"
    );
    let expected = grapheme_boundaries(TEXT);
    assert_eq!(expected, vec![0, 3, 6, 9, 12, 13, 14, 15, 16, 17, 20]);

    let mut s = TextInputState::new();
    s.text = TEXT.to_string();
    let mut forward = vec![s.cursor()];
    while s.cursor() < TEXT.len() {
        s.cursor_next();
        forward.push(s.cursor());
    }
    assert_eq!(forward, expected, "forward stepping");

    let mut backward = vec![s.cursor()];
    while s.cursor() > 0 {
        s.cursor_prev();
        backward.push(s.cursor());
    }
    backward.reverse();
    assert_eq!(backward, expected, "backward stepping");
    assert!(
        forward.iter().all(|&o| TEXT.is_char_boundary(o)),
        "every stop must be a char boundary"
    );
}

// ---------------------------------------------------------------------------
// Caret x: monotonic, prefix-consistent, scale-invariant
// ---------------------------------------------------------------------------

#[test]
fn caret_x_strictly_increases_in_grapheme_order_at_every_scale() {
    let mut h = harness();
    let s = input_with(MIXED);
    for scale in SCALES {
        let mut prev = f32::NEG_INFINITY;
        for offset in grapheme_boundaries(MIXED) {
            let [x, _, _, _] = h.caret_at(&s, offset, scale);
            assert!(x.is_finite(), "scale {scale} offset {offset}: x not finite");
            assert!(
                x > prev,
                "scale {scale}: caret x not strictly increasing at byte {offset}: prev={prev} x={x}"
            );
            prev = x;
        }
    }
}

#[test]
fn caret_x_equals_shaped_prefix_width_at_char_boundaries() {
    let mut h = harness();
    for offset in grapheme_boundaries(MIXED) {
        let expected = if offset == 0 {
            0.0
        } else {
            h.width_of(&MIXED[..offset], 1.0)
        };
        let got = byte_offset_to_x(MIXED, offset, &mut h.fonts, &h.style, 1.0);
        assert!(
            (got - expected).abs() <= 1e-3,
            "offset {offset}: byte_offset_to_x={got} prefix width={expected}"
        );
    }
}

#[test]
fn caret_x_is_scale_invariant_in_logical_pixels() {
    let mut h = harness();
    let s = input_with(MIXED);
    let offsets = grapheme_boundaries(MIXED);
    let base: Vec<f32> = offsets.iter().map(|&o| h.caret_at(&s, o, 1.0)[0]).collect();

    let mut worst = 0.0_f32;
    for scale in SCALES {
        for (i, &offset) in offsets.iter().enumerate() {
            let x = h.caret_at(&s, offset, scale)[0];
            worst = worst.max((x - base[i]).abs());
        }
    }
    println!(
        "MEASURED caret logical max |dx| vs scale 1.0: {worst} px over {} boundaries x {} scales",
        offsets.len(),
        SCALES.len()
    );
    assert!(
        worst <= LOGICAL_TOLERANCE_PX,
        "logical caret x drifts with scale: max |dx|={worst} > {LOGICAL_TOLERANCE_PX}"
    );
}

#[test]
fn layout_widths_and_glyph_x_are_scale_invariant() {
    // Same property one layer down: the shaper's logical output must not move
    // with scale. The raster placement is the only scale-dependent part.
    let mut h = harness();
    let base = h
        .fonts
        .shape(MIXED, &h.style, TextConstraints::default(), 1.0);
    for scale in SCALES {
        let layout = h
            .fonts
            .shape(MIXED, &h.style, TextConstraints::default(), scale);
        assert!(
            (layout.width - base.width).abs() <= LOGICAL_TOLERANCE_PX,
            "scale {scale}: width {} vs {}",
            layout.width,
            base.width
        );
        assert_eq!(
            layout.glyphs.len(),
            base.glyphs.len(),
            "scale {scale}: glyph count"
        );
        for (g, b) in layout.glyphs.iter().zip(base.glyphs.iter()) {
            assert!(
                (g.x - b.x).abs() <= LOGICAL_TOLERANCE_PX,
                "scale {scale}: glyph x {} vs {}",
                g.x,
                b.x
            );
        }
    }
}

#[test]
fn caret_end_matches_layout_width_and_cursor_x_at_every_scale() {
    let mut h = harness();
    let s = input_with(MIXED);
    for scale in SCALES {
        let [x_end, _, _, _] = h.caret(&s, scale);
        let width = h.width_of(MIXED, scale);
        assert!(
            (x_end - width).abs() <= LOGICAL_TOLERANCE_PX,
            "scale {scale}: caret at end {x_end} vs layout width {width}"
        );
        let cursor_x = s.cursor_x(&mut h.fonts, &h.style, scale);
        assert_eq!(
            cursor_x, x_end,
            "scale {scale}: cursor_x and ime_caret_area disagree"
        );
    }
}

#[test]
fn caret_at_start_is_zero_at_every_scale() {
    let mut h = harness();
    let s = input_with(MIXED);
    for scale in SCALES {
        let [x0, y, w, hh] = h.caret_at(&s, 0, scale);
        assert_eq!(x0, 0.0, "scale {scale}");
        assert_eq!(y, 0.0);
        assert!(
            w > 0.0 && hh > 0.0,
            "scale {scale}: degenerate caret {w}x{hh}"
        );
    }
}

#[test]
fn non_finite_or_non_positive_scale_falls_back_to_one() {
    let mut h = harness();
    let s = input_with(MIXED);
    let reference = h.caret(&s, 1.0);
    for bad in [0.0_f32, -1.0, f32::NAN, f32::INFINITY] {
        let got = h.caret(&s, bad);
        assert!(
            got.iter().all(|v| v.is_finite()),
            "scale {bad}: non-finite {got:?}"
        );
        assert!(
            (got[0] - reference[0]).abs() <= LOGICAL_TOLERANCE_PX,
            "scale {bad}: x {} vs reference {}",
            got[0],
            reference[0]
        );
    }
}

// ---------------------------------------------------------------------------
// IME candidate rect versus field bounds
// ---------------------------------------------------------------------------

/// Candidate rect in field-local logical px, replicating the formula used by
/// the Gallery's `compute_ime_caret_rect` (field origin + padding + caret area)
/// with the field at (0, 0). The Gallery function itself is private to its binary.
fn candidate_rect(h: &mut Harness, s: &TextInputState, scale: f32) -> [f32; 4] {
    let [cx, cy, cw, ch] = h.caret(s, scale);
    [h.padding + cx, h.padding + cy, cw, ch.max(1.0)]
}

#[test]
fn candidate_rect_stays_inside_field_when_text_fits() {
    let mut h = harness();
    let s = focused(input_with(MIXED));
    for scale in SCALES {
        let [x, y, w, hh] = candidate_rect(&mut h, &s, scale);
        println!(
            "MEASURED candidate rect scale {scale}: x={x} y={y} w={w} h={hh} right={} bottom={}",
            x + w,
            y + hh
        );
        assert!(
            x >= 0.0 && x + w <= FIELD_W,
            "scale {scale}: x-span [{x}, {}] outside field width {FIELD_W}",
            x + w
        );
        assert!(
            y >= 0.0 && y + hh <= FIELD_H,
            "scale {scale}: y-span [{y}, {}] outside field height {FIELD_H}",
            y + hh
        );
    }
}

#[test]
fn candidate_rect_stays_inside_field_with_preedit_span() {
    let mut h = harness();
    let mut s = focused(input_with(MIXED));
    s.set_preedit(PREEDIT, Some((3, 6)));
    for scale in SCALES {
        let before = candidate_rect(&mut h, &s, scale);
        let span = h.width_of(PREEDIT, scale);
        // The Gallery anchors the candidate window at the committed caret, so the
        // composition span starts at the same x. Check the underlined span too.
        assert!(
            before[0] + span <= FIELD_W - h.padding,
            "scale {scale}: preedit span ends at {} past content edge {}",
            before[0] + span,
            FIELD_W - h.padding
        );
    }
}

#[test]
fn overflowing_caret_is_not_clamped_without_owner_scroll() {
    // GAP pin: the crate never updates scroll_offset itself. Without an owner
    // scroll, a caret past the field edge stays past it.
    let mut h = harness();
    let long = MIXED.repeat(4);
    let mut s = focused(input_with(&long));
    let scale = 1.0;
    let [x_raw, _, w, _] = h.caret(&s, scale);
    assert!(
        h.padding + x_raw > FIELD_W,
        "fixture must overflow: caret at {}",
        h.padding + x_raw
    );

    // The owner scrolls so the caret's right edge sits at the content edge.
    let content_w = FIELD_W - 2.0 * h.padding;
    s.scroll_offset = x_raw - (content_w - w);
    let [x_scrolled, _, _, _] = h.caret(&s, scale);
    assert!(
        h.padding + x_scrolled + w <= FIELD_W - h.padding + LOGICAL_TOLERANCE_PX,
        "owner scroll must bring caret into the content box"
    );
}

#[test]
fn preedit_does_not_move_caret_or_candidate_anchor() {
    let mut h = harness();
    let s = focused(input_with(MIXED));
    for scale in SCALES {
        let before = h.caret(&s, scale);
        let mut c = s.clone();
        c.set_preedit(PREEDIT, None);
        assert_eq!(
            c.cursor(),
            s.cursor(),
            "scale {scale}: preedit moved the committed cursor"
        );
        assert_eq!(
            c.text, MIXED,
            "scale {scale}: preedit changed committed text"
        );
        assert_eq!(
            h.caret(&c, scale),
            before,
            "scale {scale}: preedit moved the caret rect"
        );
    }
}

// ---------------------------------------------------------------------------
// Preedit cancel / commit consistency
// ---------------------------------------------------------------------------

#[test]
fn cancel_preedit_restores_caret_geometry() {
    let mut h = harness();
    let s = focused(input_with(MIXED));
    for scale in SCALES {
        let before = h.caret(&s, scale);
        let mut c = s.clone();
        c.set_preedit(PREEDIT, Some((3, 6)));
        c.cancel_preedit();
        assert!(c.preedit.is_empty(), "scale {scale}: preedit not cleared");
        assert_eq!(
            c.preedit_cursor, None,
            "scale {scale}: preedit cursor not cleared"
        );
        assert_eq!(c.text, MIXED, "scale {scale}: text changed by cancel");
        assert_eq!(
            c.cursor(),
            s.cursor(),
            "scale {scale}: cursor changed by cancel"
        );
        assert_eq!(
            h.caret(&c, scale),
            before,
            "scale {scale}: caret not restored"
        );
    }
}

#[test]
fn commit_preedit_advances_caret_by_preedit_width_at_every_scale() {
    let mut h = harness();
    let s = focused(input_with(MIXED));
    for scale in SCALES {
        let before = h.caret(&s, scale);
        let mut c = s.clone();
        c.set_preedit(PREEDIT, None);
        c.commit_preedit();

        assert_eq!(
            c.text,
            format!("{MIXED}{PREEDIT}"),
            "scale {scale}: committed text"
        );
        assert_eq!(
            c.cursor(),
            MIXED.len() + PREEDIT.len(),
            "scale {scale}: cursor must advance by preedit UTF-8 bytes"
        );
        assert!(
            c.preedit.is_empty(),
            "scale {scale}: preedit left after commit"
        );

        let after = h.caret(&c, scale);
        let expected_x = before[0] + h.width_of(PREEDIT, scale);
        let diff = (after[0] - expected_x).abs();
        println!("MEASURED commit caret sum-diff at scale {scale}: {diff} px");
        assert!(
            diff <= COMPOSITION_SUM_TOLERANCE_PX,
            "scale {scale}: caret after commit {} vs before+preedit {expected_x} (diff {diff})",
            after[0]
        );
        assert_eq!(
            after[0],
            c.cursor_x(&mut h.fonts, &h.style, scale),
            "scale {scale}: cursor_x and ime_caret_area disagree after commit"
        );
    }
}

#[test]
fn undo_after_commit_restores_caret_and_redo_reapplies_it() {
    let mut h = harness();
    let s = focused(input_with(MIXED));
    for scale in SCALES {
        let before = h.caret(&s, scale);
        let mut c = s.clone();
        c.set_preedit(PREEDIT, None);
        c.commit_preedit();
        let committed = h.caret(&c, scale);

        assert!(
            c.undo(),
            "scale {scale}: commit must be one undo transaction"
        );
        assert_eq!(c.text, MIXED, "scale {scale}: undo text");
        assert_eq!(c.cursor(), MIXED.len(), "scale {scale}: undo cursor");
        assert_eq!(h.caret(&c, scale), before, "scale {scale}: undo caret");

        assert!(c.redo(), "scale {scale}: redo");
        assert_eq!(h.caret(&c, scale), committed, "scale {scale}: redo caret");
    }
}

#[test]
fn handle_text_commit_inserts_at_caret_with_byte_length() {
    // Direct text commit (the path the Gallery's ImeCommit handler uses).
    let mut h = harness();
    let mut c = focused(input_with(MIXED));
    assert!(handle_text(&mut c, "注"));
    assert_eq!(c.text, "注音輸入 abc注");
    assert_eq!(c.cursor(), c.text.len());
    let end = h.caret(&c, 1.0)[0];
    let expected = h.width_of(&c.text, 1.0);
    assert!((end - expected).abs() <= LOGICAL_TOLERANCE_PX);
}

// ---------------------------------------------------------------------------
// Regressions for findings fixed in ROADMAP 005 (were ignored before the fix)
// ---------------------------------------------------------------------------

#[test]
fn move_cursor_to_mid_char_offset_is_snapped_to_boundary() {
    // Minimal repro: offset 1 is inside the 3-byte "注".
    let mut s = input_with("注音");
    s.move_cursor_to_offset(1);
    assert!(
        "注音".is_char_boundary(s.cursor()),
        "cursor {} is not a char boundary",
        s.cursor()
    );
    let mut h = harness();
    let _ = s.ime_caret_area(&mut h.fonts, &h.style, 1.0);
}

#[test]
fn handle_text_commit_clears_preedit() {
    let mut s = TextInputState::new();
    s.set_preedit(PREEDIT, None);
    assert!(handle_text(&mut s, "注"));
    assert!(
        s.preedit.is_empty(),
        "stale preedit {:?} after commit",
        s.preedit
    );
}

#[test]
fn escape_cancels_preedit() {
    let mut s = focused(input_with(MIXED));
    s.set_preedit(PREEDIT, None);
    assert!(handle_key(
        &mut s,
        &PlatformKey::Escape,
        true,
        None,
        false,
        false
    ));
    assert!(
        s.preedit.is_empty(),
        "preedit {:?} survives Escape",
        s.preedit
    );
}
