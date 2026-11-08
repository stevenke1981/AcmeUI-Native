# BRIEF R1 — CPU renderer fast path (ROADMAP 018)

Owner: agy. Lead: Claude. Worktree branch `sdk/roadmap-018-perf`.

## Problem (measured)
`crates/acme-render-cpu/tests/frame_timing.rs` (an ignored probe added by the lead) renders a
CrewDesk-like scene — a few large rounded panels with borders, ~60 small rounded rows/buttons,
two clips — and takes **153 ms/frame at 1280×790 (scale 1.0) and 241 ms at 1600×987
(scale 1.25)** in release. A native CrewDesk window repaints on every caret blink and every
PTY update, so the app feels laggy and burns CPU.

Run: `cargo test --release -p acme-render-cpu --test frame_timing -- --ignored --nocapture`

Cause: `draw_quad` evaluates `clipped_quad_coverage` (exact f64 corner integrals, twice when
bordered) for **every** pixel of every quad, including the large interiors that are
trivially fully covered, and blends even when opaque at full coverage.

## Goal
≤ 15 ms/frame for the probe at scale 1.25 (aim lower), with **byte-identical** output for
every existing CPU test (goldens, coverage, clip_area, regression, dpi).

## Approach (suggested, your call)
1. Per quad, compute the axis-aligned "interior" where coverage is exactly 1: the rect
   shrunk by the corner radius horizontally for the top/bottom `radius` rows (or the full
   rect width for rows outside the corner bands), intersected with the clip, on whole
   pixels only. Pixels there get coverage 1 without calling the exact integrator; for a
   bordered quad also the inner rect's interior gives `inside = 1`.
2. Fill opaque spans directly (`fill` alpha 255, coverage 1 → write, no blend); keep
   `blend` for partial coverage or translucent colours. Make `blend(.., 1.0)` with an
   opaque source return `src` exactly as before so results stay identical.
3. Exact integration only on edge/corner pixels.
4. Clear: fill the framebuffer with one value (slice fill).
5. Optional if still above target: row-span iteration instead of per-pixel closures,
   and skipping glyph/image work outside the clip.

## Rules
Only `crates/acme-render-cpu/**` and `docs/team/roadmap-018/REPORT-R1-cpu-fast-path.md`.
No git writes, no cargo clean, no GPUI, no local paths in docs, public API unchanged.
Acceptance: `cargo test -p acme-render-cpu` (all green, outputs identical — goldens must not
be regenerated), clippy `-D warnings`, rustfmt check, and the probe numbers before/after in
the report. Finish with a line `TRACK-R1 DONE`.
