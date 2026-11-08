//! Rough frame-cost probe for a CrewDesk-like scene (large rounded panels under clips).
//! Ignored by default; run with
//! `cargo test --release -p acme-render-cpu --test frame_timing -- --ignored --nocapture`.
use acme_core::{Color, DrawCommand, Logical, QuadPrimitive, Rect, Scene};
use acme_render_cpu::CpuRenderer;
use std::time::Instant;

fn quad(x: f32, y: f32, w: f32, h: f32, radius: f32, border: f32) -> DrawCommand {
    DrawCommand::Quad(QuadPrimitive {
        rect: Rect::<Logical>::new(x, y, w, h),
        color: Color::rgba(0.1, 0.12, 0.16, 1.0),
        radius,
        border_width: border,
        border_color: Color::rgba(0.2, 0.25, 0.3, 1.0),
    })
}

fn crewdesk_like() -> Scene {
    let mut scene = Scene::with_clear(Color::rgba(0.04, 0.06, 0.09, 1.0));
    // Titlebar, sidebar, pane frame, status bar.
    scene.push(quad(10.0, 10.0, 1260.0, 46.0, 8.0, 1.0));
    scene.push(quad(10.0, 64.0, 280.0, 680.0, 8.0, 1.0));
    scene.push(quad(298.0, 64.0, 972.0, 680.0, 8.0, 1.0));
    scene.push(DrawCommand::PushClip(Rect::new(306.0, 94.0, 956.0, 640.0)));
    scene.push(quad(306.0, 94.0, 956.0, 640.0, 0.0, 0.0));
    scene.push(DrawCommand::PopClip);
    scene.push(DrawCommand::PushClip(Rect::new(10.0, 64.0, 280.0, 680.0)));
    for i in 0..30 {
        scene.push(quad(22.0, 80.0 + i as f32 * 22.0, 256.0, 20.0, 6.0, 1.0));
    }
    scene.push(DrawCommand::PopClip);
    for i in 0..30 {
        scene.push(quad(400.0 + i as f32 * 20.0, 20.0, 16.0, 16.0, 4.0, 0.0));
    }
    scene.push(quad(10.0, 752.0, 1260.0, 30.0, 8.0, 1.0));
    scene
}

#[test]
#[ignore = "timing probe"]
fn crewdesk_like_frame_cost() {
    let scene = crewdesk_like();
    for scale in [1.0_f32, 1.25] {
        let (w, h) = ((1280.0 * scale) as u32, (790.0 * scale) as u32);
        let mut r = CpuRenderer::new(w, h, scale).unwrap();
        r.render_scene(&scene).unwrap();
        let n = 20;
        let start = Instant::now();
        for _ in 0..n {
            r.render_scene(&scene).unwrap();
        }
        let ms = start.elapsed().as_secs_f64() * 1000.0 / f64::from(n);
        println!("scale {scale}: {w}x{h} -> {ms:.2} ms/frame");
    }
}
