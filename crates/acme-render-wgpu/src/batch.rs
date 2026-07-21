//! Adjacent batch compiler for deterministic ordered rendering.
//!
//! Walks a [`Scene`]'s [`DrawCommand`] list in painter's order, produces
//! [`RenderBatch`] descriptors grouped by pipeline kind and clip rect.
//! Adjacent commands with the same pipeline and clip are merged into a
//! single batch — **no merging across clip or layer boundaries**.
//!
//! Also provides [`scene_from_frame`] to bridge the legacy [`Frame`] format.

use crate::Frame;
use acme_core::{ClipStack, DrawCommand, GlyphFormat, Scene};

/// The GPU pipeline used to draw a batch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatchPipeline {
    /// Solid-colour quads (rounded rect / border).
    Quad,
    /// Alpha8 text glyphs (mask atlas).
    TextAlpha,
    /// RGBA text glyphs (colour emoji atlas).
    TextRgba,
}

/// A batch of adjacent draw commands sharing the same pipeline and clip rect.
///
/// `command_start` / `command_end` are indices into the original
/// [`Scene`]'s command list.  The renderer later iterates these
/// ranges to build GPU instance data.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderBatch {
    pub pipeline: BatchPipeline,
    /// Physical-pixel scissor rect `[x, y, w, h]`, or `None` for full viewport.
    pub clip: Option<[u32; 4]>,
    /// Layer depth (currently always 0; future composited-layer support).
    pub layer: u32,
    /// Index of the first draw command in this batch (within Scene.commands).
    pub command_start: usize,
    /// One-past-the-last draw command index.
    pub command_end: usize,
}

impl RenderBatch {
    fn new(pipeline: BatchPipeline, clip: Option<[u32; 4]>, layer: u32, cmd_idx: usize) -> Self {
        Self {
            pipeline,
            clip,
            layer,
            command_start: cmd_idx,
            command_end: cmd_idx + 1,
        }
    }

    /// Try to extend this batch with the next command.  Returns `true` if
    /// the command was compatible and the batch was extended, `false` if
    /// the merge was rejected (caller must flush and start a new batch).
    #[allow(dead_code)]
    fn try_extend(&mut self, _cmd: &DrawCommand, _clip: Option<[u32; 4]>) -> bool {
        // For Phase 2 we only merge identically-typed adjacent commands.
        // This prevents merging Quad with Text, or across clip changes.
        //
        // Future phases may add more sophisticated merging (e.g. same
        // border-radius quads), but adjacency remains the hard constraint.
        //
        // For now, `try_extend` is a placeholder — the compile_scene loop
        // already handles the adjacency rule by flushing on pipeline/clip
        // changes, so no extra merge logic is needed yet.
        false
    }
}

/// Compile a [`Scene`] into an ordered list of [`RenderBatch`] descriptors.
///
/// The walk is single-pass and deterministic: same Scene always produces
/// identical batches.
///
/// # Rules
///
/// 1. Commands are processed in painter's order.
/// 2. `PushClip` / `PopClip` flush any in-progress batch.
/// 3. `BeginLayer` / `EndLayer` flush and increment/decrement layer depth.
/// 4. Adjacent `Quad` commands with the same clip rect are merged.
/// 5. Adjacent `Text` commands with the same clip AND glyph format are merged.
/// 6. Text commands containing mixed-format glyphs are split by format.
/// 7. No merging happens across clip or layer boundaries.
///
/// Get current clip as logical `[x, y, w, h]`, or `None` for full viewport.
fn clip_as_rect(clip_stack: &ClipStack) -> Option<[u32; 4]> {
    let r = clip_stack.current()?;
    Some([
        r.origin.x.get() as u32,
        r.origin.y.get() as u32,
        r.size.width.get() as u32,
        r.size.height.get() as u32,
    ])
}

