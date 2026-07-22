# AcmeUI Native

**Rust-native declarative desktop UI runtime — powered by wgpu.**

> **🌐 Languages:** [English](README.md) · [繁體中文](README-zhtw.md)

---

## What is AcmeUI Native?

AcmeUI Native is a **Rust-native desktop UI framework** that renders via **wgpu** (DirectX 12 / Vulkan / Metal). It is the independent runtime successor path for AcmeUIKit — **without GPUI dependency**.

It combines:

- **Declarative widget trees** — build UIs with a builder DSL (column, row, button, label…)
- **Flexbox layout** — via Taffy
- **GPU-accelerated rendering** — batched quads, rounded rects, borders, clips, textures
- **CJK + emoji text** — cosmic-text shaping with glyph atlas
- **Traditional Chinese IME** — preedit/commit, caret geometry, candidate window support
- **Semantic design tokens** — light/dark/high-contrast themes
- **AccessKit accessibility** — screen reader support
- **Animation engine** — tween/ease/bounce/yoyo/loop
- **165 high-level components** — via `acme-ui` (shadcn/ui + Material UI + Ant Design inspired)

### Default Template (acme-ui 0.2.0)

New applications can start from the built-in semantic template and prelude:

```rust
use acme_ui::prelude::*;

let view = default_template("Dashboard")
    .subtitle("A calm, consistent starting point")
    .child(card::<AppMessage>().build())
    .build();
```

For an Apple-inspired shell with quieter spacing and a more restrained
hierarchy, use `apple_template("Dashboard")` from the same prelude.

Platform presets are also available: `windows11_template("Dashboard")` and
`ubuntu25_template("Dashboard")`.

The template provides a stable root key, semantic light theme defaults, and a
token-driven surface for composing foundations, inputs, layout, and overlay
components. See [`DEFAULT_TEMPLATE.md`](docs/architecture/DEFAULT_TEMPLATE.md).

### Architecture

```text
App → WidgetNode DSL → Retained Tree → Taffy Layout → Scene → wgpu → OS Surface
```

| Layer | Crate | Role |
|-------|-------|------|
| UI Components | `acme-ui` | 165 high-level widgets (Slider, Switch, DatePicker, Toast, Dock, Masonry, FileUpload, VideoPlayer, Heatmap, MobileCard …) |
| Widget Primitives | `acme-widgets` | WidgetNode enum, builder DSL, overlay manager, visual states |
| Text Editing | `acme-textinput` | Cursor, selection, clipboard, IME preedit/commit, undo/redo (✓ 100 tests) |
| Layout | `acme-layout` | Taffy-based flexbox layout engine |
| Text | `acme-text` | cosmic-text shaping, glyph atlas, CJK + emoji fallback |
| Theme | `acme-theme` | Semantic color tokens V1 + V2, light/dark/high-contrast |
| Animation | `acme-animation` | Tween engine, easing, yoyo, delay, loop |
| Rendering | `acme-render-wgpu` | GPU surface lifecycle, batched rect/path rendering, clip stack |
| Platform | `acme-platform` | winit event loop, Application trait, WindowId, IME, GPU recovery hook |
| Accessibility | `acme-accessibility` | AccessKit bridge, focus management, action routing |
| Core | `acme-core` | Tree, geometry, events, scene model — platform-independent |
| DevTools | `acme-devtools` | Widget inspector, layout debugger, frame metrics, surface status |

### Gallery Apps

| App | Package | Purpose |
|-----|---------|---------|
| `apps/gallery` | `acme-gallery` | Primary demo — 8-category navigation, live data/nav demos (Tree, Table, DataGrid, VirtualList), screenshot mode |
| `apps/acme-gallery` | `acme-ui-gallery` | V2 component showcase — 165 high-level `acme-ui` components |
| `apps/playground` | `playground` | Minimal dev sandbox for quick experiments |
| `apps/benchmark` | `benchmark` | Headless layout/reconciliation/frame-build benchmarks |

---

## Quick Start

### Prerequisites

- **Rust** 1.85+ (MSRV, edition 2024)
- **Windows 10/11** (primary target; Linux/macOS secondary)
- A GPU with **DirectX 12** or **Vulkan** support

### Run the Gallery

```powershell
cargo run -p acme-gallery
```

The Gallery demonstrates: rectangles, rounded borders, clipped scrolling, cosmic-text rendering (English, Traditional Chinese, emoji), light/dark theme toggle, IME input, and interactive Tree/Table/DataGrid/VirtualList demos.

