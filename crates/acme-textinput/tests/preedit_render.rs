//! ROADMAP 012 D2: `render_text_input` must draw the IME preedit glyphs, not
//! just the underline, and must shift the committed text after the caret by the
//! preedit width. Uses only the public API of `acme-textinput`, `acme-text`,
//! `acme-core` and `acme-theme`.
//!
//! Scale matrix as in `ime_dpi.rs`/`preedit_scroll.rs`. Glyph `x`/`width` are
//! physical pixels (`acme-render-cpu` and wgpu both add `origin * scale`), so the
//! helpers below convert them back to logical pixels before comparing with the
//! shaped widths of `byte_offset_to_x` peers.
//!
//! NOT covered: real 注音 composition under Windows IME (manual, NOT_RUN).

use acme_core::{DrawCommand, Scene};
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextStyle};
use acme_textinput::{TextInputState, render_text_input};
use acme_theme::Theme;

/// Logical tolerance for positions and widths.
const EPS: f32 = 0.01;
const SCALES: [f32; 4] = [1.0, 1.25, 1.5, 2.0];

/// Committed text for the plain cases.
const COMMITTED: &str = "abc";
/// A short bopomofo preedit (3 chars, 8 bytes) for 注.
const PREEDIT: &str = "ㄓㄨˋ";
/// A longer composition that does not fit in the field of the scroll cases.
const LONG_PREEDIT: &str = "ㄓㄨˋㄧㄣ注音輸入法測試字串";

struct Harness {
    fonts: FontSystem,
    atlas: GlyphAtlas,
    theme: Theme,
    style: TextStyle,
    /// Field rect `[x, y, w, h]` (logical px, x = y = 0).
    field: [f32; 4],
    padding: f32,
}

impl Harness {
    fn new() -> Self {
        let theme = Theme::light();
        let body = theme.typography.body;
        Self {
            fonts: FontSystem::new(),
            atlas: GlyphAtlas::new(1024, 1024),
            style: TextStyle {
                font_size: body,
                line_height: body * theme.typography.line_height,
                ..TextStyle::default()
            },
            padding: theme.spacing.px2,
            field: [0.0, 0.0, 240.0, 40.0],
            theme,
        }
    }

    /// Logical advance width of `text` in the field style.
    fn width(&mut self, text: &str) -> f32 {
        self.fonts
            .shape(text, &self.style, TextConstraints::default(), 1.0)
            .width
    }

    fn content_x(&self) -> f32 {
        self.field[0] + self.padding
    }

    fn render(&mut self, state: &mut TextInputState, scale: f32) -> Scene {
        let mut scene = Scene::default();
        render_text_input(
            &mut scene,
            state,
            &mut self.fonts,
            &mut self.atlas,
            self.field,
            &self.theme,
            scale,
            state.focused,
            None,
        );
        scene
    }
}

/// One `DrawCommand::Text` run, in logical pixels.
#[derive(Debug)]
struct Run {
    origin_x: f32,
    /// `(left, right)` of every drawn glyph.
    glyphs: Vec<(f32, f32)>,
}

impl Run {
    fn ink_right(&self) -> f32 {
        self.glyphs.iter().map(|(_, r)| *r).fold(f32::MIN, f32::max)
    }
}

fn runs(scene: &Scene, scale: f32) -> Vec<Run> {
    scene
        .commands()
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Text(text) => {
                let origin_x = text.origin.x.get();
                Some(Run {
                    origin_x,
                    glyphs: text
                        .glyphs
                        .iter()
                        .map(|glyph| {
                            (
                                origin_x + glyph.x / scale,
                                origin_x + (glyph.x + glyph.width) / scale,
                            )
                        })
                        .collect(),
                })
            }
            _ => None,
        })
        .collect()
}

/// Height and `(x, width)` of every quad with the given height.
fn quads_of_height(scene: &Scene, height: f32) -> Vec<(f32, f32)> {
    scene
        .commands()
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Quad(quad) if (quad.rect.size.height.get() - height).abs() < EPS => {
                Some((quad.rect.origin.x.get(), quad.rect.size.width.get()))
            }
            _ => None,
        })
        .collect()
}