pub fn compile_scene(scene: &Scene) -> Vec<RenderBatch> {
    let mut batches: Vec<RenderBatch> = Vec::new();
    let mut clip_stack = ClipStack::new();
    let mut layer_depth: u32 = 0;

    let mut i = 0;
    while i < scene.commands().len() {
        let cmd = &scene.commands()[i];
        match cmd {
            DrawCommand::PushClip(rect) => {
                clip_stack.push(*rect);
                i += 1;
            }
            DrawCommand::PopClip => {
                let _ = clip_stack.pop();
                i += 1;
            }
            DrawCommand::BeginLayer(_) => {
                layer_depth += 1;
                i += 1;
            }
            DrawCommand::EndLayer => {
                layer_depth = layer_depth.saturating_sub(1);
                i += 1;
            }
            DrawCommand::Quad(_) => {
                let clip = clip_as_rect(&clip_stack);
                batches.push(RenderBatch::new(BatchPipeline::Quad, clip, layer_depth, i));
                // Walk forward merging adjacent Quads with same clip.
                i += 1;
                while i < scene.commands().len() {
                    let next = &scene.commands()[i];
                    match next {
                        DrawCommand::Quad(_) => {
                            let next_clip = clip_as_rect(&clip_stack);
                            if next_clip == clip {
                                // Extend current batch.
                                if let Some(last) = batches.last_mut() {
                                    last.command_end = i + 1;
                                }
                                i += 1;
                            } else {
                                break;
                            }
                        }
                        DrawCommand::PushClip(_)
                        | DrawCommand::PopClip
                        | DrawCommand::BeginLayer(_)
                        | DrawCommand::EndLayer => {
                            break;
                        }
                        DrawCommand::Text(_) => {
                            break;
                        }
                    }
                }
            }
            DrawCommand::Text(prim) => {
                // Split glyphs by format.
                let has_alpha = prim.glyphs.iter().any(|g| g.format == GlyphFormat::Alpha8);
                let has_rgba = prim.glyphs.iter().any(|g| g.format == GlyphFormat::Rgba8);

                let clip = clip_as_rect(&clip_stack);

                if has_alpha {
                    batches.push(RenderBatch::new(
                        BatchPipeline::TextAlpha,
                        clip,
                        layer_depth,
                        i,
                    ));
                }
                if has_rgba {
                    batches.push(RenderBatch::new(
                        BatchPipeline::TextRgba,
                        clip,
                        layer_depth,
                        i,
                    ));
                }
                i += 1;

                // Merge adjacent Text commands with the same clip.
                while i < scene.commands().len() {
                    let next = &scene.commands()[i];
                    match next {
                        DrawCommand::Text(t2) => {
                            let next_clip = clip_as_rect(&clip_stack);
                            if next_clip == clip {
                                let t2_alpha =
                                    t2.glyphs.iter().any(|g| g.format == GlyphFormat::Alpha8);
                                let t2_rgba =
                                    t2.glyphs.iter().any(|g| g.format == GlyphFormat::Rgba8);
                                // Extend existing format batches.
                                if t2_alpha
                                    && has_alpha
                                    && let Some(last) = batches
                                        .iter_mut()
                                        .rev()
                                        .find(|b| b.pipeline == BatchPipeline::TextAlpha)
                                {
                                    last.command_end = i + 1;
                                }
                                if t2_rgba
                                    && has_rgba
                                    && let Some(last) = batches
                                        .iter_mut()
                                        .rev()
                                        .find(|b| b.pipeline == BatchPipeline::TextRgba)
                                {
                                    last.command_end = i + 1;
                                }
                                i += 1;
                            } else {
                                break;
                            }
                        }
                        DrawCommand::Quad(_)
                        | DrawCommand::PushClip(_)
                        | DrawCommand::PopClip
                        | DrawCommand::BeginLayer(_)
                        | DrawCommand::EndLayer => {
                            break;
                        }
                    }
                }
            }
        }
    }

    batches
}

