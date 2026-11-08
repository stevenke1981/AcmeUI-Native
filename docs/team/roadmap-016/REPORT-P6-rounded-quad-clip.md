# ROADMAP 016 — Track P6 Report: Exact Rounded-Quad Clip Masks

- **Owner:** agy
- **Track:** P6 (`BRIEF-P3-next.md` — ROADMAP 015 rounded-quad clip masks)
- **Branch:** `sdk/roadmap-016-terminal-host`

---

## 1. Files Changed

| Path | Summary |
|---|---|
| `crates/acme-render-cpu/src/coverage.rs` | Added `clipped_quad_coverage(bounds, radius, clip, cx, cy)` integrating the four corner cutouts over the clipped pixel sub-rectangle `pixel ∩ clip` (`(kcx, kcy, hx, hy)`); kept `quad_coverage` as `clipped_quad_coverage(..., None, ...)` |
| `crates/acme-render-cpu/src/lib.rs` | Unified `draw_quad` to evaluate `clipped_quad_coverage(rect, radius, clip, x, y)` and `clipped_quad_coverage(inner, inner_radius, clip, x, y)` directly for both sharp and rounded quads without double-multiplying `clip_coverage` |
| `crates/acme-render-cpu/src/coverage/analytic_tests.rs` | Added `clipped_analytic_coverage_matches_fine_reference` (256 randomized rounded-rect + clip cases vs 512x512 reference) and `clipped_analytic_coverage_is_additive_across_partition_splits` (128 four-quadrant split cases) |
| `crates/acme-render-cpu/tests/clip_area.rs` | Added `rounded_coincident_outer_clip_preserves_edges_at_every_dpi`, `rounded_border_coincident_outer_clip_preserves_edges_at_every_dpi`, `rounded_border_clip_at_or_inside_flat_inner_has_only_exact_fill_at_every_dpi`, `rounded_disjoint_parts_of_one_pixel_have_zero_joint_area`, and updated `rounded_shape_area_and_clip_area_are_multiplied` to verify the exact clipped quarter-disc integral (`pi/12 + sqrt(3)/8` and complementary half `pi/4 - (pi/12 + sqrt(3)/8)`) |
| `docs/status/todos.md` | Marked the ROADMAP 015 rounded-quad clip mask item complete |

---

## 2. Design Summary

For any rounded rectangle $Q = B \setminus \bigcup_{i=1}^4 O_i$ with bounding box $B$ and four disjoint corner cutouts $O_i = S_i \setminus D_i \subseteq B$, and any axis-aligned clip rectangle $C$, the intersection with a unit pixel $P = [c_x - 0.5, c_x + 0.5] \times [c_y - 0.5, c_y + 0.5]$ is:

$$Q \cap C \cap P = Q \cap K = (B \cap K) \setminus \bigcup_{i=1}^4 (O_i \cap K), \quad K = P \cap C$$

Because $K$ is an axis-aligned rectangle $[k_{x0}, k_{x1}] \times [k_{y0}, k_{y1}]$ with midpoint $(k_{cx}, k_{cy})$ and half-extents $(h_x, h_y)$, reflecting $K$ into each corner's positive quadrant and clamping to $[0, r]^2$ yields an axis-aligned box $[x_0, x_1] \times [y_0, y_1] \subseteq [0, r]^2$ whose area inside the corner disc is evaluated exactly by the existing `disc_rectangle_area(x0, y0, x1, y1, radius)` routine:

- Unclipped draws (`clip == None`) pass `(cx, cy, 0.5, 0.5)` and are bit-identical to the prior implementation.
- Sharp draws (`radius == 0.0`) return `sharp_coverage(bounds.intersect(clip), cx, cy)` without entering the corner loop.
- Rounded draws (`radius > 0.0`, filled or bordered) evaluate exact outer and inner clipped areas:
  - Coincident outer clips on fractional rounded quads (`PushClip(rect)`) produce byte-identical output to unclipped rounded quads at all five DPI scales (`1.0, 1.25, 1.5, 1.75, 2.0`).
  - Clips inside the flat inner core of a bordered rounded quad produce zero border stroke seam.
  - Partitioning a pixel across four complementary clips sums to the unclipped `quad_coverage` within `1e-6`.
  - Maximum error across 256 randomized rounded-quad + clip cases against a 512x512 supersampled reference is `1.95e-5` (`0.000019486`).
  - All 6 committed golden scenes (`tests/golden.rs`) and all 25 integer-clip frames (`integer_clips_are_byte_identical_to_legacy_centre_mask`) remain byte-identical with zero re-bless.

---

## 3. Automated Verification

| Command | Result |
|---|---|
| `cargo test -p acme-render-cpu` | **PASS** — 87 passed, 0 failed, 0 ignored (20 unit + 8 `atlas_pages` + 20 `clip_area` + 9 `coverage` + 8 `dpi` + 7 `golden` + 15 `regression`) |
| `cargo clippy -p acme-render-cpu --all-targets -- -D warnings` | **PASS** (exit 0, 0 warnings) |
| `cargo check --workspace --all-targets` | **PASS** (exit 0) |
| `rustfmt --edition 2024 --check crates/acme-render-cpu/src/lib.rs crates/acme-render-cpu/src/coverage.rs crates/acme-render-cpu/src/coverage/analytic_tests.rs crates/acme-render-cpu/tests/clip_area.rs` | **PASS** (exit 0) |

---

## 4. Manual / Hardware Checks

- **GPU fractional clip parity:** `NOT_RUN` (software rasterizer only).
- **Live Gallery visual / physical DPI / IME / screen-reader checks:** `NOT_RUN` (headless automated tests only).
