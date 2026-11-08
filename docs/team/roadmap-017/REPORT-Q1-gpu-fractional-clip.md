# ROADMAP 017 — Track Q1 Report: GPU Parity for Fractional Clips

- **Owner:** agy
- **Track:** Q1 (`BRIEF-Q1-gpu-fractional-clip.md`)
- **Branch:** `sdk/roadmap-017`

---

## 1. Audit of Previous `acme-render-wgpu` Behaviour on Fractional Clips

1. **Batch Compilation (`crates/acme-render-wgpu/src/batch.rs`):**
   - `compile_scene` intersected `PushClip` rectangles in logical coordinates via `acme_core::ClipStack` and stored `batch.clip: Option<[f32; 4]>` (`[x, y, width, height]`) preserving fractional logical coordinates.
2. **Whole-Pixel Hardware Scissor Rounding (`crates/acme-render-wgpu/src/draws.rs` & `src/renderer/frame.rs`):**
   - `scissor(rect, scale, width, height)` converted the logical clip into an outward-rounded integer physical box (`left.floor()`, `top.floor()`, `right.ceil()`, `bottom.ceil()`) and passed it to `wgpu::RenderPass::set_scissor_rect`.
   - In addition, when `rect[0]` or `rect[1]` was fractional and `rect[2] == 0.0` or `rect[3] == 0.0` (e.g. `[0.2, 0.2, 0.0, 0.5]`), `0.2.floor() = 0` and `(0.2 + 0.0).ceil() = 1` caused `scissor` to return a `1×1` non-empty scissor rect `Some([0, 0, 1, 1])` for a zero-area clip.
3. **Missing Shader-Side Clip Bounds & Coverage (`src/instances.rs`, `src/quad.wgsl`, `src/text.wgsl`):**
   - Neither `Instance` (quads) nor `GlyphInstance` (text glyphs and images) carried the physical clip bounds.
   - Consequently, every pixel inside the outward-expanded integer scissor rectangle received `1.0` clip weight instead of fractional edge coverage.
   - In `quad.wgsl`:
     - Vertex positions were placed at `[rect.x, rect.y] .. [rect.x + rect.w, rect.y + rect.h]` without expanding to the enclosing integer pixel grid `[floor, ceil]`. Under 1x pixel-center rasterization, any subpixel quad/clip or fractional edge that did not cover a pixel center `(x + 0.5, y + 0.5)` failed to invoke the fragment shader for that pixel.
     - Fragment coverage used an unclipped signed-distance `smoothstep(-0.75, 0.75, dist)` approximation and a binary `dist > -border_width` border threshold rather than exact analytic `(unit pixel) ∩ quad ∩ clip` area integration.

---

## 2. Changes Implemented

| Path | Summary |
|---|---|
| `crates/acme-render-wgpu/src/clip.rs` | New module implementing `physical_clip_bounds`, `quad_raster_bounds`, `overlap`, `rect_coverage`, `clipped_quad_coverage` (exact analytic sharp + rounded quad ∩ clip pixel-area integral), `shade_quad_fragment`, `shade_glyph_fragment`, and unit tests (including `naga` WGSL validation and full DPI parity tests) |
| `crates/acme-render-wgpu/src/draws.rs` | Updated `scissor` to derive integer scissor bounds from `physical_clip_bounds`, rejecting zero-area or negative-sized fractional clips |
| `crates/acme-render-wgpu/src/instances.rs` | Added `@location(4) clip: [f32; 4]` (`[left, top, right, bottom]` physical clip bounds) to `Instance` and `GlyphInstance`; threaded `clip: Option<[f32; 4]>` through `quad_primitive_to_instance`, `image_primitive_to_instance`, `text_primitive_glyphs`, and `collect_text_glyphs` |
| `crates/acme-render-wgpu/src/pipelines.rs` | Extended `Instance` and `GlyphInstance` vertex buffer layouts with attribute `4 => Float32x4` |
| `crates/acme-render-wgpu/src/renderer/frame.rs` | Passed `batch.clip` when constructing quad, text, and image instances |
| `crates/acme-render-wgpu/src/quad.wgsl` | Updated `vs_main` to expand vertex positions to `[floor(vis_min), ceil(vis_max)]` of `quad ∩ clip ∩ viewport`; updated `fs_main` to compute exact `clipped_quad_coverage` for outer fill and inner border and composite stroke over fill identically to `acme-render-cpu` |
| `crates/acme-render-wgpu/src/text.wgsl` | Forwarded `@location(4) clip` to `fs_main` and multiplied `Alpha8` glyph, `Rgba8` glyph, and `Image` fragment alpha by `clip_coverage(input.clip_bounds, input.position.xy)` while keeping `textureSample` in uniform control flow |
| `crates/acme-render-wgpu/src/lib.rs` | Registered internal `mod clip` |
| `crates/acme-render-wgpu/src/tests.rs` | Updated instance helper call sites and added zero/negative-size fractional clip assertions to `scissor_intersects_the_physical_viewport` |

---

## 3. Automated Verification

| Command | Result |
|---|---|
| `cargo test -p acme-render-wgpu` | **PASS** — 71 passed, 0 failed, 2 ignored (`atlas_pages_ensure_gpu`, `device_recovery_smoke_ignored`) |
| `cargo clippy -p acme-render-wgpu --all-targets -- -D warnings` | **PASS** (exit 0, 0 warnings) |
| `rustfmt --edition 2024 --check` on changed files | **PASS** (exit 0) |

---

## 4. Manual / Runtime Checks

- **Real GPU hardware drawing validation:** `NOT_RUN` (verified headlessly via `wgpu::naga` WGSL parser/validator, instance/scissor preparation tests, and CPU-side evaluation of the exact WGSL shader formulas across 1.0×–2.0× DPI scales).
