//! GPU surface lifecycle and the small batched rectangle renderer used by AcmeUI.
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod golden;

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{any::Any, sync::Arc};

use acme_text::{AtlasFormat, PreparedText};
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
    /// Percentage of atlas uploads that were already resident (0.0-100.0).
    pub atlas_hit_rate: f64,
}

impl RenderStats {
    /// Compact one-line summary for debug / devtools.
    pub fn summary(&self) -> String {
        format!(
            "quads={} glyphs={} draws={} grows={} uploaded={}B atlas_hit={:.1}%",
            self.quad_count,
            self.glyph_count,
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

const ATLAS_SIZE: u32 = 2048;

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
    alpha_atlas: wgpu::Texture,
    rgba_atlas: wgpu::Texture,
    alpha_bind_group: wgpu::BindGroup,
    rgba_bind_group: wgpu::BindGroup,
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
        let (pipeline, text_pipeline, alpha_atlas, rgba_atlas, alpha_bind_group, rgba_bind_group) =
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
            alpha_atlas,
            rgba_atlas,
            alpha_bind_group,
            rgba_bind_group,
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

    pub fn render(&mut self, frame: &Frame) -> SurfaceAction {
        let suspended = self.status == SurfaceStatus::Suspended;
        let lost = is_device_lost(&self.device_lost_flag);
        // Decide fast-exit actions without touching the surface where possible.
        let pre_acquire_action = resolve_surface_action(suspended, lost, AcquireOutcome::Success);
        if pre_acquire_action != SurfaceAction::Rendered {
            return pre_acquire_action;
        }
        let mut reconfigure_after_present = false;
        let acquired = self.surface.get_current_texture();
        // Re-check after acquire in case a callback fired concurrently.
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
            // Handle the reconfigure side effect for the outdated/lost case.
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
            // Unreachable: any non-render outcome returned above.
            _ => return SurfaceAction::Skip,
        };
        // ---- atlas uploads with per-frame dedup ----
        let mut uploaded_regions: HashSet<(u32, u32, u32, u32)> = HashSet::new();
        let mut atlas_total = 0u64;
        let mut atlas_skipped = 0u64;
        let mut atlas_bytes = 0u64;
        for run in &frame.text {
            let (total, skipped, bytes) = self.upload_atlas(&run.prepared, &mut uploaded_regions);
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
        // ---- prepare draw data ----
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // Regular (unclipped) quads.
        let regular_instances: Vec<Instance> =
            frame.quads.iter().map(|q| self.quad_instance(q)).collect();
        // Clip batching - group clipped quads by their clip (rounded to nearest integer).
        // Store the scissor rect alongside so DPI scaling is respected.
        let mut clip_groups: std::collections::HashMap<[u32; 4], (Vec<Instance>, [u32; 4])> =
            std::collections::HashMap::new();
        for clipped in &frame.clipped_quads {
            let key = clip_key(clipped.clip);
            if let Some(scissor_rect) = scissor(
                clipped.clip,
                self.scale_factor,
                self.config.width,
                self.config.height,
            ) {
                let entry = clip_groups
                    .entry(key)
                    .or_insert_with(|| (Vec::new(), scissor_rect));
                entry.0.push(self.quad_instance(&clipped.quad));
            }
        }
        // Collect text groups by (format, clip).
        let mut text_groups: std::collections::HashMap<
            (AtlasFormat, Option<[u32; 4]>),
            Vec<GlyphInstance>,
        > = std::collections::HashMap::new();
        for run in &frame.text {
            let clip = run.clip.and_then(|value| {
                scissor(
                    value,
                    self.scale_factor,
                    self.config.width,
                    self.config.height,
                )
            });
            if run.clip.is_some() && clip.is_none() {
                continue;
            }
            for format in [AtlasFormat::Alpha8, AtlasFormat::Rgba8] {
                let glyphs = glyph_instances(
                    run,
                    format,
                    self.scale_factor,
                    self.config.width,
                    self.config.height,
                );
                if !glyphs.is_empty() {
                    text_groups
                        .entry((format, clip))
                        .or_default()
                        .extend(glyphs);
                }
            }
        }
        // ---- flatten and write persistent quad buffer ----
        let buf_idx = self.current_frame;
        let instance_size = std::mem::size_of::<Instance>() as u64;
        let mut all_quads: Vec<Instance> = regular_instances;
        struct ClipBatch {
            scissor: [u32; 4],
            start: u32,
            count: u32,
        }
        let mut clip_batches: Vec<ClipBatch> = Vec::new();
        for (quads, scissor_rect) in clip_groups.values() {
            let start = all_quads.len() as u32;
            let count = quads.len() as u32;
            all_quads.extend_from_slice(quads);
            clip_batches.push(ClipBatch {
                scissor: *scissor_rect,
                start,
                count,
            });
        }
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
        // ---- flatten and write persistent glyph buffer ----
        let glyph_size = std::mem::size_of::<GlyphInstance>() as u64;
        struct GlyphBatch {
            format: AtlasFormat,
            clip: Option<[u32; 4]>,
            start: u32,
            count: u32,
        }
        let mut all_glyphs: Vec<GlyphInstance> = Vec::new();
        let mut glyph_batches: Vec<GlyphBatch> = Vec::new();
        for ((format, clip), instances) in &text_groups {
            let start = all_glyphs.len() as u32;
            let count = instances.len() as u32;
            all_glyphs.extend_from_slice(instances);
            glyph_batches.push(GlyphBatch {
                format: *format,
                clip: *clip,
                start,
                count,
            });
        }
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
        // ---- render pass ----
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
                            r: frame.clear[0] as f64,
                            g: frame.clear[1] as f64,
                            b: frame.clear[2] as f64,
                            a: frame.clear[3] as f64,
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
            // - draw regular quads (full viewport) -
            let regular_count = frame.quads.len() as u32;
            if regular_count > 0 {
                pass.set_pipeline(&self.pipeline);
                pass.set_vertex_buffer(0, self.quad_buffers[buf_idx].slice(..));
                pass.draw(0..6, 0..regular_count);
                draw_count += 1;
            }
            // - draw each clip batch with its scissor rect -
            for batch in &clip_batches {
                pass.set_scissor_rect(
                    batch.scissor[0],
                    batch.scissor[1],
                    batch.scissor[2],
                    batch.scissor[3],
                );
                pass.set_pipeline(&self.pipeline);
                pass.set_vertex_buffer(0, self.quad_buffers[buf_idx].slice(..));
                pass.draw(0..6, batch.start..batch.start + batch.count);
                draw_count += 1;
            }
            // - draw text batches -
            pass.set_scissor_rect(0, 0, self.config.width, self.config.height);
            for batch in &glyph_batches {
                let clip = batch
                    .clip
                    .unwrap_or([0, 0, self.config.width, self.config.height]);
                pass.set_scissor_rect(clip[0], clip[1], clip[2], clip[3]);
                pass.set_pipeline(&self.text_pipeline);
                pass.set_bind_group(
                    0,
                    match batch.format {
                        AtlasFormat::Alpha8 => &self.alpha_bind_group,
                        AtlasFormat::Rgba8 => &self.rgba_bind_group,
                    },
                    &[],
                );
                pass.set_vertex_buffer(0, self.glyph_buffers[buf_idx].slice(..));
                pass.draw(0..6, batch.start..batch.start + batch.count);
                draw_count += 1;
            }
            self.stats.draw_calls = draw_count;
        }
        self.queue.submit(Some(encoder.finish()));
        output.present();
        // ---- update stats ----
        self.stats.quad_count = quad_instances_needed;
        self.stats.glyph_count = all_glyphs.len() as u64;
        // ---- flip frame ring ----
        self.current_frame ^= 1;
        if reconfigure_after_present {
            self.surface.configure(&self.device, &self.config);
            SurfaceAction::Reconfigure
        } else {
            SurfaceAction::Rendered
        }
    }

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

        let (pipeline, text_pipeline, alpha_atlas, rgba_atlas, alpha_bind_group, rgba_bind_group) =
            build_render_resources(&self.device, &self.config);
        self.pipeline = pipeline;
        self.text_pipeline = text_pipeline;
        self.alpha_atlas = alpha_atlas;
        self.rgba_atlas = rgba_atlas;
        self.alpha_bind_group = alpha_bind_group;
        self.rgba_bind_group = rgba_bind_group;

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

    fn upload_atlas(
        &self,
        prepared: &PreparedText,
        uploaded: &mut HashSet<(u32, u32, u32, u32)>,
    ) -> (u64, u64, u64) {
        let mut total = 0u64;
        let mut skipped = 0u64;
        let mut atlas_bytes = 0u64;
        for upload in &prepared.uploads {
            if upload.width == 0
                || upload.height == 0
                || upload.x.saturating_add(upload.width) > ATLAS_SIZE
                || upload.y.saturating_add(upload.height) > ATLAS_SIZE
            {
                continue;
            }
            let bytes_per_pixel = match upload.format {
                AtlasFormat::Alpha8 => 1,
                AtlasFormat::Rgba8 => 4,
            };
            let expected = upload.width as usize * upload.height as usize * bytes_per_pixel;
            if upload.pixels.len() != expected {
                tracing::warn!("rejected malformed glyph atlas upload");
                continue;
            }
            total += 1;
            if !uploaded.insert((upload.x, upload.y, upload.width, upload.height)) {
                skipped += 1;
                continue;
            }
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: match upload.format {
                        AtlasFormat::Alpha8 => &self.alpha_atlas,
                        AtlasFormat::Rgba8 => &self.rgba_atlas,
                    },
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: upload.x,
                        y: upload.y,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &upload.pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(upload.width * bytes_per_pixel as u32),
                    rows_per_image: Some(upload.height),
                },
                wgpu::Extent3d {
                    width: upload.width,
                    height: upload.height,
                    depth_or_array_layers: 1,
                },
            );
            atlas_bytes += upload.width as u64 * upload.height as u64 * bytes_per_pixel as u64;
        }
        (total, skipped, atlas_bytes)
    }
}

