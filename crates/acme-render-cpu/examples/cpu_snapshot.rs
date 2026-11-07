//! Render deterministic pixels without a window or GPU. Output: binary PPM.
use std::io::Write;

use acme_core::{Color, DrawCommand, LayerParams, QuadPrimitive, Rect, Scene};
use acme_render_cpu::CpuRenderer;

fn quad(x: f32, y: f32, w: f32, h: f32, color: Color) -> DrawCommand {
    DrawCommand::Quad(QuadPrimitive {
        rect: Rect::new(x, y, w, h),
        color,
        radius: 10.0,
        border_width: 1.0,
        border_color: Color::rgba(0.4, 0.5, 0.65, 1.0),
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "cpu-frame.ppm".into());
    let mut scene = Scene::with_clear(Color::rgba(0.06, 0.08, 0.12, 1.0));
    scene.push(quad(
        12.0,
        12.0,
        70.0,
        156.0,
        Color::rgba(0.12, 0.16, 0.23, 1.0),
    ));
    for row in 0..3 {
        scene.push(quad(
            94.0,
            12.0 + row as f32 * 54.0,
            214.0,
            44.0,
            Color::rgba(0.14, 0.2, 0.3, 1.0),
        ));
    }
    scene.push(DrawCommand::PushClip(Rect::new(100.0, 20.0, 196.0, 26.0)));
    scene.push(quad(
        110.0,
        18.0,
        260.0,
        30.0,
        Color::rgba(0.2, 0.7, 0.9, 1.0),
    ));
    scene.push(DrawCommand::PopClip);
    scene.push(DrawCommand::BeginLayer(LayerParams { opacity: 0.6 }));
    scene.push(quad(
        112.0,
        76.0,
        80.0,
        24.0,
        Color::rgba(1.0, 0.6, 0.2, 1.0),
    ));
    scene.push(quad(
        160.0,
        76.0,
        80.0,
        24.0,
        Color::rgba(1.0, 0.6, 0.2, 1.0),
    ));
    scene.push(DrawCommand::EndLayer);
    let mut cpu = CpuRenderer::new(320, 180, 1.0)?;
    cpu.render_scene(&scene)?;
    let mut file = std::io::BufWriter::new(std::fs::File::create(&output)?);
    file.write_all(b"P6\n320 180\n255\n")?;
    for pixel in cpu.pixels() {
        file.write_all(&[(pixel >> 16) as u8, (pixel >> 8) as u8, *pixel as u8])?;
    }
    file.flush()?;
    println!("CPU snapshot: {output}");
    Ok(())
}