fn composing(text: &str, caret: usize, preedit: &str) -> TextInputState {
    let mut state = TextInputState::new();
    state.focused = true;
    state.text = text.to_string();
    state.move_cursor_to_offset(caret);
    state.set_preedit(preedit, None);
    state
}

fn texts_of(state: &mut TextInputState, preedit: Option<&str>) -> Scene {
    let mut h = Harness::new();
    if let Some(preedit) = preedit {
        state.set_preedit(preedit, None);
    }
    h.render(state, 1.0)
}

/// A run starting at `expected_x` that actually contains glyphs.
fn run_at<'a>(runs: &'a [Run], expected_x: f32, ctx: &str) -> &'a Run {
    runs.iter()
        .find(|run| (run.origin_x - expected_x).abs() < EPS)
        .unwrap_or_else(|| {
            panic!(
                "{ctx}: no text run at x {expected_x}; runs at {:?}",
                runs.iter().map(|r| r.origin_x).collect::<Vec<_>>()
            )
        })
}

#[test]
fn preedit_glyphs_are_drawn_while_composing() {
    // Today the only DrawCommand::Text holds the committed text, so the
    // composition appears as a bare underline.
    let mut h = Harness::new();
    let w_committed = h.width(COMMITTED);
    let mut state = composing(COMMITTED, COMMITTED.len(), PREEDIT);
    let scene = h.render(&mut state, 1.0);
    let runs = runs(&scene, 1.0);
    let composition = run_at(&runs, h.content_x() + w_committed, "scale 1.0");
    assert!(
        !composition.glyphs.is_empty(),
        "composition run at x {} draws no glyphs",
        composition.origin_x
    );
}

#[test]
fn composition_run_covers_the_preedit_width() {
    for scale in SCALES {
        let mut h = Harness::new();
        let w_committed = h.width(COMMITTED);
        let w_preedit = h.width(PREEDIT);
        let mut state = composing(COMMITTED, COMMITTED.len(), PREEDIT);
        let scene = h.render(&mut state, scale);
        let runs = runs(&scene, scale);
        let composition = run_at(
            &runs,
            h.content_x() + w_committed,
            &format!("scale {scale}"),
        );
        // Ink must extend past the second-to-last glyph, i.e. across the
        // composition rather than just its first glyph.
        let last_glyph = h.width("ˋ");
        let right = composition.ink_right();
        assert!(
            right >= composition.origin_x + w_preedit - last_glyph,
            "scale {scale}: preedit ink ends at {right}, expected about \
             {} wide preedit from {}",
            w_preedit,
            composition.origin_x
        );
    }
}

#[test]
fn committed_text_after_the_caret_shifts_by_the_preedit_width() {
    let mut h = Harness::new();
    let w_first = h.width("a");
    let w_preedit = h.width(PREEDIT);
    let caret = 1;
    let mut state = composing("ab", caret, PREEDIT);
    let scene = h.render(&mut state, 1.0);
    let runs = runs(&scene, 1.0);
    // before-caret run at the content origin
    run_at(&runs, h.content_x(), "lead run");
    // composition right after it
    let composition = run_at(&runs, h.content_x() + w_first, "composition run");
    assert!(!composition.glyphs.is_empty(), "composition run empty");
    // trailing committed text pushed by the whole preedit width
    let trailing = run_at(&runs, h.content_x() + w_first + w_preedit, "trailing run");
    assert_eq!(trailing.glyphs.len(), 1, "trailing run must hold only 'b'");
}

