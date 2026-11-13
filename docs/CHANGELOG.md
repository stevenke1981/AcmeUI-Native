# Changelog

## Unreleased - Close-request veto hook (2026-11-13)

- `Application::close_requested(&mut self, WindowId) -> bool` (default `true`) is
  consulted after `PlatformEvent::WindowCloseRequested` is delivered. Returning
  `false` keeps the window open (title-bar close, Alt+F4, taskbar close) so an
  application can confirm unsaved changes and close later with
  `WindowRequest::Close`. Existing applications are unaffected.

## Unreleased - Gallery theme-pack previews (ROADMAP 039, 2026-11-11)

- Design System now shows accent, surface, text and background swatches for
  every registered theme pack. The existing toolbar light/dark toggle updates
  all previews. Colors come directly from each pack; application theme cycling
  and the SDK API are unchanged.

## Unreleased - DataList density (ROADMAP 038, 2026-11-11)

- `data_list().density(DataListDensity::Compact | Default | Relaxed)` controls
  item and label/value spacing in both orientations. Default preserves the
  original geometry; typography, content and keys are unchanged. Gallery's
  Data page compares all six variants. See ADR 0004 for exact spacing values.

## Unreleased - Minimal UI feature dependencies (ROADMAP 037, 2026-11-11)

- `acme-ui` now declares `inputs → foundations` and `layout → inputs`.
  Selecting either `inputs` or `layout` with `default-features = false` now
  brings in its required icon/Form-validation modules and compiles. The default
  and full feature sets are unchanged. CI tests both minimal selections using
  existing suites and external consumer regressions.

## Unreleased - Paper square corners (ROADMAP 036, 2026-11-11)

- `PaperBuilder::square(bool)` forces zero corner radius independently of the
  existing elevation. Default false retains the previous radius, padding,
  children and outlined/elevated variant. External `PaperBuilder` struct
  literals must add `square: false` (or use `paper()` and setters); see ADR 0003.

## Unreleased - Full sizing percentage correction (ROADMAP 035, 2026-11-11)

- `Styled::{w_full,h_full}`, Tailwind `{w,h}::{full,screen}` and
  `Button.full_width` now use `Length::Percent(1.0)` (100% of the containing
  content box). Previously `100.0` made a 240 px root 24000 px wide. Existing
  explicit width/height overrides still apply; the layout engine's fractional
  unit is unchanged. Callers compensating for the old overflow should remove
  those workarounds. `screen()` remains a parent-relative alias for `full()`;
  no viewport-relative length unit is introduced.

## Unreleased - Multi-page glyph atlas (ROADMAP 011, 2026-11-07)

- `acme-core`: `ATLAS_PAGE_EDGE` (2048) and `ATLAS_MAX_PAGES` (8);
  `GlyphDraw::atlas_page`; `GlyphFormat` derives `Hash`. **Breaking** for
  `GlyphDraw` struct literals (MIGRATION M14).
- `acme-text`: `GlyphAtlasConfig`, `GlyphAtlas::with_config` and `Default`
  (2048² × 4 pages), `page_count`, `max_pages`, `page_evictions`;
  `AtlasUpload::page` and `PreparedGlyph::atlas_page` (**breaking** for
  literals). A full page opens the next; `begin_frame` evicts at most one
  least-recently-used stale page instead of clearing everything. Mid-frame
  eviction stays deferred. `GlyphAtlas::new` stays single-page; page edges are
  clamped to 2048; placement no longer moves the cursor after a failed glyph.
  Whitespace is no longer counted in `atlas_full_glyphs`.
- `acme-render-cpu`: per-page, lazily allocated atlases for pages `< 8`;
  whole-scene preflight unchanged; goldens unchanged.
- `acme-render-wgpu`: lazy 2048² page textures per format replace the two eager
  4096² atlases; uploads deduplicated by (page, format, rect) — the old key
  ignored page and format; text batches split per page in painter order.
  Real GPU drawing and device loss are NOT_RUN.
