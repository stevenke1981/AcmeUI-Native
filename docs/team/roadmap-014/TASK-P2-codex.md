# ROADMAP 014 — TASK P2 (Codex GPT-6.1-Sol xhigh): analytic arc coverage in the CPU renderer

Worktree: this one (branch `sdk/roadmap-014-arc`, based on `sdk/roadmap-013`). Work only in this worktree.
You own **only** `crates/acme-render-cpu/**` (plus re-blessing its own golden images if any change). p5 works in parallel on `crates/acme-accessibility`.

## Background
Open todo (ROADMAP 009): "Area-based clip edges; analytic arc area (currently 8x8 sampled)."
`crates/acme-render-cpu/src/coverage.rs` computes exact pixel-area coverage for sharp rectangles and for pixels wholly inside/outside a corner arc, but pixels crossed by the arc use an 8x8 supersample (`supersampled_coverage`, `bounded_sample`). That quantises coverage to 1/64 steps and is the slow path.

## Required
1. Replace the crossed-pixel supersample with the **exact area** of `pixel square ∩ rounded rectangle` within the corner square: the area of the intersection of an axis-aligned unit square with a disc quadrant (closed-form via the circle segment / integral of `sqrt(r² − x²)`; handle the pixel straddling the square edges and radii < 1 px). Use f64 internally if needed for stability; output f32.
2. Properties to test (add property-style tests with a fixed seed, no new deps):
   - agrees with a very fine reference (e.g. 256x256 samples) within 1e-3 over many random rects/radii/offsets including fractional DPI positions;
   - monotonic: larger radius never yields more coverage for the same pixel;
   - coverage ∈ [0, 1]; sum over all pixels ≈ exact rounded-rect area `w·h − (4 − π)·r²` within a tight tolerance;
   - degenerate cases: radius 0, radius = half the side, sub-pixel rects, rect crossing pixel edges.
3. **Clip edges (if time allows, separate commit-ready change):** make rectangular clip edges area-based (fractional coverage at clip boundaries) instead of the current behaviour — first read how clipping is applied and say in the report whether this is needed or already exact.
4. Golden/pixel tests: if any existing golden image changes, report the max per-channel delta and the number of pixels changed, and re-bless only with that evidence. Run the existing CPU benchmark (if present) before/after and report timings honestly (no invented numbers).

## Hard rules
- TDD: failing tests first (RED evidence), GREEN, plus a mutation check.
- Only `crates/acme-render-cpu/**` and `docs/team/roadmap-014/` (your report). No git add/commit/stash/checkout/push. No new dependencies. Rust 1.88.0.
- Verify: `cargo fmt -p acme-render-cpu -- --check`, `cargo clippy -p acme-render-cpu --all-targets -- -D warnings`, `cargo test -p acme-render-cpu`, `cargo check --workspace --all-targets`.
- Do not write absolute local paths into docs (the repo is public).

## Report
Write `docs/team/roadmap-014/REPORT-P2-codex.md`: math/design, files changed, RED/GREEN/mutation evidence, golden deltas, benchmark before/after, commands and results, NOT_RUN. End your final message with a line containing only `P2 DONE`.
