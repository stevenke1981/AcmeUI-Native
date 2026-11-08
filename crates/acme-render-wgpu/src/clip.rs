//! Exact pixel-area clip coverage and geometry preparation for the wgpu backend.
//!
//! Hardware scissor rects (`wgpu::RenderPass::set_scissor_rect`) operate on whole
//! physical pixels (`[floor(left), ceil(right)) × [floor(top), ceil(bottom))`).
//! To match the CPU renderer's fractional clip semantics (ROADMAP 015 / 016 P6):
//!
//! 1. [`physical_clip_bounds`] resolves a batch's logical clip `[x, y, w, h]` into
//!    physical viewport-clamped bounds `[left, top, right, bottom]` stored on each
//!    GPU [`Instance`] and [`GlyphInstance`].
//! 2. [`quad_raster_bounds`] expands a quad's vertex bounding box to the enclosing
//!    integer pixel grid `[floor(vis_min), ceil(vis_max)]` of `quad ∩ clip ∩ viewport`
//!    so 1x pixel-center rasterization invokes the fragment shader for every pixel
//!    whose unit square overlaps the clipped quad (including sub-pixel shapes whose
//!    bounds do not cover any `(x + 0.5, y + 0.5)` pixel center).
//! 3. `quad.wgsl` and [`clipped_quad_coverage`] compute the exact area of
//!    `(unit pixel) ∩ quad ∩ clip` for sharp and rounded quads (both outer fill and
//!    inner border), avoiding double attenuation on coincident edges.
//! 4. `text.wgsl` and [`rect_coverage`] multiply glyph and image alpha by the exact
//!    fractional `(unit pixel) ∩ clip` area mask.

use crate::instances::normalize_scale;
#[cfg(test)]
use crate::instances::{GlyphInstance, Instance};

/// Convert an optional logical clip rect `[x, y, width, height]` into physical
/// viewport-clamped bounds `[left, top, right, bottom]`.
///
/// When `clip` is `None`, returns the full physical viewport `[0.0, 0.0, width, height]`.
/// Non-finite, zero-sized, negative-sized, or off-viewport clips return `[0.0; 4]`.
pub(crate) fn physical_clip_bounds(
    clip: Option<[f32; 4]>,
    scale: f32,
    width: u32,
    height: u32,
) -> [f32; 4] {
    let scale = normalize_scale(scale);
    let vw = width as f32;
    let vh = height as f32;
    let Some(rect) = clip else {
        return [0.0, 0.0, vw, vh];
    };
    if rect.iter().any(|value| !value.is_finite()) || rect[2] <= 0.0 || rect[3] <= 0.0 {
        return [0.0; 4];
    }
    let left = (rect[0] * scale).clamp(0.0, vw);
    let top = (rect[1] * scale).clamp(0.0, vh);
    let right = ((rect[0] + rect[2]) * scale).clamp(0.0, vw);
    let bottom = ((rect[1] + rect[3]) * scale).clamp(0.0, vh);
    if right <= left || bottom <= top {
        [0.0; 4]
    } else {
        [left, top, right, bottom]
    }
}

/// Compute the expanded integer-aligned physical vertex bounds `([min_x, min_y], [max_x, max_y])`
/// for rasterizing a quad `rect = [x, y, w, h]` under `clip_bounds = [left, top, right, bottom]`
/// within `viewport = [vw, vh]`.
///
/// Mirrors the vertex-position expansion in `quad.wgsl` (`vs_main`).
#[cfg(test)]
pub(crate) fn quad_raster_bounds(
    rect: [f32; 4],
    clip_bounds: [f32; 4],
    viewport: [f32; 2],
) -> ([f32; 2], [f32; 2]) {
    let quad_min = [rect[0], rect[1]];
    let quad_max = [rect[0] + rect[2].max(0.0), rect[1] + rect[3].max(0.0)];
    let vis_min = [
        quad_min[0].max(clip_bounds[0]).max(0.0),
        quad_min[1].max(clip_bounds[1]).max(0.0),
    ];
    let vis_max = [
        quad_max[0].min(clip_bounds[2]).min(viewport[0]),
        quad_max[1].min(clip_bounds[3]).min(viewport[1]),
    ];
    if vis_max[0] > vis_min[0] && vis_max[1] > vis_min[1] {
        (
            [vis_min[0].floor(), vis_min[1].floor()],
            [vis_max[0].ceil(), vis_max[1].ceil()],
        )
    } else {
        ([0.0, 0.0], [0.0, 0.0])
    }
}

