//! GPU instance layouts and the conversions from scene primitives into instances.

use acme_core::{
    ATLAS_MAX_PAGES, ATLAS_PAGE_EDGE, DrawCommand, GlyphFormat as SceneGlyphFormat, ImagePrimitive,
    Scene,
};
use bytemuck::{Pod, Zeroable};

use crate::clip::physical_clip_bounds;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct Instance {
    pub(crate) rect: [f32; 4],
    pub(crate) color: [f32; 4],
    pub(crate) border_color: [f32; 4],
    pub(crate) extras: [f32; 4],
    pub(crate) clip: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct GlyphInstance {
    pub(crate) rect: [f32; 4],
    pub(crate) uv: [f32; 4],
    pub(crate) color: [f32; 4],
    pub(crate) viewport_mode: [f32; 4],
    pub(crate) clip: [f32; 4],
}

pub(crate) fn normalize_scale(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}
pub(crate) fn normalize_color(mut color: [f32; 4]) -> [f32; 4] {
    for value in &mut color {
        *value = if value.is_finite() {
            value.clamp(0.0, 1.0)
        } else {
            0.0
        };
    }
    color
}

/// Convert a [`QuadPrimitive`] (logical pixels, acme_core types) into a GPU
/// [`Instance`] by applying scale factor, physical clip bounds, and normalizing colors.
pub(crate) fn quad_primitive_to_instance(
    prim: &acme_core::QuadPrimitive,
    clip: Option<[f32; 4]>,
    scale: f32,
    config_w: u32,
    config_h: u32,
) -> Instance {
    let scale = normalize_scale(scale);
    let clean = |value: f32| if value.is_finite() { value } else { 0.0 };
    Instance {
        rect: [
            clean(prim.rect.origin.x.get()) * scale,
            clean(prim.rect.origin.y.get()) * scale,
            clean(prim.rect.size.width.get()).max(0.0) * scale,
            clean(prim.rect.size.height.get()).max(0.0) * scale,
        ],
        color: normalize_color([prim.color.r, prim.color.g, prim.color.b, prim.color.a]),
        border_color: normalize_color([
            prim.border_color.r,
            prim.border_color.g,
            prim.border_color.b,
            prim.border_color.a,
        ]),
        extras: [
            clean(prim.radius).max(0.0) * scale,
            clean(prim.border_width).max(0.0) * scale,
            config_w as f32,
            config_h as f32,
        ],
        clip: physical_clip_bounds(clip, scale, config_w, config_h),
    }
}

pub(crate) fn image_primitive_to_instance(
    primitive: &ImagePrimitive,
    clip: Option<[f32; 4]>,
    scale: f32,
    width: u32,
    height: u32,
) -> GlyphInstance {
    let scale = normalize_scale(scale);
    let rect = primitive.fitted_rect();
    GlyphInstance {
        rect: [
            rect.origin.x.get() * scale,
            rect.origin.y.get() * scale,
            rect.size.width.get().max(0.0) * scale,
            rect.size.height.get().max(0.0) * scale,
        ],
        uv: [0.0, 0.0, 1.0, 1.0],
        color: [1.0; 4],
        viewport_mode: [width as f32, height as f32, 1.0, 0.0],
        clip: physical_clip_bounds(clip, scale, width, height),
    }
}

/// Extracts `(atlas_page, GlyphInstance)` pairs from a [`TextPrimitive`] for
/// `format`, in glyph order. Applies origin offset, scale, and physical clip bounds.
///
/// Glyphs whose `atlas_page` is `>= ATLAS_MAX_PAGES` are skipped rather than
/// clamped: clamping would sample another page's texels. UVs are normalized by
/// [`ATLAS_PAGE_EDGE`].
pub(crate) fn text_primitive_glyphs(
    prim: &acme_core::TextPrimitive,
    format: SceneGlyphFormat,
    clip: Option<[f32; 4]>,
    scale: f32,
    width: u32,
    height: u32,
) -> Vec<(u32, GlyphInstance)> {
    let scale = normalize_scale(scale);
    let color = normalize_color([prim.color.r, prim.color.g, prim.color.b, prim.color.a]);
    let clip_bounds = physical_clip_bounds(clip, scale, width, height);
    let edge = ATLAS_PAGE_EDGE as f32;
    let is_rgba = format == SceneGlyphFormat::Rgba8;
    prim.glyphs
        .iter()
        .filter(|glyph| glyph.format == format && glyph.atlas_page < ATLAS_MAX_PAGES)
        .map(|glyph| {
            let instance = GlyphInstance {
                rect: [
                    glyph.x + prim.origin.x.get() * scale,
                    glyph.y + prim.origin.y.get() * scale,
                    glyph.width,
                    glyph.height,
                ],
                uv: [
                    glyph.atlas_x as f32 / edge,
                    glyph.atlas_y as f32 / edge,
                    glyph.width / edge,
                    glyph.height / edge,
                ],
                color,
                viewport_mode: [
                    width as f32,
                    height as f32,
                    if is_rgba { 1.0 } else { 0.0 },
                    0.0,
                ],
                clip: clip_bounds,
            };
            (glyph.atlas_page, instance)
        })
        .collect()
}

/// Collects the glyphs of every text command in `range` for `format`, in
/// painter order.
pub(crate) fn collect_text_glyphs(
    scene: &Scene,
    range: std::ops::Range<usize>,
    format: SceneGlyphFormat,
    clip: Option<[f32; 4]>,
    scale: f32,
    width: u32,
    height: u32,
) -> Vec<(u32, GlyphInstance)> {
    range
        .filter_map(|index| match &scene.commands()[index] {
            DrawCommand::Text(prim) => Some(text_primitive_glyphs(
                prim, format, clip, scale, width, height,
            )),
            _ => None,
        })
        .flatten()
        .collect()
}
