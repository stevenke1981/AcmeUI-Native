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
