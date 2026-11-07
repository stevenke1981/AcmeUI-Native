//! CPU rasterizer cases: `CpuRenderer::render_scene` on a fixed 1100x760
//! framebuffer with 2000 deterministic quads, at scale 1.0 and 2.0.
//!
//! The logical extent of the scene is framebuffer / scale, so each scale covers
//! the same physical pixels. A FNV-1a checksum of the output is recorded so a
//! run that silently draws nothing is visible in the JSON.

use acme_core::{Color, DrawCommand, QuadPrimitive, Rect, Scene};
use acme_render_cpu::CpuRenderer;
use anyhow::Result;

use crate::harness::{CaseResult, MetaValue, RunConfig, measure};

const FRAMEBUFFER: (u32, u32) = (1100, 760);
const QUAD_COUNT: usize = 2000;
const SCALES: [f32; 2] = [1.0, 2.0];
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

pub fn run(config: RunConfig) -> Result<Vec<CaseResult>> {
    SCALES
        .iter()
        .map(|&scale| render_case(config, scale))
        .collect()
}

fn render_case(config: RunConfig, scale: f32) -> Result<CaseResult> {
    let (width, height) = FRAMEBUFFER;
    let scene = build_scene(scale);
    let mut renderer = CpuRenderer::new(width, height, scale)?;
    renderer.render_scene(&scene)?;
    let checksum = fnv1a64(renderer.pixels());
    let samples = measure(config, || {
        renderer.render_scene(&scene)?;
        Ok(())
    })?;
    CaseResult::from_samples(
        format!("render_cpu/{QUAD_COUNT}_quads_scale_{scale:.1}"),
        &samples,
        vec![
            ("quads", MetaValue::Count(QUAD_COUNT as u64)),
            ("framebuffer", MetaValue::Text(format!("{width}x{height}"))),
            (
                "checksum_fnv1a64",
                MetaValue::Text(format!("{checksum:016x}")),
            ),
        ],
    )
}

/// Deterministic scene: a dark clear colour and `QUAD_COUNT` quads, a third of
/// them rounded with a border, spread over the logical extent.
fn build_scene(scale: f32) -> Scene {
    let (width, height) = FRAMEBUFFER;
    let logical_width = width as f32 / scale;
    let logical_height = height as f32 / scale;
    let mut scene = Scene::with_clear(Color::rgba(0.08, 0.09, 0.12, 1.0));
    for i in 0..QUAD_COUNT {
        let n = i as f32;
        let size = 12.0 + (i % 5) as f32 * 6.0;
        let x = (n * 37.0) % (logical_width - size);
        let y = (n * 23.0) % (logical_height - size);
        let rounded = i % 3 == 0;
        scene.push(DrawCommand::Quad(QuadPrimitive {
            rect: Rect::new(x, y, size, size),
            color: Color::rgba(channel(i * 7), channel(i * 13), channel(i * 29), 1.0),
            radius: if rounded { 4.0 } else { 0.0 },
            border_width: if rounded { 1.0 } else { 0.0 },
            border_color: Color::rgba(0.9, 0.9, 0.9, 1.0),
        }));
    }
    scene
}

fn channel(seed: usize) -> f32 {
    (seed % 256) as f32 / 255.0
}

fn fnv1a64(pixels: &[u32]) -> u64 {
    pixels
        .iter()
        .flat_map(|pixel| pixel.to_le_bytes())
        .fold(FNV_OFFSET, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_has_the_documented_quad_count() {
        let scene = build_scene(1.0);
        assert_eq!(scene.commands().len(), QUAD_COUNT);
        assert!(scene.validate().is_ok());
    }

    #[test]
    fn checksum_is_empty_offset_for_no_pixels_and_sensitive_to_content() {
        assert_eq!(fnv1a64(&[]), FNV_OFFSET);
        assert_ne!(fnv1a64(&[0x0011_2233]), fnv1a64(&[0x0011_2234]));
        assert_eq!(fnv1a64(&[7, 8, 9]), fnv1a64(&[7, 8, 9]));
    }
}
