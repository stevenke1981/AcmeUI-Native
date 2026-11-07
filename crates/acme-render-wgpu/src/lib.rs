//! GPU surface lifecycle and the small batched rectangle renderer used by AcmeUI.
#![forbid(unsafe_op_in_unsafe_fn)]

mod atlas_pages;
pub mod batch;
pub mod golden;
pub use batch::scene_from_frame;

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{any::Any, sync::Arc};

use acme_core::{
    ATLAS_MAX_PAGES, ATLAS_PAGE_EDGE, DrawCommand, GlyphFormat as SceneGlyphFormat, ImageKey,
    ImagePrimitive, Rgba8Image, Scene,
};
use acme_text::PreparedText;
use atlas_pages::{AtlasPages, UploadKey, UploadReject, check_upload, page_runs, upload_key};
use bytemuck::{Pod, Zeroable};
use thiserror::Error;

/// A renderer-owned rectangle expressed in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad {
    pub rect: [f32; 4],
    pub color: [f32; 4],
    pub radius: f32,
    pub border_width: f32,
    pub border_color: [f32; 4],
}

impl Quad {
    pub fn solid(rect: [f32; 4], color: [f32; 4]) -> Self {
        Self {
            rect,
            color,
            radius: 0.0,
            border_width: 0.0,
            border_color: color,
        }
    }
}

/// A complete frame. Clips are rectangular and intersected on the CPU by callers.
#[derive(Clone, Debug)]
pub struct Frame {
    pub clear: [f32; 4],
    pub quads: Vec<Quad>,
    pub clipped_quads: Vec<ClippedQuad>,
    pub text: Vec<TextRun>,
}

#[derive(Clone, Debug)]
pub struct ClippedQuad {
    pub quad: Quad,
    pub clip: [f32; 4],
}

#[derive(Clone, Debug)]
pub struct TextRun {
    pub prepared: PreparedText,
    /// Logical-pixel origin added to physical glyph positions after DPI scaling.
    pub origin: [f32; 2],
    pub color: [f32; 4],
    pub clip: Option<[f32; 4]>,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            clear: [0.08, 0.09, 0.12, 1.0],
            quads: Vec::new(),
            clipped_quads: Vec::new(),
            text: Vec::new(),
        }
    }
}

/// Per-frame rendering statistics.
#[derive(Clone, Debug, Default)]
pub struct RenderStats {
    pub buffer_grows: u64,
    pub bytes_uploaded: u64,
    pub draw_calls: u64,
    pub quad_count: u64,
    pub glyph_count: u64,
    pub image_count: u64,
    /// Percentage of atlas uploads that were already resident (0.0-100.0).
    pub atlas_hit_rate: f64,
}

