//! Draw batch descriptors, scissor rectangles and per-page glyph draw splitting.

use acme_core::GlyphFormat as SceneGlyphFormat;

use crate::atlas_pages::page_runs;
use crate::images::ImageResourceKey;
use crate::instances::GlyphInstance;

/// Quad draw covering `count` instances starting at `start`.
pub(crate) struct QuadBatchDesc {
    pub(crate) clip: Option<[f32; 4]>,
    pub(crate) start: u32,
    pub(crate) count: u32,
}

/// Glyph draw: one atlas page, one format, contiguous instances.
pub(crate) struct GlyphBatchDesc {
    pub(crate) format: SceneGlyphFormat,
    pub(crate) page: u32,
    pub(crate) clip: Option<[f32; 4]>,
    pub(crate) start: u32,
    pub(crate) count: u32,
}

/// Image draw for one resident RGBA8 resource.
pub(crate) struct ImageBatchDesc {
    pub(crate) key: ImageResourceKey,
    pub(crate) clip: Option<[f32; 4]>,
    pub(crate) start: u32,
    pub(crate) count: u32,
}

pub(crate) enum DrawBatchDesc {
    Quad(QuadBatchDesc),
    Glyph(GlyphBatchDesc),
    Image(ImageBatchDesc),
}

pub(crate) fn scissor(rect: [f32; 4], scale: f32, width: u32, height: u32) -> Option<[u32; 4]> {
    if rect.iter().any(|value| !value.is_finite()) {
        return None;
    }
    let x = (rect[0].max(0.0) * scale).floor().clamp(0.0, width as f32) as u32;
    let y = (rect[1].max(0.0) * scale).floor().clamp(0.0, height as f32) as u32;
    let right = ((rect[0] + rect[2].max(0.0)) * scale)
        .ceil()
        .clamp(0.0, width as f32) as u32;
    let bottom = ((rect[1] + rect[3].max(0.0)) * scale)
        .ceil()
        .clamp(0.0, height as f32) as u32;
    (right > x && bottom > y).then_some([x, y, right - x, bottom - y])
}

/// Appends one text batch's glyphs to `instances` and records one draw per
/// maximal same-page run. Instances stay contiguous and in painter order.
pub(crate) fn push_text_batch(
    glyphs: Vec<(u32, GlyphInstance)>,
    format: SceneGlyphFormat,
    clip: Option<[f32; 4]>,
    instances: &mut Vec<GlyphInstance>,
    draws: &mut Vec<DrawBatchDesc>,
) {
    let base = instances.len();
    let runs = page_runs(glyphs.iter().map(|(page, _)| *page));
    instances.extend(glyphs.into_iter().map(|(_, glyph)| glyph));
    draws.extend(runs.into_iter().map(|run| {
        DrawBatchDesc::Glyph(GlyphBatchDesc {
            format,
            page: run.page,
            clip,
            start: (base + run.start) as u32,
            count: run.len as u32,
        })
    }));
}
