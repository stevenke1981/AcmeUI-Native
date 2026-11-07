//! The wgpu-backed [`Renderer`] and its construction, frame, atlas and recovery steps.

mod atlas;
mod buffers;
mod frame;
mod init;
mod recovery;
mod scene_images;
mod state;

pub use state::Renderer;

/// Initial per-buffer instance capacity (1024 instances approx 64 KiB each).
const INITIAL_QUAD_CAPACITY: u64 = 1024;
const INITIAL_GLYPH_CAPACITY: u64 = 1024;
