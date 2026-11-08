use super::*;

fn random(seed: &mut u64) -> f32 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    (*seed >> 40) as f32 / (1_u32 << 24) as f32
}

fn random_rect(seed: &mut u64, scale: f32) -> Bounds {
    let left = (random(seed) * 4.0 - 2.0) * scale;
    let top = (random(seed) * 4.0 - 2.0) * scale;
    Bounds {
        left,
        top,
        right: left + (0.02 + random(seed) * 12.0) * scale,
        bottom: top + (0.02 + random(seed) * 12.0) * scale,
    }
}

// Integrate a much finer independent point-in-shape reference over the clipped
// bounding box. Clipping the sample domain exactly avoids quantising straight
// rectangle edges in the reference itself.
fn fine_reference(bounds: Bounds, radius: f32, cx: f32, cy: f32) -> f64 {
    const FINE: usize = 512;
    let (left, top, right, bottom) = (
        f64::from(bounds.left),
        f64::from(bounds.top),
        f64::from(bounds.right),
        f64::from(bounds.bottom),
    );
    let radius = f64::from(radius).clamp(0.0, ((right - left).min(bottom - top)) * 0.5);
    let x0 = (f64::from(cx) - 0.5).max(left);
    let x1 = (f64::from(cx) + 0.5).min(right);
    let y0 = (f64::from(cy) - 0.5).max(top);
    let y1 = (f64::from(cy) + 0.5).min(bottom);
    if x0 >= x1 || y0 >= y1 {
        return 0.0;
    }
    let mut hits = 0;
    for row in 0..FINE {
        let y = y0 + (row as f64 + 0.5) * (y1 - y0) / FINE as f64;
        let dy = y - y.max(top + radius).min(bottom - radius);
        for column in 0..FINE {
            let x = x0 + (column as f64 + 0.5) * (x1 - x0) / FINE as f64;
            let dx = x - x.max(left + radius).min(right - radius);
            hits += usize::from(dx * dx + dy * dy <= radius * radius);
        }
    }
    (x1 - x0) * (y1 - y0) * hits as f64 / (FINE * FINE) as f64
}

#[test]
fn analytic_coverage_resolves_subpixel_discs_and_quarter_discs() {
    for (left, side, requested_radius, want) in [
        (0.0, 2.0, 1.0, std::f64::consts::FRAC_PI_4),
        (0.0, 1.0, 0.5, std::f64::consts::FRAC_PI_4),
        (0.25, 0.5, 0.25, std::f64::consts::PI / 16.0),
        (0.0, 1.0, 10.0, std::f64::consts::FRAC_PI_4),
    ] {
        let bounds = Bounds {
            left,
            top: left,
            right: left + side,
            bottom: left + side,
        };
        let got = f64::from(quad_coverage(bounds, requested_radius, 0.5, 0.5));
        assert!(
            (got - want).abs() < 1e-6,
            "{side}/{requested_radius}: {got} vs {want}"
        );
    }
}

#[test]
fn analytic_coverage_matches_fine_reference() {
    let mut seed = 0x014a_7c5e_ed01_u64;
    let mut max_error = 0.0_f64;
    for index in 0..256 {
        let scale = [1.0, 1.25, 1.5, 2.0][index % 4];
        let bounds = random_rect(&mut seed, scale);
        let radius =
            random(&mut seed) * (bounds.right - bounds.left).min(bounds.bottom - bounds.top) * 0.5;
        let right = index % 2 == 0;
        let bottom = index % 4 < 2;
        let arc_x = if right {
            bounds.right - radius
        } else {
            bounds.left + radius
        };
        let arc_y = if bottom {
            bounds.bottom - radius
        } else {
            bounds.top + radius
        };
        let angle = random(&mut seed) * std::f32::consts::FRAC_PI_2;
        let cx = (arc_x + if right { 1.0 } else { -1.0 } * radius * angle.cos()).floor() + 0.5;
        let cy = (arc_y + if bottom { 1.0 } else { -1.0 } * radius * angle.sin()).floor() + 0.5;
        let got = f64::from(quad_coverage(bounds, radius, cx, cy));
        let want = fine_reference(bounds, radius, cx, cy);
        max_error = max_error.max((got - want).abs());
        assert!(
            (got - want).abs() <= 1e-3,
            "case {index}, scale {scale}, radius {radius}, pixel ({cx},{cy}): {got} vs {want}"
        );
    }
    println!("512x512 reference: 256 cases, max error {max_error:.9}");
}

