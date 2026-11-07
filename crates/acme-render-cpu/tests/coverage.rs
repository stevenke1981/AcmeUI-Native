//! Pixel-area coverage for CPU quads (ROADMAP 009, P2).
//!
//! Only the public API is used. Coverage of a pixel is the fraction of its unit
//! square `[x, x + 1) x [y, y + 1)` that lies inside the quad shape. A red quad over
//! a transparent clear stores alpha `round(255 * coverage)` and the final composite
//! over black keeps that value in the red byte, so `red_coverage` reads it back.
//! Each byte carries at most 1/255 of quantisation error, which the tolerances allow.

use acme_core::{Color, DrawCommand, QuadPrimitive, Rect, Scene};
use acme_render_cpu::CpuRenderer;

/// One red byte step, used as the quantisation tolerance for byte comparisons.
const BYTE_STEP: f64 = 1.0 / 255.0;

fn red() -> Color {
    Color::rgba(1.0, 0.0, 0.0, 1.0)
}

fn quad(x: f32, y: f32, w: f32, h: f32, radius: f32) -> DrawCommand {
    DrawCommand::Quad(QuadPrimitive {
        rect: Rect::new(x, y, w, h),
        color: red(),
        radius,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    })
}

fn render_one(scene: &Scene, width: u32, height: u32) -> CpuRenderer {
    let mut cpu = CpuRenderer::new(width, height, 1.0).expect("valid frame");
    cpu.render_scene(scene).expect("valid scene");
    cpu
}

/// Red-channel coverage of pixel `(x, y)` in `[0, 1]`.
fn red_coverage(cpu: &CpuRenderer, x: u32, y: u32) -> f64 {
    let (width, _) = cpu.dimensions();
    let packed = cpu.pixels()[(y * width + x) as usize];
    f64::from((packed >> 16) & 0xff) / 255.0
}

/// Sum of red coverage over the whole frame, i.e. the covered area in pixels.
fn total_coverage(cpu: &CpuRenderer) -> f64 {
    let (width, height) = cpu.dimensions();
    (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| red_coverage(cpu, x, y))
        .sum()
}

#[test]
fn half_pixel_vertical_edge_has_half_area_coverage() {
    // Right edge at x = 2.5 cuts pixel column 2 in half.
    let mut scene = Scene::new();
    scene.push(quad(0.0, 0.0, 2.5, 4.0, 0.0));
    let cpu = render_one(&scene, 4, 4);

    for y in 0..4 {
        assert!(
            (red_coverage(&cpu, 1, y) - 1.0).abs() <= BYTE_STEP,
            "interior"
        );
        assert!(
            (red_coverage(&cpu, 2, y) - 0.5).abs() <= BYTE_STEP,
            "half edge"
        );
        assert_eq!(red_coverage(&cpu, 3, y), 0.0, "outside");
    }
}

#[test]
fn quarter_pixel_corner_has_quarter_area_coverage() {
    // Pixel (0, 0) overlaps the rect [0.5, 2.5)^2 in a 0.5 x 0.5 square.
    let mut scene = Scene::new();
    scene.push(quad(0.5, 0.5, 2.0, 2.0, 0.0));
    let cpu = render_one(&scene, 3, 3);

    assert!((red_coverage(&cpu, 0, 0) - 0.25).abs() <= BYTE_STEP);
    assert!((red_coverage(&cpu, 2, 2) - 0.25).abs() <= BYTE_STEP);
    assert!((red_coverage(&cpu, 1, 1) - 1.0).abs() <= BYTE_STEP);
}

#[test]
fn full_circle_total_coverage_is_within_one_percent_of_pi_r_squared() {
    // radius == half side, so the shape is a circle of radius r.
    for &(offset, radius) in &[(0.0_f32, 20.0_f32), (0.3, 20.0), (0.7, 40.0)] {
        let side = 2.0 * radius;
        let frame = (side + 2.0 * offset.ceil() + 8.0) as u32;
        let mut scene = Scene::new();
        scene.push(quad(4.0 + offset, 4.0 + offset, side, side, radius));
        let cpu = render_one(&scene, frame, frame);

        let expected = std::f64::consts::PI * f64::from(radius) * f64::from(radius);
        let actual = total_coverage(&cpu);
        let relative = (actual - expected).abs() / expected;
        assert!(
            relative <= 0.01,
            "offset {offset} radius {radius}: coverage {actual:.2} vs pi r^2 {expected:.2} ({:.3}%)",
            relative * 100.0
        );
    }
}

