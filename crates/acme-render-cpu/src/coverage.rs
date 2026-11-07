//! Pixel-area coverage for quads (ROADMAP 009, P2).
//!
//! The coverage of a pixel is the area of its unit square `[x, x + 1) x [y, y + 1)`
//! that lies inside the shape. Sharp rectangles use the exact product of the x and
//! y overlaps. A rounded rectangle equals its bounding rectangle except inside the
//! four corner squares, where the arc removes area. Per corner, a pixel is one of
//! three cases: wholly inside the arc (nothing removed), wholly outside it (the
//! removed area is exactly `pixel ∩ corner square`), or crossed by the arc. Only
//! crossed pixels use the analytic integral of the circle boundary.

use crate::Bounds;

#[cfg(test)]
mod analytic_tests;

/// Area of the unit pixel square centred at `center` that lies within `[lo, hi)`.
fn overlap(lo: f32, hi: f32, center: f32) -> f32 {
    ((center + 0.5).min(hi) - (center - 0.5).max(lo)).clamp(0.0, 1.0)
}

/// Exact coverage of a sharp rectangle: the product of the two axis overlaps.
fn sharp_coverage(bounds: Bounds, cx: f32, cy: f32) -> f32 {
    overlap(bounds.left, bounds.right, cx) * overlap(bounds.top, bounds.bottom, cy)
}

/// Area of `[x0, x1] × [y0, y1]` inside the radius-`r` disc, in its
/// positive quadrant. The caller clips this box to `[0, r]²` first.
fn disc_rectangle_area(x0: f64, y0: f64, x1: f64, y1: f64, r: f64) -> f64 {
    let area = (x1 - x0) * (y1 - y0);
    let r2 = r * r;
    if x1 * x1 + y1 * y1 <= r2 {
        return area;
    }
    if x0 * x0 + y0 * y0 >= r2 {
        return 0.0;
    }
    // Before `full_end` the disc covers the whole box height; after `end`
    // it covers none. Between them, integrate sqrt(r² - x²) - y0.
    let full_end = ((r - y1) * (r + y1)).max(0.0).sqrt().clamp(x0, x1);
    let end = ((r - y0) * (r + y0)).max(0.0).sqrt().clamp(full_end, x1);
    let start_y = ((r - full_end) * (r + full_end)).max(0.0).sqrt();
    let end_y = ((r - end) * (r + end)).max(0.0).sqrt();
    let width = end - full_end;
    let chord = width.hypot(start_y - end_y);
    let angle = 2.0 * (chord / (2.0 * r)).clamp(0.0, 1.0).asin();
    // The integral is the trapezoid under the chord plus its circular segment.
    // Local heights avoid subtracting O(r²) antiderivatives for a unit pixel.
    // A short Taylor series also keeps angle - sin(angle) stable for thin caps.
    let segment = if angle < 0.1 {
        let a2 = angle * angle;
        angle * a2 * (1.0 / 6.0 - a2 * (1.0 / 120.0 - a2 * (1.0 / 5040.0 - a2 / 362880.0)))
    } else {
        angle - angle.sin()
    };
    let covered = (full_end - x0) * (y1 - y0)
        + width * ((start_y - y0) + (end_y - y0)) * 0.5
        + 0.5 * r2 * segment;
    covered.clamp(0.0, area)
}

/// Subtract the area outside each disc within its disjoint corner square.
/// Reflecting each corner into the positive quadrant lets all four share the
/// same integral. Clip the pixel to each square *before* integrating: one pixel
/// can cross several corners when the whole rounded rectangle is sub-pixel.
fn rounded_coverage(bounds: Bounds, radius: f32, cx: f32, cy: f32, base: f32) -> f32 {
    let (left, top, right, bottom, radius, cx, cy) = (
        f64::from(bounds.left),
        f64::from(bounds.top),
        f64::from(bounds.right),
        f64::from(bounds.bottom),
        f64::from(radius),
        f64::from(cx),
        f64::from(cy),
    );
    let mut removed = 0.0;
    for (arc_x, arc_y, sx, sy) in [
        (left + radius, top + radius, -1.0, -1.0),
        (right - radius, top + radius, 1.0, -1.0),
        (left + radius, bottom - radius, -1.0, 1.0),
        (right - radius, bottom - radius, 1.0, 1.0),
    ] {
        let x0 = ((cx - arc_x) * sx - 0.5).max(0.0);
        let x1 = ((cx - arc_x) * sx + 0.5).min(radius);
        let y0 = ((cy - arc_y) * sy - 0.5).max(0.0);
        let y1 = ((cy - arc_y) * sy + 0.5).min(radius);
        if x0 >= x1 || y0 >= y1 {
            continue;
        }
        removed += (x1 - x0) * (y1 - y0) - disc_rectangle_area(x0, y0, x1, y1, radius);
    }
    (f64::from(base) - removed).clamp(0.0, f64::from(base)) as f32
}

