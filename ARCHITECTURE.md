# Architecture

```text
Application
  -> declarative view descriptions
  -> reconciliation
  -> retained node tree
  -> Taffy layout
  -> hit-test + accessibility + scene
  -> paint batching
  -> wgpu render passes
  -> OS surface
```

## Crate Hierarchy

```text
acme-core          Platform-independent data model (tree, geometry, event, scene)
  ├── acme-layout    Taffy-based flexbox layout engine
  ├── acme-text      cosmic-text shaping + glyph atlas
  ├── acme-theme     Semantic color tokens (V1 + V2, light/dark/high-contrast)
  ├── acme-animation Tween engine (linear/ease/bounce/yoyo, delay, loop)
  └── acme-widgets   Declarative WidgetNode enum + builder DSL
       └── acme-ui   High-level component library (80+ components)
            ├── foundations/  Badge, Alert, Tag, Avatar, Progress, etc.
            ├── inputs/       Slider, Switch, Radio, DatePicker, etc.
            ├── layout/       Tabs, Toolbar, Grid, Pagination, etc.
            ├── overlay/      Toast, Drawer, ConfirmDialog, etc.
            ├── desktop/      TitleBar, Dock, SideNav, Menubar, etc.
            ├── mobile/       BottomNav, BottomSheet, PullToRefresh
            ├── browser/      Carousel, Lightbox, ZoomView
            └── charts/       LineChart, PieChart, Sparkline, AreaChart

acme-platform      winit event loop + Application trait
acme-render-wgpu   GPU surface lifecycle + batched rect renderer
acme-textinput     Text editing state machine (cursor, selection, undo/redo, IME)
acme-accessibility AccessKit bridge + action routing
acme-devtools      Widget inspector, layout debugger, frame metrics
```

### acme-widgets vs acme-ui

- **acme-widgets** — Low-level widget primitives: `WidgetNode<M>` enum, builder DSL,
  visual states, overlay manager. Each variant maps 1:1 to a layout container.
- **acme-ui** — High-level components composed from `acme-widgets` primitives.
  Each component is a builder that converts into `WidgetNode<M>` via `From` impl.
  Inspired by shadcn/ui, Material UI, and Ant Design.

## Gallery Apps

| App | Package | Purpose |
|-----|---------|---------|
| `apps/gallery` | `acme-gallery` | Primary demo — 8-category navigation, live Data/Nav demos, screenshot mode. `default-members` target. |
| `apps/acme-gallery` | `acme-ui-gallery` | V2 component showcase — demonstrates `acme-ui` high-level components. |
| `apps/playground` | `playground` | Minimal dev sandbox for quick experiments. |
| `apps/benchmark` | `benchmark` | Headless layout/reconciliation/frame-build benchmarks. |

## Dirty flags
Style, layout, paint, semantics, children. Propagate only as far as required.

## Overlay layers
Main, Floating, Modal, Tooltip, Drag, Debug.

## Coordinates
Use typed PhysicalPixels, LogicalPixels, WindowSpace and LocalSpace. Avoid naked f32 across boundaries.
