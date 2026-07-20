# AcmeUI Native

Rust-native desktop UI runtime powered by wgpu.

## Recommended repository name

`AcmeUI-Native`

Alternatives: `AcmeUI-Runtime`, `AcmeNativeUI`, `AcmeUI-Engine`.

## Positioning

AcmeUI Native is the independent runtime successor path for AcmeUIKit. It does not depend on GPUI.

```text
App API -> Widgets -> Retained Tree -> Taffy Layout -> Scene -> wgpu -> OS Surface
```

Initial platform priority: Windows 10/11, Linux, then macOS.

Read `spec.md`, `plan.md`, `todos.md`, `test.md`, `ARCHITECTURE.md`, and `AGENTS.md`.

## Run the Gallery

```powershell
cargo run -p acme-gallery
```

The Windows Gallery renders rectangles, borders, clipped scrolling content, and
cosmic-text glyphs (English, Traditional Chinese and emoji) through wgpu. Click
the first button to switch Light/Dark themes, use the mouse wheel over the sample
list, and use Tab/Shift+Tab plus Enter/Space to operate the buttons.

## Validate

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The current visual smoke evidence is `docs/gallery-smoke.png`. Traditional Chinese
IME, AccessKit integration and cross-platform support are not claimed in v0.1.
