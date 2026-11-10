# A4 Scene preflight handoff

- Actor: `/root/scene_contract`, independent worker; isolated worktree `D:/AcmeUI-Native/.worktrees/scene-031`.
- Base SHA: `1f23c910f05d7c975094df1e3b2be65ee2437ec9`.
- Code SHA: `9244e0c84e240245b0fa4e29df2a2c1f4117947c`.
- Owned files: core `scene.rs`, CPU `lib.rs`, GPU `batch.rs`; no other source edits.
- Status: PASS for targeted source/test slice; renderer entry integration remains lead-owned.

## Changes

`Scene::validate` owns clip/layer underflow and balance checks. Clip and layer stacks remain independent. CPU resource preflight keeps upload validation and allocation budgets, then maps shared structure errors to existing CPU error messages, before clearing framebuffer or applying uploads. For scenes containing several simultaneous faults, a CPU resource error may precede a structural error; rejection and state preservation remain guaranteed.

GPU public `compile_scene` preserves its return signature, warns and returns no batches for malformed structure. Private `try_compile_scene` returns the specific error and prevents partial compilation. The lead must invoke it at the start of `Renderer::render_scene`, before `update_live_scene_images`, return `SurfaceAction::Exit` with a diagnostic on Err, and reuse the resulting batches instead of invoking public `compile_scene` later. This validates once at the renderer boundary.

Compatibility: `SceneError` gains `LayerUnderflow` and `LayerUnbalanced`. Downstream exhaustive matches must add those arms. Valid-scene rendering and public function signatures are preserved. This batch does not implement GPU composited group opacity.

## Evidence

- `scene-core-red.json/.txt`: baseline plus regressions; `cargo test --locked -p acme-core validate_layer`, exit 101, both layer tests fail because validation previously accepted invalid layers.
- `scene-gpu-red.json/.txt`: clean baseline plus regression; `cargo test --locked -p acme-render-wgpu malformed_scene_produces_no_partial_batches`, exit 101, invalid PopClip still produces draw batches.
- `scene-green.json/.txt`: `cargo test --locked -p acme-core -p acme-render-cpu -p acme-render-wgpu`, exit 0. 215 nonignored tests passed; 4 ignored tests/doc-tests remain NOT_RUN. Existing CPU rejected-scene atlas preservation and allocation budget regressions pass.
- `rustfmt --edition 2024 --check` for the three source files: PASS, exit 0.
- `git diff --check`: PASS, exit 0.
- `scene-source-manifest.json`: hashes for the code tested before committing. Tests ran on a dirty base with precisely these source changes, not on a clean committed revision; integrated candidate verification is still required.
- `scene-gpu-interleaved-build.json/.txt`: INVALID as baseline evidence. Source editing overlapped compilation and produced mixed cached core/new batch code. Preserved for transparency, then replaced by a clean-base regression run; never counted as product-failure reproduction or PASS.

## Remaining / next owner

Lead: integrate code and renderer guard; perform CPU profile, integrated workspace gates, native GPU skipped-frame checks, independent Rust review, CI, merge and release build. This worker did not push, merge, publish, run GPU rendering, or validate native IME, DPI or screen readers. Self-check is not independent review.