- Apps and the SDK consumer use `GlyphAtlas::default()`; new end-to-end test
  (`acme-textinput/tests/multipage_e2e.rs`) and benchmark case
  `text/atlas_pressure_multipage`.

## Unreleased - Accessibility walk mirrors layout IDs (ROADMAP 010, 2026-11-07)

- `acme-accessibility`: the sequential and bound walks consume IDs in exactly
  layout's order (tree rows, table header leaves and header row, cell leaves,
  row containers), so TreeItem, ColumnHeader, Row and cell nodes read their own
  layout rects, and widgets after a Tree/Table/DataGrid no longer read another
  node's rect. Sequential-path IDs equal layout IDs for lowered nodes; nodes
  layout does not lower (cell content, Dialog content, VirtualList children,
  Tooltip/Popover wrappers) take virtual IDs >= 2^56 and have no rect. Ragged
  rows group by their own cells; `column_count` covers the widest row; grids
  whose rows changed after `build()` keep the flat fallback.

## Unreleased - Structural accessibility and exact-area CPU coverage (ROADMAP 009, 2026-11-07)

- `acme-accessibility`: `Tree` exposes one virtual `TreeItem` per visible item
  (level, expanded, selected, position/size among visible siblings, bounds from
  the tree rect). `Table`/`DataGrid` cells are grouped into `Row` nodes under a
  header `Row` of `ColumnHeader`s; the table carries row/column counts (the
  header row counts as row 0), rows and cells carry indices, row/header bounds
  come from the table rect, and unselected rows/cells state `selected=false`
  when the selection mode can select them. Malformed grids keep the 008 flat
  layout. Retained-path virtual nodes use IDs from `2^56`, disjoint from
  retained (< `2^48`) and layout synthetic IDs. Sequential-path IDs of widgets
  after a Tree/Table/DataGrid shift by the virtual nodes inserted.
- `acme-render-cpu`: quad coverage is pixel area (analytic for sharp edges and
  corners, 8x8 supersample for pixels crossed by a rounded-corner arc); border
  coverage is outer minus inner area. `rounded_bordered.ppm` golden re-blessed
  (188 px, max channel diff 12). Clip edges remain pixel-centre snapped.

## Unreleased - Interaction and accessibility follow-ups (ROADMAP 008, 2026-11-07)

- `acme-accessibility`: a `TextInput` with `validation_message` is now
  `Invalid::True` and exposes the message as its AccessKit description
  (the `invalid` flag alone still marks it invalid, without description).
  Table and DataGrid cells get `Role::Cell` when their content is a label or
  container; controls inside cells keep their own role and actions. Tree items
  are unchanged: `Tree` exposes no per-item widget yet (see ROADMAP 008 report).
- Gallery: a pointer press no longer survives a page switch or a hit-list
  change, so a release cannot activate a different widget; hover is
  re-resolved from the pointer each frame.
- `examples/sdk-consumer/tests/migration_snippets.rs` compiles the MIGRATION.md
  after-form snippets (11 tests, M4/M10 have no library API). Fixed the M5
  snippet (undefined `snapshot`) and the M2 parsing table (`from_env_value`
  returns `Err`; only `from_env` warns and falls back to `Auto`).
- CI `CPU renderer evidence`: release `acme-benchmark --quick` smoke run,
  `benchmark.json` stored in the artifact. No thresholds.

## Unreleased - Packaging, licenses, migration (ROADMAP 007, 2026-11-07)

- Added `LICENSE-MIT` and `LICENSE-APACHE` (the workspace already declared
  `MIT OR Apache-2.0`) and `docs/guides/LICENSES.md`. `deny.toml` gains a
  `[licenses]` allow list of the 9 licenses actually used; CI runs
  `cargo deny check bans licenses sources`.
- Internal dependencies use `[workspace.dependencies]` with `path` + `version`,
  so every library crate packages; apps are `publish = false`. CI runs
  `scripts/check_package.py` (per-crate `cargo package --no-verify`).
- New `docs/guides/MIGRATION.md` (0.2.0 to the next release) and
  `docs/guides/RELEASE.md` (checklist, publish order, version drift; proposes
  0.3.0 — not applied).