impl RenderStats {
    /// Compact one-line summary for debug / devtools.
    pub fn summary(&self) -> String {
        format!(
            "quads={} glyphs={} images={} draws={} grows={} uploaded={}B atlas_hit={:.1}%",
            self.quad_count,
            self.glyph_count,
            self.image_count,
            self.draw_calls,
            self.buffer_grows,
            self.bytes_uploaded,
            self.atlas_hit_rate,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceStatus {
    Ready,
    Suspended,
    Recovering,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SurfaceAction {
    Rendered,
    Reconfigure,
    Skip,
    Exit,
    DeviceLost,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AcquireOutcome {
    Success,
    Suboptimal,
    OutdatedOrLost,
    TimeoutOrOccluded,
    Validation,
}

fn resolve_surface_action(
    suspended: bool,
    device_lost: bool,
    acquire: AcquireOutcome,
) -> SurfaceAction {
    if suspended {
        return SurfaceAction::Skip;
    }
    if device_lost {
        return SurfaceAction::DeviceLost;
    }
    match acquire {
        AcquireOutcome::Success | AcquireOutcome::Suboptimal => SurfaceAction::Rendered,
        AcquireOutcome::OutdatedOrLost => SurfaceAction::Reconfigure,
        AcquireOutcome::TimeoutOrOccluded | AcquireOutcome::Validation => SurfaceAction::Skip,
    }
}

fn is_device_lost(flag: &AtomicBool) -> bool {
    flag.load(Ordering::Acquire)
}

fn complete_recovery_state(
    device_lost: &AtomicBool,
    status: &mut SurfaceStatus,
    gpu_epoch: &mut u64,
) {
    device_lost.store(false, Ordering::Release);
    *status = SurfaceStatus::Ready;
    *gpu_epoch = gpu_epoch.wrapping_add(1);
}

/// Register wgpu callbacks that mark the shared device-lost flag.
///
/// - `set_device_lost_callback`: real device loss
/// - `on_uncaptured_error`: Internal / OutOfMemory → treat as lost; Validation is logged only
fn register_device_error_handlers(device: &wgpu::Device, flag: &Arc<AtomicBool>) {
    let lost_flag = Arc::clone(flag);
    device.set_device_lost_callback(move |reason, message| {
        tracing::error!(?reason, %message, "GPU device lost callback fired");
        lost_flag.store(true, Ordering::Release);
    });

    let err_flag = Arc::clone(flag);
    device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| match &error {
        wgpu::Error::Validation { description, .. } => {
            tracing::error!(%description, "wgpu validation error (not marking device lost)");
        }
        wgpu::Error::OutOfMemory { .. } => {
            tracing::error!(%error, "wgpu out-of-memory; marking device lost");
            err_flag.store(true, Ordering::Release);
        }
        wgpu::Error::Internal { description, .. } => {
            tracing::error!(%description, "wgpu internal error; marking device lost");
            err_flag.store(true, Ordering::Release);
        }
    }));
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("unsupported window surface target")]
    WindowTarget,
    #[error("failed to create GPU surface: {0}")]
    Surface(String),
    #[error("no compatible GPU adapter was found")]
    Adapter,
    #[error("failed to request GPU device: {0}")]
    Device(String),
    #[error("device lost")]
    DeviceLost,
    #[error("failed to recreate GPU resources after device loss")]
    DeviceRecoveryFailed,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Instance {
    rect: [f32; 4],
    color: [f32; 4],
    border_color: [f32; 4],
    extras: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GlyphInstance {
    rect: [f32; 4],
    uv: [f32; 4],
    color: [f32; 4],
    viewport_mode: [f32; 4],
}

struct ImageResource {
    image: Rgba8Image,
    _texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

type ImageResourceKey = (ImageKey, u64);

/// Quad draw covering `count` instances starting at `start`.
struct QuadBatchDesc {
    clip: Option<[f32; 4]>,
    start: u32,
    count: u32,
}

/// Glyph draw: one atlas page, one format, contiguous instances.
struct GlyphBatchDesc {
    format: SceneGlyphFormat,
    page: u32,
    clip: Option<[f32; 4]>,
    start: u32,
    count: u32,
}

/// Image draw for one resident RGBA8 resource.
struct ImageBatchDesc {
    key: ImageResourceKey,
    clip: Option<[f32; 4]>,
    start: u32,
    count: u32,
}

enum DrawBatchDesc {
    Quad(QuadBatchDesc),
    Glyph(GlyphBatchDesc),
    Image(ImageBatchDesc),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ImageUploadLayout {
    bytes_per_row: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImageUploadError {
    ExceedsMaxDimension { width: u32, height: u32, max: u32 },
    RowPitchOverflow,
}

/// Initial per-buffer instance capacity (1024 instances approx 64 KiB each).
const INITIAL_QUAD_CAPACITY: u64 = 1024;
const INITIAL_GLYPH_CAPACITY: u64 = 1024;

/// Owns a wgpu surface. Third-party GPU types never appear in its public API.
pub struct Renderer {
    _instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    text_pipeline: wgpu::RenderPipeline,
    /// Lazily created glyph atlas pages, one texture per (format, page).
    atlas_pages: AtlasPages,
    texture_layout: wgpu::BindGroupLayout,
    image_sampler: wgpu::Sampler,
    live_images: HashMap<ImageResourceKey, Rgba8Image>,
    images: HashMap<ImageResourceKey, ImageResource>,
    size: (u32, u32),
    status: SurfaceStatus,
    scale_factor: f32,
    /// Shared with wgpu uncaptured-error / device-lost callbacks.
    device_lost_flag: Arc<AtomicBool>,
    gpu_epoch: u64,
    // Persistent double-buffered instance buffers.
    quad_buffers: [wgpu::Buffer; 2],
    glyph_buffers: [wgpu::Buffer; 2],
    quad_capacities: [u64; 2],
    glyph_capacities: [u64; 2],
    current_frame: usize,
    stats: RenderStats,
}

impl Renderer {
    /// Creates a renderer from an owned window/display handle without exposing a
    /// concrete platform window type.
    pub async fn new(
        window: Arc<dyn Any + Send + Sync>,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Result<Self, RenderError> {
        let window = window
            .downcast::<winit::window::Window>()
            .map_err(|_| RenderError::WindowTarget)?;
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .map_err(|error| RenderError::Surface(error.to_string()))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .map_err(|_| RenderError::Adapter)?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|error| RenderError::Device(error.to_string()))?;
        let device_lost_flag = Arc::new(AtomicBool::new(false));
        register_device_error_handlers(&device, &device_lost_flag);
        let safe_width = width.max(1);
        let safe_height = height.max(1);
        let mut config = surface
            .get_default_config(&adapter, safe_width, safe_height)
            .ok_or(RenderError::Adapter)?;
        config.present_mode = wgpu::PresentMode::Fifo;
        surface.configure(&device, &config);
        let (pipeline, text_pipeline, texture_layout, image_sampler) =
            build_render_resources(&device, &config);
        let quad_cap = INITIAL_QUAD_CAPACITY * std::mem::size_of::<Instance>() as u64;
        let glyph_cap = INITIAL_GLYPH_CAPACITY * std::mem::size_of::<GlyphInstance>() as u64;
        let mk_buf = |device: &wgpu::Device, size: u64, label: &str| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            })
        };
        let quad_buffers = [
            mk_buf(&device, quad_cap, "acme quad buffer"),
            mk_buf(&device, quad_cap, "acme quad buffer"),
        ];
        let glyph_buffers = [
            mk_buf(&device, glyph_cap, "acme glyph buffer"),
            mk_buf(&device, glyph_cap, "acme glyph buffer"),
        ];
        Ok(Self {
            _instance: instance,
            surface,
            device,
            queue,
            config,
            pipeline,
            text_pipeline,
            atlas_pages: AtlasPages::default(),
            texture_layout,
            image_sampler,
            live_images: HashMap::new(),
            images: HashMap::new(),
            size: (width, height),
            status: if width == 0 || height == 0 {
                SurfaceStatus::Suspended
            } else {
                SurfaceStatus::Ready
            },
            scale_factor: normalize_scale(scale_factor),
            device_lost_flag,
            gpu_epoch: 0,
            quad_buffers,
            glyph_buffers,
            quad_capacities: [quad_cap, quad_cap],
            glyph_capacities: [glyph_cap, glyph_cap],
            current_frame: 0,
            stats: RenderStats::default(),
        })
    }

    pub fn stats(&self) -> &RenderStats {
        &self.stats
    }

    pub fn status(&self) -> SurfaceStatus {
        self.status
    }

    /// Monotonically increases after each successful GPU device recovery.
    pub fn gpu_epoch(&self) -> u64 {
        self.gpu_epoch
    }

    pub fn resize(&mut self, width: u32, height: u32, scale_factor: f32) {
        self.size = (width, height);
        self.scale_factor = normalize_scale(scale_factor);
        if width == 0 || height == 0 {
            self.status = SurfaceStatus::Suspended;
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        self.status = SurfaceStatus::Ready;
    }

    fn update_live_scene_images(&mut self, scene: &Scene) {
        let max_dimension = self.device.limits().max_texture_dimension_2d;
        let scene_images = collect_scene_images(scene);
        let mut live_images = HashMap::new();
        for (resource_key, image) in scene_images {
            match validate_image_upload(image.width(), image.height(), max_dimension) {
                Ok(_) => {
                    live_images.insert(resource_key, image.clone());
                }
                Err(error) => {
                    tracing::warn!(
                        key = image.key().get(),
                        revision = image.revision(),
                        ?error,
                        "rejected RGBA8 image upload"
                    );
                }
            }
        }
        self.live_images = live_images;
    }

    fn sync_scene_images(&mut self) -> u64 {
        let max_dimension = self.device.limits().max_texture_dimension_2d;
        let live: HashSet<ImageResourceKey> = self.live_images.keys().copied().collect();
        let valid: Vec<(ImageResourceKey, Rgba8Image, ImageUploadLayout)> = self
            .live_images
            .iter()
            .filter_map(|(key, image)| {
                validate_image_upload(image.width(), image.height(), max_dimension)
                    .ok()
                    .map(|layout| (*key, image.clone(), layout))
            })
            .collect();
        let mut uploaded = 0;
        for (resource_key, image, layout) in valid {
            if self.images.contains_key(&resource_key) {
                continue;
            }

            let reusable_key = self
                .images
                .iter()
                .find(|(existing_key, resource)| {
                    existing_key.0 == resource_key.0
                        && !live.contains(existing_key)
                        && resource.image.width() == image.width()
                        && resource.image.height() == image.height()
                })
                .map(|(existing_key, _)| *existing_key);
            if let Some(reusable_key) = reusable_key {
                let mut resource = self
                    .images
                    .remove(&reusable_key)
                    .expect("selected reusable image resource must exist");
                write_rgba8_texture(&self.queue, &resource._texture, &image, layout);
                resource.image = image.clone();
                self.images.insert(resource_key, resource);
            } else {
                let resource = create_image_resource(
                    &self.device,
                    &self.queue,
                    &self.texture_layout,
                    &self.image_sampler,
                    image.clone(),
                    layout,
                );
                self.images.insert(resource_key, resource);
            }
            uploaded += image.pixels().len() as u64;
        }
        retain_live_image_resources(&mut self.images, &live);
        uploaded
    }

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

    /// Grow the current quad buffer if needed_bytes exceeds capacity.
    fn ensure_quad_capacity(&mut self, needed_bytes: u64, buf_idx: usize) {
        if needed_bytes > self.quad_capacities[buf_idx] {
            let new_cap = (self.quad_capacities[buf_idx] as f64 * 1.5) as u64;
            let new_cap = new_cap.max(needed_bytes);
            self.quad_buffers[buf_idx] = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("acme quad buffer"),
                size: new_cap,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
            self.quad_capacities[buf_idx] = new_cap;
            self.stats.buffer_grows += 1;
        }
    }

    /// Grow the current glyph buffer if needed_bytes exceeds capacity.
    fn ensure_glyph_capacity(&mut self, needed_bytes: u64, buf_idx: usize) {
        if needed_bytes > self.glyph_capacities[buf_idx] {
            let new_cap = (self.glyph_capacities[buf_idx] as f64 * 1.5) as u64;
            let new_cap = new_cap.max(needed_bytes);
            self.glyph_buffers[buf_idx] = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("acme glyph buffer"),
                size: new_cap,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
            self.glyph_capacities[buf_idx] = new_cap;
            self.stats.buffer_grows += 1;
        }
    }

    /// Recreates the GPU device, surface configuration, and all pipelines/atlases
    /// after device loss. Call this when `render()` returns `SurfaceAction::DeviceLost`.
    pub fn on_device_lost(&mut self) -> Result<(), RenderError> {
        tracing::info!("attempting GPU device recovery");

        let adapter = pollster::block_on(self._instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&self.surface),
            },
        ))
        .map_err(|_| RenderError::Adapter)?;

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .map_err(|error| RenderError::Device(error.to_string()))?;

        register_device_error_handlers(&device, &self.device_lost_flag);
        self.device = device;
        self.queue = queue;

        let safe_width = self.size.0.max(1);
        let safe_height = self.size.1.max(1);
        let mut config = self
            .surface
            .get_default_config(&adapter, safe_width, safe_height)
            .ok_or(RenderError::Adapter)?;
        config.present_mode = wgpu::PresentMode::Fifo;
        self.config = config;

        self.surface.configure(&self.device, &self.config);

        let (pipeline, text_pipeline, texture_layout, image_sampler) =
            build_render_resources(&self.device, &self.config);
        self.pipeline = pipeline;
        self.text_pipeline = text_pipeline;
        // Pages are recreated lazily on next use; the app restores their
        // content by calling `GlyphAtlas::clear()` after recovery.
        self.atlas_pages = AtlasPages::default();
        self.texture_layout = texture_layout;
        self.image_sampler = image_sampler;

        self.images.clear();
        let max_dimension = self.device.limits().max_texture_dimension_2d;
        for (key, image) in &self.live_images {
            let Ok(layout) = validate_image_upload(image.width(), image.height(), max_dimension)
            else {
                continue;
            };
            self.images.insert(
                *key,
                create_image_resource(
                    &self.device,
                    &self.queue,
                    &self.texture_layout,
                    &self.image_sampler,
                    image.clone(),
                    layout,
                ),
            );
        }

        // Recreate persistent buffers for the new device.
        let quad_cap = INITIAL_QUAD_CAPACITY * std::mem::size_of::<Instance>() as u64;
        let glyph_cap = INITIAL_GLYPH_CAPACITY * std::mem::size_of::<GlyphInstance>() as u64;
        for i in 0..2 {
            self.quad_buffers[i] = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("acme quad buffer"),
                size: quad_cap,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
            self.glyph_buffers[i] = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("acme glyph buffer"),
                size: glyph_cap,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                mapped_at_creation: false,
            });
            self.quad_capacities[i] = quad_cap;
            self.glyph_capacities[i] = glyph_cap;
        }
        self.current_frame = 0;
        self.stats = RenderStats::default();

        complete_recovery_state(
            &self.device_lost_flag,
            &mut self.status,
            &mut self.gpu_epoch,
        );
        tracing::info!("GPU device recovery successful");
        Ok(())
    }

