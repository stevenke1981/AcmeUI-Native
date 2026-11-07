//! IME preedit rendering for [`TextInputState`].
//!
//! A composition is drawn as its own text run, placed after the committed text
//! that precedes the caret, with the committed text after the caret shifted by
//! the composition width. Password fields draw the composition masked, exactly
//! like the committed text, so a password field never shows composed plaintext.
//!
//! Geometry is shared with the rest of the widget: `PreeditRuns::x` is the same
//! content-local offset `ime_caret_area` reports, and the widths come from the
//! same string `scroll_caret_into_view` measures, so the glyphs, the underline,
//! the composition cursor and the scroll all agree.
//!
//! An empty preedit takes no part in this module: the caller keeps its single-run
//! path unchanged.

use acme_core::{
    AtlasUpload as SceneAtlasUpload, Color, DrawCommand, GlyphDraw, GlyphFormat, Point,
    QuadPrimitive, Rect, Scene, TextPrimitive,
};
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextStyle, TextWrap};
use unicode_segmentation::UnicodeSegmentation;

use crate::{CARET_WIDTH, TextInputState, byte_offset_to_x_inner, floor_char_boundary};

/// A composition split into the three runs the widget draws.
pub(crate) struct PreeditRuns {
    /// Committed display text before the caret.
    pub before: String,
    /// The preedit as it is drawn: mask characters in a password field.
    pub preedit: String,
    /// Committed display text from the caret on.
    pub after: String,
    /// Content-local x of the composition start (the committed caret).
    pub x: f32,
}

/// Split `state` into its preedit runs, or `None` when no preedit is active.
pub(crate) fn preedit_runs(
    state: &TextInputState,
    display: &str,
    fonts: &mut FontSystem,
    style: &TextStyle,
    scale: f32,
) -> Option<PreeditRuns> {
    if state.preedit.is_empty() {
        return None;
    }
    let caret = floor_char_boundary(display, state.display_cursor());
    Some(PreeditRuns {
        before: display[..caret].to_string(),
        preedit: preedit_text(state),
        after: display[caret..].to_string(),
        x: byte_offset_to_x_inner(display, caret, fonts, style, scale),
    })
}

/// Where the selection highlight must be drawn, as `(x, width)` pairs in
/// content-local logical pixels.
///
/// Without a composition this is the single span of the one text run. With a
/// composition the display text is drawn as up to three runs (text before the
/// caret, the composition at the caret, the rest pushed right by the composition
/// width), so the highlight is emitted per run: bytes before the caret keep their
/// place and bytes from the caret on are offset by the composition width only —
/// their own advance already contains the committed prefix. The composition
/// itself is never inside a selection, because every selection API leaves the
/// caret at one end of the selection and the composition is inserted at the caret.
pub(crate) fn selection_spans(
    state: &TextInputState,
    display: &str,
    runs: Option<&PreeditRuns>,
    fonts: &mut FontSystem,
    style: &TextStyle,
    scale: f32,
) -> Vec<(f32, f32)> {
    let Some((sel_start, sel_end)) = state.selection else {
        return Vec::new();
    };
    let sel_start = floor_char_boundary(display, sel_start.min(display.len()));
    let sel_end = floor_char_boundary(display, sel_end.min(display.len()));
    if sel_start >= sel_end {
        return Vec::new();
    }
    if runs.is_none() {
        return span(display, sel_start, sel_end, 0.0, fonts, style, scale)
            .into_iter()
            .collect();
    }
    let caret = floor_char_boundary(display, state.display_cursor());
    // Only the composition pushes the tail right; the committed prefix width is
    // already part of every tail byte's own offset (a `runs.x` here would shift
    // the highlight one prefix too far).
    let tail_shift = preedit_width(state, fonts, style, scale);
    let mut spans = Vec::with_capacity(2);
    spans.extend(span(
        display,
        sel_start,
        sel_end.min(caret),
        0.0,
        fonts,
        style,
        scale,
    ));
    spans.extend(span(
        display,
        sel_start.max(caret),
        sel_end,
        tail_shift,
        fonts,
        style,
        scale,
    ));
    spans
}

