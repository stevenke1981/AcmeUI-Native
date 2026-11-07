# CPU Renderer — Agent Team Workflow implementation record

## Goal and scope

Introduce a software renderer for AcmeUI's backend-neutral `Scene`, retain the
existing wgpu path, support explicit CPU selection, and fall back to software
when GPU startup or GPU recovery fails.

## Lean team responsibilities (logical workflow, not a claim of independent agents)

| Stage | Owner | Scope / evidence |
|---|---|---|
| Plan / Architect | Planning role | Keep `Scene` as the shared contract; do not clone WidgetNode/layout APIs |
| Work / Implementer | Implementation role | Add `acme-render-cpu`, a private softbuffer presenter, and backend selection |
| Verification | Test role | Headless pixel tests; CI matrix on Windows/macOS/Linux; interactive Windows smoke |
| Review | Reviewer needed | Inspect alpha blending, bounds, GPU-to-CPU fallback, accessibility and DPI |
| PR / Delivery | Maintainer | Commit to a feature branch, open Draft PR, require CI and human sign-off before merge |

No separate AI subagents or independent reviewers are asserted by this document.

## Architecture

```text
App / WidgetNode → Layout → backend-neutral Scene
                             ├─ acme-render-wgpu → GPU / OS Surface
                             └─ acme-render-cpu  → XRGB8888 buffer
                                                     ↓
                                           softbuffer / OS window
```

The `acme-render-cpu` crate is headless, requires no GPU context, and provides
`CpuRenderer::new/resize/render_scene/pixels`.

`acme-platform` defaults to GPU. Set `ACME_RENDER_BACKEND=cpu` to force
software rendering. When GPU initialization fails, it attempts CPU software
presentation. When GPU device-loss recovery fails, it attempts a one-way switch
to CPU for the affected window. The `cpu-renderer` Cargo feature is enabled
by default on `acme-platform`; disable default features to omit it.

## Supported in first implementation

- Ordered scene quads, solid fills, rounded borders, alpha blending
- Nested rectangular clipping in physical pixels
- RGBA image `Contain` and `Fill` sampling
- Alpha8 and RGBA8 glyph uploads through the single-page 2048x2048 atlas
- Composited nested opacity layers with transparent intermediate surfaces
- DPI scaling, resize, maximum framebuffer capacity, malformed-command checks
- Pixel-level unit tests without a GPU or display

## Limitations / follow-up

- CPU output is opaque XRGB8888. Transparent windows are not supported.
- Glyph atlas page 0 only (matches the current wgpu bridge). Reject other pages.
- CPU image scaling is nearest-neighbour; subpixel/complex path rendering and
  GPU-identical antialiasing are not guaranteed.
- No performance parity claim or proven FPS on production hardware.
- The feature is desktop-targeted; Android/iOS/Web runtime native integration
  is not proven by `acme-ui --lib` cross-target compilation.
- Actual Windows/Linux/macOS application visual tests, CJK IME, DPI, and GPU
  device-loss fallback require interactive sign-off.
- Update `Cargo.lock` from the pinned dependency graph when building with
  Cargo; the authoring environment did not have the Rust toolchain.

## Verification gates (results must be filled from actual executions)

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test -p acme-render-cpu
cargo test --workspace
cargo clippy -p acme-render-cpu --all-targets -- -D warnings
cargo run -p acme-gallery
```

### Windows forced software smoke

```powershell
$env:ACME_RENDER_BACKEND = 'cpu'
cargo run -p acme-gallery
Remove-Item Env:ACME_RENDER_BACKEND
```

Verify text, CJK, overlay clipping, scrolling, resize, mouse interaction,
125/150/200% DPI, normal exit. Compare with default GPU rendering.

### Rollback

Disable CPU support with `acme-platform = { ..., default-features = false }`
or run with `ACME_RENDER_BACKEND=gpu` (forced GPU preference, but initialization
failure still falls back when support is compiled in). Revert the PR to remove
the new crate, dependency and private platform adapter. No user data migration.
