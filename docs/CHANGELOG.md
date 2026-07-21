# Changelog

## v0.2.3 — Interactive Gallery demos + manual validation checklists (2026-07-20)

### Interactive Data / Nav (Gallery state on app, rebuild each frame)
- Tree: expand/collapse + selection (click, chevron, ←/→); state in `tree_expanded` bits
- Table: header sort (reorders row data), row select + highlight
- VirtualList: independent wheel scroll when pointer over list viewport
- NavRail / TabBar: click selection via `on_click` messages
- Hit testing respects page scroll (`HitRegion.scrolled` + clip rect)

### Manual validation docs (honest)
- Added `docs/guides/MANUAL_VALIDATION.md` for GPU device-loss and 注音 IME
- Both checklists status: **NOT YET MANUALLY VALIDATED**
- STATUS links to checklist; no false “manual done” claims

### Verification
- Workspace fmt/clippy/test green

## v0.2.2 — Gallery demos, navigation widgets, device-lost detection (2026-07-20)

### Gallery Data demos
- Real Tree / Table / DataGrid / VirtualList pages with in-category tabs
- Custom paint paths for Tree visible nodes, Table/DataGrid header+rows, VirtualList labels

### Navigation widgets
- `NavRail`, `Sidebar`, `TabBar`, `Breadcrumb` with builders, `to_layout`, a11y roles, devtools kinds
- Gallery Navigation category demos all four widgets

### Device-lost detection (wgpu 29)
- `device_lost_flag: Arc<AtomicBool>` shared with callbacks
- `set_device_lost_callback` marks lost; `on_uncaptured_error` marks lost on Internal/OutOfMemory (Validation logged only)
- Handlers re-registered after successful `on_device_lost`
- Tests: flag→DeviceLost mapping; `#[ignore]` adapter/device smoke

### Misc
- Fixed flaky `frame_metrics_begin_end_frame_records_timing` (1ms sleep)

### Verification
- Workspace fmt/check/clippy `-D warnings` clean; ~341 unit tests pass (2 ignored)

## v0.2.1 — Recovery + IME caret hardening (2026-07-20)

### Surface / device recovery truth
- Extracted pure `resolve_surface_action` state machine (suspended / device_lost / acquire outcomes) — GPU-free unit tests cover all transitions
- Added `Renderer::gpu_epoch()`; increments after successful device recovery
- Added `Application::on_gpu_recovered(window)` hook; Gallery and Playground clear CPU `GlyphAtlas` to prevent blank text after GPU atlas rebuild
- Added `atlas_clear_forces_reupload_after_recovery` contract test in `acme-text`
- Replaced checkbox-only surface recovery claim with behavioral tests (real hardware device loss still manual)

### IME caret geometry wiring
- Added `Application::ime_cursor_area() -> Option<[f32; 4]>` (app-authoritative)
- Added `resolve_ime_cursor_area(app_rect, mouse)` — mouse is fallback only
- `TextInputState::ime_caret_area` now subtracts `scroll_offset`
- Gallery returns field-relative caret rect (origin + padding + caret)
- **Does not claim** Traditional Chinese 注音 manual validation

### Docs truthfulness
- Corrected `todos.md` IME and surface-recovery checkboxes
- Renamed misleading "Manually Validated" STATUS section to "Automated Only (manual still pending)"
- Removed stale final.md claims (Tree/Table unimplemented, empty `windows()` iterator)

### Verification
- 331 unit tests pass workspace-wide; fmt/check/clippy `-D warnings` clean

## v0.2.0 — Agent Improvement Pack v0.2 (2026-07-20)

### P0-03: Renderer Buffering & Batching
- **Persistent quad/glyph buffers**: Double-buffered with 1.5× auto-grow, frame ring alternates between buffers each frame, avoids per-frame re-allocation
- **Clip-based batching**: Quads grouped by clip rect → one scissored draw call per clip group; text runs grouped by `(AtlasFormat, scissor)`
- **`RenderStats` struct**: Tracks `quad_count`, `glyph_count`, `draw_calls`, `buffer_grows`, `bytes_uploaded`, `atlas_hit_rate`; `summary()` display method
- **Atlas upload dedup**: Per-frame `HashSet<region>` tracks already-uploaded atlas regions
- **Upgraded logging**: `eprintln!` → `tracing::warn!`

