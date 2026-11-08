# ROADMAP 016 — Track P2 Report: GPU/CPU Colour Parity (sRGB Surface)

## Changed Files

- `crates/acme-render-wgpu/src/surface.rs`
- `crates/acme-render-wgpu/src/renderer/init.rs`
- `crates/acme-render-wgpu/src/renderer/recovery.rs`
- `crates/acme-render-wgpu/src/atlas_pages.rs`
- `crates/acme-render-wgpu/src/images.rs`
- `crates/acme-render-wgpu/src/tests.rs`

## Summary of Changes

1. **Non-sRGB Swapchain Format (`surface_format` & `configure_surface_srgb_passthrough`)**:
   - Added pure helper `surface_format(caps_default: wgpu::TextureFormat) -> wgpu::TextureFormat` (`caps_default.remove_srgb_suffix()`) and `configure_surface_srgb_passthrough(&mut wgpu::SurfaceConfiguration)` in `crates/acme-render-wgpu/src/surface.rs`.
   - Applied `configure_surface_srgb_passthrough(&mut config)` in both `Renderer::new` (`crates/acme-render-wgpu/src/renderer/init.rs`) and `Renderer::on_device_lost` (`crates/acme-render-wgpu/src/renderer/recovery.rs`), keeping `view_formats` compatible and documenting:
     `"Scene colours are sRGB-encoded; the GPU path writes them unchanged, like the CPU path."`
2. **Non-sRGB RGBA8 Atlas Pages and Image Textures (`Rgba8Unorm`)**:
   - Updated `GlyphFormat::Rgba8` in `crates/acme-render-wgpu/src/atlas_pages.rs` (`atlas_texture_format`) from `wgpu::TextureFormat::Rgba8UnormSrgb` to `wgpu::TextureFormat::Rgba8Unorm`, and updated the format assertion in `atlas_pages_ensure_gpu` plus added headless unit test `atlas_texture_format_uses_non_srgb_rgba8`.
   - Updated `create_image_resource` in `crates/acme-render-wgpu/src/images.rs` (`IMAGE_TEXTURE_FORMAT`) from `wgpu::TextureFormat::Rgba8UnormSrgb` to `wgpu::TextureFormat::Rgba8Unorm`.
3. **Blending**:
   - Left pipeline blend state unchanged (sRGB-space blending matching the CPU renderer).

## Verification & Test Counts

| Command / Check | Status | Result |
| --- | --- | --- |
| `cargo test -p acme-render-wgpu` | PASS | 62 passed, 0 failed, 2 ignored (unit tests); 0 failed, 1 ignored (doc-test) |
| `cargo clippy -p acme-render-wgpu --all-targets -- -D warnings` | PASS | 0 warnings, 0 errors |
| `cargo check --workspace --all-targets` | PASS | All workspace crates and app targets compiled cleanly |
| `rustfmt --edition 2024 --check <changed files>` | PASS | 0 formatting diffs |
| Real GPU visual drawing parity check | NOT_RUN | Headless environment; lead verifies visually in the consumer app |