    /// Simulates device loss for testing. Only available in test builds.
    #[cfg(test)]
    pub fn simulate_device_loss(&mut self) {
        self.device_lost_flag.store(true, Ordering::Release);
        self.status = SurfaceStatus::Recovering;
    }

    /// Ensures every (format, page) pair used by `draws` has its texture and bind group.
    fn ensure_glyph_pages(&mut self, draws: &[DrawBatchDesc]) {
        for draw in draws {
            if let DrawBatchDesc::Glyph(run) = draw {
                self.atlas_pages.ensure(
                    &self.device,
                    &self.texture_layout,
                    &self.image_sampler,
                    run.format,
                    run.page,
                );
            }
        }
    }

    /// Upload atlas regions from a [`Scene`]'s text primitives into their pages,
    /// deduplicating by [`UploadKey`] within the frame.
    /// Returns (total_count, skipped_count, bytes_uploaded).
    fn upload_scene_atlas(
        &mut self,
        scene: &Scene,
        uploaded: &mut HashSet<UploadKey>,
    ) -> (u64, u64, u64) {
        let mut total = 0u64;
        let mut skipped = 0u64;
        let mut atlas_bytes = 0u64;
        for cmd in scene.commands() {
            let DrawCommand::Text(prim) = cmd else {
                continue;
            };
            for upload in &prim.uploads {
                let key = match validate_scene_upload(upload) {
                    Ok(key) => key,
                    Err(reject) => {
                        tracing::warn!(page = upload.page, ?reject, "rejected glyph atlas upload");
                        continue;
                    }
                };
                total += 1;
                if !uploaded.insert(key) {
                    skipped += 1;
                    continue;
                }
                let bytes_per_pixel = scene_bytes_per_pixel(upload.format) as u32;
                let [x, y] = upload.origin;
                let [width, height] = upload.size;
                let page = self.atlas_pages.ensure(
                    &self.device,
                    &self.texture_layout,
                    &self.image_sampler,
                    upload.format,
                    upload.page,
                );
                self.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &page.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x, y, z: 0 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &upload.pixels,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(width * bytes_per_pixel),
                        rows_per_image: Some(height),
                    },
                    wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                );
                atlas_bytes += width as u64 * height as u64 * bytes_per_pixel as u64;
            }
        }
        (total, skipped, atlas_bytes)
    }
}

