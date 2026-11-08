# ROADMAP 016 — Track P8 Report: CPU Window Clipped After DPI Change

- **Owner:** agy
- **Track:** P8 (`BRIEF-P8-dpi-change.md`)
- **Branch:** `sdk/roadmap-016-terminal-host`

---

## 1. Files Changed

| Path | Summary |
|---|---|
| `crates/acme-platform/src/lib.rs` | Added `needs_resize(renderer_dims, renderer_scale, inner_size, scale_factor)`; tracked `renderer_dims` and `renderer_scale` on `WindowState`; updated `WindowEvent::ScaleFactorChanged` to pass the new scale factor into `handle_resize`, emit `PlatformEvent::Resized`, and keep window-request draining active; added pre-render self-healing check in `handle_redraw` so any size or scale drift resizes the renderer and emits `PlatformEvent::Resized` before `app.frame` and `render_scene`; added unit test `needs_resize_detects_dimension_or_scale_mismatch_and_ignores_zero_size` |
| `crates/acme-platform/src/cpu_window.rs` | Tracked `surface_dims` and `scale_factor` on `CpuWindowRenderer`; exposed `dimensions()` and `scale_factor()`; made `CpuWindowRenderer::present` self-heal `softbuffer::Surface::resize` if `surface_dims` or `buffer.len()` diverges from `CpuRenderer::dimensions()`; added `sync_and_render_raster` helper and unit test `scale_change_and_self_healing_resize_cover_full_cpu_buffer` |

---

## 2. Root Cause & Design

1. **Why the CPU window was clipped after a cross-DPI transition:**
   - In `winit 0.30.13` on Windows (`src/platform_impl/windows/event_loop.rs`), `WM_DPICHANGED` dispatches `WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer }` **before** calling `SetWindowPos` with the suggested rect. Inside `ScaleFactorChanged`, `window.inner_size()` therefore still returns the pre-`SetWindowPos` physical client size (`old_physical_inner_size`), while `InnerSizeWriter` is write-only (`request_inner_size`).
   - Previously, `WindowEvent::ScaleFactorChanged` called `handle_resize(app, windows, id, None)` which ignored the event's `scale_factor`, suppressed `PlatformEvent::Resized` (`None => true`), and set `should_drain_requests = false`.
   - When a window was created at 100% (`1280x770`) and moved to 125% (`1600x962`) or vice versa, any ordering where `WM_SIZE` arrived before `WM_DPICHANGED` or was coalesced left `CpuRenderer` and `softbuffer::Surface` at a stale physical size (`1043x747` or `1600x927`) while `handle_redraw` constructed `FrameContext` from the live `state.window.inner_size()`. Because `softbuffer` on Win32 blits only `0..buffer.width` × `0..buffer.height` without stretching, the right and bottom strips of the client area remained unpainted.
2. **Fix & Self-Healing Guarantees:**
   - `WindowEvent::ScaleFactorChanged { scale_factor, .. }` now forwards `Some(scale_factor as f32)` to `handle_resize`, resizes the renderer with the updated scale factor, and always emits `PlatformEvent::Resized { window, logical_width, logical_height, scale_factor }` to the application.
   - `handle_redraw` queries the renderer's active physical dimensions and scale factor (`state.renderer.dimensions(state.renderer_dims)` and `state.renderer.scale_factor(state.renderer_scale)`) and compares them against the live `state.window.inner_size()` and `state.window.scale_factor()` via `needs_resize(...)`. Whenever either physical dimensions or scale factor differ, `handle_redraw` resizes the renderer and emits `PlatformEvent::Resized` **before** calling `app.frame(...)` and `render_scene(...)`.
   - `CpuWindowRenderer` tracks both `raster.dimensions()` and `surface_dims`, and `present` verifies that `softbuffer::Surface` matches the raster dimensions and pixel buffer length before copying and presenting.

---

## 3. Automated Verification

| Command | Result |
|---|---|
| `cargo test -p acme-platform` | **PASS** — 51 passed, 0 failed, 1 doc-test ignored |
| `cargo test -p acme-platform --no-default-features` | **PASS** — 50 passed, 0 failed, 1 doc-test ignored |
| `cargo clippy -p acme-platform --all-targets -- -D warnings` | **PASS** (exit 0, 0 warnings) |
| `cargo clippy -p acme-platform --no-default-features --all-targets -- -D warnings` | **PASS** (exit 0, 0 warnings) |
| `rustfmt --edition 2024 --check crates/acme-platform/src/lib.rs crates/acme-platform/src/cpu_window.rs` | **PASS** (exit 0) |

---

## 4. Manual / Runtime Checks

- **Real two-monitor DPI transition validation (dragging CPU window between 100% and 125%+ monitors):** `NOT_RUN` (requires physical/interactive multi-DPI display setup).
