# ROADMAP 016 — Track P3+: next AcmeUI-Native work (queue, do in order)

Owner: agy. Lead: Claude. Same hard rules as BRIEF-P-platform.md. One report per item:
`docs/team/roadmap-016/REPORT-P<n>-<slug>.md`, then reply `TRACK-P<n> DONE` and continue
with the next item without waiting.

## P3 — app-selected renderer (BLOCKING for the consumer; do first)

The owner wants CrewDesk to prefer the CPU renderer.
- `Application::render_backend(&self) -> RenderBackendPreference` (default `Auto`).
- Resolution: an explicitly set, valid `ACME_RENDER_BACKEND` env value wins; otherwise the
  app's preference; an invalid env value keeps today's behaviour. Pure resolver fn +
  unit tests for every combination. `Cpu` without the `cpu-renderer` feature must
  surface the existing error path, not panic.

## P4 — pointer cursor icon

`WindowRequest::SetCursor(CursorIcon)` with a public, winit-free
`pub enum CursorIcon { Default, Text, Pointer, ResizeHorizontal, ResizeVertical }`.
Terminals show the I-beam over the grid. Unit-test the mapping fn.

## P5 — window bounds for persistence

`PlatformEvent::Moved { window, x, y }` (logical outer position) on `WindowEvent::Moved`,
and include the logical outer size in `Resized` consumers can already compute; document
which is client vs outer. Tests for the conversion helpers.

## P6 — continue the roadmap

Then pick bounded open items from `docs/status/todos.md` (prefer ROADMAP 015 rounded-quad
clip masks, then P3 mobile components), one at a time, each with tests and a report.
Never claim manual IME/GPU/screen-reader checks; mark them NOT_RUN.