fn image_resource_key(image: &Rgba8Image) -> ImageResourceKey {
    (image.key(), image.revision())
}

fn collect_scene_images(scene: &Scene) -> HashMap<ImageResourceKey, &Rgba8Image> {
    scene
        .commands()
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Image(primitive) => {
                Some((image_resource_key(&primitive.image), &primitive.image))
            }
            _ => None,
        })
        .collect()
}

fn retain_live_image_resources<T>(
    resources: &mut HashMap<ImageResourceKey, T>,
    live: &HashSet<ImageResourceKey>,
) {
    resources.retain(|key, _| live.contains(key));
}

fn validate_image_upload(
    width: u32,
    height: u32,
    max_dimension: u32,
) -> Result<ImageUploadLayout, ImageUploadError> {
    if width > max_dimension || height > max_dimension {
        return Err(ImageUploadError::ExceedsMaxDimension {
            width,
            height,
            max: max_dimension,
        });
    }
    let bytes_per_row = width
        .checked_mul(4)
        .ok_or(ImageUploadError::RowPitchOverflow)?;
    Ok(ImageUploadLayout { bytes_per_row })
}

fn create_image_resource(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    image: Rgba8Image,
    upload_layout: ImageUploadLayout,
) -> ImageResource {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("acme rgba8 image"),
        size: wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    write_rgba8_texture(queue, &texture, &image, upload_layout);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("acme rgba8 image bind group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });
    ImageResource {
        image,
        _texture: texture,
        bind_group,
    }
}

fn write_rgba8_texture(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    image: &Rgba8Image,
    upload_layout: ImageUploadLayout,
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        image.pixels(),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(upload_layout.bytes_per_row),
            rows_per_image: Some(image.height()),
        },
        wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        },
    );
}

