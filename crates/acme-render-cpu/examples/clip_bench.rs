//! Focused clip benchmark: no fonts, native windows, GPU or new dependencies.
use std::hint::black_box;
use std::time::Instant;

use acme_core::{
    AtlasUpload, Color, DrawCommand, GlyphDraw, GlyphFormat, ImageFit, ImageKey, ImagePrimitive,
    Point, QuadPrimitive, Rect, Rgba8Image, Scene, TextPrimitive,
};
use acme_render_cpu::CpuRenderer;

fn scene(path: &str, clip: Option<Rect<acme_core::Logical>>) -> Scene {
    let mut scene = Scene::with_clear(Color::rgba(0.1, 0.2, 0.3, 1.0));
    if let Some(clip) = clip {
        scene.push(DrawCommand::PushClip(clip));
    }
    match path {
        "quad" => {
            for index in 0..80 {
                scene.push(DrawCommand::Quad(QuadPrimitive {
                    rect: Rect::new(
                        (index % 10) as f32 * 18.0,
                        (index / 10) as f32 * 16.0,
                        24.0,
                        22.0,
                    ),
                    radius: 4.0,
                    border_width: 1.0,
                    color: Color::rgba(0.9, 0.2, 0.4, 0.7),
                    border_color: Color::rgba(0.2, 0.9, 0.4, 0.8),
                }));
            }
        }
        "image" => {
            let image = Rgba8Image::new(
                ImageKey::new(1),
                0,
                2,
                2,
                vec![
                    255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 200, 255, 255, 0, 255,
                ],
            )
            .unwrap();
            for index in 0..8 {
                scene.push(DrawCommand::Image(ImagePrimitive {
                    image: image.clone(),
                    rect: Rect::new(index as f32 * 12.0, index as f32 * 8.0, 120.0, 80.0),
                    fit: ImageFit::Fill,
                }));
            }
        }
        "alpha8" | "rgba8" => {
            let format = if path == "alpha8" {
                GlyphFormat::Alpha8
            } else {
                GlyphFormat::Rgba8
            };
            let pixels = if format == GlyphFormat::Alpha8 {
                vec![192; 8 * 12]
            } else {
                [255, 128, 64, 192].repeat(8 * 12)
            };
            let glyphs = (0..300)
                .map(|index| GlyphDraw {
                    x: (index % 25) as f32 * 9.0,
                    y: (index / 25) as f32 * 13.0,
                    width: 8.0,
                    height: 12.0,
                    atlas_x: 0,
                    atlas_y: 0,
                    atlas_page: 0,
                    format,
                })
                .collect();
            scene.push(DrawCommand::Text(TextPrimitive {
                origin: Point::new(0.0, 0.0),
                color: Color::rgba(1.0, 1.0, 1.0, 0.9),
                glyphs,
                uploads: vec![AtlasUpload {
                    page: 0,
                    origin: [0, 0],
                    size: [8, 12],
                    format,
                    pixels,
                }],
            }));
        }
        _ => unreachable!("fixed workload catalog"),
    }
    if clip.is_some() {
        scene.push(DrawCommand::PopClip);
    }
    scene
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    assert!(!cfg!(debug_assertions), "use --release");
    let output = std::env::args().nth(1).expect("output JSON path");
    let mut records = Vec::new();
    for (name, path, clip) in [
        ("quad_unclipped", "quad", None),
        (
            "quad_integer",
            "quad",
            Some(Rect::new(8.0, 8.0, 160.0, 112.0)),
        ),
        (
            "quad_fractional",
            "quad",
            Some(Rect::new(8.25, 8.375, 160.5, 112.25)),
        ),
        (
            "image_fractional",
            "image",
            Some(Rect::new(8.25, 8.375, 160.5, 112.25)),
        ),
        (
            "alpha8_fractional",
            "alpha8",
            Some(Rect::new(8.25, 8.375, 160.5, 112.25)),
        ),
        (
            "rgba8_fractional",
            "rgba8",
            Some(Rect::new(8.25, 8.375, 160.5, 112.25)),
        ),
    ] {
        let scene = scene(path, clip);
        let mut cpu = CpuRenderer::new(256, 192, 1.25)?;
        for _ in 0..10 {
            cpu.render_scene(black_box(&scene))?;
        }
        let mut samples = Vec::new();
        for _ in 0..51 {
            let start = Instant::now();
            cpu.render_scene(black_box(&scene))?;
            samples.push(start.elapsed().as_nanos() as u64);
            black_box(cpu.pixels());
        }
        let checksum = cpu
            .pixels()
            .iter()
            .flat_map(|p| p.to_le_bytes())
            .fold(0xcbf2_9ce4_8422_2325u64, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
            });
        let mut sorted = samples.clone();
        sorted.sort_unstable();
        let median = sorted[25];
        let p95 = sorted[48];
        println!("{name}: median {median} ns, p95 {p95} ns, checksum {checksum:016x}");
        records.push(format!("{{\"name\":\"{name}\",\"median_ns\":{median},\"p95_ns\":{p95},\"checksum\":\"{checksum:016x}\",\"samples_ns\":{samples:?}}}"));
    }
    std::fs::write(
        output,
        format!(
            "{{\"warmup\":10,\"samples\":51,\"framebuffer\":[256,192],\"scale\":1.25,\"cases\":[{}]}}\n",
            records.join(",")
        ),
    )?;
    Ok(())
}