#[test]
fn trailing_run_is_unshifted_without_a_preedit() {
    // Guard: the shift must come from the preedit, not from splitting the run.
    let mut h = Harness::new();
    let w_first = h.width("a");
    let mut state = TextInputState::new();
    state.focused = true;
    state.text = "ab".to_string();
    state.move_cursor_to_offset(1);
    let scene = h.render(&mut state, 1.0);
    let runs = runs(&scene, 1.0);
    assert_eq!(runs.len(), 1, "no preedit keeps a single text run");
    let run = &runs[0];
    assert!((run.origin_x - h.content_x()).abs() < EPS);
    assert_eq!(run.glyphs.len(), 2);
    // The second glyph sits where 'b' is in the unsplit run.
    let w_ab = h.width("ab");
    let (b_left, _) = run.glyphs[1];
    assert!(
        b_left < h.content_x() + w_ab && b_left > h.content_x() + w_first - EPS,
        "'b' at {b_left} outside ({}, {})",
        h.content_x() + w_first - EPS,
        h.content_x() + w_ab
    );
}

#[test]
fn underline_and_composition_segment_line_up_with_the_preedit_run() {
    let mut h = Harness::new();
    let w_committed = h.width(COMMITTED);
    let w_preedit = h.width(PREEDIT);
    // The composition cursor covers the second bopomofo ('ㄨ'), so the thick
    // segment must sit after the first one.
    let start = PREEDIT.find('ㄨ').expect("bopomofo fixture");
    let end = start + 'ㄨ'.len_utf8();
    let w_prefix = h.width(&PREEDIT[..start]);
    let w_segment = h.width(&PREEDIT[start..end]);
    let mut state = composing(COMMITTED, COMMITTED.len(), PREEDIT);
    state.preedit_cursor = Some((start, end));
    let scene = h.render(&mut state, 1.0);
    let runs = runs(&scene, 1.0);
    let composition = run_at(&runs, h.content_x() + w_committed, "composition run");

    let underlines = quads_of_height(&scene, 1.0);
    assert_eq!(
        underlines.len(),
        1,
        "one preedit underline, got {underlines:?}"
    );
    let (underline_x, underline_w) = underlines[0];
    assert!(
        (underline_x - composition.origin_x).abs() < EPS,
        "underline x {underline_x} != preedit run x {}",
        composition.origin_x
    );
    assert!(
        (underline_w - w_preedit).abs() < EPS,
        "underline width {underline_w} != preedit width {w_preedit}"
    );

    let segments = quads_of_height(&scene, 3.0);
    assert_eq!(
        segments.len(),
        1,
        "one composition segment, got {segments:?}"
    );
    let (segment_x, segment_w) = segments[0];
    let expected = composition.origin_x + w_prefix;
    assert!(
        (segment_x - expected).abs() < EPS,
        "composition segment x {segment_x} != preedit run x + prefix {expected}"
    );
    assert!(
        (segment_w - w_segment).abs() < EPS,
        "composition segment width {segment_w} != preedit-cursor range {w_segment}"
    );
}

#[test]
fn password_preedit_is_masked_not_plaintext() {
    // A password field must never render the raw composition; it is drawn with
    // the password mask like the committed text.
    let mut h = Harness::new();
    let mask = '•';
    let w_committed = h.width("••");
    let w_masked = h.width("•••");
    let w_plain = h.width(PREEDIT);
    let mut state = composing("ab", 2, PREEDIT);
    state.password = true;
    state.password_char = mask;
    let scene = h.render(&mut state, 1.0);
    let runs = runs(&scene, 1.0);
    let composition = run_at(&runs, h.content_x() + w_committed, "masked composition run");
    assert_eq!(
        composition.glyphs.len(),
        3,
        "masked preedit must draw one glyph per grapheme"
    );
    let underlines = quads_of_height(&scene, 1.0);
    assert_eq!(underlines.len(), 1, "one preedit underline");
    let (_, underline_w) = underlines[0];
    assert!(
        (underline_w - w_masked).abs() < EPS,
        "underline width {underline_w} != masked width {w_masked} (plaintext {w_plain})"
    );
    assert!(
        (underline_w - w_plain).abs() > EPS,
        "underline width {underline_w} equals the plaintext preedit width"
    );
}

