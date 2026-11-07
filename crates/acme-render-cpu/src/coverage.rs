//! Pixel-area coverage for quads (ROADMAP 009, P2).
//!
//! The coverage of a pixel is the area of its unit square `[x, x + 1) x [y, y + 1)`
//! that lies inside the shape. Sharp rectangles use the exact product of the x and
//! y overlaps. A rounded rectangle equals its bounding rectangle except inside the
//! four corner squares, where the arc removes area. Per corner, a pixel is one of
//! three cases: wholly inside the arc (nothing removed), wholly outside it (the
//! removed area is exactly `pixel ∩ corner square`), or crossed by the arc. Only
//! crossed pixels are supersampled.

use crate::Bounds;

/// Samples per axis for pixels that the arc crosses.
const SUBSAMPLES: usize = 8;
/// Half the diagonal of a unit pixel square. A pixel whose centre is further than
/// this from an arc lies wholly on one side of it.
const BOX_REACH: f32 = std::f32::consts::FRAC_1_SQRT_2;

/// Area of the unit pixel square centred at `center` that lies within `[lo, hi)`.
fn overlap(lo: f32, hi: f32, center: f32) -> f32 {
    ((center + 0.5).min(hi) - (center - 0.5).max(lo)).clamp(0.0, 1.0)
}

/// Exact coverage of a sharp rectangle: the product of the two axis overlaps.
fn sharp_coverage(bounds: Bounds, cx: f32, cy: f32) -> f32 {
    overlap(bounds.left, bounds.right, cx) * overlap(bounds.top, bounds.bottom, cy)
}

/// Whether two rectangles share a region of positive area.
fn overlaps(a: Bounds, b: Bounds) -> bool {
    a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top
}

/// A corner square of a rounded rectangle, with the centre of its arc.
#[derive(Clone, Copy)]
struct Corner {
    square: Bounds,
    arc_x: f32,
    arc_y: f32,
}

fn corners(bounds: Bounds, radius: f32) -> [Corner; 4] {
    let (l, t, r, b) = (bounds.left, bounds.top, bounds.right, bounds.bottom);
    let corner = |square: Bounds, arc_x: f32, arc_y: f32| Corner {
        square,
        arc_x,
        arc_y,
    };
    let square = |left, top, right, bottom| Bounds {
        left,
        top,
        right,
        bottom,
    };
    [
        corner(square(l, t, l + radius, t + radius), l + radius, t + radius),
        corner(square(r - radius, t, r, t + radius), r - radius, t + radius),
        corner(square(l, b - radius, l + radius, b), l + radius, b - radius),
        corner(square(r - radius, b - radius, r, b), r - radius, b - radius),
    ]
}

/// Whether point `(x, y)` lies inside the rounded rectangle. The nearest point of the
/// core rectangle, shrunk by `radius`, must be within `radius` of the point.
fn inside_rounded(bounds: Bounds, radius: f32, x: f32, y: f32) -> bool {
    if x < bounds.left || x > bounds.right || y < bounds.top || y > bounds.bottom {
        return false;
    }
    // `max`/`min` rather than `clamp`: rounding can make the core bounds cross
    // when the radius equals half the side, and `clamp` panics in that case.
    let nearest_x = x.max(bounds.left + radius).min(bounds.right - radius);
    let nearest_y = y.max(bounds.top + radius).min(bounds.bottom - radius);
    let (dx, dy) = (x - nearest_x, y - nearest_y);
    dx * dx + dy * dy <= radius * radius
}

/// Coverage by a fixed `SUBSAMPLES x SUBSAMPLES` grid over the pixel square.
fn supersampled_coverage(bounds: Bounds, radius: f32, cx: f32, cy: f32) -> f32 {
    let step = 1.0 / SUBSAMPLES as f32;
    let mut hits = 0usize;
    for row in 0..SUBSAMPLES {
        let y = cy - 0.5 + (row as f32 + 0.5) * step;
        for column in 0..SUBSAMPLES {
            let x = cx - 0.5 + (column as f32 + 0.5) * step;
            hits += usize::from(inside_rounded(bounds, radius, x, y));
        }
    }
    hits as f32 / (SUBSAMPLES * SUBSAMPLES) as f32
}

/// Sampled coverage, clamped to the bounding-rectangle coverage `base`. The clamp
/// keeps a coarse grid from overshooting the exact area, so a larger radius can never
/// report more coverage than a smaller one.
fn bounded_sample(bounds: Bounds, radius: f32, cx: f32, cy: f32, base: f32) -> f32 {
    supersampled_coverage(bounds, radius, cx, cy).min(base)
}

/// Coverage of a rounded rectangle whose bounding rectangle has coverage `base`.
/// The four corner squares are disjoint, so their removed areas can be summed.
fn rounded_coverage(bounds: Bounds, radius: f32, cx: f32, cy: f32, base: f32) -> f32 {
    let pixel = Bounds {
        left: cx - 0.5,
        top: cy - 0.5,
        right: cx + 0.5,
        bottom: cy + 0.5,
    };
    let mut removed = 0.0;
    for corner in corners(bounds, radius) {
        if !overlaps(pixel, corner.square) {
            continue;
        }
        let distance = (cx - corner.arc_x).hypot(cy - corner.arc_y);
        if distance + BOX_REACH <= radius {
            // Wholly inside the arc: this corner removes nothing.
            continue;
        }
        if distance - BOX_REACH >= radius {
            // Wholly outside the arc: the whole overlap with the square is removed.
            removed += sharp_coverage(corner.square, cx, cy);
            continue;
        }
        // The arc crosses this pixel.
        return bounded_sample(bounds, radius, cx, cy, base);
    }
    (base - removed).max(0.0)
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
    fn interior_pixels_skip_sampling_and_are_fully_covered() {
        // Well inside the core of a rounded rect: exact 1.0, no arc test needed.
        let b = bounds(0.0, 0.0, 20.0, 20.0);
        assert_eq!(quad_coverage(b, 4.0, 10.5, 10.5), 1.0);
        assert_eq!(quad_coverage(b, 4.0, 1.5, 10.5), 1.0);
    }
}