/// Convert a legacy [`Frame`] into a [`Scene`], preserving the current
/// bucket-order (all quads → all clipped quads → all text).
///
/// This is a **bridge** for migration: new code should produce `Scene`
/// directly via [`Scene::push`] / [`DrawCommand`].
pub fn scene_from_frame(frame: &Frame) -> Scene {
    use acme_core::{AtlasUpload, GlyphDraw, QuadPrimitive, TextPrimitive};
    use acme_text::AtlasFormat as TextAtlasFormat;

    let mut scene = Scene::with_clear(acme_core::Color::rgba(
        frame.clear[0],
        frame.clear[1],
        frame.clear[2],
        frame.clear[3],
    ));

    // Helper: convert text AtlasFormat to scene GlyphFormat.
    let to_glyph_format = |f: TextAtlasFormat| -> GlyphFormat {
        match f {
            TextAtlasFormat::Alpha8 => GlyphFormat::Alpha8,
            TextAtlasFormat::Rgba8 => GlyphFormat::Rgba8,
        }
    };

    // 1. Unclipped quads.
    for q in &frame.quads {
        scene.push(DrawCommand::Quad(QuadPrimitive {
            rect: acme_core::Rect::new(q.rect[0], q.rect[1], q.rect[2], q.rect[3]),
            color: acme_core::Color::rgba(q.color[0], q.color[1], q.color[2], q.color[3]),
            radius: q.radius,
            border_width: q.border_width,
            border_color: acme_core::Color::rgba(
                q.border_color[0],
                q.border_color[1],
                q.border_color[2],
                q.border_color[3],
            ),
        }));
    }

    // 2. Clipped quads – wrap each unique clip group in PushClip/PopClip.
    //    We walk the clipped_quads in order and group consecutive runs
    //    with the same clip rect.
    let mut clip_start = 0usize;
    while clip_start < frame.clipped_quads.len() {
        let clip_rect = frame.clipped_quads[clip_start].clip;
        let mut clip_end = clip_start + 1;
        while clip_end < frame.clipped_quads.len()
            && frame.clipped_quads[clip_end].clip == clip_rect
        {
            clip_end += 1;
        }

        let clip_prim =
            acme_core::Rect::new(clip_rect[0], clip_rect[1], clip_rect[2], clip_rect[3]);
        scene.push(DrawCommand::PushClip(clip_prim));

        for cq in &frame.clipped_quads[clip_start..clip_end] {
            scene.push(DrawCommand::Quad(QuadPrimitive {
                rect: acme_core::Rect::new(
                    cq.quad.rect[0],
                    cq.quad.rect[1],
                    cq.quad.rect[2],
                    cq.quad.rect[3],
                ),
                color: acme_core::Color::rgba(
                    cq.quad.color[0],
                    cq.quad.color[1],
                    cq.quad.color[2],
                    cq.quad.color[3],
                ),
                radius: cq.quad.radius,
                border_width: cq.quad.border_width,
                border_color: acme_core::Color::rgba(
                    cq.quad.border_color[0],
                    cq.quad.border_color[1],
                    cq.quad.border_color[2],
                    cq.quad.border_color[3],
                ),
            }));
        }

        scene.push(DrawCommand::PopClip);
        clip_start = clip_end;
    }

    // 3. Text runs — each run becomes one Text command.
    for run in &frame.text {
        let mut glyphs = Vec::new();
        let mut uploads = Vec::new();

        // Extract glyphs and uploads from the PreparedText.
        // Each PreparedText contains atlas-upload regions and positioned glyphs.
        let prepared = &run.prepared;

        // Atlas upload regions from the prepared text.
        for region in &prepared.uploads {
            uploads.push(AtlasUpload {
                page: 0, // single-page atlas in current implementation
                origin: [region.x, region.y],
                size: [region.width, region.height],
                format: to_glyph_format(region.format),
                pixels: region.pixels.clone(),
            });
        }

        // Positioned glyphs.
        for glyph in &prepared.glyphs {
            glyphs.push(GlyphDraw {
                x: glyph.x as f32,
                y: glyph.y as f32,
                width: glyph.width as f32,
                height: glyph.height as f32,
                atlas_x: glyph.atlas_x,
                atlas_y: glyph.atlas_y,
                format: to_glyph_format(glyph.format),
            });
        }

        scene.push(DrawCommand::Text(TextPrimitive {
            origin: acme_core::Point::new(run.origin[0], run.origin[1]),
            color: acme_core::Color::rgba(run.color[0], run.color[1], run.color[2], run.color[3]),
            glyphs,
            uploads,
        }));
    }

    scene
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::{AtlasUpload, Color, Point, QuadPrimitive, Rect, TextPrimitive};

    // ------------------------------------------------------------------
    // Basic batching tests
    // ------------------------------------------------------------------

    fn make_quad(x: f32, y: f32, w: f32, h: f32) -> DrawCommand {
        DrawCommand::Quad(QuadPrimitive {
            rect: Rect::new(x, y, x + w, y + h),
            color: Color::rgba(1.0, 0.0, 0.0, 1.0),
            radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        })
    }

    fn make_text(alpha: bool) -> DrawCommand {
        let format = if alpha {
            GlyphFormat::Alpha8
        } else {
            GlyphFormat::Rgba8
        };
        DrawCommand::Text(TextPrimitive {
            origin: Point::new(0.0, 0.0),
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            glyphs: vec![acme_core::GlyphDraw {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
                atlas_x: 0,
                atlas_y: 0,
                format,
            }],
            uploads: vec![AtlasUpload {
                page: 0,
                origin: [0, 0],
                size: [10, 10],
                format,
                pixels: vec![0u8; 100],
            }],
        })
    }

    #[test]
    fn empty_scene_produces_no_batches() {
        let scene = Scene::new();
        let batches = compile_scene(&scene);
        assert!(batches.is_empty(), "empty scene → no batches");
    }

    #[test]
    fn single_quad_produces_one_batch() {
        let mut scene = Scene::new();
        scene.push(make_quad(0.0, 0.0, 100.0, 50.0));
        let batches = compile_scene(&scene);
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].pipeline, BatchPipeline::Quad);
        assert_eq!(batches[0].command_start, 0);
        assert_eq!(batches[0].command_end, 1);
    }

    #[test]
    fn adjacent_quads_are_merged() {
        let mut scene = Scene::new();
        scene.push(make_quad(0.0, 0.0, 10.0, 10.0));
        scene.push(make_quad(10.0, 0.0, 10.0, 10.0));
        scene.push(make_quad(20.0, 0.0, 10.0, 10.0));
        let batches = compile_scene(&scene);
        assert_eq!(batches.len(), 1, "three adjacent quads → one batch");
        assert_eq!(batches[0].command_start, 0);
        assert_eq!(batches[0].command_end, 3);
    }

    #[test]
    fn text_and_quad_are_separate_batches() {
        let mut scene = Scene::new();
        scene.push(make_quad(0.0, 0.0, 10.0, 10.0));
        scene.push(make_text(true));
        let batches = compile_scene(&scene);
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].pipeline, BatchPipeline::Quad);
        assert_eq!(batches[1].pipeline, BatchPipeline::TextAlpha);
    }

    #[test]
    fn clip_boundary_flushes_batches() {
        let mut scene = Scene::new();
        scene.push(make_quad(0.0, 0.0, 10.0, 10.0));
        scene.push(DrawCommand::PushClip(Rect::new(0.0, 0.0, 50.0, 50.0)));
        scene.push(make_quad(5.0, 5.0, 20.0, 20.0));
        scene.push(DrawCommand::PopClip);
        scene.push(make_quad(30.0, 30.0, 10.0, 10.0));
        let batches = compile_scene(&scene);
        // Expect: Quad(unclipped) + PushClip → Quad(clipped) + PopClip → Quad(unclipped)
        // = 3 quad batches (separated by clip boundaries)
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0].command_start, 0);
        assert_eq!(batches[0].command_end, 1);
        assert_eq!(batches[1].command_start, 2);
        assert_eq!(batches[1].command_end, 3);
        assert_eq!(batches[2].command_start, 4);
        assert_eq!(batches[2].command_end, 5);
    }

    #[test]
    fn layer_boundary_flushes_batches() {
        let mut scene = Scene::new();
        scene.push(make_quad(0.0, 0.0, 10.0, 10.0));
        scene.push(DrawCommand::BeginLayer(acme_core::LayerParams {
            opacity: 0.5,
        }));
        scene.push(make_quad(5.0, 5.0, 10.0, 10.0));
        scene.push(DrawCommand::EndLayer);
        scene.push(make_quad(20.0, 20.0, 10.0, 10.0));
        let batches = compile_scene(&scene);
        // 3 quad batches separated by layer boundaries
        assert!(batches.len() >= 3, "should have at least 3 batches");
        // First batch has layer=0, second has layer=1 during BeginLayer
        // but we don't increment until after the command, so the quad
        // inside the layer also has layer=0 (the BeginLayer command itself
        // is not a draw command and doesn't produce a batch).
        // Actually, layer depth increments after seeing BeginLayer,
        // so the quad after BeginLayer has layer=1.
        assert_eq!(batches[1].layer, 1, "quad inside layer should have layer=1");
        assert_eq!(
            batches[2].layer, 0,
            "quad after EndLayer should have layer=0"
        );
    }

    #[test]
    fn adjacent_text_with_same_format_and_clip_are_merged() {
        let mut scene = Scene::new();
        scene.push(make_text(true));
        scene.push(make_text(true));
        let batches = compile_scene(&scene);
        assert_eq!(
            batches.len(),
            1,
            "two adjacent alpha-text commands → one batch"
        );
        assert_eq!(batches[0].command_start, 0);
        assert_eq!(batches[0].command_end, 2);
    }

    #[test]
    fn text_alpha_and_rgba_in_same_command_produce_separate_batches() {
        // A single TextPrimitive with mixed-format glyphs.
        let prim = TextPrimitive {
            origin: Point::new(0.0, 0.0),
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            glyphs: vec![
                acme_core::GlyphDraw {
                    x: 0.0,
                    y: 0.0,
                    width: 10.0,
                    height: 10.0,
                    atlas_x: 0,
                    atlas_y: 0,
                    format: GlyphFormat::Alpha8,
                },
                acme_core::GlyphDraw {
                    x: 10.0,
                    y: 0.0,
                    width: 10.0,
                    height: 10.0,
                    atlas_x: 0,
                    atlas_y: 0,
                    format: GlyphFormat::Rgba8,
                },
            ],
            uploads: vec![],
        };
        let mut scene = Scene::new();
        scene.push(DrawCommand::Text(prim));
        let batches = compile_scene(&scene);
        // Should produce TWO batches: TextAlpha and TextRgba (same command index).
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].pipeline, BatchPipeline::TextAlpha);
        assert_eq!(batches[1].pipeline, BatchPipeline::TextRgba);
        // Both reference the same command (index 0).
        assert_eq!(batches[0].command_start, 0);
        assert_eq!(batches[1].command_start, 0);
    }

    // ------------------------------------------------------------------
    // Determinism: 100-run test
    // ------------------------------------------------------------------

    #[test]
    fn compile_scene_is_deterministic() {
        let mut scene = Scene::new();
        // Build a non-trivial scene with mixed commands.
        scene.push(make_quad(0.0, 0.0, 100.0, 50.0));
        scene.push(DrawCommand::PushClip(Rect::new(10.0, 10.0, 80.0, 30.0)));
        scene.push(make_quad(20.0, 20.0, 30.0, 30.0));
        scene.push(make_quad(50.0, 20.0, 30.0, 30.0));
        scene.push(DrawCommand::PopClip);
        scene.push(make_text(true));
        scene.push(make_text(false));
        scene.push(DrawCommand::BeginLayer(acme_core::LayerParams {
            opacity: 0.5,
        }));
        scene.push(make_quad(0.0, 0.0, 10.0, 10.0));
        scene.push(DrawCommand::EndLayer);

        let reference = compile_scene(&scene);
        for run in 0..100 {
            let result = compile_scene(&scene);
            assert_eq!(
                result, reference,
                "run {run}: compile_scene must be deterministic (same Scene → same batches)"
            );
        }
    }

    // ------------------------------------------------------------------
    // scene_from_frame bridge tests
    // ------------------------------------------------------------------

    #[test]
    fn scene_from_frame_preserves_quad_count() {
        let mut frame = Frame::default();
        frame
            .quads
            .push(crate::Quad::solid([0.0, 0.0, 10.0, 10.0], [1.0; 4]));
        frame
            .quads
            .push(crate::Quad::solid([10.0, 0.0, 10.0, 10.0], [0.5; 4]));

        let scene = scene_from_frame(&frame);
        let quad_count = scene
            .commands()
            .iter()
            .filter(|c| matches!(c, DrawCommand::Quad(_)))
            .count();
        assert_eq!(quad_count, 2, "bridge preserves quad count");
    }

    #[test]
    fn scene_from_frame_preserves_clear_color() {
        let frame = Frame {
            clear: [0.1, 0.2, 0.3, 1.0],
            ..Frame::default()
        };

        let scene = scene_from_frame(&frame);
        let c = scene.clear_color();
        assert!((c.r - 0.1).abs() < 0.001);
        assert!((c.g - 0.2).abs() < 0.001);
        assert!((c.b - 0.3).abs() < 0.001);
        assert!((c.a - 1.0).abs() < 0.001);
    }

    #[test]
    fn scene_from_frame_groups_adjacent_clip_quads() {
        let mut frame = Frame::default();
        frame.clipped_quads.push(crate::ClippedQuad {
            quad: crate::Quad::solid([0.0, 0.0, 10.0, 10.0], [1.0; 4]),
            clip: [0.0, 0.0, 50.0, 50.0],
        });
        frame.clipped_quads.push(crate::ClippedQuad {
            quad: crate::Quad::solid([10.0, 0.0, 10.0, 10.0], [1.0; 4]),
            clip: [0.0, 0.0, 50.0, 50.0],
        });

        let scene = scene_from_frame(&frame);
        let commands = scene.commands();
        // Expect: PushClip + 2×Quad + PopClip = 4 commands
        assert_eq!(commands.len(), 4);
        assert!(matches!(commands[0], DrawCommand::PushClip(_)));
        assert!(matches!(commands[1], DrawCommand::Quad(_)));
        assert!(matches!(commands[2], DrawCommand::Quad(_)));
        assert!(matches!(commands[3], DrawCommand::PopClip));
    }

    #[test]
    fn scene_from_frame_handles_empty_frame() {
        let frame = Frame::default();
        let scene = scene_from_frame(&frame);
        assert!(scene.commands().is_empty(), "empty frame → empty scene");
    }

    #[test]
    fn compiled_scene_from_frame_is_valid() {
        let mut frame = Frame {
            clear: [0.9, 0.9, 0.9, 1.0],
            ..Frame::default()
        };
        frame
            .quads
            .push(crate::Quad::solid([0.0, 0.0, 100.0, 100.0], [1.0; 4]));
        frame.clipped_quads.push(crate::ClippedQuad {
            quad: crate::Quad::solid([10.0, 10.0, 20.0, 20.0], [0.5; 4]),
            clip: [0.0, 0.0, 200.0, 200.0],
        });

        let scene = scene_from_frame(&frame);
        assert!(scene.validate().is_ok(), "bridge scene must be valid");

        let batches = compile_scene(&scene);
        assert!(!batches.is_empty(), "non-empty frame → at least one batch");
    }
}