#[test]
fn analytic_coverage_large_radii_do_not_cancel_area() {
    for radius in [128.0_f32, 4096.0, 65536.0, 1e6, 1e7] {
        // A large, mostly offscreen circle still crosses a visible unit pixel.
        let arc = 10.0 - (radius * std::f32::consts::FRAC_1_SQRT_2).round();
        let bounds = Bounds {
            left: arc - radius,
            top: arc - radius,
            right: arc + radius,
            bottom: arc + radius,
        };
        let center = (f64::from(radius) * std::f64::consts::FRAC_1_SQRT_2 + f64::from(arc)).floor()
            as f32
            + 0.5;
        let got = f64::from(quad_coverage(bounds, radius, center, center));
        let want = fine_reference(bounds, radius, center, center);
        assert!(
            (got - want).abs() <= 1e-3,
            "radius {radius}: {got} vs {want}"
        );
    }
}

#[test]
fn analytic_coverage_is_bounded_and_monotone() {
    let mut seed = 0x0140_b0ad_5eed_u64;
    for index in 0..96 {
        let bounds = random_rect(&mut seed, [1.0, 1.25, 1.5, 2.0][index % 4]);
        let max_radius = (bounds.right - bounds.left).min(bounds.bottom - bounds.top) * 0.5;
        for y in bounds.top.floor() as i32..bounds.bottom.ceil() as i32 {
            for x in bounds.left.floor() as i32..bounds.right.ceil() as i32 {
                let mut previous = 1.0;
                for fraction in [0.0, 0.1, 0.4, 0.7, 1.0, 2.0] {
                    let got = quad_coverage(
                        bounds,
                        max_radius * fraction,
                        x as f32 + 0.5,
                        y as f32 + 0.5,
                    );
                    assert!((0.0..=1.0).contains(&got), "case {index}: {got}");
                    assert!(
                        got <= previous + 2e-6,
                        "case {index}, pixel ({x},{y}), radius {fraction}: {previous} -> {got}"
                    );
                    previous = got;
                }
            }
        }
    }
}

#[test]
fn analytic_coverage_sums_to_exact_rounded_rect_area() {
    let mut seed = 0x0140_a2ea_5eed_u64;
    let mut max_error = 0.0_f64;
    for index in 0..128 {
        let bounds = random_rect(&mut seed, [1.0, 1.25, 1.5, 2.0][index % 4]);
        let max_radius = (bounds.right - bounds.left).min(bounds.bottom - bounds.top) * 0.5;
        let radius = max_radius * [0.0, 0.01, 0.37, 1.0][index % 4];
        let mut actual = 0.0_f64;
        for y in bounds.top.floor() as i32..bounds.bottom.ceil() as i32 {
            for x in bounds.left.floor() as i32..bounds.right.ceil() as i32 {
                actual += f64::from(quad_coverage(
                    bounds,
                    radius,
                    x as f32 + 0.5,
                    y as f32 + 0.5,
                ));
            }
        }
        let expected = (f64::from(bounds.right) - f64::from(bounds.left))
            * (f64::from(bounds.bottom) - f64::from(bounds.top))
            - (4.0 - std::f64::consts::PI) * f64::from(radius).powi(2);
        max_error = max_error.max((actual - expected).abs());
        assert!(
            (actual - expected).abs() <= 5e-5,
            "case {index}, radius {radius}: area {actual} vs {expected}"
        );
    }
    println!("area conservation: 128 rects, max error {max_error:.9}");
}

fn fine_clipped_reference(bounds: Bounds, radius: f32, clip: Bounds, cx: f32, cy: f32) -> f64 {
    const FINE: usize = 512;
    let (left, top, right, bottom) = (
        f64::from(bounds.left),
        f64::from(bounds.top),
        f64::from(bounds.right),
        f64::from(bounds.bottom),
    );
    let radius = f64::from(radius).clamp(0.0, ((right - left).min(bottom - top)) * 0.5);
    let x0 = (f64::from(cx) - 0.5).max(left).max(f64::from(clip.left));
    let x1 = (f64::from(cx) + 0.5).min(right).min(f64::from(clip.right));
    let y0 = (f64::from(cy) - 0.5).max(top).max(f64::from(clip.top));
    let y1 = (f64::from(cy) + 0.5)
        .min(bottom)
        .min(f64::from(clip.bottom));
    if x0 >= x1 || y0 >= y1 {
        return 0.0;
    }
    let mut hits = 0;
    for row in 0..FINE {
        let y = y0 + (row as f64 + 0.5) * (y1 - y0) / FINE as f64;
        let dy = y - y.max(top + radius).min(bottom - radius);
        for column in 0..FINE {
            let x = x0 + (column as f64 + 0.5) * (x1 - x0) / FINE as f64;
            let dx = x - x.max(left + radius).min(right - radius);
            hits += usize::from(dx * dx + dy * dy <= radius * radius);
        }
    }
    (x1 - x0) * (y1 - y0) * hits as f64 / (FINE * FINE) as f64
}