- Correction: the v0.2.0 entry below says `route_action` returns `Option`; the
  code changed to `Vec` the next day (`229ca22`). Not listed before: `RenderBatch.clip`
  changed from `Option<[u32; 4]>` to `Option<[f32; 4]>`, and `Application` gained
  default methods `background_tick` / `background_poll_interval` (see MIGRATION.md).

## Unreleased - Audit reconciliation (PR #13, 2026-11-07)

- Added a font-independent allocator regression for frame-boundary overflow
  eviction, and multi-frame retained accessibility identity/focus/bounds
  coverage across insertion, reorder and removal. Retained binding already
  existed; the legacy sequential accessibility APIs are unchanged.
- Reconciled STATUS/todos, Rust 1.88 MSRV and the English/Traditional Chinese
  source inventory (199 category modules, not 199 fully interactive widgets).
- No atlas behavior change: `evict_stale` and `clear` remain immediate explicit
  operations, as defined by ROADMAP 006.

## Unreleased - Benchmarks and atlas lifecycle (ROADMAP 006, 2026-11-07)

- **Behaviour change (`acme-text` `GlyphAtlas`):** automatic eviction no longer
  clears the atlas in the middle of a frame once any glyph was used in it; that
  overwrote texels of glyphs already handed out, so glyphs in one
  `PreparedText` could overlap. A glyph that does not fit mid-frame is now
  dropped for that frame (`atlas_full_glyphs`), and `begin_frame` evicts
  entries idle for more than 60 frames after an overflow or above 75 % fill.
  `atlas_generation` therefore bumps at frame start, possibly earlier than
  before. `clear()` also resets a pending eviction request.
- `apps/benchmark` is headless (no window, no GPU device): warm-up plus sampled
  min/median/p95/mean, `--quick`, `--json PATH` (schema v1 with git SHA, rustc,
  OS/arch, profile, CPU), retained-tree and CPU-render cases. The `--window`
  path was removed. Methodology: `docs/guides/BENCHMARKS.md`; no FPS targets or
  CI thresholds.
- New atlas lifecycle tests (`crates/acme-text/tests/atlas_lifecycle.rs`):
  600-frame Latin/CJK/emoji rotation on a 256x256 atlas, explicit clear,
  hot-set accounting, deferred-eviction regression. Known limitation (ignored
  test): a glyph is dropped while every entry was used within 60 frames.

## Unreleased - Retained IDs in remaining apps (2026-11-07)

- V2 Showcase and Playground frame layouts now consume retained identities,
  with complete sequential fallback when reconciliation or binding fails.
- Showcase sidebar keys are unique and category pages have separate identities.
- Playground preserves its legacy layout styles/measurement and fixes obsolete
  child indices that made its real frame panic on a leaf node.
- Added real-builder/frame, stable-ID, layout/hit parity and fallback regressions.
  Native visual, physical-DPI, IME and screen-reader acceptance remains separate.

## Unreleased - Focus, semantics, DPI/IME automation (ROADMAP 004-005, 2026-11-07)

- Gallery keyboard focus is keyed by stable identity (retained node + message).
  Fixes an Enter/Space index-out-of-bounds panic after switching to a page with
  fewer controls, and focus drift on page/tree/table changes. Accessibility
  focus follows keyboard focus; `AccessibilityAdapter::clear_widget_focus` added.
- Accessibility semantics: TextInput name/value/`PasswordInput` (password text
  never exposed), disabled/read-only/invalid, `SetValue` only when editable;
  Buttons expose `Click` only when enabled; TabBar children are `Tab` with
  selected state; selected NavRail item; modal dialogs.
- Text input: `move_cursor_to_offset` snaps to a grapheme boundary (a mid-char
  offset made caret geometry panic); committed text clears the preedit; Escape
  cancels an active composition before blurring; new `scroll_caret_into_view`.
- Gallery TextInput: the field was laid out one line (21 px) tall, clipping text
  to a ~5 px band; it now reserves the theme control height. Overflowing text
  scrolls so the caret and IME candidate rect stay inside the field.
