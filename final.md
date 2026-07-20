# Final Delivery

## Delivered

- Windows winit application lifecycle and real wgpu Gallery window.
- Typed DPI geometry, keyed retained tree, reconciliation, dirty propagation,
  hit testing, capture-target-bubble events, focus traversal and IME event architecture.
- Taffy Row/Column/Stack/Scroll facade and layout snapshots.
- Semantic Light/Dark themes and declarative Label, Button, Card, Separator and
  ScrollView builders.
- cosmic-text shaping, system CJK/emoji fallback, CPU glyph atlas bookkeeping,
  R8/RGBA GPU atlas uploads and instanced text rendering.
- Batched rectangles, rounded corners, borders and DPI-aware scissor clips.
- CI plus a reproducible Gallery screenshot script and evidence image.

## Validation

- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace --all-targets`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: passed (35 unit tests, 0 failed).
- Gallery executable: built and stayed alive with a non-zero Windows window handle.
- Visual evidence: `docs/gallery-smoke.png` shows real Traditional Chinese and emoji glyphs.

## Performance

The renderer batches unclipped rectangles and instanced glyphs. No numeric frame-time
or 10k-node target is claimed yet; the current test strategy has no accepted numeric
threshold or controlled hardware baseline.

## Known limitations

- Deterministic surface-loss recreation is not yet complete; resize, zero-size
  suspension, outdated/suboptimal reconfiguration and validation diagnostics exist.
- Traditional Chinese IME has architecture-only support (preedit/commit/cancel events in
  PlatformEvent, TextInputState handles IME composition) but has not received manual 注音
  validation.
- Manual interaction at physical 125/150/200% Windows display scaling remains to be
  performed; automated DPI conversion and glyph-scale tests do not replace it.
- Tree, Table, DataGrid not yet implemented.
- Application::run() multi-window support is designed (WindowId on all events) but the
  concrete Runtime loop still drives a single window; the `windows()` default returns an
  empty iterator.

## Risks

- Renderer window acquisition currently uses a framework-owned `Any` boundary and an
  internal winit downcast. It keeps public APIs free of platform types but should become
  a dedicated private runtime bridge before additional platform backends are added.
- The Gallery layout IDs follow the current deterministic declarative traversal; the
  retained-tree-to-layout adapter should replace traversal IDs in the next milestone.

## Milestone 2 additions

- **acme-devtools**: FrameMetrics, WidgetTreeDump, LayoutInspector, RenderDiagnostics
  with rolling FPS, widget tree debug dump, layout hit testing, and frame diagnostics.
- **acme-accessibility**: AccessKit TreeUpdate builder mapping all widget types
  to AccessKit roles (Group, Label, Button, Splitter, ScrollView), bounds from layout
  snapshot, focus tracking, 17 tests.
- **apps/benchmark**: Layout benchmark (100/500/1000 nodes), reconciliation benchmark
  (5 orderings), frame build benchmark (1000+ quads, text runs).
- **apps/playground**: Interactive widget test app with theme toggle, 5 button variants,
  click counter, CJK/emoji rendering, keyboard navigation, scroll, 840-line implementation.
- Fixed `LayoutSnapshot.iter()` API exposure.
- 68 unit tests total across all crates, all passing.

## Milestone 3 additions

- **Clipboard**: `acme-platform/src/clipboard.rs` with arboard-based `Clipboard { get_text, set_text, is_available }`, thread-safe via `Mutex`, 2 tests.
- **Tooltip**: `WidgetNode::Tooltip(Tooltip<M>)` with label + child content, wraps child in layout, 3 tests.
- **Animation**: New `acme-animation` crate with `AnimationEngine`, `Tween<T>`, `Easing` (Linear, QuadIn, QuadOut, QuadInOut, SmoothStep, Bounce, Elastic), `AnimationUpdate { value, progress, done }`, 22 tests.
- **Multi-window**: `WindowId(pub u64)` added to all `PlatformEvent` variants, `Application::windows()` default method returning empty iterator, `HashMap<WinitWindowId, WindowState>` in Runtime, 7 tests.
- **TextInput**: New `acme-textinput` crate with `TextInputState` — grapheme-aware cursor movement, selection (shift+arrow), clipboard cut/copy/paste, IME preedit/commit/password masking, `render_text_input()` returning styled quads + cursor, 45 tests.
- **VirtualList**: `WidgetNode::VirtualList(VirtualList<M>)` with `visible_range()`, `content_height`, `Arc<dyn Fn(usize) -> WidgetNode<M>>` item builder, 7 tests.
- **Popover**: `WidgetNode::Popover(Popover<M>)` with anchor + content nodes, `PopoverPlacement::Bottom/Top/Left/Right`, tests.
- **Menu**: `WidgetNode::Menu(Menu<M>)` with `Vec<MenuItem<M>>` (label, disabled, separator, submenu), tests.
- **Dialog**: `WidgetNode::Dialog(Dialog<M>)` with title, content, modal flag, explicit width/height, tests.
- 174 unit tests total across all crates, all passing with zero warnings.

## Milestone 4 additions

- **Tree**: `WidgetNode::Tree(Tree<M>)` with `TreeNode<M>` (label, depth, disabled, expanded, submenu/message), column-layout with indentation, 5 tests.
- **Table**: `WidgetNode::Table(Table<M>)` with `TableColumn` (title, width), `rows: Vec<Vec<WidgetNode<M>>>`, header visibility toggle, 5 tests.
- **DataGrid**: `WidgetNode::DataGrid(DataGrid<M>)` with `DataGridColumn` (title, width, sortable), `DataGridRow<M>` (cells, selected), sort column and selected row tracking, 5 tests.
- **PlatformKey extended**: `ArrowLeft`, `ArrowRight`, `Backspace`, `Delete`, `Home`, `End` + `ctrl` and `text` fields on `PlatformEvent::Key`.
- **TextInput keyboard shortcuts**: `handle_key()` handles arrows (cursor navigation), Backspace/Delete, Home/End, Escape (blur). New `handle_keyboard_shortcut()` handles Ctrl+A (select all), Ctrl+C (copy), Ctrl+V (paste), Ctrl+X (cut). 60 textinput tests total (+15 new).
- **IME Gallery demo**: TextInput section with focus-on-click, IME preedit/commit handling, rendered via `render_text_input()` with theme tokens, committed text display.
- **Multi-window test**: `multiple_window_configs` test verifies `app.windows()` returns N configs with correct titles.
- **Surface/device recreation**: `SurfaceAction::DeviceLost`, `Renderer::on_device_lost()` recreates device/pipelines/atlases/bind-groups, `simulate_device_loss()` test method, `device_lost_action_is_distinct` test.
- **Devtools + Accessibility**: Tree/Table/DataGrid match arms in `node_kind()`, `key_string()`, `extra_info()`, and `walk_node()` (Tree → Role::Tree, Table → Role::Table, DataGrid → Role::Grid).
- 211 unit tests total across all crates, all passing with zero warnings.
- `cargo fmt --all`, `cargo check`, `cargo clippy -D warnings`, `cargo test` — all gates pass.

## Remaining (next milestone)

Deterministic surface/device recreation automated test, retained NodeId layout identity,
intrinsic text measurement for auto-sized Label/Stack, manual DPI validation at
125/150/200% Windows display scaling.