/// Shared helper: creates pipelines, atlas textures, and bind groups from a device + config.
/// Used by both `Renderer::new()` and `Renderer::on_device_lost()`.
fn build_render_resources(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> (
    wgpu::RenderPipeline,
    wgpu::RenderPipeline,
    wgpu::Texture,
    wgpu::Texture,
    wgpu::BindGroup,
    wgpu::BindGroup,
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
    let alpha_atlas = create_atlas(device, wgpu::TextureFormat::R8Unorm, "alpha");
    let rgba_atlas = create_atlas(device, wgpu::TextureFormat::Rgba8UnormSrgb, "rgba");
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("acme glyph sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let alpha_bind_group =
        create_atlas_bind_group(device, &texture_layout, &alpha_atlas, &sampler, "alpha");
    let rgba_bind_group =
        create_atlas_bind_group(device, &texture_layout, &rgba_atlas, &sampler, "rgba");
    (
        pipeline,
        text_pipeline,
        alpha_atlas,
        rgba_atlas,
        alpha_bind_group,
        rgba_bind_group,
    )
}

fn create_atlas(device: &wgpu::Device, format: wgpu::TextureFormat, name: &str) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(name),
        size: wgpu::Extent3d {
            width: ATLAS_SIZE,
            height: ATLAS_SIZE,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn create_atlas_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    texture: &wgpu::Texture,
    sampler: &wgpu::Sampler,
    name: &str,
) -> wgpu::BindGroup {
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(name),
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
    })
}