#[test]
fn empty_preedit_renders_exactly_as_without_one() {
    // The D2 change must not alter the no-composition frame, byte for byte.
    let plain = {
        let mut state = TextInputState::new();
        state.focused = true;
        state.text = COMMITTED.to_string();
        state.move_cursor_to_offset(2);
        texts_of(&mut state, None)
    };
    let empty = {
        let mut state = TextInputState::new();
        state.focused = true;
        state.text = COMMITTED.to_string();
        state.move_cursor_to_offset(2);
        texts_of(&mut state, Some(""))
    };
    assert_eq!(
        plain.commands(),
        empty.commands(),
        "an empty preedit changed the rendered scene"
    );
}

#[test]
fn long_preedit_right_edge_stays_inside_the_scrolled_field() {
    for scale in SCALES {
        let mut h = Harness::new();
        let narrow = [0.0_f32, 0.0, 120.0, 40.0];
        h.field = narrow;
        let content_w = narrow[2] - 2.0 * h.padding;
        let mut state = composing("", 0, LONG_PREEDIT);
        let (fonts, style) = (&mut h.fonts, h.style.clone());
        acme_textinput::scroll_caret_into_view(&mut state, fonts, &style, scale, content_w);
        let scene = h.render(&mut state, scale);
        let runs = runs(&scene, scale);
        let composition = runs
            .iter()
            .find(|run| !run.glyphs.is_empty())
            .unwrap_or_else(|| panic!("no composition run drawn at scale {scale}"));
        let right = composition.ink_right();
        let visible_right = h.content_x() + content_w - 1.5;
        assert!(
            right <= visible_right + 1.0,
            "scale {scale}: composition ink ends at {right}, past the visible edge \
             {visible_right}"
        );
        assert!(
            right >= h.content_x(),
            "scale {scale}: composition ink ends at {right}, left of the content origin {}",
            h.content_x()
        );
    }
}
// ---------------------------------------------------------------------------
// End to end: the composition reaches the pixels
// ---------------------------------------------------------------------------

/// Render `scene` headlessly and return the `0x00RRGGBB` pixels.
fn raster(scene: &Scene, width: u32, height: u32, scale: f32) -> Vec<u32> {
    let mut renderer = acme_render_cpu::CpuRenderer::new(width, height, scale).expect("renderer");
    renderer.render_scene(scene).expect("scene renders");
    renderer.pixels().to_vec()
}

/// Dark (glyph) pixels inside `x_range` and the content rows.
fn ink_pixels(pixels: &[u32], width: u32, rows: u32, x_range: (u32, u32)) -> usize {
    let mut ink = 0;
    for y in 0..rows as usize {
        for x in x_range.0..x_range.1 {
            if (pixels[y * width as usize + x as usize] & 0xff) < 128 {
                ink += 1;
            }
        }
    }
    ink
}

#[test]
fn composition_reaches_the_cpu_rasterizer() {
    // Glyph ink can only appear inside the composition area when the composition
    // is drawn: without a preedit that range holds no text at all.
    let mut h = Harness::new();
    let w_committed = h.width(COMMITTED);
    let w_preedit = h.width(PREEDIT);
    let scale = 2.0;
    let (width, height) = (960u32, 160u32);
    // Only inside the field: outside it the surface is black, which would count
    // as ink.
    let rows = ((h.field[3] - 2.0 * h.padding) * scale) as u32;
    let range = (
        ((h.content_x() + w_committed) * scale) as u32 + 2,
        ((h.content_x() + w_committed + w_preedit) * scale) as u32 - 2,
    );

    let mut state = composing(COMMITTED, COMMITTED.len(), PREEDIT);
    state.focused = false;
    let pixels = raster(&h.render(&mut state, scale), width, height, scale);
    let with = ink_pixels(&pixels, width, rows, range);

    let mut plain_state = composing(COMMITTED, COMMITTED.len(), "");
    plain_state.focused = false;
    let plain = raster(&h.render(&mut plain_state, scale), width, height, scale);
    let without = ink_pixels(&plain, width, rows, range);

    assert_eq!(
        without, 0,
        "the control frame already inks {without} pixels in the composition area"
    );
    assert!(
        with >= 20,
        "the composition only inked {with} device pixels in {range:?}"
    );
}

