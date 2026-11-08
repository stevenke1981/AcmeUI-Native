# ROADMAP 016 — Track P2: GPU/CPU colour parity (sRGB surface)

Owner: agy. Lead: Claude. Same hard rules as BRIEF-P-platform.md (no git writes, no new
deps, Rust 1.88, no local paths in docs).

## Defect (reproduced by the lead)

The same Scene renders visibly lighter/washed out on the wgpu path than on the CPU path
(`ACME_RENDER_BACKEND=cpu`). Theme colours are sRGB-encoded values (e.g. `#0a0f17`).
`crates/acme-render-wgpu/src/renderer/init.rs` and `recovery.rs` use
`surface.get_default_config(..)`, which picks an `*UnormSrgb` swapchain format on Windows,
so the GPU encodes the already-sRGB quad/text colours a second time.

## Fix (contract)

1. Configure the surface with the non-sRGB variant of the chosen format
   (`format.remove_srgb_suffix()`), in both init and recovery; `view_formats` stays
   compatible. Factor the choice into a pure fn (e.g. `surface_format(caps_default)`)
   with unit tests.
2. Colour glyph atlas pages and image textures currently use `Rgba8UnormSrgb`, which
   decode to linear on sampling. With a non-sRGB target they must be `Rgba8Unorm` so the
   sRGB bytes pass through unchanged (matching the CPU renderer). Update the format
   assertions in tests accordingly.
3. Blending stays as is (sRGB-space blending, same as the CPU renderer).
4. Document the colour-space rule in a short comment at the surface configuration:
   "Scene colours are sRGB-encoded; the GPU path writes them unchanged, like the CPU path."

Files you own: `crates/acme-render-wgpu/src/**`.

## Acceptance

```
cargo test -p acme-render-wgpu
cargo clippy -p acme-render-wgpu --all-targets -- -D warnings
cargo check --workspace --all-targets
```
Real GPU drawing cannot be checked headless; mark it NOT_RUN in the report — the lead
verifies visually in the consumer app.

Report: `docs/team/roadmap-016/REPORT-P2-srgb.md`, then reply `TRACK-P2 DONE`.