### P0-04: Unified Platform Event Model
- **9 additive `PlatformEvent` variants**: `ImePreeditDetailed`, `ImeCommitDetailed`, `ImeEnabled`, `ImeDisabled`, `PointerButtonDetailed`, `FocusChanged`, `CursorEntered`, `CursorLeft`, `FileDropped`
- **IME + WindowId**: `WindowId` field on ImePreeditDetailed/ImeCommitDetailed; `Application::set_ime_cursor_area()` trait method
- **Pointer tracking**: `PointerButtonDetailed` with `x`, `y`, `button`, `pointer` fields; shift/control/alt/meta modifier tracking on all pointer events
- **Right/middle mouse**: Right button → `button: 2`, middle → `button: 1`

### P0-01: Unified Node Identity
- **`LayoutNode.id`** changed from `u64` to `NodeId` (public `NodeId::new(u64)` constructor, `Hash`/`Eq`/`Ord`)
- **`LayoutSnapshot.rects/scroll`**: `HashMap<u64, _>` → `HashMap<NodeId, _>`; `get(&self, NodeId)` and `scroll_metrics(&self, NodeId)` accept the new type
- **`WidgetNode::to_layout()`**: Signature changed from `(&mut u64)` to `(NodeId)` — caller provides identity
- **`RuntimeNode<M>`**: Added with `id`, `widget`, `children` fields; `WidgetNode::compile(&mut u64)` for future retained-tree pipeline
- **Gallery/Playground magic numbers eliminated**: Both apps compute layout IDs structurally via `extract_gallery_ids()` / `extract_playground_ids()` — no more `snapshot.get(7)`

### P0-02: Intrinsic Text Layout
- **`measure_text()`**: Extracted text shaping into reusable `acme_layout::measure_text(text, fonts, font_size, scale) -> ShapedText`
- **`PositionedGlyph` / `ShapedText`**: Cache-friendly types containing `glyphs`, `width`, `height`, `text`, `font_size`
- **`render_text_input()` cache**: Stores `TextLayout` keyed by rendered text; auto-invalidates on text change
- **Label widget**: `font_size` and `cached: Option<ShapedText>` fields; `label_with_size()` builder

### P0-05: Accessibility Runtime Integration
- **`AccessibilityAdapter`**: Per-window bridge; `update(root, snapshot)` rebuilds tree, `tree_ref()` current tree, `route_action(&AccessibilityAction) -> Option<PlatformEvent>`
- **`AccessibilityAction`**: `Focus`, `Click`, `SetValue`, `ScrollIntoView`, `Activate` — each maps to a `PlatformEvent`
- **Focus management**: `focus_widget(target_id)` tracks focused node; `build_tree_update()` produces AccessKit `TreeUpdate`
- **Gallery integration**: Adapter initialized in `new()`, called in `frame()`
- **27 tests** in acme-accessibility (9 new adapter tests)

### P1-01: Theme V2
- **11 new public types**: `SurfaceTokens`, `TextTokens`, `BorderTokens`, `SemanticColor`/`SemanticTokens`, `Typeface`/`TypographyScale`, `SpacingScale`, `RadiusScale`, `ControlDimensions`/`ControlSizes`, `Density`, `Elevation`, `ThemeV2`
- **Theme integration**: `pub v2: ThemeV2` on existing `Theme` struct; `light_v2()`, `dark_v2()`, `high_contrast()` constructors
- **Validation**: Test checks all tokens are present

### P2-01: TextInput Hardening
- **Data safety**: `cursor`/`selection` fields made private; accessors `cursor()`, `selection()`, `has_selection()`; debug assertions in all mutations
- **Undo/redo**: `EditTransaction` stack (max 50), `undo()`, `redo()`, `clear_history()`
- **Mouse selection**: `move_cursor_to_offset()`, `set_selection_range()`, `select_word_at_offset()` (double-click word select via `UnicodeSegmentation`)
- **Extended keyboard**: Shift+arrows (extend selection), Ctrl+arrows (word boundary), Ctrl+Backspace/Delete (word delete), Shift+Home/End
- **Placeholder/readonly/invalid**: Placeholder text in muted color; `readonly` blocks all mutations; `invalid` shows danger border
- **IME caret area**: `ime_caret_area(fonts, style, scale) -> [f32; 4]`
- **Horizontal scrolling**: `scroll_offset: f32`, text rendered at `content_x - scroll_offset`
- **96 tests** (+36 new)

