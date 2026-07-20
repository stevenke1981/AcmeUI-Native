# AcmeUI Native — STATUS

## Stable

| Component | Since | Notes |
|-----------|-------|-------|
| `acme-core` (tree, geometry, event, scene) | M1 | Keyed reorder, focus/capture, clip, event order |
| `acme-layout` (Taffy wrapper) | M1 | NodeId-based lookup, intrinsic text measure |
| `acme-text` (swash/rustybuzz shaping) | M1 | Atlas dedup, CJK/emoji; `clear()` bumps generation for recovery |
| `acme-animation` (tween engine) | M1 | Linear/ease/bounce/yoyo, delay, loop, events |
| `acme-theme` (color tokens) | M1 | V1 + V2 tokens, light/dark/high-contrast |
| `acme-platform` (event model) | M1 | Event variants, WindowId, IME caret query, GPU recovery hook |
| `acme-render-wgpu` (WGPU backend) | M3 | Persistent buffers, clip batching, pure surface-action state machine, `gpu_epoch` |
| `acme-textinput` | M4 | 100 tests; cursor, selection, undo/redo, IME caret geometry, extended keys, readonly |
| `acme-widgets` (WidgetNode + submodules) | M4 | Visual states, button variants/sizes, card variants, overlay mgr |
| `acme-accessibility` (adapter + tree) | P0-05 | AccessKit bridge, action routing, focus management |
| `acme-devtools` (inspector, metrics) | M3 | WidgetTreeDump, LayoutInspector, FrameMetrics, SurfaceStatus |

## Experimental

| Component | Notes |
|-----------|-------|
| `acme-widgets::data::Tree` | Typeahead, Arrow/Home/End, expand/collapse, visible node culling |
| `acme-widgets::data::Table` | Column resize, sort, selection, viewport virtualization, sticky header |
| `acme-widgets::data::DataGrid` | Frozen cells, merged cells, bidirectional virtualization |
| `acme-widgets::data::VirtualList` | Visible range, variable height cache, scroll anchoring |
| `acme-gallery` | 8-category navigation; IME caret wired; Data demos still placeholders |

## Architecture Only

| Component | Notes |
|-----------|-------|
| `acme-widgets::navigation` | Module structure reserved; widgets not yet implemented |
| Glyph atlas eviction/aging | Fixed 2048² shelf; full → drop glyphs until `clear()` |
| Real wgpu device-loss detection | `device_lost` flag + recovery path exist; uncaptured-error hook not wired |
| Screenshot golden pipeline | `ScreenshotConfig` scaffolded; no capture/diff CI |

## Automated Only (manual still pending)

| Component | Automated evidence | Manual still needed |
|-----------|--------------------|---------------------|
| Surface status machine | `resolve_surface_action` covers suspended/device-lost/acquire outcomes | Real GPU device loss on Windows |
| Post-recovery text integrity | `atlas_clear_forces_reupload`; Gallery/Playground `on_gpu_recovered` clears CPU atlas | Trigger real device loss and confirm glyphs |
| IME caret geometry | `ime_caret_area` + `resolve_ime_cursor_area` + Gallery field-relative rect | Traditional Chinese 注音 candidate placement |
| CJK shaping | `acme-text` shapes TC + emoji without panic | Visual glyph quality at 125/150/200% DPI |
| Multi-window routing | Unit tests for WindowId / configs | Interactive multi-window smoke |
| Theme V2 | Constructors + validation test | Visual Light/Dark/High Contrast pass |

## Not Validated

- UI pixel test / screenshot golden file comparison
- Performance baseline thresholds (clean build, warm incremental, frame prep)
- `cargo-deny` / `cargo-audit`
- WSL / macOS / CI matrix beyond local Windows
- MSRV Rust 1.85 check
- Manual Traditional Chinese 注音 IME