#[test]
fn clipped_analytic_coverage_matches_fine_reference() {
    let mut seed = 0x0160_c11b_5eed_u64;
    let mut max_error = 0.0_f64;
    for index in 0..256 {
        let scale = [1.0, 1.25, 1.5, 2.0][index % 4];
        let bounds = random_rect(&mut seed, scale);
        let radius =
            random(&mut seed) * (bounds.right - bounds.left).min(bounds.bottom - bounds.top) * 0.5;
        let right = index % 2 == 0;
        let bottom = index % 4 < 2;
        let arc_x = if right {
            bounds.right - radius
        } else {
            bounds.left + radius
        };
        let arc_y = if bottom {
            bounds.bottom - radius
        } else {
            bounds.top + radius
        };
        let angle = random(&mut seed) * std::f32::consts::FRAC_PI_2;
        let cx = (arc_x + if right { 1.0 } else { -1.0 } * radius * angle.cos()).floor() + 0.5;
        let cy = (arc_y + if bottom { 1.0 } else { -1.0 } * radius * angle.sin()).floor() + 0.5;
        let clip = Bounds {
            left: cx - 0.5 + random(&mut seed) * 0.75,
            top: cy - 0.5 + random(&mut seed) * 0.75,
            right: cx - 0.25 + random(&mut seed) * 1.25,
            bottom: cy - 0.25 + random(&mut seed) * 1.25,
        };
        let got = f64::from(clipped_quad_coverage(bounds, radius, Some(clip), cx, cy));
        let want = fine_clipped_reference(bounds, radius, clip, cx, cy);
        max_error = max_error.max((got - want).abs());
        assert!(
            (got - want).abs() <= 1e-3,
            "case {index}, scale {scale}, radius {radius}, pixel ({cx},{cy}): {got} vs {want}"
        );
    }
    println!("clipped 512x512 reference: 256 cases, max error {max_error:.9}");
}

#[test]
fn clipped_analytic_coverage_is_additive_across_partition_splits() {
    let mut seed = 0x0160_5b17_5eed_u64;
    for index in 0..128 {
        let scale = [1.0, 1.25, 1.5, 2.0][index % 4];
        let bounds = random_rect(&mut seed, scale);
        let radius =
            random(&mut seed) * (bounds.right - bounds.left).min(bounds.bottom - bounds.top) * 0.5;
        let cx = bounds.left.floor() + 0.5;
        let cy = bounds.top.floor() + 0.5;
        let split_x = cx - 0.5 + random(&mut seed);
        let split_y = cy - 0.5 + random(&mut seed);
        let q00 = Bounds {
            left: cx - 10.0,
            top: cy - 10.0,
            right: split_x,
            bottom: split_y,
        };
        let q10 = Bounds {
            left: split_x,
            top: cy - 10.0,
            right: cx + 10.0,
            bottom: split_y,
        };
        let q01 = Bounds {
            left: cx - 10.0,
            top: split_y,
            right: split_x,
            bottom: cy + 10.0,
        };
        let q11 = Bounds {
            left: split_x,
            top: split_y,
            right: cx + 10.0,
            bottom: cy + 10.0,
        };
        let sum = clipped_quad_coverage(bounds, radius, Some(q00), cx, cy)
            + clipped_quad_coverage(bounds, radius, Some(q10), cx, cy)
            + clipped_quad_coverage(bounds, radius, Some(q01), cx, cy)
            + clipped_quad_coverage(bounds, radius, Some(q11), cx, cy);
        let unclipped = quad_coverage(bounds, radius, cx, cy);
        assert!(
            (sum - unclipped).abs() <= 1e-6,
            "case {index}: partition sum {sum} vs unclipped {unclipped}"
        );
    }
}
