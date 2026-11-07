//! GPU surface lifecycle and the small batched rectangle renderer used by AcmeUI.
#![forbid(unsafe_op_in_unsafe_fn)]

mod atlas_pages;
pub mod batch;
mod draws;
pub mod golden;
mod images;
mod instances;
mod pipelines;
mod renderer;
mod surface;
#[cfg(test)]
mod tests;
mod types;
mod uploads;

pub use batch::scene_from_frame;
pub use renderer::Renderer;
pub use types::{
    ClippedQuad, Frame, Quad, RenderError, RenderStats, SurfaceAction, SurfaceStatus, TextRun,
};