### Run the V2 Showcase

```powershell
cargo run -p acme-ui-gallery
```

### Validate

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Using the Component Library

`acme-ui` is a typed, declarative component layer. A component is built in
three stages: choose a builder, configure it with chainable methods, then add
the resulting `WidgetNode<M>` to a parent container. The application owns the
message enum and state; components emit messages through `on_click` or other
event methods.

### 1. Add the crate and select features

```toml
[dependencies]
acme-ui = { path = "crates/acme-ui", features = ["foundations", "inputs", "layout", "overlay"] }
```

The four feature families above are enabled by default. Add `desktop`,
`charts`, `mobile`, or `browser` only when needed. This keeps compile time and
binary size predictable.

### 2. Compose a screen

```rust
use acme_ui::prelude::*;

#[derive(Clone, Debug, PartialEq)]
enum AppMessage {
    Save,
    Cancel,
}

fn settings_view() -> WidgetNode<AppMessage> {
    let actions = row::<AppMessage>()
        .gap(8.0)
        .child(button("cancel", "Cancel").on_click(AppMessage::Cancel))
        .child(button("save", "Save").primary().on_click(AppMessage::Save))
        .build();

    default_template::<AppMessage>("Settings")
        .subtitle("Keep your workspace focused")
        .child(card::<AppMessage>().child(label("Preferences")).build())
        .child(actions)
        .build()
}
```

`column`, `row`, and `stack` are layout primitives. `card`, `label`, `button`,
and `scroll_view` are common foundation primitives. Every child is a node, so
the same composition model works for dialogs, forms, navigation, and pages.

### 3. Handle messages in the application

The builder does not mutate application state. Route emitted messages in your
application event/update layer:

```rust
match message {
    AppMessage::Save => state.save(),
    AppMessage::Cancel => state.close_settings(),
}
```

This keeps rendering deterministic: derive a new widget tree from state, let
the retained tree reconcile identity by keys, and render the resulting scene.
Use explicit keys for dynamic lists and interactive controls.

### 4. Choose a starter template

```rust
let view = default_template::<AppMessage>("Dashboard");
let apple = apple_template::<AppMessage>("Dashboard");
let windows = windows11_template::<AppMessage>("Dashboard");
let ubuntu = ubuntu25_template::<AppMessage>("Dashboard");
```

All templates expose `.subtitle(...)`, `.child(...)`, and `.build()`. They
provide spacing and hierarchy only; colors remain semantic theme tokens. The
stable root keys are `acmeui-default-template`, `acmeui-apple-template`,
`acmeui-windows11-template`, and `acmeui-ubuntu25-template`.

### 5. Use themes and tokens

```rust
let theme = default_theme();
let dark = Theme::dark();
assert!(theme.validate().is_ok());
```

Pass the active theme to the renderer/component resolution layer. Do not put
literal color values inside application widgets; use `ThemeColor` and the
semantic fields on `Theme` so light, dark, and high-contrast modes remain
consistent.

### 6. Find components and examples

- Foundations: labels, cards, badges, alerts, progress, calendar, lists.
- Inputs: buttons, text input, slider, select, date/time picker, checkbox,
  autocomplete, transfer, rating.
- Layout: forms, sections, toolbars, tabs, split panels, settings pages.
- Overlay: modal, drawer, dropdown menu, tooltip, toast, command palette.
- Optional families: desktop chrome, charts, mobile surfaces, browser media.

Run `cargo run -p acme-ui-gallery` for the broad component showcase and
`cargo run -p acme-gallery` for the runtime, input, accessibility, and data
component demonstrations.

---

## Project Status

| Area | Status |
|------|--------|
| Core framework | ✅ **Stable** — tree, layout, render, text, theme, animation, accessibility, widgets |
| Text input + IME | ✅ **Stable** — 100 tests, caret geometry, Traditional Chinese preedit/commit |
| Data components | 🧪 **Experimental** — Tree, Table, DataGrid, VirtualList with live Gallery demos |
| UI component library | 🧪 **Experimental** — 165 components (acme-ui), showcased in acme-ui-gallery |
| GPU device-loss recovery | 🧪 **Wired** — pure-test state machine + `on_gpu_recovered` hook; **manual validation pending** |
| Traditional Chinese 注音 IME | 🧪 **Architecture done** — **manual validation pending** |
| Screenshot golden tests | 📋 **Scaffolded** — not yet in CI |
| CI benchmarks | 📋 **Not yet** — no performance thresholds |