/// The highlight covered by `selection` inside `run` (`[start, end)` display
/// bytes), placed at `shift` plus the run-relative offset.
fn span(
    display: &str,
    start: usize,
    end: usize,
    shift: f32,
    fonts: &mut FontSystem,
    style: &TextStyle,
    scale: f32,
) -> Option<(f32, f32)> {
    if start >= end {
        return None;
    }
    let before = byte_offset_to_x_inner(display, start, fonts, style, scale);
    let through = byte_offset_to_x_inner(display, end, fonts, style, scale);
    let width = through - before;
    if width <= 0.0 {
        return None;
    }
    Some((shift + before, width))
}

/// The composition as drawn: the mask for password fields, else the preedit.
///
/// Masking is the safer option. A password field already hides its committed
/// text, so drawing the raw composition would reveal the characters the user is
/// about to commit. One mask character per preedit grapheme keeps the head/tail
/// split and the composition cursor proportional to the drawn run.
pub(crate) fn preedit_text(state: &TextInputState) -> String {
    if !state.password {
        return state.preedit.clone();
    }
    let count = state.preedit.graphemes(true).count();
    let mut out = String::with_capacity(count * state.password_char.len_utf8());
    for _ in 0..count {
        out.push(state.password_char);
    }
    out
}

/// Byte offset into [`preedit_text`] for a byte offset into `state.preedit`.
///
/// Identity for plain fields; for password fields the offset is re-counted in
/// mask characters, like `display_cursor` does for the committed text. The
/// offset is clamped to a char boundary first, so a mid-char or out-of-range
/// platform cursor cannot slice inside a character.
pub(crate) fn drawn_offset(state: &TextInputState, plain: usize) -> usize {
    if !state.password {
        return plain.min(state.preedit.len());
    }
    let plain = floor_char_boundary(&state.preedit, plain);
    state.preedit[..plain].graphemes(true).count() * state.password_char.len_utf8()
}

/// Width of the drawn composition in logical pixels.
pub(crate) fn preedit_width(
    state: &TextInputState,
    fonts: &mut FontSystem,
    style: &TextStyle,
    scale: f32,
) -> f32 {
    if state.preedit.is_empty() {
        return 0.0;
    }
    // The composition is never wrapped: measure it unconstrained so the underline
    // and the shifted tail use its real advance width.
    fonts
        .shape(
            &preedit_text(state),
            style,
            TextConstraints::default(),
            scale,
        )
        .width
}

/// Push the committed text and the composition as separate runs.
///
/// The text after the caret is shifted by the composition width, so it sits
/// where it will be once the preedit is committed.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_composing_text(
    scene: &mut Scene,
    state: &TextInputState,
    runs: &PreeditRuns,
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
    style: &TextStyle,
    scale: f32,
    text_origin_x: f32,
    text_y: f32,
    color: [f32; 4],
    clip: [f32; 4],
) {
    let w_preedit = preedit_width(state, fonts, style, scale);
    // Absolute x of the composition start and of the shifted tail.
    let composition_x = text_origin_x + runs.x;
    let mut items = Vec::with_capacity(3);
    if !runs.before.is_empty() {
        items.push((runs.before.as_str(), text_origin_x));
    }
    items.push((runs.preedit.as_str(), composition_x));
    if !runs.after.is_empty() {
        items.push((runs.after.as_str(), composition_x + w_preedit));
    }

    scene.push(clip_rect(clip));
    for (text, origin_x) in items {
        let (glyphs, uploads) = prepare_run(fonts, atlas, text, style, scale);
        scene.push(DrawCommand::Text(TextPrimitive {
            origin: Point::new(origin_x, text_y),
            color: Color::rgba(color[0], color[1], color[2], color[3]),
            glyphs,
            uploads,
        }));
    }
    scene.push(DrawCommand::PopClip);
}