- Automated DPI evidence at 1.0/1.25/1.5/1.75/2.0: Gallery logical layout and IME
  caret drift measured 0.0 px on every page; CPU renderer exact physical-pixel
  mapping; mixed 注音/ASCII caret geometry. **Manual 注音 IME and Windows DPI
  sessions remain NOT validated.**

## Unreleased - Retained IDs in the app path (ROADMAP 003, 2026-11-07)

- `acme-widgets`: `retained_binding` (`bind_retained`, `RetainedBinding`,
  `SyntheticIds`, `SYNTHETIC_ID_BASE`) and `WidgetNode::to_layout_with_binding`.
  Layout node IDs come from the `RetainedTree` and survive reorder/insert/remove;
  synthetic rows/cells use IDs from `1 << 48`. `to_layout_with_context` output is
  byte-identical to before (single shared layout body).
- `acme-accessibility`: `update_with_binding` / `build_tree_update_with_binding`
  give AccessKit nodes the same stable IDs and read bounds from the bound layout.
  `update` is unchanged.
- Gallery: the retained tree is no longer a shadow. Sidebar category buttons had
  duplicate empty keys, so every reconcile failed and the retained tree was never
  populated; they are now keyed `category:{i}`. Frames bind retained IDs for
  layout and accessibility and fall back to sequential IDs (logged once) on a
  reconcile error or partial binding.
- Evidence: 8 Gallery tests over every page (zero binding mismatches, identical
  computed rects to the sequential path, stable IDs across frames and category
  switches, accessibility bounds equal layout rects).

## Unreleased - SDK production hardening (batch-002, 2026-11-07)

- Merge the CPU software renderer with GPU fallback (PR #6) and focus-navigation
  hardening (PR #5).
- `acme-platform`: add `RenderBackendPreference` (`ACME_RENDER_BACKEND` =
  `auto` | `gpu` | `cpu`, trimmed and case-insensitive). Invalid values now log a
  warning and fall back to `auto`; previously a value such as ` cpu` was ignored.
- **Breaking (0.x):** `PlatformError` gains `NoWindows { reasons }` and
  `Renderer(String)`. `run()` now returns `Err` when no window can be created or
  the loop stops on an unrecoverable renderer failure, instead of `Ok(())`. A
  recorded renderer error takes precedence over an event-loop error. Normal
  close of the last window still returns `Ok(())`.
- `acme-render-cpu`: deterministic, text-free golden pixel regression tests
  (`tests/golden.rs`, six 64x64 PPM fixtures, tolerance 2 per channel; regenerate
  with `ACME_BLESS_GOLDEN=1`).
- Packaging: crate descriptions, repository, rust-version, keywords and
  categories; new standalone `examples/sdk-consumer` built in CI against the
  workspace lockfile; new `docs/guides/SDK_GUIDE.md`.
- `acme-ui`: remove all 21 test-only dead-code warnings by exercising each
  component's message wiring (14 new tests); inline format arguments flagged by
  Clippy in touched files.
- Not verified: native windows, injected GPU device loss, 注音 IME, DPI
  125/150/200%, screen readers. See `docs/team/sdk-hardening/REPORT.md`.

## Unreleased - Core invalidation and text editing

- Invalidate a parent's structure, layout, paint and semantics after child insertion,
  removal or reorder, preserving keyed node IDs.
- Propagate layout invalidation to ancestors after reconciliation changes a node's
  kind, focusability, disabled state or children. Unchanged frames remain clean.
- Add regression coverage and a headless `reconcile_dirty` example.
- Preserve text-input readonly state and history during undo/redo; record selected
  Ctrl+Backspace/Delete as undoable edits and clear obsolete redo branches.
- Wire Gallery Ctrl+Z, Ctrl+Y and Ctrl+Shift+Z; ignore key-release shortcuts.
- Windows GNU / Rust 1.94.1 workspace check/build/tests and formatting passed;
  Clippy has no changed-file warnings. Pinned Rust 1.88, cross-platform and
  Gallery visual gates remain pending. See
  [the validation report](reports/retained-tree-invalidation.md).

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