### P1-02: Widget Visual States + Module Refactor
- **Module restructure**: `acme-widgets/src/` split into `foundations/`, `inputs/`, `navigation/`, `overlay/`, `data/`, `prelude.rs`, `visual_state.rs`, `overlay_manager.rs`
- **`VisualState` enum**: 8 states — Default, Hover, Pressed, FocusVisible, Selected, Disabled, Loading, Invalid
- **Button**: `ButtonVariant` (Primary/Secondary/Ghost/Danger), `ButtonSize` (XS/S/M/L), loading/full_width/leading_icon/trailing_icon; pressed uses separate theme token
- **Card**: `CardVariant` (Plain/Outlined/Elevated/Interactive/Muted); separate struct
- **Input**: Builder with label/description/placeholder/clearable/readonly/password/invalid/validation
- **OverlayManager**: Layer-based stacking (Main/Floating/Modal/Tooltip/Drag/Debug), push/raise/dismiss/top_of
- **14 new theme tokens**: button_*_bg, button_hover/pressed, card_*, input_*, disabled_bg
- **118 widget tests**

### P1-03: Gallery Templates
- **8-category sidebar**: Foundations, Inputs, Navigation, Overlay, Data, Patterns, Accessibility, Stress Tests
- **Component page template**: Anatomy, Variants, Sizes, States, Density, Light/Dark, Keyboard, Accessibility, Traditional Chinese
- **4 reference templates**: Settings (sidebar+content), Dashboard (KPI+insight), IDE Layout (rail+sidebar+editor), SpeakType (recording UI)
- **Theme/density toggle**: Light/Dark toggle, Compact/Comfortable, focus ring visibility
- **Screenshot mode**: `ScreenshotConfig` with configurable width/height/theme/density
- **No magic numbers**: All layout IDs from `extract_gallery_ids()` structural walk

### P2-02: Data Widgets Productionization
- **VirtualList**: `visible_range()` viewport query, `VariableHeightCache`, scroll anchoring via `save_anchor()`/`restore_anchor()`
- **Tree**: Expand/collapse keyed by `WidgetKey`, Arrow/Home/End navigation, typeahead, `visible_nodes()` viewport
- **Table**: Column resize (clamped to min/max), sort toggle with direction, row/cell selection modes, viewport row virtualization, sticky header, `TableState::Normal/Empty/Loading/Error`
- **DataGrid**: Frozen rows/cols, cell merge (colspan/rowspan), bidirectional viewport virtualization, row numbers
- **10 new theme tokens**: `table_header_bg`, `table_row_*_bg`, `table_sticky_header_shadow`, `tree_indent_guide`, `tree_expand_chevron`, `datagrid_gridline`, `datagrid_frozen_shadow`

## v0.1.0 — Milestone 4 (2026-07-19)
- Tree, Table, DataGrid widgets (struct + tests)
- PlatformKey extended: ArrowLeft/Right, Backspace, Delete, Home, End
- TextInput keyboard shortcuts: arrow navigation, shortcuts (Ctrl+A/C/V/X)
- IME Gallery demo
- Multi-window test
- Surface/device recreation
- Devtools + Accessibility: match arms for Tree/Table/DataGrid
- 211 unit tests total

## v0.1.0-rc3 — Milestone 3 (2026-07-18)
- Clipboard (arboard), Tooltip, Animation (tween engine), Multi-window (WindowId)
- TextInput (cursor, selection, clipboard, IME), VirtualList, Popover, Menu, Dialog
- 174 unit tests total

## v0.1.0-rc2 — Milestone 2 (2026-07-17)
- acme-devtools (FrameMetrics, WidgetTreeDump, LayoutInspector)
- acme-accessibility (AccessKit TreeUpdate, focus, 17 tests)
- benchmark app, playground app
- 68 unit tests total

## v0.1.0-rc1 — Milestone 1 (2026-07-15)
- Windows winit application lifecycle + wgpu Gallery window
- Typed DPI geometry, keyed retained tree, reconciliation, hit testing
- Taffy Row/Column/Stack/Scroll layout
- Semantic Light/Dark themes, Label/Button/Card/Separator/ScrollView
- cosmic-text shaping, CJK/emoji, glyph atlas, GPU text rendering
- Batched rectangles, rounded corners, borders, DPI-aware clips
- 35 unit tests, Gallery smoke screenshot
