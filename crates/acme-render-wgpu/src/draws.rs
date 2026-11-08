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
    let [left, top, right, bottom] =
        crate::clip::physical_clip_bounds(Some(rect), scale, width, height);
    if right <= left || bottom <= top {
        return None;
    }
    let x = left.floor() as u32;
    let y = top.floor() as u32;
    let r = right.ceil() as u32;
    let b = bottom.ceil() as u32;
    (r > x && b > y).then_some([x, y, r - x, b - y])
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
