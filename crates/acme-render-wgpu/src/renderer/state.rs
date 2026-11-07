//! Renderer state shared by its implementation modules.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use acme_core::Rgba8Image;

use crate::atlas_pages::AtlasPages;
use crate::images::{ImageResource, ImageResourceKey};
use crate::{RenderStats, SurfaceStatus};

/// Owns a wgpu surface. Third-party GPU types never appear in its public API.
pub struct Renderer {
    pub(super) _instance: wgpu::Instance,
    pub(super) surface: wgpu::Surface<'static>,
    pub(super) device: wgpu::Device,
    pub(super) queue: wgpu::Queue,
    pub(super) config: wgpu::SurfaceConfiguration,
    pub(super) pipeline: wgpu::RenderPipeline,
    pub(super) text_pipeline: wgpu::RenderPipeline,
    /// Lazily created glyph atlas pages, one texture per (format, page).
    pub(super) atlas_pages: AtlasPages,
    pub(super) texture_layout: wgpu::BindGroupLayout,
    pub(super) image_sampler: wgpu::Sampler,
    pub(super) live_images: HashMap<ImageResourceKey, Rgba8Image>,
    pub(super) images: HashMap<ImageResourceKey, ImageResource>,
    pub(super) size: (u32, u32),
    pub(super) status: SurfaceStatus,
    pub(super) scale_factor: f32,
    /// Shared with wgpu uncaptured-error / device-lost callbacks.
    pub(super) device_lost_flag: Arc<AtomicBool>,
    pub(super) gpu_epoch: u64,
    // Persistent double-buffered instance buffers.
    pub(super) quad_buffers: [wgpu::Buffer; 2],
    pub(super) glyph_buffers: [wgpu::Buffer; 2],
    pub(super) quad_capacities: [u64; 2],
    pub(super) glyph_capacities: [u64; 2],
    pub(super) current_frame: usize,
    pub(super) stats: RenderStats,
}
