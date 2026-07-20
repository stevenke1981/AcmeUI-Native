//! GPU surface lifecycle and the small batched rectangle renderer used by AcmeUI.
#![forbid(unsafe_op_in_unsafe_fn)]

use std::{any::Any, sync::Arc};

use acme_text::{AtlasFormat, PreparedText};
use bytemuck::{Pod, Zeroable};
use thiserror::Error;
use wgpu::util::DeviceExt;

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
    device_lost: bool,
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
        let safe_width = width.max(1);
        let safe_height = height.max(1);
        let mut config = surface
            .get_default_config(&adapter, safe_width, safe_height)
            .ok_or(RenderError::Adapter)?;
        config.present_mode = wgpu::PresentMode::Fifo;
        surface.configure(&device, &config);
        let (pipeline, text_pipeline, alpha_atlas, rgba_atlas, alpha_bind_group, rgba_bind_group) =
            build_render_resources(&device, &config);
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
            device_lost: false,
        })
    }

    pub fn status(&self) -> SurfaceStatus {
        self.status
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
        if self.status == SurfaceStatus::Suspended {
            return SurfaceAction::Skip;
        }
        if self.device_lost {
            return SurfaceAction::DeviceLost;
        }
        let mut reconfigure_after_present = false;
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                reconfigure_after_present = true;
                frame
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                self.status = SurfaceStatus::Recovering;
                self.surface.configure(&self.device, &self.config);
                self.status = SurfaceStatus::Ready;
                return SurfaceAction::Reconfigure;
            }
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return SurfaceAction::Skip;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                eprintln!("acme-render-wgpu: surface acquisition validation failure");
                return SurfaceAction::Skip;
            }
        };
        for run in &frame.text {
            self.upload_atlas(&run.prepared);
        }
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let instances: Vec<Instance> = frame.quads.iter().map(|q| self.quad_instance(q)).collect();
        let buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("acme quad instances"),
                contents: bytemuck::cast_slice(&instances),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let clipped_buffers: Vec<_> = frame
            .clipped_quads
            .iter()
            .filter_map(|clipped| {
                let clip = scissor(
                    clipped.clip,
                    self.scale_factor,
                    self.config.width,
                    self.config.height,
                )?;
                let one = [self.quad_instance(&clipped.quad)];
                let buffer = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("acme clipped quad"),
                        contents: bytemuck::cast_slice(&one),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
                Some((buffer, clip))
            })
            .collect();
        let mut text_buffers = Vec::new();
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
                    let buffer =
                        self.device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("acme glyph instances"),
                                contents: bytemuck::cast_slice(&glyphs),
                                usage: wgpu::BufferUsages::VERTEX,
                            });
                    text_buffers.push((buffer, glyphs.len() as u32, format, clip));
                }
            }
        }
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
            if !instances.is_empty() {
                pass.set_pipeline(&self.pipeline);
                pass.set_vertex_buffer(0, buffer.slice(..));
                pass.draw(0..6, 0..instances.len() as u32);
            }
            for (clipped_buffer, clip) in &clipped_buffers {
                pass.set_scissor_rect(clip[0], clip[1], clip[2], clip[3]);
                pass.set_pipeline(&self.pipeline);
                pass.set_vertex_buffer(0, clipped_buffer.slice(..));
                pass.draw(0..6, 0..1);
            }
            pass.set_scissor_rect(0, 0, self.config.width, self.config.height);
            for (text_buffer, count, format, clip) in &text_buffers {
                let clip = clip.unwrap_or([0, 0, self.config.width, self.config.height]);
                pass.set_scissor_rect(clip[0], clip[1], clip[2], clip[3]);
                pass.set_pipeline(&self.text_pipeline);
                pass.set_bind_group(
                    0,
                    match format {
                        AtlasFormat::Alpha8 => &self.alpha_bind_group,
                        AtlasFormat::Rgba8 => &self.rgba_bind_group,
                    },
                    &[],
                );
                pass.set_vertex_buffer(0, text_buffer.slice(..));
                pass.draw(0..6, 0..*count);
            }
        }
        self.queue.submit(Some(encoder.finish()));
        output.present();
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

        self.device_lost = false;
        self.status = SurfaceStatus::Ready;
        tracing::info!("GPU device recovery successful");
        Ok(())
    }

    /// Simulates device loss for testing. Only available in test builds.
    #[cfg(test)]
    pub fn simulate_device_loss(&mut self) {
        self.device_lost = true;
        self.status = SurfaceStatus::Recovering;
    }

    fn upload_atlas(&self, prepared: &PreparedText) {
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
                eprintln!("acme-render-wgpu: rejected malformed glyph atlas upload");
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
        }
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
    fn invalid_scale_and_color_are_normalized() {
        assert_eq!(normalize_scale(f32::NAN), 1.0);
        assert_eq!(
            normalize_color([f32::NAN, -1.0, 2.0, 0.5]),
            [0.0, 0.0, 1.0, 0.5]
        );
    }
}