> **Full status:** [`STATUS.md`](docs/status/STATUS.md) · **Manual checklists:** [`MANUAL_VALIDATION.md`](docs/guides/MANUAL_VALIDATION.md)

---

## Repository Structure

```
AcmeUI-Native/
├── crates/
│   ├── acme-core/          # Tree, geometry, events, scene
│   ├── acme-platform/      # winit loop, Application trait, IME
│   ├── acme-render-wgpu/   # GPU surface + batched renderer
│   ├── acme-layout/        # Taffy layout wrapper
│   ├── acme-text/          # cosmic-text shaping + glyph atlas
│   ├── acme-textinput/     # Text editing state machine
│   ├── acme-theme/         # Design tokens (light/dark/high-contrast)
│   ├── acme-animation/     # Tween engine
│   ├── acme-style/         # Styling abstraction layer
│   ├── acme-widgets/       # WidgetNode enum + builder DSL
│   ├── acme-ui/            # 165 high-level components
│   ├── acme-accessibility/ # AccessKit bridge
│   └── acme-devtools/      # Inspector, metrics, debugger
├── apps/
│   ├── gallery/            # Primary demo app
│   ├── acme-gallery/       # V2 component showcase
│   ├── playground/         # Dev sandbox
│   └── benchmark/          # Headless benchmarks
├── docs/                   # Architecture, manual validation, ADRs
├── scripts/                # CI / dev scripts
├── spec.md                 # Project specification
├── plan.md                 # Development plan
├── todos.md                # Task tracking
├── docs/architecture/      # Architecture docs
└── AGENTS.md               # Agent workflow rules
```

---

## Design Principles

- **Semantic-first**: All colors, spacing, typography via design tokens — no hardcoded values
- **Foreground/background pairs**: Every surface token has a matching text token
- **Desktop-optimized**: Denser than web defaults, more breathing room than classic Win32
- **WCAG AA**: Minimum contrast on all text/background pairs; visible focus rings
- **Functional pipeline**: Event handling layered as `hit → activate → dispatch → match` (no monolithic `&mut self`)
- **GPU-friendly**: Batch everything; minimize state changes; persistent buffers with epoch-based invalidation

> **Full design system:** [`DESIGN_SYSTEM.md`](docs/architecture/DESIGN_SYSTEM.md)

---

---

## Functional & Layered Architecture — Conversion Extent

The `apps/gallery` (primary demo) has been fully converted from a 2593-line monolith to a layered, functional architecture:

### Event Pipeline (Layer 1 → 4)

| Layer | File | Role | Functions | Methods on `Gallery` |
|-------|------|------|-----------|---------------------|
| **Layer 4** | `main.rs` (event match) | Pure match-to-dispatch | 0 | `event()` (3 lines per arm) |
| **Layer 3** | `events/dispatch.rs` | Per-event-type handlers | **10 pub fn** | 0 |
| **Layer 2** | `events/activate.rs` + `events/ime.rs` | State transition | **2 pub fn** | 0 |
| **Layer 1** | `events/hit.rs` | Pure query | **1 pub fn** | 0 |

**All 13 event handlers are free functions** — zero `impl Gallery` methods, zero `&mut self`.

### Render Pipeline (Layer 1 → 4)

| Layer | Module | Role | Functions |
|-------|--------|------|-----------|
| **Layer 4** | `render/frame.rs` | Pipeline orchestration | **8 pub fn** (build_theme, render_sidebar, …) |
| **Layer 3** | `render/content.rs` + `render/hit_test.rs` | Widget rendering | **6 pub fn** |
| **Layer 2** | `render/style.rs` + `render/text.rs` | Style & text helpers | **3 pub fn** |
| **Layer 1** | `render/geometry.rs` + `render/layout.rs` | Primitives | **6 pub fn** |

**All 23 render functions are free functions** — zero `impl Gallery` methods, zero `&mut self`.

### Page Builders

| File | Category | Functions added to `Gallery` |
|------|----------|------------------------------|
| `pages/component.rs` | Foundations + page dispatcher | 4 |
| `pages/inputs.rs` | Inputs | 2 |
| `pages/navigation.rs` | Navigation | 5 |
| `pages/overlay.rs` | Overlay | 1 |
| `pages/data.rs` | **Data** (Tree, Table, DataGrid, VirtualList) | 6 |
| `pages/patterns.rs` | Patterns | 5 |
| `pages/accessibility.rs` | Accessibility | 1 |
| `pages/stress.rs` | Stress tests | 1 |

