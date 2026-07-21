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
- **80+ high-level components** — via `acme-ui` (shadcn/ui + Material UI + Ant Design inspired)

### Architecture

```text
App → WidgetNode DSL → Retained Tree → Taffy Layout → Scene → wgpu → OS Surface
```

| Layer | Crate | Role |
|-------|-------|------|
| UI Components | `acme-ui` | 80+ high-level widgets (Slider, Switch, DatePicker, Toast, Dock …) |
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
| `apps/acme-gallery` | `acme-ui-gallery` | V2 component showcase — 80+ high-level `acme-ui` components |
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

---

## Project Status

| Area | Status |
|------|--------|
| Core framework | ✅ **Stable** — tree, layout, render, text, theme, animation, accessibility, widgets |
| Text input + IME | ✅ **Stable** — 100 tests, caret geometry, Traditional Chinese preedit/commit |
| Data components | 🧪 **Experimental** — Tree, Table, DataGrid, VirtualList with live Gallery demos |
| UI component library | 🧪 **Experimental** — 80+ components (acme-ui), showcased in acme-ui-gallery |
| GPU device-loss recovery | 🧪 **Wired** — pure-test state machine + `on_gpu_recovered` hook; **manual validation pending** |
| Traditional Chinese 注音 IME | 🧪 **Architecture done** — **manual validation pending** |
| Screenshot golden tests | 📋 **Scaffolded** — not yet in CI |
| CI benchmarks | 📋 **Not yet** — no performance thresholds |

> **Full status:** [`STATUS.md`](STATUS.md) · **Manual checklists:** [`docs/MANUAL_VALIDATION.md`](docs/MANUAL_VALIDATION.md)

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
│   ├── acme-ui/            # 80+ high-level components
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
├── ARCHITECTURE.md         # Detailed architecture
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

> **Full design system:** [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md)

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
| [`spec.md`](spec.md) | Project specification and gates |
| [`plan.md`](plan.md) | Phase plan (P0–P10) |
| [`todos.md`](todos.md) | Task tracking with completion status |
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | Crate hierarchy and data flow |
| [`STATUS.md`](STATUS.md) | Component maturity (stable/experimental/architecture-only) |
| [`AGENTS.md`](AGENTS.md) | Agent workflow rules |
| [`docs/adr/`](docs/adr/) | Architecture Decision Records |

---

## License

MIT OR Apache-2.0
