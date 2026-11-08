# ROADMAP 016 — Track P4 Report: Pointer Cursor Icon

## Changed Files

- `crates/acme-platform/src/waker.rs`
- `crates/acme-platform/src/lib.rs`

## Public API Diff (`acme-platform`)

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CursorIcon {
    #[default]
    Default,
    Text,
    Pointer,
    ResizeHorizontal,
    ResizeVertical,
}

pub enum WindowRequest {
    SetTitle(String),
    Focus,
    Minimize,
    Close,
    SetImeAllowed(bool),
    SetCursor(CursorIcon),
}
```

## Summary of Changes

1. **Winit-free `CursorIcon` Enum**:
   - Added `pub enum CursorIcon { Default, Text, Pointer, ResizeHorizontal, ResizeVertical }` in `crates/acme-platform/src/waker.rs` and re-exported it from `crates/acme-platform/src/lib.rs`.
2. **`WindowRequest::SetCursor(CursorIcon)` & Mapping**:
   - Added `map_cursor_icon(CursorIcon) -> winit::window::CursorIcon` (`Default` -> `Default`, `Text` -> `Text`, `Pointer` -> `Pointer`, `ResizeHorizontal` -> `EwResize`, `ResizeVertical` -> `NsResize`) and unit-tested all 5 variants in `waker::tests::map_cursor_icon_maps_every_variant`.
   - Wired `WindowRequest::SetCursor(icon)` in `process_window_requests` via `state.window.set_cursor(map_cursor_icon(icon))`.

## Verification & Test Counts

| Command / Check | Status | Result |
| --- | --- | --- |
| `cargo test -p acme-platform` | PASS | 47 passed, 0 failed (unit tests); 0 failed, 1 ignored (doc-test) |
| `cargo clippy -p acme-platform --all-targets -- -D warnings` | PASS | 0 warnings, 0 errors |
| `cargo check --workspace --all-targets` | PASS | All workspace crates and app targets compiled cleanly |
| `rustfmt --edition 2024 --check <changed files>` | PASS | 0 formatting diffs |