### main.rs Reduction

| Metric | Before | After |
|--------|--------|-------|
| Total lines | 2,593 | **538** (-79%) |
| `impl Gallery` methods | ~25 | 2 (new, window_config) + 3 (Application trait) |
| Free functions | 0 | **40+** across events/, render/, pages/, helpers/ |
| Event handlers | Inline in `event()` | **13 free functions** in `events/dispatch.rs` |
| Render steps | Inline in `frame()` | **23 free functions** in `render/` (8 files) |
| Gallery page builders | Inline | **9 files** in `pages/` |

### Data Components (acme-widgets)

| Component | Lines | Builder API | State Management | Tests |
|-----------|-------|-------------|-----------------|-------|
| `data/tree.rs` | 563 | ✅ TreeNode, Tree | ✅ Expand/collapse, selection, typeahead | 12 |
| `data/table.rs` | 824 | ✅ TableColumn, TableRow | ✅ Sort, select, resize, keyboard nav | 24 |
| `data/datagrid.rs` | 663 | ✅ DataGridColumn, DataGridRow | ✅ Frozen cells, merge, bidirectional virtual | 14 |
| `data/virtual_list.rs` | 562 | ✅ Item height, overscan | ✅ Visible range, anchor, height cache | 15 |
| **Total** | **2,612** | — | — | **65 tests** |

### UI Component Library (acme-ui)

| Module | Count | Maturity | Feature Gate | Default |
|--------|-------|----------|-------------|---------|
| `foundations/` | 42 | S1 | `foundations` | ✅ |
| `inputs/` | 29 | S2 | `inputs` | ✅ |
| `layout/` | 15 | S1 | `layout` | ✅ |
| `overlay/` | 10 | S1 | `overlay` | ✅ |
| `desktop/` | 13 | S1 | `desktop` | — |
| `charts/` | 20 | S0 (experimental) | `charts` | — |
| `mobile/` | 20 | S1 (experimental) | `mobile` | — |
| `browser/` | 16 | S0 (experimental) | `browser` | — |
| **Total** | **165 component files** | — | 8 feature gates | 4 default |

> Component maturity levels: **S0** Scaffold, **S1** Visual, **S2** Interactive, **S3** Accessible, **S4** Production.  
> Interactive features (pointer, keyboard, message dispatch) are listed per module.  
> Accessibility (S3+), visual regression goldens, and benchmarks (S4) are not yet implemented for any component.

Each component follows the **builder pattern**: `Component::new() → .option(value) → .on_event(message) → .build() → WidgetNode<M>`.

---

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| **No GPUI** | Independent runtime path, no Zed dependency |
| **wgpu** | Cross-platform GPU abstraction (D3D12/Vulkan/Metal) |
| **Taffy** | Battle-tested flexbox layout in pure Rust |
| **cosmic-text** | Mature shaping + glyph caching with CJK support |
| **Message-driven** | Widgets communicate via `Message<M>` enums — no callbacks |
| **Layered events** | `hit → activate → dispatch → match` enables field-level borrow checking |
| **Persistent buffers** | Re-upload only when `gpu_epoch` increments |
| **No `cargo clean`** | Never used as routine fix |

---

## For AI Agents

**This project uses a Controlled Workflow.** See [`AGENTS.md`](AGENTS.md) for:

- Scope gates and boundary contracts
- Task classification (T0–T3)
- Verification-first workflow
- Subagent handoff rules
- Persistent memory via `.opencode/memory/`

Key files for agent context:

| File | Purpose |
|------|---------|
| [`spec.md`](docs/specs/spec.md) | Project specification and gates |
| [`plan.md`](docs/specs/plan.md) | Phase plan (P0–P10) |
| [`todos.md`](docs/status/todos.md) | Task tracking with completion status |
| [`ARCHITECTURE.md`](docs/architecture/ARCHITECTURE.md) | Crate hierarchy and data flow |
| [`STATUS.md`](docs/status/STATUS.md) | Component maturity (stable/experimental/architecture-only) |
| [`AGENTS.md`](AGENTS.md) | Agent workflow rules |
| [`docs/adr/`](docs/adr/) | Architecture Decision Records |

---

## License

MIT OR Apache-2.0
