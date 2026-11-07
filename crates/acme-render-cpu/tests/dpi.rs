//! DPI and fractional-scale acceptance for the CPU rasterizer (ROADMAP 005, D2).
//!
//! Only the public API is used. The rules asserted here are read from
//! `crates/acme-render-cpu/src/lib.rs`:
//!
//! R1 Logical to physical: `Bounds::from_rect` multiplies every logical origin and
//!    size by the scale given to `CpuRenderer::new` or `resize`. Nothing is rounded.
//! R2 Frame size: the crate stores the caller's physical `width x height` verbatim
//!    (`dimensions()`, `pixels().len() == width * height`). The crate never derives
//!    the frame from the logical size, so any ceil or floor choice belongs to the
//!    host. The platform host passes the OS physical inner size.
//! R3 Pixel centres: pixel `(x, y)` is sampled at `(x + 0.5, y + 0.5)`. Clip rects
//!    are half-open (`Rect::contains`), and a clipped draw paints a pixel only when
//!    its centre lies inside the clip (`for_pixels`).
//! R4 Quad edges: a pixel's quad coverage is the exact area of the pixel square
//!    inside the quad (P2, ROADMAP 009). Sharp quads take the product of the x and y
//!    overlaps, so a corner pixel is the product of two axis coverages. Rounded
//!    corners are supersampled only near the arc. An edge pixel on a fractional
//!    boundary is therefore partially painted by its exact area.
//! R5 Byte mapping: a red quad over a transparent clear stores alpha
//!    `round(255 * coverage)`. The final composite over black gives red byte
//!    `round(255 * coverage)`. Full coverage is `0x00ff0000`.
//! R6 Border: stroke = outer coverage minus inner coverage, where the inner rect is
//!    inset by `border_width * scale` physical px.
//!
//! Every frame here uses a logical size whose product with the scale is an integer,
//! so no host rounding choice enters the pixel expectations.

use acme_core::{Color, DrawCommand, Logical, QuadPrimitive, Rect, Scene};
use acme_render_cpu::CpuRenderer;

/// The DPI matrix from BRIEF D2. 1.75 is included because it is cheap.
const SCALES: [f32; 5] = [1.0, 1.25, 1.5, 1.75, 2.0];

const RED_PIXEL: u32 = 0x00ff_0000;
const GREEN_PIXEL: u32 = 0x0000_ff00;

fn red() -> Color {
    Color::rgba(1.0, 0.0, 0.0, 1.0)
}

fn green() -> Color {
    Color::rgba(0.0, 1.0, 0.0, 1.0)
}

/// Physical extent of a logical length at `scale`. Asserts that the product is
/// integral so that the frame size is unambiguous.
fn physical(logical: u32, scale: f32) -> u32 {
    let exact = logical as f32 * scale;
    assert_eq!(
        exact.fract(),
        0.0,
        "{logical} logical px at {scale}x is not an integral physical size"
    );
    exact as u32
}

fn solid(rect: Rect<Logical>, color: Color) -> DrawCommand {
    DrawCommand::Quad(QuadPrimitive {
        rect,
        color,
        radius: 0.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    })
}

fn bordered(
    rect: Rect<Logical>,
    fill: Color,
    border_width: f32,
    border_color: Color,
) -> DrawCommand {
    DrawCommand::Quad(QuadPrimitive {
        rect,
        color: fill,
        radius: 0.0,
        border_width,
        border_color,
    })
}

fn render(scene: &Scene, width: u32, height: u32, scale: f32) -> CpuRenderer {
    let mut cpu = CpuRenderer::new(width, height, scale).expect("valid test frame");
    cpu.render_scene(scene).expect("valid test scene");
    cpu
}

/// Packed pixel at `(x, y)` of the presented buffer.
fn at(cpu: &CpuRenderer, x: u32, y: u32) -> u32 {
    let (width, _) = cpu.dimensions();
    cpu.pixels()[(y * width + x) as usize]
}

/// Packed pixel for a red quad with the given coverage (R5).
fn red_at_coverage(coverage: f32) -> u32 {
    ((255.0 * coverage).round() as u32) << 16
}

/// Exact 1D area fraction of pixel `x` inside `[lo, hi)`. Along an axis-aligned
/// edge this equals the R4 coverage, so it is an independent derivation.
fn box_coverage(x: u32, lo: f32, hi: f32) -> f32 {
    let x = x as f32;
    ((x + 1.0).min(hi) - x.max(lo)).clamp(0.0, 1.0)
}