/// Push the preedit underline and the composition-cursor segment.
///
/// Both line up with the composition run: the underline spans the composition
/// width from `PreeditRuns::x`, and the thick segment covers the composition
/// cursor, or the caret when the platform reports no cursor inside the preedit.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_preedit_underline(
    scene: &mut Scene,
    state: &TextInputState,
    runs: &PreeditRuns,
    fonts: &mut FontSystem,
    style: &TextStyle,
    scale: f32,
    text_origin_x: f32,
    underline_y: f32,
    color: [f32; 4],
    clip: [f32; 4],
) {
    let w_preedit = preedit_width(state, fonts, style, scale);
    if w_preedit <= 0.0 {
        return;
    }
    let x = text_origin_x + runs.x;
    let (offset, seg_w) = composition_segment(state, runs, fonts, style, scale);
    push_quad(
        scene,
        Rect::new(x, underline_y, w_preedit, 1.0),
        color,
        clip,
    );
    push_quad(
        scene,
        Rect::new(x + offset, underline_y - 1.0, seg_w, 3.0),
        color,
        clip,
    );
}

/// Offset from the composition start and width of the composition cursor.
fn composition_segment(
    state: &TextInputState,
    runs: &PreeditRuns,
    fonts: &mut FontSystem,
    style: &TextStyle,
    scale: f32,
) -> (f32, f32) {
    let len = runs.preedit.len();
    let Some((pc_start, pc_end)) = state.preedit_cursor else {
        let end = byte_offset_to_x_inner(&runs.preedit, len, fonts, style, scale);
        return (end, CARET_WIDTH);
    };
    let start = byte_offset_to_x_inner(
        &runs.preedit,
        drawn_offset(state, pc_start),
        fonts,
        style,
        scale,
    );
    let end = byte_offset_to_x_inner(
        &runs.preedit,
        drawn_offset(state, pc_end),
        fonts,
        style,
        scale,
    );
    (start, (end - start).max(CARET_WIDTH))
}

/// Shape and rasterise one run into scene glyphs and atlas uploads.
fn prepare_run(
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
    text: &str,
    style: &TextStyle,
    scale: f32,
) -> (Vec<GlyphDraw>, Vec<SceneAtlasUpload>) {
    let constraints = TextConstraints {
        max_width: None,
        wrap: TextWrap::None,
    };
    let layout = fonts.shape(text, style, constraints, scale);
    let prepared = fonts.prepare(&layout, atlas);
    let uploads = prepared
        .uploads
        .iter()
        .map(|region| SceneAtlasUpload {
            page: region.page,
            origin: [region.x, region.y],
            size: [region.width, region.height],
            format: glyph_format(region.format),
            pixels: region.pixels.clone(),
        })
        .collect();
    let glyphs = prepared
        .glyphs
        .iter()
        .map(|glyph| GlyphDraw {
            x: glyph.x as f32,
            y: glyph.y as f32,
            width: glyph.width as f32,
            height: glyph.height as f32,
            atlas_x: glyph.atlas_x,
            atlas_y: glyph.atlas_y,
            atlas_page: glyph.atlas_page,
            format: glyph_format(glyph.format),
        })
        .collect();
    (glyphs, uploads)
}

fn glyph_format(format: acme_text::AtlasFormat) -> GlyphFormat {
    match format {
        acme_text::AtlasFormat::Alpha8 => GlyphFormat::Alpha8,
        acme_text::AtlasFormat::Rgba8 => GlyphFormat::Rgba8,
    }
}

fn clip_rect(clip: [f32; 4]) -> DrawCommand {
    DrawCommand::PushClip(Rect::new(clip[0], clip[1], clip[2], clip[3]))
}

fn push_quad(scene: &mut Scene, rect: Rect<acme_core::Logical>, color: [f32; 4], clip: [f32; 4]) {
    scene.push(clip_rect(clip));
    scene.push(DrawCommand::Quad(QuadPrimitive {
        rect,
        color: Color::rgba(color[0], color[1], color[2], color[3]),
        radius: 0.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    }));
    scene.push(DrawCommand::PopClip);
}
