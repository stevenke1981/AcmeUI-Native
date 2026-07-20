# AcmeUI Native — STATUS

## Stable

| Component | Since | Notes |
|-----------|-------|-------|
| `acme-core` (tree, geometry, event, scene) | M1 | Keyed reorder, focus/capture, clip, event order |
| `acme-layout` (Taffy wrapper) | M1 | NodeId-based lookup, intrinsic text measure |
| `acme-text` (swash/rustybuzz shaping) | M1 | Atlas dedup, CJK/emoji |
| `acme-animation` (tween engine) | M1 | Linear/ease/bounce/yoyo, delay, loop, events |
| `acme-theme` (color tokens) | M1 | V1 + V2 tokens, light/dark/high-contrast |
| `acme-platform` (event model) | M1 | 9 event variants, WindowId, IME, modifiers |
| `acme-render-wgpu` (WGPU backend) | M3 | Persistent buffers, frame ring, clip batching, RenderStats |
| `acme-textinput` | M4 | 96 tests; cursor, selection, undo/redo, IME, extended keys, readonly |
| `acme-widgets` (WidgetNode + submodules) | M4 | Visual states, button variants/sizes, card variants, overlay mgr |
| `acme-accessibility` (adapter + tree) | P0-05 | AccessKit bridge, action routing, focus management |
| `acme-devtools` (inspector, metrics) | M3 | WidgetTreeDump, LayoutInspector, FrameMetrics, SurfaceStatus |

## Experimental

| Component | Notes |
|-----------|-------|
| `acme-widgets::data::Tree` (keyboard nav + virtualization) | Typeahead, Arrow/Home/End, expand/collapse, visible node culling |
| `acme-widgets::data::Table` (column resize, sort, selection) | Viewport virtualization, sticky header |
| `acme-widgets::data::DataGrid` (frozen cells, merged cells) | Bidirectional virtualization |
| `acme-widgets::data::VirtualList` (visible range, variable height) | Item anchor, scroll anchoring |
| `acme-gallery` (8-category navigation) | Foundation/Inputs/Overlay/Data/Patterns/Accessibility/Stress |

## Architecture Only

| Component | Notes |
|-----------|-------|
| `acme-widgets::navigation` | Module structure reserved, widgets not yet implemented |
| `apps/benchmark` | Scaffold only |
| `acme-render-wgpu` atlas resize | Growing implemented; eviction/aging not yet |
| `acme-textinput::ime_caret_area()` | Returns position; platform integration pending |

## Manually Validated

| Component | Validation |
|-----------|------------|
| CJK shaping (Traditional Chinese + emoji) | `acme-text` test passes; manual visual not yet run |
| Multi-window event routing | Unit tests pass; manual visual not yet run |
| Surface/device recreation | `acme-devtools` SurfaceStatus tracks state; manual trigger not yet tested |
| Light/Dark/High Contrast theme | Theme V2 constructors + validation test; visual not yet run |
| Gallery screenshot matrix | 8 resolutions, Light/Dark, Compact/Comfortable scaffolded |

## Not Validated

- UI pixel test / screenshot golden file comparison
- Performance baseline (clean build, warm incremental, frame preparation)
- `cargo-deny` / `cargo-audit` dependency auditing
- WSL / macOS / CI pipeline
- MSRV Rust 1.85 check
