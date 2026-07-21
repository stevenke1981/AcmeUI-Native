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

- Surface/device recovery has a **deterministic pure state machine** and CPU-atlas
  invalidation contract, but real wgpu device-loss detection (uncaptured error) and
  manual GPU recovery on Windows are still pending.
- Traditional Chinese IME has architecture + caret geometry wiring (preedit/commit,
  `ime_caret_area`, Gallery field-relative candidate rect) but has **not** received
  manual 注音 validation (AGENTS.md forbids claiming otherwise).
- Manual interaction at physical 125/150/200% Windows display scaling remains to be
  performed; automated DPI conversion and glyph-scale tests do not replace it.
- Tree, Table, DataGrid are implemented with unit tests; Gallery Data pages are still
  placeholder component templates (widgets not yet demoed live).
- Multi-window: `WindowId` on events, runtime multi-config support, and non-empty
  `windows()` default exist; interactive multi-window smoke is still manual.

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

## Agent Improvement Pack v0.2 additions

### P0-03: Renderer Buffering & Batching
- **Persistent double-buffered buffers**: Quad/glyph buffers with 1.5× auto-grow, frame ring alternates between buffers each frame — eliminates per-frame re-allocation
- **Clip-based batching**: Quads grouped by clip rect → one scissored draw call per group; text runs by `(AtlasFormat, scissor)`
- **`RenderStats`**: quad_count, glyph_count, draw_calls, buffer_grows, bytes_uploaded, atlas_hit_rate — `summary()` display
- **Atlas upload dedup**: `HashSet<region>` prevents redundant uploads per frame

### P0-04: Unified Platform Event Model
- **9 additive `PlatformEvent` variants**: `ImePreeditDetailed`, `ImeCommitDetailed`, `ImeEnabled`, `ImeDisabled`, `PointerButtonDetailed`, `FocusChanged`, `CursorEntered`, `CursorLeft`, `FileDropped`
- **IME + WindowId**: `WindowId` on preedit/commit; `set_ime_cursor_area()` trait method
- **Pointer tracking**: `PointerButtonDetailed` with x/y/button/pointer; modifier tracking (shift/ctrl/alt/meta)

### P0-01: Unified Node Identity
- **`LayoutNode.id`**: `u64` → `NodeId`; `LayoutSnapshot` keyed by `NodeId`
- **`to_layout(id: NodeId)`**: Caller provides identity instead of traversal counter
- **`RuntimeNode<M>`**: Compiled node with id/widget/children; `compile()` for future pipeline
- **No magic numbers**: Gallery/Playground use `extract_gallery_ids()` structural walk

### P0-02: Intrinsic Text Layout
- **`measure_text()`**: Extracted into `acme_layout`, returns `ShapedText` with glyphs + bounds
- **`render_text_input()` cache**: Auto-caches `TextLayout` by rendered text string
- **Label `font_size` / `cached`**: `label_with_size()` builder

### P0-05: Accessibility Runtime Integration
- **`AccessibilityAdapter`**: Per-window bridge; `update()` rebuilds tree, `route_action()` maps actions to PlatformEvent
- **`AccessibilityAction`**: Focus, Click, SetValue, ScrollIntoView, Activate
- **Gallery integration**: Adapter in `new()`, called every frame

### P1-01: Theme V2
- **11 new types**: SurfaceTokens, TextTokens, BorderTokens, SemanticColor/Tokens, Typeface, TypographyScale, SpacingScale, RadiusScale, ControlDimensions/Sizes, Density, Elevation, ThemeV2
- **`pub v2: ThemeV2`** on Theme struct; `light_v2()`/`dark_v2()`/`high_contrast()`

### P2-01: TextInput Hardening
- **Private cursor/selection**: Accessors, debug asserts; 96 tests
- **Undo/redo**: Transaction stack (max 50); mouse click/drag/word selection
- **Extended keys**: Shift+arrows, Ctrl+arrows, Ctrl+Backspace/Delete, Shift+Home/End
- **Placeholder/readonly/invalid**: UI states; IME caret area; horizontal scrolling

