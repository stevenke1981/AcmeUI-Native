# BRIEF Q1 — GPU parity for fractional clips (ROADMAP 017)

Owner: agy. Lead: Claude. Branch `sdk/roadmap-017` (worktree created by the lead).

## Goal
`docs/status/todos.md` item: "GPU renderer parity for fractional clips — NOT_RUN". The CPU
renderer now computes exact coverage for sharp and rounded quads under fractional clips
(ROADMAP 016 P6). Bring `acme-render-wgpu` to the same semantics where it can be verified
without a physical GPU:
1. Audit how the wgpu path applies `PushClip` rects with fractional edges (scissor rounding,
   shader-side clip, rounded quads). Document the current behaviour precisely.
2. Where the GPU path rounds a fractional clip to whole pixels, add shader-side (or
   CPU-prepared) clip coverage so edge pixels get fractional coverage like the CPU path.
3. Add tests that run without a GPU (pure geometry/uniform preparation, WGSL validation via
   naga if already a dependency). Do not add new heavy dependencies.

## Rules
Only `crates/acme-render-wgpu/**`, its tests and
`docs/team/roadmap-017/REPORT-Q1-gpu-fractional-clip.md`. No git writes, no cargo clean, no
GPUI, no local paths in docs, keep public APIs backward compatible. Real GPU drawing stays
NOT_RUN. Acceptance: `cargo test -p acme-render-wgpu`, clippy `-D warnings`, rustfmt check.
Finish with a line `TRACK-Q1 DONE`.
