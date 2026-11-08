# ROADMAP 016 — Track P5 Report: Window Bounds for Persistence

## Changed Files

- `crates/acme-platform/src/lib.rs`

## Public API Diff (`acme-platform`)

```rust
pub enum PlatformEvent {
    /// Window client (inner) area resized. `logical_width` and `logical_height`
    /// are logical client dimensions matching [`WindowConfig::width`] and
    /// [`WindowConfig::height`].
    Resized {
        window: WindowId,
        logical_width: f32,
        logical_height: f32,
        scale_factor: f32,
    },
    /// Window outer top-left position moved. `x` and `y` are logical outer
    /// coordinates matching [`WindowConfig::position`].
    Moved {
        window: WindowId,
        x: f32,
        y: f32,
    },
    // ...
}
```

## Client vs. Outer Bounds Documentation

- **Client (inner) dimensions**: `WindowConfig::width`, `WindowConfig::height`, `WindowConfig::min_width`, `WindowConfig::min_height`, and `PlatformEvent::Resized { logical_width, logical_height, .. }` are all **logical client (inner) dimensions**, passed to `winit::window::WindowAttributes::with_inner_size` / `with_min_inner_size` and derived from `WindowEvent::Resized(PhysicalSize<u32>)`.
- **Outer position**: `WindowConfig::position` and `PlatformEvent::Moved { window, x, y }` are **logical outer top-left coordinates**, passed to `winit::window::WindowAttributes::with_position` and derived from `WindowEvent::Moved(PhysicalPosition<i32>)`.
- Added pure conversion helpers `physical_size_to_logical` and `physical_position_to_logical` with fallback guards for non-positive/NaN scale factors, plus unit tests in `tests::physical_bounds_conversions_scale_and_handle_invalid_scale_factors`.

## Verification & Test Counts

| Command / Check | Status | Result |
| --- | --- | --- |
| `cargo test -p acme-platform` | PASS | 48 passed, 0 failed (unit tests); 0 failed, 1 ignored (doc-test) |
| `cargo clippy -p acme-platform --all-targets -- -D warnings` | PASS | 0 warnings, 0 errors |
| `cargo check --workspace --all-targets` | PASS | All workspace crates and app targets compiled cleanly |
| `rustfmt --edition 2024 --check <changed files>` | PASS | 0 formatting diffs |
