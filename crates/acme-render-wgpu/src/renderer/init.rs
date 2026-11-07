//! Renderer construction, surface resize and read-only accessors.

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::atlas_pages::AtlasPages;
use crate::instances::{GlyphInstance, Instance, normalize_scale};
use crate::pipelines::build_render_resources;
use crate::surface::register_device_error_handlers;
use crate::{RenderError, RenderStats, SurfaceStatus};

use super::{INITIAL_GLYPH_CAPACITY, INITIAL_QUAD_CAPACITY, Renderer};

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
}
