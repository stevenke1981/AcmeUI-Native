# ROADMAP 018 — Track R1 Report: CPU Renderer Fast Path

- **Owner:** agy
- **Lead:** Claude
- **Track:** R1 (`BRIEF-R1-cpu-fast-path.md`)
- **Branch:** `sdk/roadmap-018-perf`

---

## 1. Problem & Baseline Measurements

`crates/acme-render-cpu/tests/frame_timing.rs` renders a CrewDesk-like scene (large rounded panels with borders, ~60 small rounded rows/buttons, nested clips).

Before optimization, `draw_quad` invoked `clipped_quad_coverage` (f64 circle integrals and corner arc math, twice per bordered quad) for every single pixel in every quad's bounding box, followed by per-pixel alpha blending even when coverage and opacity were 1.0.

### Baseline (Before)
Run: `cargo test --release -p acme-render-cpu --test frame_timing -- --ignored --nocapture`
- **Scale 1.0 (1280×790):** 249.33 ms/frame
- **Scale 1.25 (1600×987):** 507.65 ms/frame

---

## 2. Changes Implemented

| Path | Summary |
|---|---|
| `crates/acme-render-cpu/src/coverage.rs` | Added non-corner band fast-path in `clipped_quad_coverage`: when a pixel lies in the central column `[left + radius, right - radius]` or the central horizontal band `[top + radius, bottom - radius]`, no corner arc can remove area, returning `sharp_coverage` immediately without circle integration. |
| `crates/acme-render-cpu/src/lib.rs` | 1. `blend`: Added zero-cost early exits for `opacity <= 0.0` (returns `dst`) and `opacity >= 1.0 && (src >> 24) == 255` (returns `src`).<br>2. `row_interior_span`: Computes exact horizontal integer pixel interval `[x0, x1)` for row `y` where coverage is guaranteed 1.0 (fully inside rect and clip, outside all corner squares).<br>3. `draw_quad`: Decomposes each row into left edge pixels `left..x0`, fast interior span `x0..x1`, and right edge pixels `x1..right`. For bordered quads in rows outside `inner`, the interior span is 100% stroke; in rows inside `inner`, the interior span is 100% fill; for borderless quads, 100% fill. Opaque spans use direct `slice.fill`. Edge pixels evaluate exact analytic `clipped_quad_coverage`.<br>4. `render_scene`: Optimized output composite loop by detecting `a == 255` to assign `pixel & 0x00ff_ffff` directly, eliminating integer divisions on opaque frames. |

---

## 3. Performance Results (After)

Run: `cargo test --release -p acme-render-cpu --test frame_timing -- --ignored --nocapture`

| Configuration | Baseline (Before) | Fast Path (After) | Speedup | Target |
|---|---|---|---|---|
| **Scale 1.0 (1280×790)** | 249.33 ms/frame | **1.88 ms/frame** | **132.6× faster** | — |
| **Scale 1.25 (1600×987)** | 507.65 ms/frame | **9.40 ms/frame** | **54.0× faster** | **≤ 15 ms/frame (PASSED)** |

---

## 4. Automated Verification

| Command | Result |
|---|---|
| `cargo test -p acme-render-cpu` | **PASS** — 87 passed, 0 failed, 1 ignored (`crewdesk_like_frame_cost`), 0 filtered out |
| `cargo test --release -p acme-render-cpu --test frame_timing -- --ignored --nocapture` | **PASS** — 1 passed |
| `cargo clippy -p acme-render-cpu --all-targets -- -D warnings` | **PASS** — exit code 0, 0 warnings |
| `cargo fmt --check` | **PASS** — exit code 0 |
| Golden fixtures comparison (`tests/golden.rs`) | **PASS** — zero pixel divergence, byte-identical output across all scenes |

---

## 5. Acceptance & Scope Rules Compliance

- **Touched files:** Only `crates/acme-render-cpu/**` and `docs/team/roadmap-018/REPORT-R1-cpu-fast-path.md`.
- **Git writes:** None performed.
- **Cargo clean:** None performed.
- **GPUI:** None introduced.
- **Local paths in docs:** None.
- **Public API:** Completely unchanged.
- **Golden files:** None regenerated; byte-identical rendering preserved.
