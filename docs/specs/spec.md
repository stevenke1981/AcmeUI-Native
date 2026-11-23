# Specification

## Goal
Build a Rust-native desktop UI framework using wgpu, winit, Taffy, cosmic-text and AccessKit.

## MVP
- Window creation and resize
- wgpu surface/device/queue lifecycle
- DPI-aware coordinates
- Rectangle, rounded rectangle, border, clip and text rendering
- Retained node tree and reconciliation
- Row, Column, Stack and Scroll layout
- Pointer hit testing and event propagation
- Focus and keyboard traversal
- Label, Button, Card, Separator, ScrollView
- Light/Dark theme
- Gallery app

## P0
- Windows 10/11
- 100/125/150/200% DPI
- Surface-loss recovery
- CJK fallback
- Traditional Chinese IME architecture before TextInput
- No GPUI dependency

## Layout and text

Use the same `FontSystem` default family for intrinsic measurement and painting.
After changing `FontSystem::set_default_family`, recompute layout and any cached
shaped runs.

Taffy rounds layout to whole logical pixels for all font families. Automatic
text leaf dimensions round shaped measurements up before layout. The ceiling
uses the measured value directly so even a width just above an integer gets
enough space for a painter that compares the full advance to the layout width.
Explicit sizes and flex constraints can still limit the space for text.

## Non-goals for v0.1
- Rich text editor
- Full CSS
- Mobile/Web targets
- Complete DataGrid

## Gates
```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p acme-gallery
```
