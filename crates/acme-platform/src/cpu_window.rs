//! Native presentation adapter for the independent headless CPU rasterizer.
//! This module is private so softbuffer/winit types never enter public AcmeUI APIs.

use std::num::NonZeroU32;
use std::sync::Arc;

use acme_core::Scene;
use acme_render_cpu::CpuRenderer;
use softbuffer::{Context, Surface};
use winit::window::Window;

pub(crate) struct CpuWindowRenderer {
    raster: CpuRenderer,
    _context: Context<Arc<Window>>,
    surface: Surface<Arc<Window>, Arc<Window>>,
}

impl CpuWindowRenderer {
    pub(crate) fn new(window: Arc<Window>, width: u32, height: u32, scale: f32) -> Result<Self, String> {
        let context = Context::new(window.clone()).map_err(|e| format!("software context: {e}"))?;
        let surface = Surface::new(&context, window).map_err(|e| format!("software surface: {e}"))?;
        let mut instance = Self {
            raster: CpuRenderer::new(width, height, scale).map_err(|e| e.to_string())?,
            _context: context,
            surface,
        };
        instance.resize(width, height, scale)?;
        Ok(instance)
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32, scale: f32) -> Result<(), String> {
        self.raster.resize(width, height, scale).map_err(|e| e.to_string())?;
        if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
            self.surface.resize(w, h).map_err(|e| format!("software resize: {e}"))?;
        }
        Ok(())
    }

    pub(crate) fn present(&mut self, scene: &Scene) -> Result<(), String> {
        if self.raster.dimensions().0 == 0 || self.raster.dimensions().1 == 0 {
            return Ok(());
        }
        self.raster.render_scene(scene).map_err(|e| e.to_string())?;
        let mut buffer = self.surface.buffer_mut().map_err(|e| format!("software buffer: {e}"))?;
        if buffer.len() != self.raster.pixels().len() {
            return Err("software buffer dimensions mismatch".into());
        }
        buffer.copy_from_slice(self.raster.pixels());
        buffer.present().map_err(|e| format!("software present: {e}"))
    }
}