#[test]
fn framebuffer_matches_caller_physical_size_at_every_scale() {
    for scale in SCALES {
        let n = physical(8, scale);
        let cpu = render(&Scene::new(), n, n, scale);
        assert_eq!(cpu.dimensions(), (n, n), "scale {scale}");
        assert_eq!(cpu.pixels().len(), (n * n) as usize, "scale {scale}");
        // The host recovers the logical size as physical / scale (R2). It is exact here.
        assert_eq!(n as f32 / scale, 8.0, "scale {scale}");
    }
}

/// 15 logical px at 1.25x is 18.75 physical px. The crate never picks ceil or
/// floor (R2), so each frame width is drawn as-is and the last column shows the
/// partial coverage that the chosen width leaves.
#[test]
fn fractional_logical_extent_follows_the_host_chosen_frame_width() {
    let scale = 1.25;
    let height = physical(8, scale);
    let scene = {
        let mut scene = Scene::new();
        scene.push(solid(Rect::new(0.0, 0.0, 15.0, 8.0), red()));
        scene
    };

    // ceil(18.75) = 19. Column 18 covers [18, 19) and the quad edge at 18.75 lies
    // 0.25 px into it, so the pixel centre 18.5 is 0.25 px inside and coverage is 0.75.
    let ceil_cpu = render(&scene, 19, height, scale);
    for y in 0..height {
        for x in 0..18 {
            assert_eq!(at(&ceil_cpu, x, y), RED_PIXEL, "ceil frame x={x} y={y}");
        }
        assert_eq!(
            at(&ceil_cpu, 18, y),
            red_at_coverage(0.75),
            "ceil frame edge y={y}"
        );
    }

    // floor(18.75) = 18. The 18 columns are all fully inside the quad.
    let floor_cpu = render(&scene, 18, height, scale);
    for y in 0..height {
        for x in 0..18 {
            assert_eq!(at(&floor_cpu, x, y), RED_PIXEL, "floor frame x={x} y={y}");
        }
    }
}

#[test]
fn resize_through_every_scale_leaves_no_stale_pixels_and_uses_the_new_scale() {
    let mut cpu = CpuRenderer::new(8, 8, 1.0).expect("valid test frame");
    cpu.render_scene(&Scene::with_clear(red()))
        .expect("valid test scene");
    assert!(cpu.pixels().iter().all(|p| *p == RED_PIXEL));

    for scale in [1.25, 1.75, 1.5, 2.0, 1.0] {
        let n = physical(8, scale);
        cpu.resize(n, n, scale).expect("valid resize");
        cpu.render_scene(&Scene::with_clear(green()))
            .expect("valid test scene");

        assert_eq!(cpu.dimensions(), (n, n), "scale {scale}");
        assert_eq!(cpu.pixels().len(), (n * n) as usize, "scale {scale}");
        assert!(
            cpu.pixels().iter().all(|p| *p == GREEN_PIXEL),
            "stale pixel after resize to {scale}x"
        );

        // A logical quad at (2, 2, 4, 4) covers physical [2s, 6s). Its left edge
        // column must carry the R4 coverage for the new scale, which shows that
        // resize updated the scale used for drawing.
        let mut scene = Scene::new();
        scene.push(solid(Rect::new(2.0, 2.0, 4.0, 4.0), red()));
        cpu.render_scene(&scene).expect("valid test scene");
        let mid = n / 2;
        let left = (2.0 * scale).floor() as u32;
        let expected = red_at_coverage(box_coverage(left, 2.0 * scale, 6.0 * scale));
        assert_eq!(at(&cpu, left, mid), expected, "left edge at {scale}x");
    }
}

/// A 1-logical-px border on an 8x8 logical rect. Physical border width is `scale`
/// (R6). Outer ring pixels lie entirely in the stroke, and pixels at least two
/// physical px in from the outer edge lie entirely in the fill, for every tested
/// scale.
#[test]
fn one_logical_px_border_is_visible_on_the_outer_ring_at_every_scale() {
    for scale in SCALES {
        let n = physical(8, scale);
        let scene = {
            let mut scene = Scene::new();
            scene.push(bordered(Rect::new(0.0, 0.0, 8.0, 8.0), green(), 1.0, red()));
            scene
        };
        let cpu = render(&scene, n, n, scale);

        // Outer ring: inner edge is at least 0.5 px from the pixel centre, so the
        // stroke coverage is 1 and the fill coverage is 0.
        for y in 0..n {
            for x in 0..n {
                let on_ring = x == 0 || y == 0 || x == n - 1 || y == n - 1;
                if on_ring {
                    assert_eq!(at(&cpu, x, y), RED_PIXEL, "ring {scale}x ({x},{y})");
                }
            }
        }
        // Interior: the pixel centre is at least 0.5 px inside the inner edge, so
        // the fill coverage is 1 and the stroke is 0.
        for y in 2..n - 2 {
            for x in 2..n - 2 {
                assert_eq!(at(&cpu, x, y), GREEN_PIXEL, "fill {scale}x ({x},{y})");
            }
        }
        // At a fractional border width, the first inner column straddles the inner
        // edge, so it is a mix of fill and stroke.
        if scale.fract() != 0.0 {
            let mid = n / 2;
            let pixel = at(&cpu, 1, mid);
            assert!(
                (pixel >> 16) & 0xff > 0 && (pixel >> 8) & 0xff > 0,
                "expected a red/green mix at {scale}x, got {pixel:#010x}"
            );
        }
    }
}

