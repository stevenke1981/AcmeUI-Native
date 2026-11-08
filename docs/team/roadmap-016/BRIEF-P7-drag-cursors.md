# BRIEF P7 — drag cursors and pointer capture (ROADMAP 016)

Owner: agy. Lead: Claude. Consumer: CrewDesk pane drag-to-move.

## Goal
CrewDesk drags a pane header onto another pane to swap them. The platform needs:

1. `CursorIcon` gains `Grab`, `Grabbing`, `Move`, `NotAllowed` (map to winit
   `Grab`, `Grabbing`, `Move`, `NotAllowed`). Keep `Default` as default; keep the enum
   platform-neutral (no winit types in public API).
2. Pointer capture while a button is held: when the left button is pressed inside the
   window and the pointer leaves the client area, `PointerMoved` must keep arriving
   (with coordinates possibly outside the window) and the matching `PointerButton`
   release must be delivered even if released outside. On Windows winit already
   captures during a press; verify, and if not, document the limitation instead of
   adding unsafe Win32 code. Add `PlatformEvent::PointerLeft { window }` if missing so a
   consumer can cancel hover state.

## Files you own
`crates/acme-platform/src/waker.rs`, `crates/acme-platform/src/lib.rs` (event loop only),
`crates/acme-platform/tests/*`, `docs/team/roadmap-016/REPORT-P7-drag-cursors.md`.

## Hard rules
No GPUI. No git add/commit/stash/checkout. No cargo clean. No local paths in docs.
Backward compatible: only add variants. Tests for the mapping and for any new event.

## Acceptance
`cargo test -p acme-platform`, `cargo clippy -p acme-platform --all-targets -- -D warnings`,
`rustfmt --edition 2024 --check` on changed files: all PASS. Report lists NOT_RUN items
(real mouse capture outside the window is manual).