#[test]
fn coverage_never_leaves_the_rect_bounds() {
    // Pixel-box overlap with the bounding rect bounds every pixel's coverage.
    let (x, y, w, h) = (1.25_f32, 1.5_f32, 3.5_f32, 2.75_f32);
    let mut scene = Scene::new();
    scene.push(quad(x, y, w, h, 1.0));
    let cpu = render_one(&scene, 8, 8);

    for py in 0..8u32 {
        for px in 0..8u32 {
            let overlap_x = (px as f32 + 1.0).min(x + w) - (px as f32).max(x);
            let overlap_y = (py as f32 + 1.0).min(y + h) - (py as f32).max(y);
            let bound = f64::from(overlap_x.clamp(0.0, 1.0) * overlap_y.clamp(0.0, 1.0));
            let got = red_coverage(&cpu, px, py);
            assert!(
                got <= bound + BYTE_STEP,
                "pixel ({px},{py}) coverage {got} exceeds bounding-rect area {bound}"
            );
        }
    }
}

#[test]
fn coverage_is_monotone_non_increasing_in_radius() {
    // A smaller radius only adds area, so every pixel's coverage can only grow as
    // the radius shrinks. Several fractional offsets exercise different arc phases.
    let radii = [0.0_f32, 1.0, 2.0, 4.0, 6.0, 8.0];
    for &(x, y) in &[(0.4_f32, 0.6_f32), (0.9, 0.1), (3.5, 2.25)] {
        let frames: Vec<CpuRenderer> = radii
            .iter()
            .map(|&radius| {
                let mut scene = Scene::new();
                scene.push(quad(x, y, 24.0, 16.0, radius));
                render_one(&scene, 28, 20)
            })
            .collect();

        for pair in frames.windows(2) {
            for py in 0..20 {
                for px in 0..28 {
                    let (smaller_radius, larger_radius) = (
                        red_coverage(&pair[0], px, py),
                        red_coverage(&pair[1], px, py),
                    );
                    assert!(
                        larger_radius <= smaller_radius + BYTE_STEP,
                        "offset ({x},{y}) pixel ({px},{py}) grew from {smaller_radius} to {larger_radius}"
                    );
                }
            }
        }
    }
}

#[test]
fn non_finite_and_empty_quads_paint_nothing_and_do_not_panic() {
    // acme-core maps non-finite origins and sizes to zero before they reach this
    // crate, so non-finite *extents* are the inputs that reach the rasterizer.
    let cases = [
        quad(0.0, 0.0, f32::NAN, 4.0, 0.0),
        quad(0.0, 0.0, f32::INFINITY, 4.0, 1.0),
        quad(0.0, 0.0, 0.0, 4.0, 0.0),
        quad(0.0, 0.0, 4.0, 0.0, 2.0),
        quad(2.0, 2.0, -3.0, 3.0, 0.0),
    ];
    for (index, command) in cases.into_iter().enumerate() {
        let mut scene = Scene::new();
        scene.push(command);
        let cpu = render_one(&scene, 6, 6);
        assert_eq!(total_coverage(&cpu), 0.0, "case {index} painted");
    }
}

#[test]
fn physical_overflow_paints_nothing_and_does_not_panic() {
    // 4 logical px at f32::MAX scale overflows to infinity in physical space.
    let mut scene = Scene::new();
    scene.push(quad(0.0, 0.0, 4.0, 4.0, 1.0));
    let mut cpu = CpuRenderer::new(6, 6, f32::MAX).expect("finite scale");
    cpu.render_scene(&scene).expect("valid scene");
    assert_eq!(total_coverage(&cpu), 0.0);
}

#[test]
fn non_finite_radius_is_contained_within_the_rect() {
    for radius in [f32::NAN, f32::INFINITY, -1.0] {
        let mut scene = Scene::new();
        scene.push(quad(0.0, 0.0, 6.0, 6.0, radius));
        let cpu = render_one(&scene, 6, 6);
        let total = total_coverage(&cpu);
        assert!(
            total.is_finite() && total <= 36.0 + 36.0 * BYTE_STEP,
            "radius {radius}"
        );
    }
}

#[test]
fn stroke_is_outer_area_minus_inner_area_and_never_negative() {
    // Transparent fill, red 0.5 px border on a 4 x 4 rect at the origin.
    let mut scene = Scene::new();
    scene.push(DrawCommand::Quad(QuadPrimitive {
        rect: Rect::new(0.0, 0.0, 4.0, 4.0),
        color: Color::TRANSPARENT,
        radius: 0.0,
        border_width: 0.5,
        border_color: red(),
    }));
    let cpu = render_one(&scene, 4, 4);

    // Corner pixel: outer 1.0, inner [0.5, 3.5)^2 overlaps it by 0.25, stroke 0.75.
    assert!(
        (red_coverage(&cpu, 0, 0) - 0.75).abs() <= BYTE_STEP,
        "corner stroke"
    );
    // Edge pixel: outer 1.0, inner overlaps it by 0.5, stroke 0.5.
    assert!(
        (red_coverage(&cpu, 0, 1) - 0.5).abs() <= BYTE_STEP,
        "edge stroke"
    );
    // Inner pixel: fully inside the inner rect, so no stroke.
    assert_eq!(red_coverage(&cpu, 1, 1), 0.0, "inner pixel");
}
