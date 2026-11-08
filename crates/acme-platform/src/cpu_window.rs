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
    surface_dims: (u32, u32),
    scale_factor: f32,
    _context: Context<Arc<Window>>,
    surface: Surface<Arc<Window>, Arc<Window>>,
}

/// Synchronize `raster` to `(width, height, scale)` when [`crate::needs_resize`]
/// reports a dimension or scale change, then render `scene`.
pub(crate) fn sync_and_render_raster(
    raster: &mut CpuRenderer,
    current_scale: &mut f32,
    width: u32,
    height: u32,
    scale: f32,
    scene: &Scene,
) -> Result<(), String> {
    if crate::needs_resize(raster.dimensions(), *current_scale, (width, height), scale) {
        raster
            .resize(width, height, scale)
            .map_err(|e| e.to_string())?;
        *current_scale = scale;
    }
    raster.render_scene(scene).map_err(|e| e.to_string())
}

impl CpuWindowRenderer {
    pub(crate) fn new(
        window: Arc<Window>,
        width: u32,
        height: u32,
        scale: f32,
    ) -> Result<Self, String> {
        let context = Context::new(window.clone()).map_err(|e| format!("software context: {e}"))?;
        let surface =
            Surface::new(&context, window).map_err(|e| format!("software surface: {e}"))?;
        let mut instance = Self {
            raster: CpuRenderer::new(width, height, scale).map_err(|e| e.to_string())?,
            surface_dims: (0, 0),
            scale_factor: scale,
            _context: context,
            surface,
        };
        instance.resize(width, height, scale)?;
        Ok(instance)
    }

    pub(crate) fn dimensions(&self) -> (u32, u32) {
        if self.surface_dims == self.raster.dimensions() {
            self.raster.dimensions()
        } else {
            self.surface_dims
        }
    }

    pub(crate) fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32, scale: f32) -> Result<(), String> {
        self.raster
            .resize(width, height, scale)
            .map_err(|e| e.to_string())?;
        self.scale_factor = scale;
        if let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) {
            self.surface
                .resize(w, h)
                .map_err(|e| format!("software resize: {e}"))?;
            self.surface_dims = (width, height);
        } else {
            self.surface_dims = (0, 0);
        }
        Ok(())
    }

    pub(crate) fn present(&mut self, scene: &Scene) -> Result<(), String> {
        let (width, height) = self.raster.dimensions();
        if width == 0 || height == 0 {
            return Ok(());
        }
        let scale = self.scale_factor;
        sync_and_render_raster(
            &mut self.raster,
            &mut self.scale_factor,
            width,
            height,
            scale,
            scene,
        )?;
        if self.surface_dims != (width, height)
            && let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height))
        {
            self.surface
                .resize(w, h)
                .map_err(|e| format!("software resize: {e}"))?;
            self.surface_dims = (width, height);
        }
        let expected_len = self.raster.pixels().len();
        let needs_buffer_resize = {
            let buffer = self
                .surface
                .buffer_mut()
                .map_err(|e| format!("software buffer: {e}"))?;
            buffer.len() != expected_len
        };
        if needs_buffer_resize {
            let (Some(w), Some(h)) = (NonZeroU32::new(width), NonZeroU32::new(height)) else {
                return Err("software buffer dimensions mismatch".into());
            };
            self.surface
                .resize(w, h)
                .map_err(|e| format!("software resize: {e}"))?;
            self.surface_dims = (width, height);
        }
        let mut buffer = self
            .surface
            .buffer_mut()
            .map_err(|e| format!("software buffer: {e}"))?;
        if buffer.len() != expected_len {
            return Err("software buffer dimensions mismatch".into());
        }
        buffer.copy_from_slice(self.raster.pixels());
        buffer
            .present()
            .map_err(|e| format!("software present: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::{Color, DrawCommand, QuadPrimitive, Rect};

    fn full_window_scene(logical_width: f32, logical_height: f32, color: Color) -> Scene {
        let mut scene = Scene::with_clear(Color::rgba(0.0, 0.0, 0.0, 1.0));
        scene.push(DrawCommand::Quad(QuadPrimitive {
            rect: Rect::new(0.0, 0.0, logical_width, logical_height),
            color,
            radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }));
        scene
    }

    #[test]
    fn scale_change_and_self_healing_resize_cover_full_cpu_buffer() {
        // Start with the clipped 1043x747 physical buffer observed on the 100% monitor.
        let mut scale = 1.0_f32;
        let mut raster = CpuRenderer::new(1043, 747, scale).unwrap();

        // Next redraw sees the real 1280x770 client size at 100% (1.0): self-heal and render.
        let scene_100 = full_window_scene(1280.0, 770.0, Color::rgba(0.0, 1.0, 0.0, 1.0));
        sync_and_render_raster(&mut raster, &mut scale, 1280, 770, 1.0, &scene_100).unwrap();
        assert_eq!(raster.dimensions(), (1280, 770));
        assert_eq!(raster.pixels().len(), 1280 * 770);
        assert!(raster.pixels().iter().all(|&p| p == 0x0000_ff00));

        // Simulate moving to a 125% monitor where ScaleFactorChanged initially sees the
        // pre-SetWindowPos 1280x770 size at scale 1.25, then redraw sees the final 1600x962
        // client size (covering the bottom ~35 physical px that previously stayed black).
        sync_and_render_raster(&mut raster, &mut scale, 1280, 770, 1.25, &scene_100).unwrap();
        assert!((scale - 1.25).abs() < f32::EPSILON);

        let (logical_w, logical_h) = (1600.0 / 1.25, 962.0 / 1.25);
        let scene_125 = full_window_scene(logical_w, logical_h, Color::rgba(0.0, 0.0, 1.0, 1.0));
        sync_and_render_raster(&mut raster, &mut scale, 1600, 962, 1.25, &scene_125).unwrap();
        assert_eq!(raster.dimensions(), (1600, 962));
        assert_eq!(raster.pixels().len(), 1600 * 962);
        // Every pixel including the bottom 35 physical rows is covered.
        assert!(raster.pixels().iter().all(|&p| p == 0x0000_00ff));
    }
}