/// A 1-logical-px red separator at logical x = 10. The physical span is
/// `[10s, 11s)` (R1). Each edge column has coverage from R4, and the coverages
/// sum to the physical width `s`, so no ink is lost or added.
///
/// Note: at 1.25x and 1.75x no column is fully saturated. The separator is spread
/// over two or three columns with partial coverage, so "visible" here means
/// non-zero red at the expected columns with exact partial values.
#[test]
fn one_logical_px_separator_keeps_physical_ink_at_every_scale() {
    // (scale, [(column, expected red byte)])
    //
    // 1.00: [10, 11). Column 10 is fully covered.
    // 1.25: [12.5, 13.75). Column 12 has centre 12.5 on the left edge, so coverage 0.5.
    //       Column 13 has centre 13.5, 0.25 px inside the right edge at 13.75, so 0.75.
    //       Ink = 0.5 + 0.75 = 1.25.
    // 1.50: [15, 16.5). Column 15 is fully covered. Column 16 has centre 16.5 on the
    //       right edge, so 0.5. Ink = 1.5.
    // 1.75: [17.5, 19.25). Column 17 has centre 17.5 on the left edge, so 0.5. Column
    //       18 is fully covered. Column 19 has centre 19.5, 0.25 px past the right edge,
    //       so 0.25. Ink = 1.75.
    // 2.00: [20, 22). Columns 20 and 21 are fully covered.
    //
    // Coverages are dyadic, so the sums below are exact in f32.
    const CASES: &[(f32, &[(u32, f32)])] = &[
        (1.0, &[(10, 1.0)]),
        (1.25, &[(12, 0.5), (13, 0.75)]),
        (1.5, &[(15, 1.0), (16, 0.5)]),
        (1.75, &[(17, 0.5), (18, 1.0), (19, 0.25)]),
        (2.0, &[(20, 1.0), (21, 1.0)]),
    ];

    for (scale, columns) in CASES {
        let (width, height) = (physical(16, *scale), physical(8, *scale));
        let mut scene = Scene::new();
        scene.push(solid(Rect::new(10.0, 0.0, 1.0, 8.0), red()));
        let cpu = render(&scene, width, height, *scale);

        let ink: f32 = columns.iter().map(|(_, c)| c).sum();
        assert!(
            (ink - scale).abs() < 1e-6,
            "physical ink {ink} != {scale} at {scale}x"
        );

        for y in 0..height {
            for x in 0..width {
                let expected = columns
                    .iter()
                    .find(|(column, _)| *column == x)
                    .map_or(0, |(_, c)| red_at_coverage(*c));
                assert_eq!(at(&cpu, x, y), expected, "separator {scale}x ({x},{y})");
            }
        }
    }
}

/// Clip rect at logical `[1.375, 3.5)` on both axes. Both values are dyadic, so
/// the physical edges are exact at every tested scale. A pixel is painted only if
/// its centre is inside the physical clip (R3).
#[test]
fn fractional_clip_follows_pixel_centre_rule_at_every_scale() {
    // (scale, painted columns, the same set applies to rows)
    //
    // Column x is painted iff 1.375s <= x + 0.5 < 3.5s.
    // 1.00: [1.375, 3.5). Centres 1.5 and 2.5 are in. 3.5 is out (half-open).
    // 1.25: [1.71875, 4.375). Centre 1.5 is out. 2.5 and 3.5 are in. 4.5 is out.
    // 1.50: [2.0625, 5.25). Centre 1.5 is out. 2.5, 3.5, 4.5 are in. 5.5 is out.
    // 1.75: [2.40625, 6.125). Centre 1.5 is out. 2.5 through 5.5 are in. 6.5 is out.
    // 2.00: [2.75, 7.0). Centre 2.5 is out (below 2.75). 3.5 through 6.5 are in.
    const CASES: &[(f32, &[u32])] = &[
        (1.0, &[1, 2]),
        (1.25, &[2, 3]),
        (1.5, &[2, 3, 4]),
        (1.75, &[2, 3, 4, 5]),
        (2.0, &[3, 4, 5, 6]),
    ];

    for (scale, painted) in CASES {
        let n = physical(8, *scale);
        let mut scene = Scene::new();
        scene.push(DrawCommand::PushClip(Rect::new(1.375, 1.375, 2.125, 2.125)));
        // Full-frame quad, so each painted pixel has full coverage.
        scene.push(solid(Rect::new(-1.0, -1.0, 10.0, 10.0), red()));
        scene.push(DrawCommand::PopClip);
        let cpu = render(&scene, n, n, *scale);

        for y in 0..n {
            for x in 0..n {
                let inside = painted.contains(&x) && painted.contains(&y);
                let expected = if inside { RED_PIXEL } else { 0 };
                assert_eq!(at(&cpu, x, y), expected, "clip {scale}x ({x},{y})");
            }
        }
    }
}