/// Shared helper: creates pipelines, the glyph atlas bind group layout, and the
/// image sampler from a device + config. Glyph atlas pages are created lazily by
/// [`AtlasPages`]. Used by both `Renderer::new()` and `Renderer::on_device_lost()`.
fn build_render_resources(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> (
    wgpu::RenderPipeline,
    wgpu::RenderPipeline,
    wgpu::BindGroupLayout,
    wgpu::Sampler,
) {
    let shader = device.create_shader_module(wgpu::include_wgsl!("quad.wgsl"));
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("acme quad pipeline"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Instance>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });
    let text_shader = device.create_shader_module(wgpu::include_wgsl!("text.wgsl"));
    let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("acme glyph atlas layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });
    let text_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("acme text pipeline layout"),
        bind_group_layouts: &[Some(&texture_layout)],
        immediate_size: 0,
    });
    let text_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("acme text pipeline"),
        layout: Some(&text_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &text_shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<GlyphInstance>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &text_shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("acme linear image sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    (pipeline, text_pipeline, texture_layout, sampler)
}

fn scissor(rect: [f32; 4], scale: f32, width: u32, height: u32) -> Option<[u32; 4]> {
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

fn normalize_scale(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}
fn normalize_color(mut color: [f32; 4]) -> [f32; 4] {
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
/// [`Instance`] by applying scale factor and normalizing colors.
fn quad_primitive_to_instance(
    prim: &acme_core::QuadPrimitive,
    scale: f32,
    config_w: u32,
    config_h: u32,
) -> Instance {
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
    }
}

fn image_primitive_to_instance(
    primitive: &ImagePrimitive,
    scale: f32,
    width: u32,
    height: u32,
) -> GlyphInstance {
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
    }
}

/// Extracts `(atlas_page, GlyphInstance)` pairs from a [`TextPrimitive`] for
/// `format`, in glyph order. Applies origin offset and scale in the same way as
/// the original `glyph_instances` helper.
///
/// Glyphs whose `atlas_page` is `>= ATLAS_MAX_PAGES` are skipped rather than
/// clamped: clamping would sample another page's texels. UVs are normalized by
/// [`ATLAS_PAGE_EDGE`].
fn text_primitive_glyphs(
    prim: &acme_core::TextPrimitive,
    format: SceneGlyphFormat,
    scale: f32,
    width: u32,
    height: u32,
) -> Vec<(u32, GlyphInstance)> {
    let color = normalize_color([prim.color.r, prim.color.g, prim.color.b, prim.color.a]);
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
            };
            (glyph.atlas_page, instance)
        })
        .collect()
}

