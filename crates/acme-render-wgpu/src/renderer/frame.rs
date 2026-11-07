//! Per-frame rendering: scene compilation, instance extraction, atlas uploads and submission.

use std::collections::HashSet;

use acme_core::{DrawCommand, GlyphFormat as SceneGlyphFormat, Scene};

use crate::atlas_pages::UploadKey;
use crate::batch;
use crate::draws::{DrawBatchDesc, ImageBatchDesc, QuadBatchDesc, push_text_batch, scissor};
use crate::images::image_resource_key;
use crate::instances::{
    GlyphInstance, Instance, collect_text_glyphs, image_primitive_to_instance, normalize_color,
    quad_primitive_to_instance,
};
use crate::surface::{AcquireOutcome, is_device_lost, resolve_surface_action};
use crate::{Frame, Quad, SurfaceAction, SurfaceStatus};

use super::Renderer;

impl Renderer {
    /// Render a [`Scene`] directly, bypassing the legacy [`Frame`] format.
    ///
    /// This is the primary render entry point.  The legacy [`render`](Self::render)
    /// method bridges to this via [`batch::scene_from_frame`].
    pub fn render_scene(&mut self, scene: &Scene) -> SurfaceAction {
        // Keep backend-neutral CPU image state current even when acquisition
        // immediately reports device loss. Recovery rebuilds only this live set.
        self.update_live_scene_images(scene);

        // ---- surface acquire preamble ----
        let suspended = self.status == SurfaceStatus::Suspended;
        let lost = is_device_lost(&self.device_lost_flag);
        let pre_acquire_action = resolve_surface_action(suspended, lost, AcquireOutcome::Success);
        if pre_acquire_action != SurfaceAction::Rendered {
            return pre_acquire_action;
        }
        let mut reconfigure_after_present = false;
        let acquired = self.surface.get_current_texture();
        let lost = is_device_lost(&self.device_lost_flag);
        let outcome = match &acquired {
            wgpu::CurrentSurfaceTexture::Success(_) => AcquireOutcome::Success,
            wgpu::CurrentSurfaceTexture::Suboptimal(_) => AcquireOutcome::Suboptimal,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                AcquireOutcome::OutdatedOrLost
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                AcquireOutcome::TimeoutOrOccluded
            }
            wgpu::CurrentSurfaceTexture::Validation => AcquireOutcome::Validation,
        };
        let action = resolve_surface_action(suspended, lost, outcome);
        if action != SurfaceAction::Rendered {
            if outcome == AcquireOutcome::OutdatedOrLost {
                self.status = SurfaceStatus::Recovering;
                self.surface.configure(&self.device, &self.config);
                self.status = SurfaceStatus::Ready;
            } else if outcome == AcquireOutcome::Validation {
                tracing::warn!("surface acquisition validation failure");
            }
            return action;
        }
        let output = match acquired {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                reconfigure_after_present = true;
                frame
            }
            _ => return SurfaceAction::Skip,
        };

        self.stats.bytes_uploaded += self.sync_scene_images();

        // ---- atlas upload from scene ----
        let mut uploaded_regions: HashSet<UploadKey> = HashSet::new();
        let mut atlas_total = 0u64;
        let mut atlas_skipped = 0u64;
        let mut atlas_bytes = 0u64;
        {
            let (total, skipped, bytes) = self.upload_scene_atlas(scene, &mut uploaded_regions);
            atlas_total += total;
            atlas_skipped += skipped;
            atlas_bytes += bytes;
        }
        self.stats.bytes_uploaded += atlas_bytes;
        self.stats.atlas_hit_rate = if atlas_total > 0 {
            (atlas_skipped as f64 / atlas_total as f64) * 100.0
        } else {
            100.0
        };

        // ---- compile scene into ordered batches ----
        let batches = batch::compile_scene(scene);
        let scale = self.scale_factor;
        let config_w = self.config.width;
        let config_h = self.config.height;

        // ---- extract instances per batch ----
        let mut all_quads: Vec<Instance> = Vec::new();
        let mut all_glyphs: Vec<GlyphInstance> = Vec::new();
        let mut draw_batches: Vec<DrawBatchDesc> = Vec::new();
        let mut image_count = 0u64;

        for batch in &batches {
            let cmd_range = batch.command_start..batch.command_end;
            match batch.pipeline {
                batch::BatchPipeline::Quad => {
                    let start = all_quads.len() as u32;
                    for cmd_idx in cmd_range {
                        if let DrawCommand::Quad(prim) = &scene.commands()[cmd_idx] {
                            all_quads
                                .push(quad_primitive_to_instance(prim, scale, config_w, config_h));
                        }
                    }
                    let count = all_quads.len() as u32 - start;
                    if count > 0 {
                        draw_batches.push(DrawBatchDesc::Quad(QuadBatchDesc {
                            clip: batch.clip,
                            start,
                            count,
                        }));
                    }
                }
                batch::BatchPipeline::TextAlpha => {
                    let format = SceneGlyphFormat::Alpha8;
                    let glyphs =
                        collect_text_glyphs(scene, cmd_range, format, scale, config_w, config_h);
                    push_text_batch(
                        glyphs,
                        format,
                        batch.clip,
                        &mut all_glyphs,
                        &mut draw_batches,
                    );
                }
                batch::BatchPipeline::TextRgba => {
                    let format = SceneGlyphFormat::Rgba8;
                    let glyphs =
                        collect_text_glyphs(scene, cmd_range, format, scale, config_w, config_h);
                    push_text_batch(
                        glyphs,
                        format,
                        batch.clip,
                        &mut all_glyphs,
                        &mut draw_batches,
                    );
                }
                batch::BatchPipeline::Image => {
                    for cmd_idx in cmd_range {
                        if let DrawCommand::Image(primitive) = &scene.commands()[cmd_idx] {
                            let start = all_glyphs.len() as u32;
                            all_glyphs.push(image_primitive_to_instance(
                                primitive, scale, config_w, config_h,
                            ));
                            draw_batches.push(DrawBatchDesc::Image(ImageBatchDesc {
                                key: image_resource_key(&primitive.image),
                                clip: batch.clip,
                                start,
                                count: 1,
                            }));
                            image_count += 1;
                        }
                    }
                }
            }
        }

        // ---- write double-buffered vertex buffers ----
        let buf_idx = self.current_frame;
        let instance_size = std::mem::size_of::<Instance>() as u64;
        let quad_instances_needed = all_quads.len() as u64;
        let needed_quad_bytes = quad_instances_needed * instance_size;
        self.ensure_quad_capacity(needed_quad_bytes, buf_idx);
        if !all_quads.is_empty() {
            self.queue.write_buffer(
                &self.quad_buffers[buf_idx],
                0,
                bytemuck::cast_slice(&all_quads),
            );
            self.stats.bytes_uploaded += needed_quad_bytes;
        }

        let glyph_size = std::mem::size_of::<GlyphInstance>() as u64;
        let needed_glyph_bytes = all_glyphs.len() as u64 * glyph_size;
        self.ensure_glyph_capacity(needed_glyph_bytes, buf_idx);
        if !all_glyphs.is_empty() {
            self.queue.write_buffer(
                &self.glyph_buffers[buf_idx],
                0,
                bytemuck::cast_slice(&all_glyphs),
            );
            self.stats.bytes_uploaded += needed_glyph_bytes;
        }

        // Every page a glyph draw binds must exist before the pass starts.
        self.ensure_glyph_pages(&draw_batches);

        // ---- render pass ----
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("acme frame"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("acme main pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: scene.clear.r as f64,
                            g: scene.clear.g as f64,
                            b: scene.clear.b as f64,
                            a: scene.clear.a as f64,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            let mut draw_count = 0u64;

            // Draw all pipelines in the Scene's painter order.
            for batch in &draw_batches {
                let clip = match batch {
                    DrawBatchDesc::Quad(batch) => batch.clip,
                    DrawBatchDesc::Glyph(batch) => batch.clip,
                    DrawBatchDesc::Image(batch) => batch.clip,
                };
                let scissor_rect = match clip {
                    Some(c) => {
                        let Some(scissor_rect) = scissor(c, scale, config_w, config_h) else {
                            continue;
                        };
                        scissor_rect
                    }
                    None => [0, 0, config_w, config_h],
                };
                pass.set_scissor_rect(
                    scissor_rect[0],
                    scissor_rect[1],
                    scissor_rect[2],
                    scissor_rect[3],
                );
                match batch {
                    DrawBatchDesc::Quad(batch) => {
                        pass.set_pipeline(&self.pipeline);
                        pass.set_vertex_buffer(0, self.quad_buffers[buf_idx].slice(..));
                        pass.draw(0..6, batch.start..batch.start + batch.count);
                        draw_count += 1;
                    }
                    DrawBatchDesc::Glyph(batch) => {
                        let Some(page) = self.atlas_pages.get(batch.format, batch.page) else {
                            continue;
                        };
                        pass.set_pipeline(&self.text_pipeline);
                        pass.set_bind_group(0, &page.bind_group, &[]);
                        pass.set_vertex_buffer(0, self.glyph_buffers[buf_idx].slice(..));
                        pass.draw(0..6, batch.start..batch.start + batch.count);
                        draw_count += 1;
                    }
                    DrawBatchDesc::Image(batch) => {
                        let Some(resource) = self.images.get(&batch.key) else {
                            continue;
                        };
                        pass.set_pipeline(&self.text_pipeline);
                        pass.set_bind_group(0, &resource.bind_group, &[]);
                        pass.set_vertex_buffer(0, self.glyph_buffers[buf_idx].slice(..));
                        pass.draw(0..6, batch.start..batch.start + batch.count);
                        draw_count += 1;
                    }
                }
            }

            self.stats.draw_calls = draw_count;
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();

        // ---- update stats ----
        self.stats.quad_count = quad_instances_needed;
        self.stats.glyph_count = all_glyphs.len() as u64 - image_count;
        self.stats.image_count = image_count;
        // ---- flip frame ring ----
        self.current_frame ^= 1;
        if reconfigure_after_present {
            self.surface.configure(&self.device, &self.config);
            SurfaceAction::Reconfigure
        } else {
            SurfaceAction::Rendered
        }
    }

    /// Legacy bridge: converts a [`Frame`] to a [`Scene`] via
    /// [`batch::scene_from_frame`] and delegates to [`render_scene`](Self::render_scene).
    pub fn render(&mut self, frame: &Frame) -> SurfaceAction {
        let scene = batch::scene_from_frame(frame);
        self.render_scene(&scene)
    }

    #[allow(dead_code)]
    fn quad_instance(&self, quad: &Quad) -> Instance {
        let clean = |value: f32| if value.is_finite() { value } else { 0.0 };
        Instance {
            rect: [
                clean(quad.rect[0]) * self.scale_factor,
                clean(quad.rect[1]) * self.scale_factor,
                clean(quad.rect[2]).max(0.0) * self.scale_factor,
                clean(quad.rect[3]).max(0.0) * self.scale_factor,
            ],
            color: normalize_color(quad.color),
            border_color: normalize_color(quad.border_color),
            extras: [
                clean(quad.radius).max(0.0) * self.scale_factor,
                clean(quad.border_width).max(0.0) * self.scale_factor,
                self.config.width as f32,
                self.config.height as f32,
            ],
        }
    }
}
