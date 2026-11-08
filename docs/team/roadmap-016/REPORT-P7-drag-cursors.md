# ROADMAP 016 — Track P7 Report: Drag Cursors and Pointer Capture

- **Owner:** agy
- **Track:** P7 (`BRIEF-P7-drag-cursors.md`)
- **Branch:** `sdk/roadmap-016-terminal-host`

---

## 1. Files Changed

| Path | Summary |
|---|---|
| `crates/acme-platform/src/waker.rs` | Extended `CursorIcon` with `Grab`, `Grabbing`, `Move`, `NotAllowed` (preserving `Default` as `#[default]`); mapped each variant in `map_cursor_icon`; extended `map_cursor_icon_maps_every_variant` unit test |
| `crates/acme-platform/src/lib.rs` | Added `PlatformEvent::PointerLeft { window }`; added `pointer_position_to_logical` (unclamped logical coordinate conversion for captured drags outside client bounds) and `cursor_left_events` (emitting both `PointerLeft` and `CursorLeft`); added unit test `pointer_capture_coordinates_and_pointer_left_events_are_preserved` |

---

## 2. Design & Pointer Capture Verification

1. **Drag Cursor Icons (`CursorIcon`):**
   - Added `Grab`, `Grabbing`, `Move`, and `NotAllowed` to `CursorIcon`, mapped 1:1 to `winit::window::CursorIcon::{Grab, Grabbing, Move, NotAllowed}`.
   - `CursorIcon::Default` remains the `#[default]` variant, and no `winit` types are exposed in the public API.
2. **Windows Pointer Capture During Button Press:**
   - Verified in `winit 0.30.13` (`src/platform_impl/windows/event_loop.rs`):
     - On `WM_LBUTTONDOWN` (as well as `WM_RBUTTONDOWN`, `WM_MBUTTONDOWN`, `WM_XBUTTONDOWN`), `winit` increments `window_state.mouse.capture_count` and calls Win32 `SetCapture(window)`.
     - While captured, `WM_MOUSEMOVE` continues arriving even after the pointer leaves the client area; `winit` decodes signed 16-bit `lparam` coordinates (`get_x_lparam` / `get_y_lparam` as `i32 -> f64`) without clamping. When crossing the client boundary, `winit` emits `WindowEvent::CursorLeft` followed by `WindowEvent::CursorMoved` with the signed coordinates.
     - On `WM_LBUTTONUP` (and corresponding button-up events), `winit` decrements `capture_count`, calls `ReleaseCapture()` when it reaches zero, and delivers `WindowEvent::MouseInput { state: ElementState::Released, .. }` even when released outside the window.
   - In `crates/acme-platform/src/lib.rs`, `handle_pointer_moved` converts physical `(x, y)` via `pointer_position_to_logical` without clamping and updates `state.cursor`, so both `PlatformEvent::PointerMoved` and the subsequent outside `PlatformEvent::PointerButton { pressed: false, .. }` carry the exact logical coordinates.
   - Added `PlatformEvent::PointerLeft { window: WindowId }` and updated `handle_cursor_left` (via `cursor_left_events`) to emit `PlatformEvent::PointerLeft { window }` alongside `PlatformEvent::CursorLeft { window }` so consumers can cancel hover state without breaking existing `CursorLeft` listeners.

---

## 3. Automated Verification

| Command | Result |
|---|---|
| `cargo test -p acme-platform` | **PASS** — 49 passed, 0 failed, 0 ignored |
| `cargo clippy -p acme-platform --all-targets -- -D warnings` | **PASS** (exit 0, 0 warnings) |
| `rustfmt --edition 2024 --check crates/acme-platform/src/waker.rs crates/acme-platform/src/lib.rs` | **PASS** (exit 0) |

---

## 4. Manual / Runtime Checks

- **Real interactive mouse capture outside the window (`SetCapture` drag across/outside client bounds and outside button release):** `NOT_RUN` (manual interactive window check).