/// One row of the quad coverage table: partially covered edge columns, then the
/// half-open range of fully covered columns.
struct QuadCase {
    scale: f32,
    partial: &'static [(u32, f32)],
    first_full: u32,
    end_full: u32,
}

/// Solid quad at logical `(10, 10, 20, 20)`, which is physical `[10s, 30s)` (R1).
/// Frame is 40 logical px.
#[test]
fn solid_quad_at_10_10_20_20_covers_expected_physical_range() {
    // (scale, partial edge columns with coverage, first full column, one past last full column)
    //
    // 1.00: [10, 30). Columns 10 to 29 are full. No partial columns.
    // 1.25: [12.5, 37.5). Column 12 has centre 12.5 on the edge, so 0.5. Columns 13 to 36
    //       are full. Column 37 has centre 37.5 on the edge, so 0.5.
    // 1.50: [15, 45). Columns 15 to 44 are full.
    // 1.75: [17.5, 52.5). Column 17 is 0.5. Columns 18 to 51 are full. Column 52 is 0.5.
    // 2.00: [20, 60). Columns 20 to 59 are full.
    const CASES: &[QuadCase] = &[
        QuadCase {
            scale: 1.0,
            partial: &[],
            first_full: 10,
            end_full: 30,
        },
        QuadCase {
            scale: 1.25,
            partial: &[(12, 0.5), (37, 0.5)],
            first_full: 13,
            end_full: 37,
        },
        QuadCase {
            scale: 1.5,
            partial: &[],
            first_full: 15,
            end_full: 45,
        },
        QuadCase {
            scale: 1.75,
            partial: &[(17, 0.5), (52, 0.5)],
            first_full: 18,
            end_full: 52,
        },
        QuadCase {
            scale: 2.0,
            partial: &[],
            first_full: 20,
            end_full: 60,
        },
    ];

    for case in CASES {
        let (scale, partial, first_full, end_full) =
            (case.scale, case.partial, case.first_full, case.end_full);
        let n = physical(40, scale);
        let mut scene = Scene::new();
        scene.push(solid(Rect::new(10.0, 10.0, 20.0, 20.0), red()));
        let cpu = render(&scene, n, n, scale);

        // 1D coverage along one axis, using the expected table.
        let axis = |v: u32| -> f32 {
            if let Some((_, c)) = partial.iter().find(|(p, _)| *p == v) {
                *c
            } else if (first_full..end_full).contains(&v) {
                1.0
            } else {
                0.0
            }
        };

        // Edge rows and columns and the interior are checked. Corner pixels, where
        // both axes are partial, are skipped here; their exact-area value is pinned
        // by `sharp_quad_corner_pixel_uses_exact_area_coverage` below.
        for y in 0..n {
            for x in 0..n {
                let (cx, cy) = (axis(x), axis(y));
                let expected = if cx == 0.0 || cy == 0.0 {
                    Some(0)
                } else if cx == 1.0 && cy == 1.0 {
                    Some(RED_PIXEL)
                } else if cx == 1.0 {
                    Some(red_at_coverage(cy))
                } else if cy == 1.0 {
                    Some(red_at_coverage(cx))
                } else {
                    None
                };
                if let Some(expected) = expected {
                    assert_eq!(at(&cpu, x, y), expected, "quad {scale}x ({x},{y})");
                }
            }
        }
    }
}

/// Sharp rect whose corner pixel centre sits exactly on the corner. Exact area
/// coverage is 0.5 x 0.5 = 0.25, so the expected red byte is 64. This pins the
/// P2 exact-area model; it was an ignored DESIGN test before ROADMAP 009.
#[test]
fn sharp_quad_corner_pixel_uses_exact_area_coverage() {
    let mut scene = Scene::new();
    scene.push(solid(Rect::new(10.0, 10.0, 20.0, 20.0), red()));
    let cpu = render(&scene, 50, 50, 1.25);
    assert_eq!(at(&cpu, 12, 12), red_at_coverage(0.25));
}