/// 1D overlap of the unit pixel `[center - 0.5, center + 0.5)` with `[lo, hi)`.
#[cfg(test)]
pub(crate) fn overlap(lo: f32, hi: f32, center: f32) -> f32 {
    ((center + 0.5).min(hi) - (center - 0.5).max(lo)).clamp(0.0, 1.0)
}

/// Exact area coverage of an axis-aligned rectangle `bounds = [left, top, right, bottom]`
/// at pixel center `center = [cx, cy]`.
#[cfg(test)]
pub(crate) fn rect_coverage(bounds: [f32; 4], center: [f32; 2]) -> f32 {
    if !(bounds[2] - bounds[0] > 0.0 && bounds[3] - bounds[1] > 0.0) {
        return 0.0;
    }
    overlap(bounds[0], bounds[2], center[0]) * overlap(bounds[1], bounds[3], center[1])
}

#[cfg(test)]
fn disc_rectangle_area(x0: f32, y0: f32, x1: f32, y1: f32, r: f32) -> f32 {
    let area = (x1 - x0) * (y1 - y0);
    let r2 = r * r;
    if x1 * x1 + y1 * y1 <= r2 {
        return area;
    }
    if x0 * x0 + y0 * y0 >= r2 {
        return 0.0;
    }
    let full_end = ((r - y1) * (r + y1)).max(0.0).sqrt().clamp(x0, x1);
    let end = ((r - y0) * (r + y0)).max(0.0).sqrt().clamp(full_end, x1);
    let start_y = ((r - full_end) * (r + full_end)).max(0.0).sqrt();
    let end_y = ((r - end) * (r + end)).max(0.0).sqrt();
    let width = end - full_end;
    let dy = start_y - end_y;
    let chord = (width * width + dy * dy).sqrt();
    let angle = 2.0 * (chord / (2.0 * r)).clamp(0.0, 1.0).asin();
    let segment = if angle < 0.1 {
        let a2 = angle * angle;
        angle * a2 * (1.0 / 6.0 - a2 * (1.0 / 120.0 - a2 * (1.0 / 5040.0)))
    } else {
        angle - angle.sin()
    };
    let covered = (full_end - x0) * (y1 - y0)
        + width * ((start_y - y0) + (end_y - y0)) * 0.5
        + 0.5 * r2 * segment;
    covered.clamp(0.0, area)
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn corner_removed_area(
    arc_x: f32,
    arc_y: f32,
    sx: f32,
    sy: f32,
    kcx: f32,
    kcy: f32,
    hx: f32,
    hy: f32,
    r: f32,
) -> f32 {
    let x0 = ((kcx - arc_x) * sx - hx).max(0.0);
    let x1 = ((kcx - arc_x) * sx + hx).min(r);
    let y0 = ((kcy - arc_y) * sy - hy).max(0.0);
    let y1 = ((kcy - arc_y) * sy + hy).min(r);
    if x0 >= x1 || y0 >= y1 {
        return 0.0;
    }
    (x1 - x0) * (y1 - y0) - disc_rectangle_area(x0, y0, x1, y1, r)
}

/// Exact pixel-area coverage of a quad `bounds = [left, top, right, bottom]` with corner
/// `radius`, intersected with `clip = [clip_left, clip_top, clip_right, clip_bottom]`,
/// at pixel center `center = [cx, cy]`.
///
/// Mirrors `clipped_quad_coverage` in `quad.wgsl`.
#[cfg(test)]
pub(crate) fn clipped_quad_coverage(
    bounds: [f32; 4],
    radius: f32,
    clip: [f32; 4],
    center: [f32; 2],
) -> f32 {
    let half_x = (bounds[2] - bounds[0]) * 0.5;
    let half_y = (bounds[3] - bounds[1]) * 0.5;
    if !(half_x > 0.0 && half_y > 0.0) {
        return 0.0;
    }
    if !(clip[2] - clip[0] > 0.0 && clip[3] - clip[1] > 0.0) {
        return 0.0;
    }
    let clipped_bounds = [
        bounds[0].max(clip[0]),
        bounds[1].max(clip[1]),
        bounds[2].min(clip[2]),
        bounds[3].min(clip[3]),
    ];
    let base = rect_coverage(clipped_bounds, center);
    if base <= 0.0 {
        return 0.0;
    }
    let r = radius.max(0.0).min(half_x.min(half_y));
    if r <= 0.0 {
        return base;
    }
    let kx0 = (center[0] - 0.5).max(clip[0]);
    let kx1 = (center[0] + 0.5).min(clip[2]);
    let ky0 = (center[1] - 0.5).max(clip[1]);
    let ky1 = (center[1] + 0.5).min(clip[3]);
    let kcx = 0.5 * (kx0 + kx1);
    let kcy = 0.5 * (ky0 + ky1);
    let hx = 0.5 * (kx1 - kx0);
    let hy = 0.5 * (ky1 - ky0);
    let removed =
        corner_removed_area(
            bounds[0] + r,
            bounds[1] + r,
            -1.0,
            -1.0,
            kcx,
            kcy,
            hx,
            hy,
            r,
        ) + corner_removed_area(bounds[2] - r, bounds[1] + r, 1.0, -1.0, kcx, kcy, hx, hy, r)
            + corner_removed_area(bounds[0] + r, bounds[3] - r, -1.0, 1.0, kcx, kcy, hx, hy, r)
            + corner_removed_area(bounds[2] - r, bounds[3] - r, 1.0, 1.0, kcx, kcy, hx, hy, r);
    (base - removed).clamp(0.0, base)
}

/// Evaluate the `quad.wgsl` fragment shader output (straight RGBA) for `instance`
/// at physical pixel center `center = [x + 0.5, y + 0.5]`.
#[cfg(test)]
pub(crate) fn shade_quad_fragment(instance: &Instance, center: [f32; 2]) -> [f32; 4] {
    let quad_bounds = [
        instance.rect[0],
        instance.rect[1],
        instance.rect[0] + instance.rect[2].max(0.0),
        instance.rect[1] + instance.rect[3].max(0.0),
    ];
    let radius = instance.extras[0];
    let border = instance.extras[1];
    let coverage = clipped_quad_coverage(quad_bounds, radius, instance.clip, center);
    if coverage <= 0.0 {
        return [0.0; 4];
    }
    if border > 0.0 {
        let inner_bounds = [
            quad_bounds[0] + border,
            quad_bounds[1] + border,
            quad_bounds[2] - border,
            quad_bounds[3] - border,
        ];
        let inner_radius = (radius - border).max(0.0);
        let inside = clipped_quad_coverage(inner_bounds, inner_radius, instance.clip, center);
        let stroke = (coverage - inside).max(0.0);
        let fill_a = instance.color[3] * inside;
        let stroke_a = instance.border_color[3] * stroke;
        let out_a = stroke_a + fill_a * (1.0 - stroke_a);
        if out_a <= 0.0 {
            return [0.0; 4];
        }
        let inv_a = 1.0 / out_a;
        [
            (instance.border_color[0] * stroke_a + instance.color[0] * fill_a * (1.0 - stroke_a))
                * inv_a,
            (instance.border_color[1] * stroke_a + instance.color[1] * fill_a * (1.0 - stroke_a))
                * inv_a,
            (instance.border_color[2] * stroke_a + instance.color[2] * fill_a * (1.0 - stroke_a))
                * inv_a,
            out_a,
        ]
    } else {
        [
            instance.color[0],
            instance.color[1],
            instance.color[2],
            instance.color[3] * coverage,
        ]
    }
}

/// Evaluate the `text.wgsl` fragment shader output (straight RGBA) for `instance`
/// given sampled texel `sample` at physical pixel center `center = [x + 0.5, y + 0.5]`.
#[cfg(test)]
pub(crate) fn shade_glyph_fragment(
    instance: &GlyphInstance,
    sample: [f32; 4],
    center: [f32; 2],
) -> [f32; 4] {
    let mask = rect_coverage(instance.clip, center);
    if instance.viewport_mode[2] > 0.5 {
        [
            sample[0] * instance.color[0],
            sample[1] * instance.color[1],
            sample[2] * instance.color[2],
            sample[3] * instance.color[3] * mask,
        ]
    } else {
        [
            instance.color[0],
            instance.color[1],
            instance.color[2],
            instance.color[3] * sample[0] * mask,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::{BatchPipeline, compile_scene};
    use crate::draws::scissor;
    use crate::instances::{
        collect_text_glyphs, image_primitive_to_instance, quad_primitive_to_instance,
    };
    use acme_core::{
        AtlasUpload, Color, DrawCommand, GlyphDraw, GlyphFormat, ImageFit, ImageKey,
        ImagePrimitive, Logical, Point, QuadPrimitive, Rect, Rgba8Image, Scene, TextPrimitive,
    };

    const SCALES: [f32; 5] = [1.0, 1.25, 1.5, 1.75, 2.0];

    fn validate_wgsl(source: &str, label: &str) {
        let module = wgpu::naga::front::wgsl::parse_str(source)
            .unwrap_or_else(|err| panic!("{label} WGSL parse failed: {err}"));
        let mut validator = wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::default(),
        );
        validator
            .validate(&module)
            .unwrap_or_else(|err| panic!("{label} WGSL validation failed: {err}"));
    }

    #[test]
    fn quad_and_text_wgsl_shaders_validate_with_naga() {
        validate_wgsl(include_str!("quad.wgsl"), "quad.wgsl");
        validate_wgsl(include_str!("text.wgsl"), "text.wgsl");
    }

    #[test]
    fn physical_clip_bounds_and_raster_bounds_handle_fractional_and_subpixel_geometry() {
        // Unclipped defaults to full physical viewport.
        assert_eq!(
            physical_clip_bounds(None, 1.5, 160, 120),
            [0.0, 0.0, 160.0, 120.0]
        );
        // Fractional logical clip scaled by 1.25.
        assert_eq!(
            physical_clip_bounds(Some([1.375, 1.375, 2.125, 2.125]), 1.25, 16, 16),
            [1.71875, 1.71875, 4.375, 4.375]
        );
        // Zero, negative, off-viewport, and NaN clips normalize to empty [0; 4].
        assert_eq!(
            physical_clip_bounds(Some([0.2, 0.2, 0.0, 0.5]), 1.0, 16, 16),
            [0.0; 4]
        );
        assert_eq!(
            physical_clip_bounds(Some([0.3, 0.3, -0.1, 0.5]), 1.0, 16, 16),
            [0.0; 4]
        );
        assert_eq!(
            physical_clip_bounds(Some([20.0, 0.0, 4.0, 4.0]), 1.0, 16, 16),
            [0.0; 4]
        );
        assert_eq!(
            physical_clip_bounds(Some([f32::NAN, 0.0, 4.0, 4.0]), 1.0, 16, 16),
            [0.0; 4]
        );

        // Subpixel clip [0.125, 0.125, 0.25, 0.25] has no pixel center inside it,
        // but quad_raster_bounds expands to [0.0, 0.0]..[1.0, 1.0] so pixel (0,0) is shaded.
        let clip_bounds = physical_clip_bounds(Some([0.125, 0.125, 0.25, 0.25]), 1.0, 16, 16);
        assert_eq!(
            quad_raster_bounds([0.0, 0.0, 16.0, 16.0], clip_bounds, [16.0, 16.0]),
            ([0.0, 0.0], [1.0, 1.0])
        );

        // Disjoint quad [0.0, 0.0, 0.25, 1.0] and clip [0.75, 0.0, 1.0, 1.0] collapse to empty.
        assert_eq!(
            quad_raster_bounds([0.0, 0.0, 0.25, 1.0], [0.75, 0.0, 1.0, 1.0], [16.0, 16.0]),
            ([0.0, 0.0], [0.0, 0.0])
        );
    }

    #[test]
    fn instance_preparation_carries_scaled_physical_clip_bounds() {
        let clip = Some([0.25, 1.5, 10.75, 20.125]);
        let quad = QuadPrimitive {
            rect: Rect::new(0.375, 0.625, 5.25, 4.75),
            color: Color::rgba(1.0, 0.0, 0.0, 1.0),
            radius: 1.25,
            border_width: 0.5,
            border_color: Color::rgba(0.0, 1.0, 0.0, 0.6),
        };
        let q_inst = quad_primitive_to_instance(&quad, clip, 2.0, 100, 100);
        assert_eq!(q_inst.clip, [0.5, 3.0, 22.0, 43.25]);

        let img = Rgba8Image::new(ImageKey::new(7), 0, 1, 1, vec![255, 0, 0, 255]).unwrap();
        let img_prim = ImagePrimitive {
            image: img,
            rect: Rect::new(0.0, 0.0, 16.0, 16.0),
            fit: ImageFit::Fill,
        };
        let img_inst = image_primitive_to_instance(&img_prim, clip, 2.0, 100, 100);
        assert_eq!(img_inst.clip, [0.5, 3.0, 22.0, 43.25]);
    }

    fn blend_alpha_channel(dst: u32, src_rgba: [f32; 4]) -> u32 {
        let sa = (src_rgba[3].clamp(0.0, 1.0) * 255.0).round();
        if sa <= 0.0 {
            return dst;
        }
        let sr = (src_rgba[0].clamp(0.0, 1.0) * 255.0).round();
        let sg = (src_rgba[1].clamp(0.0, 1.0) * 255.0).round();
        let sb = (src_rgba[2].clamp(0.0, 1.0) * 255.0).round();
        let da = ((dst >> 24) & 255) as f32;
        let out_a = sa + da * (1.0 - sa / 255.0);
        if out_a <= 0.0 {
            return 0;
        }
        let c = |s: f32, shift: u32| {
            let d = ((dst >> shift) & 255) as f32;
            ((s * sa + d * da * (1.0 - sa / 255.0)) / out_a).round() as u32
        };
        ((out_a.round() as u32) << 24) | (c(sr, 16) << 16) | (c(sg, 8) << 8) | c(sb, 0)
    }

    /// Simulate the wgpu render pass (batch compilation, scissor test, vertex raster bounds,
    /// WGSL fragment shader formulas, and alpha blending) on a 16x16 physical target.
    fn simulate_gpu_scene(commands: impl IntoIterator<Item = DrawCommand>, scale: f32) -> Vec<u32> {
        const W: u32 = 16;
        const H: u32 = 16;
        let mut scene = Scene::with_clear(Color::TRANSPARENT);
        for cmd in commands {
            scene.push(cmd);
        }
        let batches = compile_scene(&scene);
        let mut fb = vec![0u32; (W * H) as usize];

        for batch in &batches {
            let scissor_rect = match batch.clip {
                Some(c) => {
                    let Some(s) = scissor(c, scale, W, H) else {
                        continue;
                    };
                    s
                }
                None => [0, 0, W, H],
            };
            let (sx0, sy0, sx1, sy1) = (
                scissor_rect[0],
                scissor_rect[1],
                scissor_rect[0] + scissor_rect[2],
                scissor_rect[1] + scissor_rect[3],
            );

            match batch.pipeline {
                BatchPipeline::Quad => {
                    for idx in batch.command_start..batch.command_end {
                        let DrawCommand::Quad(prim) = &scene.commands()[idx] else {
                            continue;
                        };
                        let inst = quad_primitive_to_instance(prim, batch.clip, scale, W, H);
                        let (rmin, rmax) =
                            quad_raster_bounds(inst.rect, inst.clip, [W as f32, H as f32]);
                        for y in sy0..sy1 {
                            for x in sx0..sx1 {
                                let (cx, cy) = (x as f32 + 0.5, y as f32 + 0.5);
                                if cx >= rmin[0] && cx < rmax[0] && cy >= rmin[1] && cy < rmax[1] {
                                    let out = shade_quad_fragment(&inst, [cx, cy]);
                                    let p = (y * W + x) as usize;
                                    fb[p] = blend_alpha_channel(fb[p], out);
                                }
                            }
                        }
                    }
                }
                BatchPipeline::Image => {
                    for idx in batch.command_start..batch.command_end {
                        let DrawCommand::Image(prim) = &scene.commands()[idx] else {
                            continue;
                        };
                        let inst = image_primitive_to_instance(prim, batch.clip, scale, W, H);
                        let px = prim.image.pixels();
                        let sample = [
                            f32::from(px[0]) / 255.0,
                            f32::from(px[1]) / 255.0,
                            f32::from(px[2]) / 255.0,
                            f32::from(px[3]) / 255.0,
                        ];
                        for y in sy0..sy1 {
                            for x in sx0..sx1 {
                                let (cx, cy) = (x as f32 + 0.5, y as f32 + 0.5);
                                if cx >= inst.rect[0]
                                    && cx < inst.rect[0] + inst.rect[2]
                                    && cy >= inst.rect[1]
                                    && cy < inst.rect[1] + inst.rect[3]
                                {
                                    let out = shade_glyph_fragment(&inst, sample, [cx, cy]);
                                    let p = (y * W + x) as usize;
                                    fb[p] = blend_alpha_channel(fb[p], out);
                                }
                            }
                        }
                    }
                }
                BatchPipeline::TextAlpha | BatchPipeline::TextRgba => {
                    let format = if batch.pipeline == BatchPipeline::TextAlpha {
                        GlyphFormat::Alpha8
                    } else {
                        GlyphFormat::Rgba8
                    };
                    let glyphs = collect_text_glyphs(
                        &scene,
                        batch.command_start..batch.command_end,
                        format,
                        batch.clip,
                        scale,
                        W,
                        H,
                    );
                    let DrawCommand::Text(tprim) = &scene.commands()[batch.command_start] else {
                        continue;
                    };
                    let upload_px = &tprim.uploads[0].pixels;
                    let sample = if format == GlyphFormat::Alpha8 {
                        [f32::from(upload_px[0]) / 255.0, 0.0, 0.0, 1.0]
                    } else {
                        [
                            f32::from(upload_px[0]) / 255.0,
                            f32::from(upload_px[1]) / 255.0,
                            f32::from(upload_px[2]) / 255.0,
                            f32::from(upload_px[3]) / 255.0,
                        ]
                    };
                    for (_, inst) in glyphs {
                        for y in sy0..sy1 {
                            for x in sx0..sx1 {
                                let (cx, cy) = (x as f32 + 0.5, y as f32 + 0.5);
                                if cx >= inst.rect[0]
                                    && cx < inst.rect[0] + inst.rect[2]
                                    && cy >= inst.rect[1]
                                    && cy < inst.rect[1] + inst.rect[3]
                                {
                                    let out = shade_glyph_fragment(&inst, sample, [cx, cy]);
                                    let p = (y * W + x) as usize;
                                    fb[p] = blend_alpha_channel(fb[p], out);
                                }
                            }
                        }
                    }
                }
            }
        }

        fb.into_iter()
            .map(|pixel| {
                let a = (pixel >> 24) & 255;
                let r = (pixel >> 16) & 255;
                let g = (pixel >> 8) & 255;
                let b = pixel & 255;
                ((r * a / 255) << 16) | ((g * a / 255) << 8) | (b * a / 255)
            })
            .collect()
    }

    fn area(rect: Rect<Logical>, scale: f32, x: u32, y: u32) -> f32 {
        let left = rect.origin.x.get() * scale;
        let top = rect.origin.y.get() * scale;
        let right = left + rect.size.width.get() * scale;
        let bottom = top + rect.size.height.get() * scale;
        let x = x as f32;
        let y = y as f32;
        (right.min(x + 1.0) - left.max(x)).clamp(0.0, 1.0)
            * (bottom.min(y + 1.0) - top.max(y)).clamp(0.0, 1.0)
    }

    fn assert_quad_self_clip(radius: f32, border_width: f32) {
        let rect = Rect::new(0.375, 0.375, 5.25, 5.25);
        for scale in SCALES {
            for alpha in [128_u8, 255] {
                let quad = QuadPrimitive {
                    rect,
                    color: Color::rgba(1.0, 0.0, 0.0, f32::from(alpha) / 255.0),
                    radius,
                    border_width,
                    border_color: Color::rgba(0.0, 1.0, 0.0, 0.6),
                };
                let uncut = simulate_gpu_scene([DrawCommand::Quad(quad)], scale);
                assert!(
                    uncut
                        .iter()
                        .any(|pixel| *pixel != 0 && *pixel != 0x00ff_0000)
                );
                let clipped = simulate_gpu_scene(
                    [
                        DrawCommand::PushClip(rect),
                        DrawCommand::Quad(quad),
                        DrawCommand::PopClip,
                    ],
                    scale,
                );
                assert_eq!(
                    clipped, uncut,
                    "{scale}x alpha {alpha} radius {radius} border {border_width}"
                );
            }
        }
    }

    #[test]
    fn sharp_and_rounded_coincident_outer_clips_preserve_edges_at_every_dpi() {
        assert_quad_self_clip(0.0, 0.0);
        assert_quad_self_clip(0.0, 0.5);
        assert_quad_self_clip(1.25, 0.0);
        assert_quad_self_clip(1.25, 0.5);
    }

    #[test]
    fn sharp_and_rounded_border_clips_inside_flat_inner_have_only_exact_fill() {
        for radius in [0.0, 1.0] {
            let quad = QuadPrimitive {
                rect: Rect::new(0.375, 0.375, 5.25, 5.25),
                color: Color::rgba(1.0, 0.0, 0.0, 0.6),
                radius,
                border_width: 1.0,
                border_color: Color::rgba(0.0, 1.0, 0.0, 1.0),
            };
            for scale in SCALES {
                for clip in [
                    Rect::new(1.375, 1.375, 3.25, 3.25),
                    Rect::new(2.375, 2.375, 1.25, 1.25),
                ] {
                    let clipped = simulate_gpu_scene(
                        [
                            DrawCommand::PushClip(clip),
                            DrawCommand::Quad(quad),
                            DrawCommand::PopClip,
                        ],
                        scale,
                    );
                    let expected = simulate_gpu_scene(
                        [DrawCommand::Quad(QuadPrimitive {
                            rect: clip,
                            radius: 0.0,
                            border_width: 0.0,
                            ..quad
                        })],
                        scale,
                    );
                    assert_eq!(clipped, expected, "radius {radius} {scale}x clip {clip:?}");
                }
            }
        }
    }

    #[test]
    fn sharp_border_inner_coincident_clip_does_not_add_a_stroke_seam() {
        let pixels = simulate_gpu_scene(
            [
                DrawCommand::PushClip(Rect::new(0.5, 0.5, 3.0, 3.0)),
                DrawCommand::Quad(QuadPrimitive {
                    rect: Rect::new(0.0, 0.0, 4.0, 4.0),
                    color: Color::rgba(1.0, 0.0, 0.0, 1.0),
                    radius: 0.0,
                    border_width: 0.5,
                    border_color: Color::rgba(0.0, 1.0, 0.0, 1.0),
                }),
                DrawCommand::PopClip,
            ],
            1.0,
        );
        assert_eq!(pixels[16], 0x0080_0000);
        assert_eq!(pixels[0], 0x0040_0000);
        assert_eq!(pixels[17], 0x00ff_0000);
    }

    #[test]
    fn rounded_shape_area_and_clip_area_are_integrated_analytically() {
        let quad = QuadPrimitive {
            rect: Rect::new(0.0, 0.0, 2.0, 2.0),
            color: Color::rgba(1.0, 0.0, 0.0, 1.0),
            radius: 1.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        };
        let right_half = simulate_gpu_scene(
            [
                DrawCommand::PushClip(Rect::new(0.5, 0.0, 1.5, 2.0)),
                DrawCommand::Quad(quad),
                DrawCommand::PopClip,
            ],
            1.0,
        );
        let exact_right = std::f32::consts::PI / 12.0 + 3.0_f32.sqrt() / 8.0;
        assert_eq!(right_half[0], ((255.0 * exact_right).round() as u32) << 16);

        let left_half = simulate_gpu_scene(
            [
                DrawCommand::PushClip(Rect::new(0.0, 0.0, 0.5, 2.0)),
                DrawCommand::Quad(quad),
                DrawCommand::PopClip,
            ],
            1.0,
        );
        let exact_left = std::f32::consts::FRAC_PI_4 - exact_right;
        assert_eq!(left_half[0], ((255.0 * exact_left).round() as u32) << 16);
    }

    #[test]
    fn subpixel_clip_and_disjoint_subpixel_quad_match_cpu_semantics() {
        let clip = Rect::new(0.125, 0.125, 0.25, 0.25);
        let pixels = simulate_gpu_scene(
            [
                DrawCommand::PushClip(clip),
                DrawCommand::Quad(QuadPrimitive {
                    rect: Rect::new(0.0, 0.0, 16.0, 16.0),
                    color: Color::rgba(1.0, 0.0, 0.0, 1.0),
                    radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }),
                DrawCommand::PopClip,
            ],
            1.0,
        );
        assert_eq!(pixels[0], 0x0010_0000);
        assert!(pixels[1..].iter().all(|pixel| *pixel == 0));

        for radius in [0.0, 0.125] {
            let disjoint = simulate_gpu_scene(
                [
                    DrawCommand::PushClip(Rect::new(0.75, 0.0, 0.25, 1.0)),
                    DrawCommand::Quad(QuadPrimitive {
                        rect: Rect::new(0.0, 0.0, 0.25, 1.0),
                        color: Color::rgba(1.0, 0.0, 0.0, 1.0),
                        radius,
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    }),
                    DrawCommand::PopClip,
                ],
                1.0,
            );
            assert!(disjoint.iter().all(|pixel| *pixel == 0));
        }
    }

    #[test]
    fn fractional_clips_on_quad_image_alpha8_and_rgba8_match_cpu_at_every_dpi() {
        let clip = Rect::new(1.375, 1.375, 2.125, 2.125);
        for scale in SCALES {
            for alpha in [128_u8, 255] {
                let quad_cmd = DrawCommand::Quad(QuadPrimitive {
                    rect: Rect::new(0.0, 0.0, 16.0, 16.0),
                    color: Color::rgba(1.0, 0.0, 0.0, f32::from(alpha) / 255.0),
                    radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                });
                let image_cmd = DrawCommand::Image(ImagePrimitive {
                    image: Rgba8Image::new(ImageKey::new(1), 0, 1, 1, vec![255, 0, 0, alpha])
                        .unwrap(),
                    rect: Rect::new(0.0, 0.0, 16.0, 16.0),
                    fit: ImageFit::Fill,
                });
                let alpha8_cmd = DrawCommand::Text(TextPrimitive {
                    origin: Point::new(0.0, 0.0),
                    color: Color::rgba(1.0, 0.0, 0.0, 1.0),
                    glyphs: vec![GlyphDraw {
                        x: 0.0,
                        y: 0.0,
                        width: 16.0,
                        height: 16.0,
                        atlas_x: 0,
                        atlas_y: 0,
                        atlas_page: 0,
                        format: GlyphFormat::Alpha8,
                    }],
                    uploads: vec![AtlasUpload {
                        page: 0,
                        origin: [0, 0],
                        size: [16, 16],
                        format: GlyphFormat::Alpha8,
                        pixels: vec![alpha; 16 * 16],
                    }],
                });
                let rgba8_cmd = DrawCommand::Text(TextPrimitive {
                    origin: Point::new(0.0, 0.0),
                    color: Color::rgba(1.0, 1.0, 1.0, 1.0),
                    glyphs: vec![GlyphDraw {
                        x: 0.0,
                        y: 0.0,
                        width: 16.0,
                        height: 16.0,
                        atlas_x: 0,
                        atlas_y: 0,
                        atlas_page: 0,
                        format: GlyphFormat::Rgba8,
                    }],
                    uploads: vec![AtlasUpload {
                        page: 0,
                        origin: [0, 0],
                        size: [16, 16],
                        format: GlyphFormat::Rgba8,
                        pixels: [255, 0, 0, alpha].repeat(16 * 16),
                    }],
                });

                for (label, cmd) in [
                    ("quad", quad_cmd),
                    ("image", image_cmd),
                    ("alpha8", alpha8_cmd),
                    ("rgba8", rgba8_cmd),
                ] {
                    let pixels = simulate_gpu_scene(
                        [DrawCommand::PushClip(clip), cmd, DrawCommand::PopClip],
                        scale,
                    );
                    for y in 0..16 {
                        for x in 0..16 {
                            let expected =
                                ((f32::from(alpha) * area(clip, scale, x, y)).round() as u32) << 16;
                            assert_eq!(
                                pixels[(y * 16 + x) as usize],
                                expected,
                                "{label} {scale}x alpha {alpha} ({x},{y})"
                            );
                        }
                    }
                }
            }
        }
    }
}