fn glyph_instances(
    run: &TextRun,
    format: AtlasFormat,
    scale: f32,
    width: u32,
    height: u32,
) -> Vec<GlyphInstance> {
    let color = normalize_color(run.color);
    run.prepared
        .glyphs
        .iter()
        .filter(|glyph| glyph.format == format)
        .map(|glyph| GlyphInstance {
            rect: [
                glyph.x as f32 + run.origin[0] * scale,
                glyph.y as f32 + run.origin[1] * scale,
                glyph.width as f32,
                glyph.height as f32,
            ],
            uv: [
                glyph.atlas_x as f32 / ATLAS_SIZE as f32,
                glyph.atlas_y as f32 / ATLAS_SIZE as f32,
                glyph.width as f32 / ATLAS_SIZE as f32,
                glyph.height as f32 / ATLAS_SIZE as f32,
            ],
            color,
            viewport_mode: [
                width as f32,
                height as f32,
                if format == AtlasFormat::Rgba8 {
                    1.0
                } else {
                    0.0
                },
                0.0,
            ],
        })
        .collect()
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

/// Round clip rect components to nearest integer for use as a grouping key.
fn clip_key(clip: [f32; 4]) -> [u32; 4] {
    [
        (clip[0].round().max(0.0)) as u32,
        (clip[1].round().max(0.0)) as u32,
        clip[2].round().max(0.0) as u32,
        clip[3].round().max(0.0) as u32,
    ]
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

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(scissor([200.0, 0.0, 10.0, 10.0], 1.0, 100, 100), None);
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
}