/// Exact-area coverage of a quad with the given corner radius, at the pixel whose
/// centre is `(cx, cy)`. Non-finite or empty bounds yield zero.
pub(crate) fn quad_coverage(bounds: Bounds, radius: f32, cx: f32, cy: f32) -> f32 {
    let half_x = (bounds.right - bounds.left) * 0.5;
    let half_y = (bounds.bottom - bounds.top) * 0.5;
    // The negated form also rejects NaN.
    if !(half_x > 0.0 && half_y > 0.0) {
        return 0.0;
    }
    let base = sharp_coverage(bounds, cx, cy);
    if base <= 0.0 {
        return 0.0;
    }
    let radius = radius.max(0.0).min(half_x.min(half_y));
    if radius <= 0.0 {
        return base;
    }
    rounded_coverage(bounds, radius, cx, cy, base)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(left: f32, top: f32, right: f32, bottom: f32) -> Bounds {
        Bounds {
            left,
            top,
            right,
            bottom,
        }
    }

    fn inside_rounded(bounds: Bounds, radius: f32, x: f32, y: f32) -> bool {
        if x < bounds.left || x > bounds.right || y < bounds.top || y > bounds.bottom {
            return false;
        }
        let nearest_x = x.max(bounds.left + radius).min(bounds.right - radius);
        let nearest_y = y.max(bounds.top + radius).min(bounds.bottom - radius);
        let (dx, dy) = (x - nearest_x, y - nearest_y);
        dx * dx + dy * dy <= radius * radius
    }

    /// Reference coverage on a fine 64 x 64 grid.
    fn reference(bounds: Bounds, radius: f32, cx: f32, cy: f32) -> f32 {
        const FINE: usize = 64;
        let step = 1.0 / FINE as f32;
        let mut hits = 0usize;
        for row in 0..FINE {
            for column in 0..FINE {
                let x = cx - 0.5 + (column as f32 + 0.5) * step;
                let y = cy - 0.5 + (row as f32 + 0.5) * step;
                hits += usize::from(inside_rounded(bounds, radius, x, y));
            }
        }
        hits as f32 / (FINE * FINE) as f32
    }

    #[test]
    fn sharp_rect_uses_axis_overlap_product() {
        let b = bounds(0.5, 0.5, 2.5, 2.5);
        assert_eq!(quad_coverage(b, 0.0, 0.5, 0.5), 0.25);
        assert_eq!(quad_coverage(b, 0.0, 1.5, 1.5), 1.0);
        // Centre 2.5 is the pixel [2, 3), which still overlaps [0.5, 2.5) by 0.5.
        assert_eq!(quad_coverage(b, 0.0, 2.5, 1.5), 0.5);
        assert_eq!(quad_coverage(b, 0.0, 3.5, 1.5), 0.0);
    }

    #[test]
    fn empty_inverted_and_non_finite_bounds_have_no_coverage() {
        assert_eq!(
            quad_coverage(bounds(1.0, 1.0, 1.0, 5.0), 0.0, 1.5, 2.5),
            0.0
        );
        assert_eq!(
            quad_coverage(bounds(3.0, 0.0, 1.0, 4.0), 0.0, 2.5, 1.5),
            0.0
        );
        assert_eq!(
            quad_coverage(bounds(0.0, 0.0, f32::NAN, 4.0), 0.0, 0.5, 0.5),
            0.0
        );
        assert_eq!(
            quad_coverage(bounds(0.0, 0.0, 4.0, 4.0), f32::NAN, 0.5, 0.5),
            1.0
        );
    }

    #[test]
    fn rounded_fast_paths_agree_with_reference() {
        let b = bounds(0.3, 0.6, 24.3, 16.6);
        for radius in [1.0_f32, 3.0, 8.0] {
            for y in 0..18 {
                for x in 0..26 {
                    let (cx, cy) = (x as f32 + 0.5, y as f32 + 0.5);
                    let got = quad_coverage(b, radius, cx, cy);
                    let want = reference(b, radius, cx, cy);
                    assert!(
                        (got - want).abs() <= 0.07,
                        "radius {radius} pixel ({x},{y}): got {got}, reference {want}"
                    );
                }
            }
        }
    }

    #[test]
    fn interior_pixels_are_fully_covered() {
        // Well inside the core of a rounded rect: exact 1.0, no arc test needed.
        let b = bounds(0.0, 0.0, 20.0, 20.0);
        assert_eq!(quad_coverage(b, 4.0, 10.5, 10.5), 1.0);
        assert_eq!(quad_coverage(b, 4.0, 1.5, 10.5), 1.0);
    }
}
