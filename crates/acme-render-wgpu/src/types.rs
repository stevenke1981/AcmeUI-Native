//! Public data types, statistics, surface state and error values of the renderer.

use acme_text::PreparedText;
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