### P1-02: Widget Visual States + Module Refactor
- **Module restructure**: `acme-widgets/src/` → foundations/ inputs/ navigation/ overlay/ data/
- **`VisualState` enum**: 8 states; `ButtonVariant`/`ButtonSize`; `CardVariant`; OverlayManager
- **14 new theme tokens**

### P1-03: Gallery Templates
- **8-category sidebar**: Foundations→Stress Tests; component page template (10 sections)
- **4 reference templates**: Settings, Dashboard, IDE Layout, SpeakType
- **Theme/density toggle**: Light/Dark, Compact/Comfortable, focus rings; screenshot config

### P2-02: Data Widgets Productionization
- **VirtualList**: Viewport culling, variable height cache, scroll anchoring
- **Tree**: Expand/collapse, Arrow/Home/End, typeahead, visible nodes
- **Table**: Column resize, sort, row/cell selection, viewport virtualization, sticky header
- **DataGrid**: Frozen rows/cols, cell merge, bidirectional virtualization
- **10 new theme tokens**: table_*, tree_*, datagrid_*

### Verification (v0.2)
- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace --all-targets`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: passed (319 unit tests, 0 failed across 11 crates).
- Full gallery navigation with 8 categories and 4 templates compiles.

## Hardening batch (post v0.2)

### Surface / device recovery truth
- **Pure state machine**: private `AcquireOutcome` + `resolve_surface_action(suspended, device_lost, acquire)` covers Skip/DeviceLost/Reconfigure/Rendered without GPU.
- **`gpu_epoch`**: increments after successful `on_device_lost()`; `complete_recovery_state` unit-tested GPU-free.
- **Blank-text fix**: `Application::on_gpu_recovered` default hook; Gallery/Playground call `atlas.clear()` so next frame re-uploads glyphs after empty GPU atlases are rebuilt.
- **Atlas contract test**: `atlas_clear_forces_reupload_after_recovery` proves cache-hit → clear → re-upload + generation bump.

### IME caret wiring (not manual 注音 validation)
- **`Application::ime_cursor_area() -> Option<[f32;4]>`**: app-authoritative; mouse is fallback only via `resolve_ime_cursor_area`.
- **`ime_caret_area`**: accounts for `scroll_offset`; Gallery composes field origin + padding + caret.
- **Tests**: caret x advances with text, scroll reduces x, height from line_height; platform prefers app rect over mouse.
- **Explicitly unvalidated**: Traditional Chinese 注音 candidate UI on real Windows IME.

### Verification (hardening)
- `cargo fmt --all -- --check`: passed.
- `cargo check --workspace --all-targets`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `cargo test --workspace`: passed (**331** unit tests, 0 failed; 1 ignored doctest).

## v0.2.2 additions

### Gallery Data + Navigation demos
- Data category: live Tree, Table (28 rows), DataGrid (frozen col + merge), VirtualList (250 items)
- Navigation category: NavRail / Sidebar / TabBar / Breadcrumb demos
- `acme-widgets::navigation` module fully implemented with tests

### Real device-lost detection
- wgpu 29 `set_device_lost_callback` + `on_uncaptured_error` → `Arc<AtomicBool>`
- Internal/OutOfMemory mark lost; Validation errors log only
- `#[ignore]` GPU smoke test for adapter/device + handler registration

### Verification (v0.2.2)
- fmt/check/clippy `-D warnings`: passed
- `cargo test --workspace`: ~341 unit tests passed; 2 ignored (GPU smoke + doctest)

## Remaining (next milestone)

Manual 注音 IME validation, manual DPI at 125/150/200%, glyph atlas eviction,
screenshot golden comparison, performance baseline thresholds, `cargo-deny` /
`cargo-audit`, WSL/macOS/CI matrix, interactive multi-window smoke.
