# BRIEF P8 — CPU window clipped after a DPI change (ROADMAP 016)

Owner: agy. Lead: Claude.

## Symptom (observed natively, CPU renderer, Windows 10, two monitors 100% and 125%)
A CrewDesk window (AcmeUI CPU backend) created on one monitor and moved to a monitor with a
different scale factor presents a frame that does not cover the client area: on the 100%
monitor only about 1043x747 of a 1280x770 client was drawn (rest white/black); after moving
to the 125% monitor the bottom ~35 physical px stayed black. Layout itself used the right
logical size (FrameContext comes from `inner_size()/scale_factor()` in `handle_redraw`).

## Suspects
- `WindowEvent::ScaleFactorChanged` -> `handle_resize(.., None)` resizes the renderer with the
  *old* `inner_size()` before Windows applies the new size, and emits no `Resized` to the app.
  If the following `WindowEvent::Resized` is missed/coalesced, raster and surface stay stale.
- `CpuWindowRenderer::present` copies `raster.pixels()` into a surface whose size may differ.

## Task
1. Make the CPU (and GPU) path self-healing: in `handle_redraw`, if the renderer's size differs
   from `inner_size()` (or scale differs), resize before rendering. Use the
   `inner_size_writer` / new size from `ScaleFactorChanged` when available, and emit
   `PlatformEvent::Resized` to the app on scale changes too.
2. Unit-test the pure decision (e.g. `needs_resize(renderer_dims, inner_size, scale)`), and a
   CPU renderer test that a scale change followed by render covers the full buffer.

Files you own: `crates/acme-platform/src/lib.rs`, `crates/acme-platform/src/cpu_window.rs`,
`crates/acme-platform/tests/*`, `docs/team/roadmap-016/REPORT-P8-dpi-change.md`.
Rules as before: no git writes, no cargo clean, no GPUI, no local paths in docs, backward
compatible. Acceptance: `cargo test -p acme-platform`, clippy `-D warnings`, rustfmt check.
Mark real two-monitor validation NOT_RUN. Finish with `TRACK-P8 DONE`.
