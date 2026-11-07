//! GPU device-loss recovery and the test-only device-loss simulation hook.

#[cfg(test)]
use std::sync::atomic::Ordering;

#[cfg(test)]
use crate::SurfaceStatus;
use crate::atlas_pages::AtlasPages;
use crate::images::{create_image_resource, validate_image_upload};
use crate::instances::{GlyphInstance, Instance};
use crate::pipelines::build_render_resources;
use crate::surface::{complete_recovery_state, register_device_error_handlers};
use crate::{RenderError, RenderStats};

use super::{INITIAL_GLYPH_CAPACITY, INITIAL_QUAD_CAPACITY, Renderer};

impl Renderer {
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
}