/// Selection-highlight quads: height matches the content area, alpha 0.25.
fn highlights(scene: &Scene) -> Vec<(f32, f32)> {
    scene
        .commands()
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Quad(quad) if (quad.color.a - 0.25).abs() < EPS => {
                Some((quad.rect.origin.x.get(), quad.rect.size.width.get()))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn selection_before_the_composition_stays_unshifted() {
    // Selecting "a" puts the caret after it, so the composition is inserted at
    // the selection end and the highlight of "a" must not move.
    let mut h = Harness::new();
    let w_a = h.width("a");
    let mut state = composing("ab", 0, PREEDIT);
    state.set_selection_range(0, 1);
    assert_eq!(state.cursor(), 1, "selection places the caret at its end");
    let scene = h.render(&mut state, 1.0);
    let spans = highlights(&scene);
    assert_eq!(spans.len(), 1, "one highlight span, got {spans:?}");
    let (x, w) = spans[0];
    assert!(
        (x - h.content_x()).abs() < EPS,
        "highlight x {x} != content origin {}",
        h.content_x()
    );
    assert!((w - w_a).abs() < EPS, "highlight width {w} != 'a' {w_a}");

    // The composition and the shifted tail are drawn after it.
    let runs = runs(&scene, 1.0);
    let w_preedit = h.width(PREEDIT);
    run_at(&runs, h.content_x() + w_a, "composition run");
    run_at(&runs, h.content_x() + w_a + w_preedit, "trailing run");
}

#[test]
fn selection_highlight_unchanged_without_a_preedit() {
    let mut h = Harness::new();
    let w_a = h.width("a");
    let mut state = TextInputState::new();
    state.focused = true;
    state.text = "ab".to_string();
    state.set_selection_range(1, 2);
    let scene = h.render(&mut state, 1.0);
    let spans = highlights(&scene);
    assert_eq!(spans.len(), 1, "one highlight span, got {spans:?}");
    let (x, _) = spans[0];
    assert!(
        (x - (h.content_x() + w_a)).abs() < EPS,
        "highlight x {x} != content origin + 'a' {}",
        h.content_x() + w_a
    );
}

#[test]
fn composition_run_shares_the_committed_baseline() {
    // The composition must sit on the same baseline as the committed text: the
    // underline and the composition cursor assume one line box for all runs.
    let mut h = Harness::new();
    let mut state = composing(COMMITTED, COMMITTED.len(), PREEDIT);
    let scene = h.render(&mut state, 1.0);
    let origins = text_origins(&scene);
    assert!(origins.len() >= 2, "expected split runs, got {origins:?}");
    let ys: Vec<f32> = origins.iter().map(|(_, y)| *y).collect();
    assert!(
        ys.windows(2).all(|w| (w[0] - w[1]).abs() < EPS),
        "runs at different y: {ys:?}"
    );
}

/// `(x, y)` origin of every text run.
fn text_origins(scene: &Scene) -> Vec<(f32, f32)> {
    scene
        .commands()
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Text(text) => Some((text.origin.x.get(), text.origin.y.get())),
            _ => None,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Fix wave: F1 (backwards / cross-caret selection) and F2 (composition colour)
// ---------------------------------------------------------------------------

/// Text runs with their colour, for the colour assertions.
fn text_runs(scene: &Scene) -> Vec<(f32, f32, f32, f32, f32)> {
    scene
        .commands()
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Text(text) => Some((
                text.origin.x.get(),
                text.color.r,
                text.color.g,
                text.color.b,
                text.color.a,
            )),
            _ => None,
        })
        .collect()
}

/// A focused field with `text`, the caret at `caret`, a backwards selection of
/// `steps` graphemes and an active preedit. A backwards selection leaves the
/// caret at the selection *start*, which `set_selection_range` cannot express,
/// so the Shift+Arrow path is driven instead.
fn backwards_selection(text: &str, caret: usize, steps: usize, preedit: &str) -> TextInputState {
    use acme_platform::PlatformKey;
    let mut state = TextInputState::new();
    state.focused = true;
    state.text = text.to_string();
    state.move_cursor_to_offset(caret);
    for _ in 0..steps {
        assert!(
            acme_textinput::handle_key(
                &mut state,
                &PlatformKey::ArrowLeft,
                true,
                None,
                false,
                true
            ),
            "Shift+Left must extend the selection"
        );
    }
    state.set_preedit(preedit, None);
    state
}

#[test]
fn backwards_selection_caret_stays_at_the_selection_start() {
    // Precondition for the F1 tests below: the caret is left of the selection.
    let state = backwards_selection("abc", 3, 1, PREEDIT);
    assert_eq!(state.selection(), Some((2, 3)), "selected the last char");
    assert_eq!(
        state.cursor(),
        2,
        "a backwards selection puts the caret first"
    );
}

#[test]
fn backwards_selection_highlight_matches_the_tail_glyphs() {
    // F1: `tail_shift` already contains the committed prefix width, so adding
    // the whole-display prefix again shifted the highlight one prefix too far.
    for scale in SCALES {
        let mut h = Harness::new();
        let mut state = backwards_selection("abc", 3, 1, PREEDIT);
        let scene = h.render(&mut state, scale);
        let spans = highlights(&scene);
        assert_eq!(
            spans.len(),
            1,
            "scale {scale}: one highlight, got {spans:?}"
        );
        let (x, w) = spans[0];
        let runs = runs(&scene, scale);
        let w_prefix = h.width("ab");
        let w_c = h.width("c");
        let tail = run_at(
            &runs,
            h.content_x() + w_prefix + h.width(PREEDIT),
            "tail run",
        );
        assert!(
            (x - tail.origin_x).abs() < EPS,
            "scale {scale}: highlight x {x} != tail origin {}",
            tail.origin_x
        );
        assert!(
            (w - w_c).abs() < EPS,
            "scale {scale}: highlight width {w} != 'c' {w_c}"
        );
        let (c_left, c_right) = tail.glyphs[0];
        assert!(
            x <= c_left + EPS && x + w >= c_right - EPS,
            "scale {scale}: highlight [{x}, {}] misses the tail glyph [{c_left}, {c_right}]",
            x + w
        );
    }
}

#[test]
fn cross_caret_selection_hits_both_runs_once() {
    // `select_all` keeps the caret where it was, so selecting everything with
    // the caret in the middle puts the composition inside the selection: the
    // head run and the shifted tail run must each be highlighted once.
    for scale in SCALES {
        let mut h = Harness::new();
        let mut state = TextInputState::new();
        state.focused = true;
        state.text = "abc".to_string();
        state.move_cursor_to_offset(1);
        state.select_all();
        assert_eq!(state.selection(), Some((0, 3)));
        assert_eq!(state.cursor(), 1, "select_all keeps the caret");
        state.set_preedit(PREEDIT, None);

        let scene = h.render(&mut state, scale);
        let spans = highlights(&scene);
        assert_eq!(spans.len(), 2, "scale {scale}: two spans, got {spans:?}");
        let w_a = h.width("a");
        let w_bc = h.width("bc");
        let head = spans
            .iter()
            .find(|(x, _)| (x - h.content_x()).abs() < EPS)
            .unwrap_or_else(|| panic!("scale {scale}: no head span; spans {spans:?}"));
        let tail_x = h.content_x() + w_a + h.width(PREEDIT);
        let tail = spans
            .iter()
            .find(|(x, _)| (x - tail_x).abs() < EPS)
            .unwrap_or_else(|| panic!("scale {scale}: no tail span at {tail_x}; spans {spans:?}"));
        assert!(
            (head.1 - w_a).abs() < EPS,
            "scale {scale}: head span {} != 'a' {w_a}",
            head.1
        );
        assert!(
            (tail.1 - w_bc).abs() < EPS,
            "scale {scale}: tail span {} != 'bc' {w_bc}",
            tail.1
        );
    }
}

#[test]
fn placeholder_is_hidden_and_composition_uses_the_text_colour() {
    // F2: an empty field with a placeholder used to draw the composition in the
    // muted placeholder colour, because the colour was chosen from `is_empty`
    // before the composing branch was known.
    let mut h = Harness::new();
    let text_color = [
        h.theme.colors.foreground.red,
        h.theme.colors.foreground.green,
        h.theme.colors.foreground.blue,
        h.theme.colors.foreground.alpha,
    ];
    let muted = [
        h.theme.colors.muted_foreground.red,
        h.theme.colors.muted_foreground.green,
        h.theme.colors.muted_foreground.blue,
        h.theme.colors.muted_foreground.alpha,
    ];
    assert_ne!(text_color, muted, "theme tokens must differ for this test");

    let mut state = TextInputState::new();
    state.focused = true;
    state.placeholder = "type here".to_string();
    state.set_preedit(PREEDIT, None);
    let scene = h.render(&mut state, 1.0);
    let runs_with_color = text_runs(&scene);
    assert!(!runs_with_color.is_empty(), "the composition must be drawn");
    for (x, r, g, b, a) in &runs_with_color {
        let color = [*r, *g, *b, *a];
        assert!(
            color
                .iter()
                .zip(text_color.iter())
                .all(|(got, want)| (got - want).abs() < EPS),
            "run at x {x} used colour {color:?}, expected the text token {text_color:?}"
        );
    }
    // No glyph of the placeholder may be drawn.
    for run in runs(&scene, 1.0) {
        assert!(
            run.glyphs.len() <= PREEDIT.chars().count() + 2,
            "placeholder glyphs leaked into the scene: {} glyphs at x {}",
            run.glyphs.len(),
            run.origin_x
        );
    }
}

#[test]
fn placeholder_keeps_the_muted_colour_without_a_composition() {
    // Guard: the muted token is still used when the placeholder is drawn.
    let mut h = Harness::new();
    let muted = h.theme.colors.muted_foreground;
    let mut state = TextInputState::new();
    state.focused = true;
    state.placeholder = "type here".to_string();
    let scene = h.render(&mut state, 1.0);
    let runs_with_color = text_runs(&scene);
    assert_eq!(runs_with_color.len(), 1, "the placeholder is one run");
    let (_, r, g, b, a) = runs_with_color[0];
    assert!(
        (r - muted.red).abs() < EPS
            && (g - muted.green).abs() < EPS
            && (b - muted.blue).abs() < EPS
            && (a - muted.alpha).abs() < EPS,
        "placeholder colour {:?} != muted {:?}",
        (r, g, b, a),
        (muted.red, muted.green, muted.blue, muted.alpha)
    );
}

#[test]
fn readonly_composition_uses_the_disabled_text_token() {
    let mut h = Harness::new();
    let disabled = h.theme.colors.disabled_text;
    let mut state = TextInputState::new();
    state.focused = true;
    state.readonly = true;
    state.set_preedit(PREEDIT, None);
    let scene = h.render(&mut state, 1.0);
    let runs_with_color = text_runs(&scene);
    assert!(!runs_with_color.is_empty(), "the composition must be drawn");
    let (_, r, g, b, a) = runs_with_color[0];
    assert!(
        (r - disabled.red).abs() < EPS
            && (g - disabled.green).abs() < EPS
            && (b - disabled.blue).abs() < EPS
            && (a - disabled.alpha).abs() < EPS,
        "readonly composition colour {:?} != disabled {:?}",
        (r, g, b, a),
        (disabled.red, disabled.green, disabled.blue, disabled.alpha)
    );
}