/// Collects the glyphs of every text command in `range` for `format`, in
/// painter order.
fn collect_text_glyphs(
    scene: &Scene,
    range: std::ops::Range<usize>,
    format: SceneGlyphFormat,
    scale: f32,
    width: u32,
    height: u32,
) -> Vec<(u32, GlyphInstance)> {
    range
        .filter_map(|index| match &scene.commands()[index] {
            DrawCommand::Text(prim) => {
                Some(text_primitive_glyphs(prim, format, scale, width, height))
            }
            _ => None,
        })
        .flatten()
        .collect()
}

/// Appends one text batch's glyphs to `instances` and records one draw per
/// maximal same-page run. Instances stay contiguous and in painter order.
fn push_text_batch(
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

/// Bytes per texel for a scene glyph format.
fn scene_bytes_per_pixel(format: SceneGlyphFormat) -> usize {
    match format {
        SceneGlyphFormat::Alpha8 => 1,
        SceneGlyphFormat::Rgba8 => 4,
    }
}

/// Validates one scene atlas upload and returns its dedup key. The key includes
/// page and format, so the same rectangle on another page is a distinct region.
fn validate_scene_upload(upload: &acme_core::AtlasUpload) -> Result<UploadKey, UploadReject> {
    check_upload(
        upload.page,
        upload.origin,
        upload.size,
        scene_bytes_per_pixel(upload.format),
        upload.pixels.len(),
    )?;
    Ok(upload_key(
        upload.page,
        upload.format,
        upload.origin,
        upload.size,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_image(revision: u64, width: u32, height: u32) -> Rgba8Image {
        Rgba8Image::new(
            ImageKey::new(1),
            revision,
            width,
            height,
            vec![255; width as usize * height as usize * 4],
        )
        .unwrap()
    }

    #[test]
    fn same_key_revisions_are_distinct_resources_within_one_scene() {
        let mut scene = Scene::new();
        for revision in [10, 11] {
            scene.push(DrawCommand::Image(ImagePrimitive::contain(
                test_image(revision, 2, 2),
                acme_core::Rect::new(0.0, 0.0, 10.0, 10.0),
            )));
        }

        let images = collect_scene_images(&scene);
        assert_eq!(images.len(), 2);
        assert!(images.contains_key(&(ImageKey::new(1), 10)));
        assert!(images.contains_key(&(ImageKey::new(1), 11)));
    }

    #[test]
    fn image_resource_retention_is_bounded_to_live_scene_keys() {
        let mut resources = HashMap::from([
            ((ImageKey::new(1), 1), "old"),
            ((ImageKey::new(1), 2), "current"),
            ((ImageKey::new(2), 1), "removed"),
        ]);
        let live = HashSet::from([(ImageKey::new(1), 2)]);
        retain_live_image_resources(&mut resources, &live);

        assert_eq!(resources.len(), 1);
        assert_eq!(resources[&(ImageKey::new(1), 2)], "current");
    }

    #[test]
    fn image_upload_layout_rejects_device_limit_and_row_pitch_overflow() {
        assert_eq!(
            validate_image_upload(4097, 10, 4096),
            Err(ImageUploadError::ExceedsMaxDimension {
                width: 4097,
                height: 10,
                max: 4096,
            })
        );
        assert_eq!(
            validate_image_upload(u32::MAX, 1, u32::MAX),
            Err(ImageUploadError::RowPitchOverflow)
        );
        assert_eq!(
            validate_image_upload(1920, 1080, 8192),
            Ok(ImageUploadLayout {
                bytes_per_row: 7680,
            })
        );
    }

    #[test]
    fn image_instance_uses_contained_rect_and_rgba_mode() {
        let primitive = ImagePrimitive::contain(
            test_image(0, 2, 1),
            acme_core::Rect::new(0.0, 0.0, 100.0, 100.0),
        );
        let instance = image_primitive_to_instance(&primitive, 2.0, 800, 600);
        assert_eq!(instance.rect, [0.0, 50.0, 200.0, 100.0]);
        assert_eq!(instance.uv, [0.0, 0.0, 1.0, 1.0]);
        assert_eq!(instance.viewport_mode, [800.0, 600.0, 1.0, 0.0]);
    }

    #[test]
    fn solid_quad_defaults_are_valid() {
        let q = Quad::solid([1.0, 2.0, 3.0, 4.0], [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(q.border_width, 0.0);
    }
    #[test]
    fn empty_frame_has_opaque_clear() {
        assert_eq!(Frame::default().clear[3], 1.0);
    }

    #[test]
    fn scissor_intersects_the_physical_viewport() {
        assert_eq!(
            scissor([-5.0, 4.0, 20.0, 10.0], 2.0, 100, 100),
            Some([0, 8, 30, 20])
        );
        assert_eq!(
            scissor([0.25, 1.5, 10.75, 20.125], 2.0, 100, 100),
            Some([0, 3, 22, 41])
        );
        assert_eq!(scissor([200.0, 0.0, 10.0, 10.0], 1.0, 100, 100), None);
        assert_eq!(scissor([10.0, 10.0, 0.0, 10.0], 1.0, 100, 100), None);
        assert_eq!(scissor([f32::NAN, 0.0, 10.0, 10.0], 1.0, 100, 100), None);
    }

    #[test]
    fn device_lost_action_is_distinct() {
        assert_ne!(SurfaceAction::DeviceLost, SurfaceAction::Exit);
        assert_ne!(SurfaceAction::DeviceLost, SurfaceAction::Reconfigure);
    }

    #[test]
    fn resolve_surface_action_covers_all_transitions() {
        use AcquireOutcome::*;
        let outcomes = [
            Success,
            Suboptimal,
            OutdatedOrLost,
            TimeoutOrOccluded,
            Validation,
        ];

        // Suspended always wins → Skip, regardless of device_lost or acquire.
        for &acquire in &outcomes {
            for &device_lost in &[false, true] {
                assert_eq!(
                    resolve_surface_action(true, device_lost, acquire),
                    SurfaceAction::Skip,
                    "suspended must Skip (device_lost={device_lost}, acquire={acquire:?})"
                );
            }
        }

        // Not suspended, device_lost → DeviceLost, regardless of acquire.
        for &acquire in &outcomes {
            assert_eq!(
                resolve_surface_action(false, true, acquire),
                SurfaceAction::DeviceLost,
                "device_lost must map to DeviceLost (acquire={acquire:?})"
            );
        }

        // Not suspended, not device_lost → depends on acquire outcome.
        assert_eq!(
            resolve_surface_action(false, false, Success),
            SurfaceAction::Rendered
        );
        assert_eq!(
            resolve_surface_action(false, false, Suboptimal),
            SurfaceAction::Rendered
        );
        assert_eq!(
            resolve_surface_action(false, false, OutdatedOrLost),
            SurfaceAction::Reconfigure
        );
        assert_eq!(
            resolve_surface_action(false, false, TimeoutOrOccluded),
            SurfaceAction::Skip
        );
        assert_eq!(
            resolve_surface_action(false, false, Validation),
            SurfaceAction::Skip
        );
    }

    #[test]
    fn recovery_resets_pure_state_and_bumps_epoch() {
        // Mimic post-device-loss state: device_lost=true, status=Recovering.
        let device_lost = AtomicBool::new(true);
        let mut status = SurfaceStatus::Recovering;
        let mut gpu_epoch = 7u64;
        complete_recovery_state(&device_lost, &mut status, &mut gpu_epoch);
        assert!(
            !is_device_lost(&device_lost),
            "device_lost must clear after recovery"
        );
        assert_eq!(status, SurfaceStatus::Ready, "status must return to Ready");
        assert_eq!(gpu_epoch, 8, "gpu_epoch must increment by one");
    }

    #[test]
    fn device_lost_flag_maps_to_device_lost_action() {
        let flag = AtomicBool::new(false);
        assert_eq!(
            resolve_surface_action(false, is_device_lost(&flag), AcquireOutcome::Success),
            SurfaceAction::Rendered
        );
        flag.store(true, Ordering::Release);
        assert_eq!(
            resolve_surface_action(false, is_device_lost(&flag), AcquireOutcome::Success),
            SurfaceAction::DeviceLost
        );
    }

    #[test]
    #[ignore = "requires real GPU adapter; run with cargo test -p acme-render-wgpu -- --ignored"]
    fn device_recovery_smoke_ignored() {
        // Adapter + device + handler registration only (no window / surface).
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: true,
            compatible_surface: None,
        }));
        let Ok(adapter) = adapter else {
            return;
        };
        let Ok((device, _queue)) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
        else {
            return;
        };
        let flag = Arc::new(AtomicBool::new(false));
        register_device_error_handlers(&device, &flag);
        assert!(!is_device_lost(&flag));
    }

    #[test]
    fn invalid_scale_and_color_are_normalized() {
        assert_eq!(normalize_scale(f32::NAN), 1.0);
        assert_eq!(
            normalize_color([f32::NAN, -1.0, 2.0, 0.5]),
            [0.0, 0.0, 1.0, 0.5]
        );
    }

    fn scene_glyph(page: u32, x: f32) -> acme_core::GlyphDraw {
        acme_core::GlyphDraw {
            x,
            y: 0.0,
            width: 4.0,
            height: 4.0,
            atlas_x: 8,
            atlas_y: 16,
            atlas_page: page,
            format: SceneGlyphFormat::Alpha8,
        }
    }

    /// Alpha text primitive whose glyph `i` sits on `pages[i]` at x = `i`.
    fn text_with_pages(pages: &[u32]) -> acme_core::TextPrimitive {
        acme_core::TextPrimitive {
            origin: acme_core::Point::new(0.0, 0.0),
            color: acme_core::Color::rgba(1.0, 1.0, 1.0, 1.0),
            glyphs: pages
                .iter()
                .enumerate()
                .map(|(index, &page)| scene_glyph(page, index as f32))
                .collect(),
            uploads: Vec::new(),
        }
    }

    fn scene_upload(
        page: u32,
        format: SceneGlyphFormat,
        origin: [u32; 2],
    ) -> acme_core::AtlasUpload {
        let bpp = scene_bytes_per_pixel(format);
        acme_core::AtlasUpload {
            page,
            origin,
            size: [2, 2],
            format,
            pixels: vec![0; 4 * bpp],
        }
    }

    #[test]
    fn text_glyphs_carry_their_atlas_page() {
        let prim = text_with_pages(&[0, 0, 1, 1, 0]);
        let glyphs = text_primitive_glyphs(&prim, SceneGlyphFormat::Alpha8, 1.0, 800, 600);
        let pages: Vec<u32> = glyphs.iter().map(|(page, _)| *page).collect();
        assert_eq!(pages, vec![0, 0, 1, 1, 0]);
    }

    #[test]
    fn text_glyph_uv_is_normalized_by_page_edge() {
        let mut prim = text_with_pages(&[0]);
        prim.glyphs[0].atlas_x = 1024;
        prim.glyphs[0].atlas_y = 512;
        let glyphs = text_primitive_glyphs(&prim, SceneGlyphFormat::Alpha8, 1.0, 800, 600);
        let uv = glyphs[0].1.uv;
        let edge = ATLAS_PAGE_EDGE as f32;
        assert_eq!(uv, [1024.0 / edge, 512.0 / edge, 4.0 / edge, 4.0 / edge]);
    }

    #[test]
    fn text_glyph_with_out_of_range_page_is_skipped() {
        let prim = text_with_pages(&[0, ATLAS_MAX_PAGES, 1, u32::MAX]);
        let glyphs = text_primitive_glyphs(&prim, SceneGlyphFormat::Alpha8, 1.0, 800, 600);
        let pages: Vec<u32> = glyphs.iter().map(|(page, _)| *page).collect();
        assert_eq!(pages, vec![0, 1]);
        let xs: Vec<f32> = glyphs.iter().map(|(_, glyph)| glyph.rect[0]).collect();
        assert_eq!(xs, vec![0.0, 2.0]);
    }

    #[test]
    fn text_batch_splits_into_per_page_runs_in_painter_order() {
        let prim = text_with_pages(&[0, 0, 1, 1, 0]);
        let glyphs = text_primitive_glyphs(&prim, SceneGlyphFormat::Alpha8, 1.0, 800, 600);
        let mut instances = Vec::new();
        let mut draws = Vec::new();
        push_text_batch(
            glyphs,
            SceneGlyphFormat::Alpha8,
            None,
            &mut instances,
            &mut draws,
        );

        let runs: Vec<(u32, u32, u32)> = draws
            .iter()
            .filter_map(|draw| match draw {
                DrawBatchDesc::Glyph(run) => Some((run.page, run.start, run.count)),
                _ => None,
            })
            .collect();
        assert_eq!(runs, vec![(0, 0, 2), (1, 2, 2), (0, 4, 1)]);
        assert_eq!(draws.len(), 3, "one draw per page run");
        let xs: Vec<f32> = instances.iter().map(|glyph| glyph.rect[0]).collect();
        assert_eq!(xs, vec![0.0, 1.0, 2.0, 3.0, 4.0], "painter order kept");
    }

    #[test]
    fn text_batch_with_no_glyphs_emits_no_draws() {
        let mut instances = Vec::new();
        let mut draws = Vec::new();
        push_text_batch(
            Vec::new(),
            SceneGlyphFormat::Rgba8,
            None,
            &mut instances,
            &mut draws,
        );
        assert!(instances.is_empty());
        assert!(draws.is_empty());
    }

    #[test]
    fn scene_upload_key_distinguishes_page_and_format_for_the_same_rect() {
        let uploads = [
            scene_upload(0, SceneGlyphFormat::Alpha8, [2, 3]),
            scene_upload(1, SceneGlyphFormat::Alpha8, [2, 3]),
            scene_upload(0, SceneGlyphFormat::Rgba8, [2, 3]),
        ];
        let keys: HashSet<_> = uploads
            .iter()
            .map(|upload| validate_scene_upload(upload).expect("valid upload"))
            .collect();
        assert_eq!(
            keys.len(),
            3,
            "same x,y,w,h on other page/format is distinct"
        );
    }

    #[test]
    fn scene_upload_validation_rejects_out_of_range_page() {
        let upload = scene_upload(ATLAS_MAX_PAGES, SceneGlyphFormat::Alpha8, [0, 0]);
        assert_eq!(
            validate_scene_upload(&upload),
            Err(UploadReject::PageOutOfRange)
        );
        let last = scene_upload(ATLAS_MAX_PAGES - 1, SceneGlyphFormat::Alpha8, [0, 0]);
        assert!(validate_scene_upload(&last).is_ok());
    }
}
