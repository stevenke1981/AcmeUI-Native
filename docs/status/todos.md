# Todos

## ROADMAP 036 — Paper square corners (2026-11-11)

- [x] Add independent square(bool), preserve default/elevation/outline behavior, cover all corner variants, real layout and retained IDs.
- [x] Add themed/clipped Gallery comparison and automated native CPU evidence; see [report](../team/roadmap-036/REPORT.md) for exact verification/delivery status and public-field compatibility.
- [ ] Human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader acceptance — NOT_RUN.

## ROADMAP 035 — full sizing percentage correction (2026-11-11)

- [x] Reproduce 24000 vs 240 px root width and padded child overflow; correct Styled/Tailwind full sizing and Button.full_width to the existing fractional Percent convention, preserving explicit overrides.
- [x] Add real-layout/theme/clip Gallery evidence and behavior-change notes; see [report](../team/roadmap-035/REPORT.md) for exact verification/delivery status.
- [ ] Human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader acceptance — NOT_RUN.

## ROADMAP 034 — SwipeActions acceptance (2026-11-11)

- [x] Add controlled mobile swipe_actions and backend-neutral horizontal threshold state, optional keyed Archive/Delete Buttons, layout/identity tests and a 21st public mobile default contract.
- [x] Add Gallery pointer reveal/close, separate action/remove/restore, cancellation, pointer matching and F7/Enter/Delete/Escape alternative; preserve composition guard and visible-row scroll position. See [report](../team/roadmap-034/REPORT.md).
- [ ] Continuous translation/animation, general gesture/capture integration and human Gallery, Windows 注音 IME, physical DPI/multitouch, GPU and screen-reader acceptance — NOT_RUN.

## ROADMAP 033 — Snackbar acceptance (2026-11-11)

- [x] Extend existing toast with keyed typed action/dismiss, wrapping Unicode content, bounded real layout and retained identities; preserve duration metadata and tone icons.
- [x] Add Gallery bottom overlay with restartable expiry, independent Undo/dismiss, stale-press cancellation and click-through protection; see [report](../team/roadmap-033/REPORT.md) for local/hosted evidence.
- [ ] Global timer/overlay management, live-region semantics and animation remain separate; human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader acceptance — NOT_RUN.

## ROADMAP 032 — floating action button acceptance (2026-11-11)

- [x] Resume interrupted FAB work without discarding changes; support mini/normal/extended sizes, themed elevated surfaces and one typed Button action.
- [x] Add component layout/retained tests and Gallery theme/scale/clip/pointer regressions; see [report](../team/roadmap-032/REPORT.md) for verification and delivery status.
- [ ] Viewport anchoring and expandable groups remain host/follow-up work; human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader acceptance — NOT_RUN.

## ROADMAP 031 — render preflight and skipped-frame reliability (2026-11-10)

- [x] Reproduce dropped one-shot glyph uploads on failed surface acquisition; upload before acquire and flush skipped-frame writes.
- [x] Retry Skip/Reconfigure at a bounded deadline without requiring unrelated input; normalize background intervals and reschedule changes.
- [x] Share clip/layer structural validation across CPU/GPU before resource mutation, preserving CPU allocation budgets.
- [x] Windows real GPU/surface regression with injected Timeout/Occluded/Outdated and exact atlas readback; 275 targeted tests and strict four-crate Clippy pass.
- [x] Gallery valid-scene evidence: 36 pages × two themes × two frames; see [batch report](../team/roadmap-031/REPORT.md).
- [ ] Hosted CI and exact merged-SHA release build: current status in batch report / PR; do not infer completion from the implementation checkboxes.
- [ ] Human IME/DPI/screen-reader validation NOT_RUN; desktop capture blocked by agent-desktop host DACL validation.

## ROADMAP 030 — mobile builder contract coverage (2026-11-10)

- [x] Add named default/layout-kind/widget-and-layout-child-count contracts for all 20 public mobile modules, with a registry-driven inventory guard (historical P3 “16” count corrected: 15 mobile_* + five legacy modules).
- [x] Verify all 15 P3 Gallery examples and stable bound roots across themes/scales; run locked touched/mobile tests, strict Clippy, actual CPU profile, workspace milestone gates and native CPU smoke; see [coverage/report](../team/roadmap-030/REPORT.md).
- [ ] Legacy composite interaction/theme/runtime gaps, generic Row focus/typed AccessKit actions, spinner animation and actual sheet dragging remain open.
- [ ] Human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader acceptance — NOT_RUN.

## ROADMAP 029 — mobile sheet handle acceptance (2026-11-10)

- [x] Render a themed 36×4 rounded indicator, preserve custom width and fixed finite geometry, add retained/layout/paint tests and centered Gallery preview.
- [x] Locked touched/mobile tests, strict Clippy, actual CPU profile, workspace milestone gates and native CPU capture; see [report](../team/roadmap-029/REPORT.md).
- [ ] Drag recognition is outside this visual indicator; human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader acceptance — NOT_RUN.

## ROADMAP 028 — mobile action acceptance (2026-11-10)

- [x] Honour destructive/disabled states, bounded wrapping labels and stable identity; add themed Gallery actions and pointer/paint regressions.
- [x] Locked UI/Gallery/mobile tests, strict Clippy, actual CPU profile, workspace milestone gates and native CPU captures; see [report](../team/roadmap-028/REPORT.md).
- [ ] Generic retained Row focus/typed AccessKit actions remain open; manual human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader acceptance — NOT_RUN.

## ROADMAP 027 — mobile list item acceptance (2026-11-10)

- [x] Add full-width themed rows, wrapping title/detail, fixed trailing icon and stable text keys; preserve existing public fields and Row on_tap messages.
- [x] Add Gallery pointer and explicit F6 keyboard host, optional-detail/reorder demos, real-layout/retained/theme/paint-index regressions and native CPU pointer/keyboard captures.
- [x] Fix Frame→Scene dropping TextRun clips, exposed by keyboard scrolling: Scene/Gallery RED/GREEN and native toolbar pixel comparison (762 changed pixels before, 0 after). Run locked touched/mobile tests, strict Clippy, actual CPU profile and workspace milestone gates; see [report](../team/roadmap-027/REPORT.md).
- [ ] Framework-wide retained focus and typed AccessKit actions for the Row composition — open; keyboard handling in this slice is Gallery-specific.
- [ ] Manual human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader acceptance — NOT_RUN.

## ROADMAP 026 — mobile notification acceptance (2026-11-10)

- [x] Add responsive wrapping title/body, stable content/footer keys and separate 44 px actions; preserve the stateless builder API and semantic elevated Card.
- [x] Honour existing Card layout Style in both paths with a RED/GREEN regression. Add Gallery action/dismiss/show flows and host-owned five-second expiry with cancellation/restart tests and native timed CPU captures.
- [x] Cancel stale pointer press/hover state on asynchronous card removal; run locked widgets/UI/Gallery and mobile-only tests, strict Clippy, actual CPU profile and workspace milestone gates; see [report](../team/roadmap-026/REPORT.md).
- [ ] Manual human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader acceptance — NOT_RUN.

## ROADMAP 025 — mobile banner acceptance (2026-11-10)

- [x] Add full-width semantic soft status surfaces, Unicode wrapping and separate instance-keyed 44 px action/dismiss buttons; preserve the builder API.
- [x] Cover optional/public fields, four tones, narrow/wide real layout, retained IDs and Gallery pointer action/dismiss/restore; capture native CPU windows.
- [x] Locked all-feature/mobile-only UI and Gallery tests, strict Clippy, actual CPU profile and workspace milestone gates; see [report](../team/roadmap-025/REPORT.md).
- [ ] Manual human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader acceptance — NOT_RUN.

## ROADMAP 024 — mobile avatar acceptance (2026-11-09)

- [x] Add circular themed avatars, 32/40/56 px sizes, centered Unicode initials and an empty-name placeholder without expanding the public API.
- [x] Honour existing Label sizing and single-line alignment; paint Stack surfaces and bound aligned text to its content box. Add retained/layout/paint tests and native CPU Gallery evidence.
- [x] Locked touched/mobile-only tests, strict Clippy, actual CPU verification and workspace milestone gates; see [report](../team/roadmap-024/REPORT.md).
- [ ] Manual human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader acceptance — NOT_RUN.

## ROADMAP 023 — mobile chip acceptance (2026-11-09)

- [x] Add rounded semantic chip surfaces and distinct child tap/dismiss actions with instance-derived keys; preserve the builder API.
- [x] Cover optional actions, selected/empty/Unicode tags, real layout and retained identity; add Gallery filter/removal/restore demos and automated native CPU pointer captures.
- [x] Locked targeted/mobile-only tests, strict Clippy, actual CPU verification and workspace milestone gates; see [report](../team/roadmap-023/REPORT.md).
- [ ] Manual human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader acceptance — NOT_RUN.

## ROADMAP 022 — mobile search acceptance (2026-11-09)

- [x] Replace the placeholder Label with a real controlled TextInput, instance-derived action keys, themed rounded surface, separate search/clear actions and retained/layout regressions.
- [x] Add a keyboard/pointer/Unicode editing Gallery demo, undoable clear, composition guards, scroll clips, native CPU interaction captures and targeted/workspace gates; see [report](../team/roadmap-022/REPORT.md).
- [x] Correct TextInput style application, render baselines and stale DPI/typography shaping cache; center Gallery content-button labels. Preserve RED/GREEN evidence.
- [x] Reconcile the historical toggle/stepper/segmented Clippy blocker after the exact all-feature strict gate and current behavior suites pass; no new implementation claimed.
- [ ] Manual human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader acceptance — NOT_RUN.

## ROADMAP 021 — mobile loader acceptance (2026-11-09)

- [x] Audit the existing foundation spinner + optional message composition; preserve API/defaults and verify retained IDs, message omission/changes and keyed instance reorder.
- [x] Wrap long loader messages at character boundaries; make Gallery honour Label wrap/width/line-height and preserve each prepared text line's baseline offset.
- [x] Add indicator-only, short-message and narrow long-message Gallery examples, automated native CPU capture, targeted tests/strict Clippy and workspace milestone checks; see [batch report](../team/roadmap-021/REPORT.md).
- [ ] Animate the foundation spinner / mobile loader through runtime scheduling and reduced-motion policy — static glyph only in this batch.
- [ ] Manual human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader acceptance — NOT_RUN.

## ROADMAP 020 — mobile determinate progress (2026-11-09)

- [x] Replace percentage-only output with a responsive themed track/fill, optional description and separate percentage text; normalize numeric edge cases.
- [x] Cover real layout, retained IDs, Gallery examples, stepper-driven updates, theme/clip/order and shaped-glyph positioning.
- [x] Correct Gallery content-label baseline alignment discovered in the automated native CPU capture; preserve RED/GREEN evidence.
- [ ] Manual human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader acceptance — NOT_RUN; see [batch report](../team/roadmap-020/REPORT.md).

## ROADMAP 019 — mobile card acceptance (2026-11-08)

- [x] Audit the existing card builder; preserve title/subtitle/media ordering and elevated/outlined semantic variants.
- [x] Preserve the card text subtree's retained IDs when optional media is inserted or removed; add combination and real-layout regressions.
- [x] Add three Gallery Mobile card examples and themed, clipped surface tests; preserve parent/child paint order for interactive media.
- [ ] Manual human Gallery review, physical DPI/touch and screen-reader acceptance — NOT_RUN; see [batch report](../team/roadmap-019/REPORT.md) for automated CPU-window evidence and checks.

## ROADMAP 015 — area-based clip edges (2026-11-08)

- [x] Clip edges contribute fractional pixel area; sharp quads use the exact rect∩clip area (no double attenuation on coincident edges).
- [x] Rounded quads (fill and border) use the exact analytic rounded-quad∩clip pixel area (`clipped_quad_coverage`, ROADMAP 016 P6); images and glyphs multiply texel alpha by the clip-area mask.
- [ ] GPU renderer parity for fractional clips — NOT_RUN.

## ROADMAP 014 — AccessKit requests, analytic arcs, mobile controls (2026-11-08)

- [x] `AccessibilityAction::from_request` / `AccessibilityAdapter::handle_request`; `from_access_id` round-trips virtual ids.
- [x] Exact pixel-area coverage for corner arcs (max error 6.7e-5 vs a 512x512 reference); golden `rounded_bordered` re-blessed with measured delta.
- [x] Interactive mobile toggle, stepper and segmented control with Gallery demos and hit tests.
- [ ] Wire a real AccessKit platform adapter (e.g. accesskit_winit) to `handle_request`.
- [ ] Quiet-host CPU benchmark: the single uncontrolled run showed +12.5% median at scale 1.0.
- [x] Whole-crate `clippy -p acme-ui --all-targets -D warnings` baseline cleared (35 diagnostics, 22 files).

## ROADMAP 013 — TreeItem actions, item content walk, textinput split (2026-11-08)

- [x] TreeItem Focus/Click on every row; Expand/Collapse on rows with children; additive `AccessibilityAction::Expand/Collapse` routed as row select + arrow key.
- [x] Non-plain tree-item content walked as the item's child (unlowered; layout-ID order unchanged).
- [x] `acme-textinput/src/lib.rs` (2829 lines) split into render/caret/input/boundary and four test files; public items unchanged.
- [x] AccessKit `ActionRequest` → `AccessibilityAction` mapping and `handle_request` (ROADMAP 014); a real platform adapter is still not wired.
- [x] TreeItem presses land at the row's trailing edge (ROADMAP 014).
- [ ] Screen-reader confirmation of TreeItem actions — NOT_RUN (manual).

## ROADMAP 012 — wgpu split, atlas starvation, IME composition (2026-11-08)

- [x] `acme-render-wgpu/src/lib.rs` split into modules (lib.rs 22 lines; public paths unchanged).
- [x] Demand LRU: a glyph no longer drops when every page holds only fresh entries; starvation test un-ignored.
- [x] Preedit scrolls into view; preedit glyphs are drawn (masked in password fields); Gallery forwards the preedit cursor and clamps the candidate anchor to the field.
- [ ] Per-format allocators and mid-frame eviction of untouched pages — rejected for now (SPEC-B); revisit with measurements.
- [x] `acme-textinput/src/lib.rs` split (ROADMAP 013).
- [ ] Real Windows 注音 composition, candidate-window placement and physical DPI — NOT_RUN (manual).
- [ ] Real GPU drawing — NOT_RUN.

## ROADMAP 011 — multi-page glyph atlas (2026-11-07)

- [x] Contract (page fields, constants, `GlyphAtlasConfig`), page-granular LRU allocator, CPU pages, wgpu lazy page textures, apps on `GlyphAtlas::default()`.
- [x] wgpu upload dedup key includes page and format.
- [ ] Real GPU multi-page drawing, device-loss recovery with live pages, GPU memory/timing — NOT_RUN (no GPU; fallback adapter segfaults on this host).
- [x] Starvation when every page holds only fresh entries — fixed in ROADMAP 012.
- [ ] Per-format allocators (both formats share one (page, x, y) namespace); mid-frame eviction of untouched pages.
- [x] `acme-render-wgpu/src/lib.rs` split (ROADMAP 012).

## ROADMAP 009 — structural accessibility and exact-area CPU coverage (2026-11-07)

- [x] Virtual TreeItem / Row / ColumnHeader nodes with counts, indices, positions, selection, bounds.
- [x] CPU quad exact-area coverage (analytic + 8x8 arc supersample); DESIGN test un-ignored.
- [x] Cell bounds / layout-ID alignment — walk mirrors layout ID order (ROADMAP 010).
- [x] Tree-item content is walked and TreeItem has Focus/Click/Expand/Collapse (ROADMAP 013).
- [ ] DataGrid merged-cell spans. (Ragged rows group correctly since ROADMAP 010.)
- [x] Analytic arc area (ROADMAP 014). - [x] Area-based clip edges (ROADMAP 015).
- [x] Multi-page glyph atlas (ROADMAP 011).
- [ ] Screen-reader confirmation of TreeItem/Row/ColumnHeader — NOT_RUN (manual).

## ROADMAP 008 — interaction and accessibility follow-ups (2026-11-07)

- [x] Gallery hovered/pressed indices no longer drift after a page switch (headless tests).
- [x] TextInput `validation_message` → AccessKit invalid + description.
- [x] Table/DataGrid cells → `Role::Cell` (label/container content).
- [x] Tree item roles — virtual `TreeItem` nodes (ROADMAP 009).
- [x] Table `Row` nodes and column headers (ROADMAP 009). TextInput `description` field still not exposed.
- [x] MIGRATION.md snippets compiled by the sdk-consumer tests.
- [x] Headless benchmark smoke run in CI (no thresholds).
- [ ] Screen-reader confirmation of the new roles — NOT_RUN (manual).

## Audit reconciliation — bounded hardening (2026-11-07)

- [x] Re-verify accessibility findings: retained-bound IDs and Gallery wiring were already fixed by merged PR #8/#9; preserve the sequential compatibility APIs.
- [x] Extend retained-ID regression coverage across insertion, reorder and removal, including focus and layout bounds.
- [x] Document Rust 1.88 MSRV and 199 component source modules (excluding category `mod.rs`); module count is not a maturity claim.
- [x] Add a font-independent regression that automatic overflow eviction waits for `begin_frame` (full clear, not per-glyph LRU). Explicit `evict_stale`/`clear` stay immediate (ROADMAP 006 contract).
- [x] Inspect PR #6: already merged, including lockfile hardening; no historical branch update needed.
- [x] Reconcile with merged PR #12: preserve benchmark work, pressure/overflow-triggered frame-boundary eviction and the 600-frame lifecycle suite.
- [ ] Manual Gallery/visual, 注音 IME, physical DPI, GPU failover and screen-reader checks — NOT_RUN in this batch.

## SDK production hardening — batch-002 (2026-11-07)

- [x] Integrate PR #5 (focus navigation) and PR #6 (CPU renderer fallback).
- [x] Explicit renderer backend policy with unit tests (`RenderBackendPreference`).
- [x] `run()` reports `NoWindows` / `Renderer` failures instead of `Ok(())`.
- [x] Deterministic CPU golden pixel tests.
- [x] ROADMAP 006 bounded atlas-lifecycle follow-up: reject invalid CPU scenes before changing glyph caches; add dual-format cache/painter-order regressions (docs/team/cpu-atlas-atomic/REPORT.md). Gallery/GPU baselines and measured performance remain open.
- [x] ROADMAP 006: headless benchmark runner with statistics/JSON and methodology (docs/guides/BENCHMARKS.md); atlas lifecycle tests and deferred mid-frame eviction fix (docs/team/roadmap-006/REPORT.md). Gallery/GPU screenshot goldens and GPU timing remain NOT_RUN.
- [x] Crate metadata, external consumer example + CI job, SDK guide.
- [x] ROADMAP 007: packageable crates (check_package.py in CI), LICENSE files + cargo-deny license policy in CI, MIGRATION.md, RELEASE.md (docs/team/roadmap-007/REPORT.md). Publish, version bump and crate-name collision are owner decisions.
- [x] acme-ui test warnings removed.
- [ ] Native runtime check of fatal paths (no-window, forced CPU failure) — NOT_RUN.
- [x] ROADMAP 003: Gallery layout and accessibility use retained NodeIds (see docs/team/roadmap-003/REPORT.md).
- [x] ROADMAP 003 follow-up: V2 Showcase and Playground frame layouts use retained IDs; fix Showcase duplicate keys and Playground stale-index panic (docs/team/retained-apps/REPORT.md).
- [ ] Retained-app follow-up: native visual/physical-DPI checks for V2 Showcase and Playground; headless frame tests do not establish manual acceptance.
- [x] ROADMAP 004: keyboard focus keyed by stable identity; core control semantics (docs/team/roadmap-004/REPORT.md).
- [x] ROADMAP 005 (automated part): DPI matrix and IME geometry tests; 3 IME bugs and 2 TextInput field bugs fixed (docs/team/roadmap-005/REPORT.md).
- [ ] ROADMAP 005 (manual): 注音 sessions at 100/125/150/200 % — human sign-off still required.
- [ ] Follow-ups (hover/press drift, validation text, Table/DataGrid cells done in ROADMAP 008; Tree item roles open): preedit scroll fixed in ROADMAP 012; CPU sharp-corner coverage (ignored test).

## Retained-tree invalidation (bounded local patch)

- [x] Detect child insertion/removal/reorder and invalidate the owning node.
- [x] Propagate reconciliation layout changes to ancestors, preserving stable IDs.
- [x] Verify unchanged frames remain clean and invalid input preserves dirty state.
- [x] Run headless core tests, Clippy and the reproducible reorder example.
- [x] Guard readonly undo/redo and make selected word deletion undoable.
- [x] Wire Gallery history shortcuts and test the actual event handler flow.
- [x] Run full Windows GNU workspace check/build/tests, all features, formatting
  and changed-file Clippy validation with existing Rust 1.94.1.
- [ ] Historical patch-wide pinned Rust 1.88 / cross-platform evidence: see later team reports and current CI; this batch runs targeted Linux checks only.
- [ ] Gallery visual validation remains pending. The retained tree drives layout/accessibility IDs since ROADMAP 003.

Evidence and scope: [retained-tree-invalidation.md](../reports/retained-tree-invalidation.md).

## AcmeUI design Skill scope (2026-07-23)

- [x] Allow `acmeui-native-design` to guide general Rust, GPUI, and Web UI projects.
- [x] Route WidgetNode/wgpu/winit implementation rules only to AcmeUI-Native targets.
- [x] Preserve each non-AcmeUI project's existing component model and validation commands.

## Documentation and visual identity (2026-07-23)

- [x] Add an original, text-free README hero illustration for the declarative UI → GPU rendering story.
- [x] Clearly distinguish generated concept art from the real Gallery capture in both English and Traditional Chinese documentation.
- [x] Optimize the project-bound hero as WebP and validate its dimensions, size, and Markdown references.

## P0
- [x] Workspace and CI
- [x] Compile-time baseline
- [x] winit window
- [x] wgpu bootstrap
- [x] Resize and surface recovery
- [x] DPI coordinate types
- [x] Paint command list
- [x] Rectangle batching
- [x] Rounded rectangle and border rendering
- [x] Clip stack
- [x] Text rendering
- [x] Stable NodeId
- [x] Reconciliation
- [x] Dirty propagation
- [x] Taffy integration
- [x] Row, Column, Stack and Scroll layout
- [x] Layout snapshots
- [x] cosmic-text shaping
- [x] Glyph atlas
- [x] CJK fallback
- [x] Hit testing
- [x] Pointer events
- [x] Capture-target-bubble event propagation
- [x] Focus manager
- [x] Keyboard traversal
- [x] Label
- [x] Button
- [x] Card
- [x] Separator
- [x] ScrollView
- [x] Light and Dark theme
- [x] Gallery

## P1
- [x] TextInput (cursor, selection, clipboard, IME preedit, password)
- [x] Clipboard (arboard integration in acme-platform)
- [x] Traditional Chinese IME (architecture + Gallery demo + caret geometry wiring; **manual 注音 validation still pending**)
- [x] Tooltip
- [x] Popover
- [x] Menu
- [x] Dialog
- [x] AccessKit
- [x] acme-devtools crate (FrameMetrics, WidgetTreeDump, LayoutInspector, RenderDiagnostics)
- [x] apps/benchmark (layout, reconciliation, frame build microbenches; no CI thresholds yet)
- [x] apps/playground (interactive widget testing app)

## P2
- [x] VirtualList (virtual scrolling, visible range calculation)
- [x] Tree (hierarchical items with depth, expand/collapse, activation)
- [x] Table (columns with titles, rows of cells, header toggle)
- [x] DataGrid (sortable columns, row selection, grid role)
- [x] Animation (tween engine, easing, looping, yoyo)
- [x] Multi-window (WindowId, per-window state, close handling, tests)
- [x] Surface recreation automated test (pure state machine + atlas invalidation contract + `on_gpu_recovered` hook)

## Remaining / next
- [x] Navigation widgets (NavRail, Sidebar, TabBar, Breadcrumb)
- [x] Gallery Data category real Tree/Table/DataGrid/VirtualList demos
- [x] Real wgpu device-lost detection (`set_device_lost_callback` + uncaptured Internal/OOM)
- [x] Interactive Gallery Data/Nav demos (Tree/Table/VL/Nav state machines)
- [x] Manual validation checklists documented (`docs/guides/MANUAL_VALIDATION.md`) — **human sign-off still open**
- [ ] Manual Traditional Chinese 注音 IME validation on Windows (checklist B)
- [ ] Manual GPU device-loss recovery on Windows (checklist A)
- [ ] Manual DPI interaction at 125/150/200% Windows scaling
- [x] Glyph atlas stale detection and full-clear eviction when full (not per-glyph LRU); mid-frame eviction deferred to `begin_frame`
- [ ] Benchmark headless CI thresholds
- [x] Pixel regression: CPU-renderer text-free goldens (batch-002); Gallery screenshot goldens still open

## P3 — Mobile component real implementations (CBM audit 2026-07-21)

> Historical CBM task list: current `mobile_*` files now contain builders; the
> old blanket label-passthrough claim is obsolete. Unchecked items below remain
> acceptance follow-ups, not proof that the implementation is absent. Re-audit
> interaction and test coverage before marking them complete.

- [x] mobile_button — sized variants (sm/md/lg), disabled state, on_press message; 36/44/52px touch heights, Gallery page, and hit-index regression tests (2026-07-23)
- [x] mobile_card — title + subtitle + optional media slot + elevated/outlined semantic variants; retained-ID regression, layout tests and Gallery demos (ROADMAP 019). Native shadows are outside this slice; manual hardware acceptance remains NOT_RUN.
- [x] mobile_toggle — switch track + thumb, checked state, on_change; ROADMAP 014 behavior/hit/accessibility/mutation evidence preserved, historical Clippy gate reverified PASS in ROADMAP 022. Manual hardware acceptance remains NOT_RUN.
- [x] mobile_stepper — minus/plus buttons, value and min/max clamp; ROADMAP 014 behavior/hit/accessibility/mutation evidence preserved, historical Clippy gate reverified PASS in ROADMAP 022. Manual hardware acceptance remains NOT_RUN.
- [x] mobile_progress — responsive determinate bar (value/total) + optional label; numeric boundaries, retained IDs and Gallery demos/tests (ROADMAP 020). Manual hardware acceptance remains NOT_RUN.
- [x] mobile_loader — existing static foundation spinner indicator + optional wrapping message; retained-ID/layout/paint tests and Gallery demos (ROADMAP 021). Rotation/runtime scheduling remains a separate unchecked follow-up; manual hardware acceptance is NOT_RUN.
- [x] mobile_search — rounded themed TextInput + search icon/action + clear button; controlled query, input marker, retained/layout/host-event tests and native CPU Gallery evidence (ROADMAP 022). Manual hardware acceptance remains NOT_RUN.
- [x] mobile_segmented — selected N-segment row and on_select; ROADMAP 014 behavior/hit/accessibility/mutation evidence preserved, historical Clippy gate reverified PASS in ROADMAP 022. Manual hardware acceptance remains NOT_RUN.
- [x] mobile_chip — compact themed tag, selected marker and optional close action; separate tap/dismiss messages, retained/layout tests and native CPU Gallery evidence (ROADMAP 023). Manual hardware acceptance remains NOT_RUN.
- [x] mobile_avatar — circular themed initials / neutral empty-name placeholder, 32/40/56 px variants; retained/layout/paint tests and native CPU Gallery evidence (ROADMAP 024). Actual image loading is outside this builder; manual hardware acceptance remains NOT_RUN.
- [x] mobile_banner — full-width themed soft alert with icon, wrapping text and optional independent action/dismiss; retained/layout/Gallery tests and native CPU evidence (ROADMAP 025). Manual hardware acceptance remains NOT_RUN.
- [x] mobile_notification — themed toast-style card with wrapping title/body, separate actions and Gallery host-owned auto-dismiss/hint; retained/layout/timer tests and native CPU captures (ROADMAP 026). The stateless component schedules no timer itself; manual hardware acceptance remains NOT_RUN.
- [x] mobile_list_item — responsive themed title/detail row with optional fixed trailing icon and on_tap; retained/layout/Gallery pointer/keyboard tests and native CPU evidence (ROADMAP 027). Framework-wide Row focus/typed semantics remain open; manual hardware acceptance is NOT_RUN.
- [x] mobile_action — themed tappable Row with bounded icon, wrapping stable label, destructive/disabled states and Gallery pointer evidence (ROADMAP 028). Generic Row focus/typed semantics and manual hardware acceptance remain open.
- [x] mobile_sheet_handle — themed 36×4 rounded visual indicator, optional custom width, fixed/finite geometry, retained/layout/Gallery tests and native CPU capture (ROADMAP 029). Actual dragging is outside this decoration; manual hardware acceptance remains NOT_RUN.
- [x] Unit tests for all public mobile modules: builder defaults, layout kind and widget/layout child counts (ROADMAP 030). Historical “16” count corrected to 20 public modules: 15 P3 mobile_* builders plus five legacy composites; this coverage does not certify their interaction/runtime completeness.

## P4 — New mobile components (gap analysis)

- [x] FloatingActionButton (FAB) — circular elevated button, mini/normal/extended; existing float_button builder, retained/layout/typed actions and native CPU Gallery evidence (ROADMAP 032). Viewport anchoring is host-owned; manual hardware acceptance remains NOT_RUN.
- [x] Snackbar — existing overlay::toast with wrapping message, optional independent action/dismiss, and Gallery bottom placement/auto-dismiss (ROADMAP 033). Stateless builder; host owns visibility/deadlines. Manual hardware acceptance remains NOT_RUN.
- [x] SwipeActions — controlled swipe-to-reveal Delete/Archive on list content, threshold state and native CPU Gallery pointer/keyboard tests (ROADMAP 034). Actions require separate activation; host owns pointer routing. Continuous translation/animation and manual hardware acceptance remain open.
- [ ] MobileTabBar — top scrollable tab strip with badge support
- [ ] MobileDrawer — side navigation drawer with overlay scrim
- [ ] MobileCarousel — horizontal paging with dot indicators
- [ ] MobileRefreshIndicator — pull-to-refresh spinner overlay (extend pull_to_refresh)

## P5 — Architecture & quality improvements (CBM audit)

- [ ] Correct UI optional-feature dependencies: layout-only fails on IconName and form_message even with Paper restored to the ROADMAP 035 baseline (ROADMAP 036 evidence). Inputs also imports foundation IconName; audit/declaration and minimal-feature CI remain open.

- [x] Reconcile `Styled::w_full()` and related full sizing helpers with fractional `Length::Percent(1.0)`; correct Button.full_width, regressions and Gallery evidence (ROADMAP 035). Engine units unchanged; screen helpers remain parent-relative.

- [ ] Decompose gallery `frame()` (244 lines in acme-gallery, 130 in gallery) into per-page helpers
- [ ] Increase trait abstraction (only 27 IMPLEMENTS edges across 4343 symbols)
- [x] Add Gallery "Mobile" page for all 15 P3 mobile_* builders — examples/evidence through ROADMAP 029; complete presence and retained-root inventory verified in ROADMAP 030. This does not claim demos or interaction acceptance for the five other legacy mobile composites.
- [ ] Add `#[cfg(test)]` coverage for overlay/layout/chart builder conversions
- [ ] Evaluate extracting `acme-gesture` crate for swipe/drag/long-press recognition
- [x] Document public component-module inventory in both READMEs (200 after ROADMAP 034; counted from eight category mod.rs declarations, excluding test/support modules; does not imply interactive completeness). Historical pre-SwipeActions count was 199.

## P6 — shadcn/ui alignment (2026-07-21)

> Coverage: 57/57 shadcn/ui components now mapped. 6 new components added this pass.

### Newly added (this pass)
- [x] FormField (`inputs/form_field.rs`) — label + description + error + control wrapper
- [x] ToggleGroup (`inputs/toggle_group.rs`) — single/multiple toggle selection
- [x] InputOTP (`inputs/input_otp.rs`) — individual character boxes for OTP entry
- [x] DateRangePicker (`inputs/date_range_picker.rs`) — start/end date range selection
- [x] Resizable (`layout/resizable.rs`) — draggable panel resizing (horizontal/vertical)
- [x] Sidebar (`layout/sidebar.rs`) — collapsible sidebar with variants (sidebar/floating/inset)

### Already covered (existing components map to shadcn/ui)
- [x] Accordion → foundations/accordion
- [x] Alert → foundations/alert
- [x] Alert Dialog → overlay/confirm_dialog
- [x] Aspect Ratio → foundations/aspect_ratio
- [x] Avatar → foundations/avatar
- [x] Badge → foundations/badge
- [x] Breadcrumb → layout/breadcrumb
- [x] Button → widgets/button
- [x] Calendar → foundations/calendar
- [x] Card → widgets/card
- [x] Carousel → browser/carousel
- [x] Chart → charts/* (21 chart types)
- [x] Checkbox → inputs/checkbox
- [x] Collapsible → foundations/collapsible
- [x] Combobox → inputs/combobox
- [x] Command → overlay/command_palette
- [x] Context Menu → overlay/context_menu
- [x] Data Table → widgets/datagrid
- [x] Date Picker → inputs/date_picker
- [x] Dialog → overlay/modal
- [x] Drawer → overlay/drawer
- [x] Dropdown Menu → overlay/dropdown_menu
- [x] Empty → foundations/empty_state
- [x] Form → layout/form
- [x] Hover Card → overlay/hover_card
- [x] Input → widgets/text_input
- [x] Input OTP → inputs/pin_input + inputs/input_otp
- [x] Kbd → foundations/kbd
- [x] Label → widgets/label
- [x] Menubar → desktop/menubar
- [x] Navigation Menu → layout/navigation_menu
- [x] Pagination → layout/pagination
- [x] Popover → widgets/popover
- [x] Progress → foundations/progress
- [x] Radio Group → inputs/radio
- [x] Resizable → layout/split_panel + layout/resizable
- [x] Scroll Area → layout/scroll_area
- [x] Select → inputs/select
- [x] Separator → foundations/divider
- [x] Sheet → overlay/drawer
- [x] Sidebar → desktop/sidenav + layout/sidebar
- [x] Skeleton → foundations/skeleton
- [x] Slider → inputs/slider + inputs/range_slider
- [x] Sonner → overlay/toast
- [x] Spinner → foundations/spinner
- [x] Status → foundations/status_dot
- [x] Switch → inputs/switch
- [x] Table → widgets/table
- [x] Tabs → layout/tabs
- [x] Tag → foundations/tag
- [x] Textarea → inputs/textarea
- [x] Toast → overlay/toast
- [x] Toggle → inputs/toggle_button
- [x] Toggle Group → inputs/button_group + inputs/toggle_group
- [x] Tooltip → widgets/tooltip

### Remaining enhancements
- [ ] Add Gallery page showcasing shadcn-aligned components
- [ ] Add async search to Combobox (command-style filtering)
- [ ] Add range mode to Calendar (visual range highlight)
- [ ] Add drag-and-drop reordering to Data Table
- [ ] Add keyboard navigation to Sidebar and Navigation Menu

## P7 — Ant Design alignment (2026-07-22)

> Coverage: 70/70 Ant Design 5.x components now mapped. 7 new components added this pass.

### Newly added (this pass)
- [x] Anchor (`layout/anchor.rs`) — page anchor navigation with nested items
- [x] QRCode (`foundations/qr_code.rs`) — QR code display placeholder
- [x] Tour (`overlay/tour.rs`) — guided tour / onboarding steps
- [x] Affix (`layout/affix.rs`) — sticky/fixed positioning wrapper
- [x] FloatButton (`overlay/float_button.rs`) — floating action button (FAB)
- [x] Notification (`overlay/notification.rs`) — desktop notification overlay
- [x] Typography (`foundations/typography.rs`) — Title/Paragraph/Text formatting

### Already covered (existing components map to Ant Design)
- [x] Button → widgets/button
- [x] Icon → foundations/icon
- [x] Typography (partial) → foundations/text_block + foundations/typography
- [x] Divider → foundations/divider
- [x] Flex → foundations/flex
- [x] Grid → layout/grid
- [x] Layout → layout/* (multiple)
- [x] Space → foundations/spacer
- [x] Splitter → layout/split_panel + layout/resizable
- [x] Breadcrumb → layout/breadcrumb
- [x] Dropdown → overlay/dropdown_menu
- [x] Menu → desktop/menubar + overlay/dropdown_menu
- [x] Pagination → layout/pagination
- [x] Steps → layout/stepper
- [x] Tabs → layout/tabs
- [x] AutoComplete → inputs/autocomplete
- [x] Cascader → inputs/cascader
- [x] Checkbox → inputs/checkbox
- [x] ColorPicker → inputs/color_picker
- [x] DatePicker → inputs/date_picker + date_range_picker
- [x] Form → layout/form + inputs/form_field
- [x] Input → widgets/text_input
- [x] InputNumber → inputs/number_input
- [x] Mentions → inputs/mentions
- [x] Radio → inputs/radio
- [x] Rate → inputs/rating
- [x] Select → inputs/select
- [x] Slider → inputs/slider + range_slider
- [x] Switch → inputs/switch
- [x] TimePicker → inputs/time_picker
- [x] Transfer → inputs/transfer
- [x] TreeSelect → inputs/tree_select
- [x] Upload → inputs/file_upload
- [x] Avatar → foundations/avatar
- [x] Badge → foundations/badge
- [x] Calendar → foundations/calendar
- [x] Card → widgets/card
- [x] Carousel → browser/carousel
- [x] Collapse → foundations/accordion + collapsible
- [x] Descriptions → foundations/descriptions
- [x] Empty → foundations/empty_state
- [x] Image → desktop/image_view + browser/lightbox
- [x] List → foundations/list
- [x] Popover → widgets/popover + overlay/hover_card
- [x] Segmented → inputs/segmented_control
- [x] Statistic → foundations/statistic
- [x] Table → widgets/table + datagrid
- [x] Tag → foundations/tag
- [x] Timeline → foundations/timeline
- [x] Tooltip → widgets/tooltip
- [x] Tree → widgets/tree
- [x] Alert → foundations/alert
- [x] Drawer → overlay/drawer
- [x] Message → overlay/toast
- [x] Modal → overlay/modal
- [x] Popconfirm → overlay/confirm_dialog
- [x] Progress → foundations/progress
- [x] Result → foundations/result
- [x] Skeleton → foundations/skeleton
- [x] Spin → foundations/spinner
- [x] Watermark → foundations/watermark

### Remaining enhancements
- [ ] Real QR code rendering (currently placeholder)
- [ ] Tour target highlighting (scroll to + spotlight element)
- [ ] Affix scroll listener (toggle fixed on scroll position)
- [ ] FloatButton group (expandable multiple FABs)
- [ ] Notification auto-dismiss timer integration

## P8 — MUI (Material UI) alignment (2026-07-22)

> Coverage: 60/60 MUI components now mapped. 7 new components added this pass.

### Newly added (this pass)
- [x] Backdrop (`overlay/backdrop.rs`) — overlay scrim behind modals/dialogs
- [x] SpeedDial (`overlay/speed_dial.rs`) — expandable FAB with multiple actions
- [x] ImageList (`layout/image_list.rs`) — image grid/masonry layout
- [x] AppBar (`layout/app_bar.rs`) — top application bar with title/actions
- [x] Paper (`layout/paper.rs`) — elevated surface container with elevation levels
- [x] TextField (`inputs/text_field.rs`) — multi-variant input (outlined/filled/standard)
- [x] BottomNavigation (`layout/bottom_navigation.rs`) — desktop bottom navigation bar

### Already covered (existing components map to MUI)
- [x] Autocomplete → inputs/autocomplete
- [x] Button → widgets/button
- [x] Button Group → inputs/button_group
- [x] Checkbox → inputs/checkbox
- [x] Floating Action Button → overlay/float_button
- [x] Radio Group → inputs/radio
- [x] Rating → inputs/rating
- [x] Select → inputs/select
- [x] Slider → inputs/slider + range_slider
- [x] Switch → inputs/switch
- [x] Toggle Button → inputs/toggle_button + toggle_group
- [x] Transfer List → inputs/transfer
- [x] Avatar → foundations/avatar
- [x] Badge → foundations/badge
- [x] Chip → foundations/chip
- [x] Divider → foundations/divider
- [x] Icons → foundations/icon
- [x] List → foundations/list
- [x] Table → widgets/table + datagrid
- [x] Tooltip → widgets/tooltip
- [x] Typography → foundations/typography
- [x] Alert → foundations/alert
- [x] Dialog → overlay/modal
- [x] Progress → foundations/progress
- [x] Skeleton → foundations/skeleton
- [x] Snackbar → overlay/toast
- [x] Bottom Navigation (mobile) → mobile/bottom_nav
- [x] Breadcrumbs → layout/breadcrumb
- [x] Drawer → overlay/drawer
- [x] Link → foundations/link
- [x] Menu → desktop/menubar + overlay/dropdown_menu
- [x] Pagination → layout/pagination
- [x] Stepper → layout/stepper
- [x] Tabs → layout/tabs
- [x] Box → foundations/flex
- [x] Container → layout/section
- [x] Grid → layout/grid
- [x] Stack → widgets/stack
- [x] Modal → overlay/modal
- [x] Popover → widgets/popover
- [x] Textarea Autosize → inputs/textarea
- [x] Accordion → foundations/accordion
- [x] Card → widgets/card
- [x] Toolbar → layout/toolbar

### Remaining enhancements
- [ ] Real image rendering in ImageList (currently placeholder labels)
- [ ] SpeedDial directional expansion (up/down/left/right)
- [ ] AppBar scroll-hide behavior
- [ ] TextField input masking and adornments (leading/trailing icons)
- [x] Paper square variant (no border radius), independent of elevation; real layout/retained/Gallery checks and native CPU evidence (ROADMAP 036). Existing shadow approximation and manual acceptance remain separate.

## P9 — Radix UI alignment (2026-07-22)

> Coverage: all Radix primitives + Radix Themes components now mapped. 7 new components added this pass.
> Radix emphasizes unstyled accessible primitives; our a11y layer (acme-accessibility) already covers the core.

### Newly added (this pass)
- [x] VisuallyHidden (`foundations/visually_hidden.rs`) — visually hidden but screen-reader accessible
- [x] LiveRegion (`foundations/live_region.rs`) — aria-live region for screen reader announcements
- [x] CheckboxCards (`inputs/checkbox_cards.rs`) — card-style checkbox selection group
- [x] RadioCards (`inputs/radio_cards.rs`) — card-style radio selection group
- [x] DataList (`foundations/data_list.rs`) — key-value pair data display
- [x] Inset (`layout/inset.rs`) — content inset within a surface
- [x] Code (`foundations/code.rs`) — inline and block code display

### Already covered (existing components map to Radix primitives/themes)
- [x] Checkbox → inputs/checkbox
- [x] Form → layout/form + inputs/form_field
- [x] Label → widgets/label
- [x] Radio Group → inputs/radio
- [x] Select → inputs/select
- [x] Slider → inputs/slider
- [x] Switch → inputs/switch
- [x] Textarea → inputs/textarea
- [x] Toggle → inputs/toggle_button
- [x] Toggle Group → inputs/toggle_group
- [x] Alert Dialog → overlay/confirm_dialog
- [x] Aspect Ratio → foundations/aspect_ratio
- [x] Avatar → foundations/avatar
- [x] Collapsible → foundations/collapsible
- [x] Context Menu → overlay/context_menu
- [x] Dialog → overlay/modal
- [x] Dropdown Menu → overlay/dropdown_menu
- [x] Hover Card → overlay/hover_card
- [x] Navigation Menu → layout/navigation_menu
- [x] Popover → widgets/popover
- [x] Scroll Area → layout/scroll_area
- [x] Toast → overlay/toast
- [x] Toolbar → layout/toolbar
- [x] Tooltip → widgets/tooltip
- [x] Accessible Icon → foundations/icon
- [x] Badge → foundations/badge
- [x] Callout → foundations/callout
- [x] Card → widgets/card
- [x] Container → layout/section
- [x] Em/Strong/Sub/Sup → foundations/typography (TextType)
- [x] Flex → foundations/flex
- [x] Grid → layout/grid
- [x] Heading → foundations/typography (TitleLevel)
- [x] Kbd → foundations/kbd
- [x] Link → foundations/link
- [x] Progress → foundations/progress
- [x] Quote → foundations/quote
- [x] Segmented Control → inputs/segmented_control
- [x] Separator → foundations/divider
- [x] Skeleton → foundations/skeleton
- [x] Spinner → foundations/spinner
- [x] Table → widgets/table
- [x] Tabs → layout/tabs
- [x] Text → foundations/typography
- [x] Theme → acme-theme crate

### Remaining enhancements
- [ ] Wire VisuallyHidden/LiveRegion into acme-accessibility tree builder
- [ ] CheckboxCards/RadioCards keyboard navigation (arrow keys)
- [ ] DataList density variants (compact/relaxed)
- [ ] Code syntax highlighting integration
- [ ] Inset clip variants (rounded corners matching parent)

## P10 — gpui-component absorption (2026-07-22)

> Absorbed strengths from longbridge/gpui-component, re-implemented in AcmeUI Native
> builder-pattern style. NO GPUI dependency added (per AGENTS.md constraint).
> No hardcoded theme colors; all components use semantic variants/tones.

### Newly added (this pass)
- [x] CopyButton (`foundations/copy_button.rs`) — copy text with idle/copied feedback state
- [x] Indicator (`foundations/indicator.rs`) — status dot (online/offline/busy/away) + ping + label
- [x] ProgressRing (`foundations/progress_ring.rs`) — circular/radial progress (complements linear)
- [x] InputGroup (`inputs/input_group.rs`) — input with prefix/suffix/leading/trailing addons
- [x] SkeletonShape (`foundations/skeleton_shape.rs`) — loading placeholder (text/circle/rect/rounded)
- [x] SliderMarks (`inputs/slider_marks.rs`) — slider with labeled tick marks
- [x] KbdCombo (`foundations/kbd_combo.rs`) — keyboard shortcut combination display (Ctrl+Shift+P)

### gpui-component strengths absorbed
- Clipboard copy-with-feedback pattern → CopyButton
- Rich status indicator (vs plain status_dot stub) → Indicator
- Circular progress variant → ProgressRing
- Input with addons (prefix/suffix) → InputGroup
- Skeleton shape variants → SkeletonShape
- Marked slider for precise selection → SliderMarks
- Keyboard shortcut display → KbdCombo

### Already covered (gpui-component components we already had)
- [x] accordion, alert, avatar, badge, breadcrumb, button, calendar
- [x] checkbox, collapsible, context_menu, date_picker
- [x] description_list → foundations/descriptions
- [x] dock → desktop/dock
- [x] dropdown_menu, form, icon, image, input, kbd, label, list, menu, modal
- [x] notification, number_input, popover, progress, radio, resizable
- [x] scrollable → layout/scroll_area
- [x] select, sidebar, skeleton, slider, switch, tab, table, text, toggle, tooltip
- [x] webview → browser/web_frame

### Remaining enhancements
- [ ] CopyButton real clipboard write (wire to acme-platform arboard)
- [ ] Indicator ping animation (wire to acme-animation tween)
- [ ] ProgressRing SVG-style arc rendering (currently glyph approximation)
- [ ] InputGroup real input focus management
- [ ] SliderMarks drag interaction on marks

## P11 — Brand theme packs (2026-07-22)

> New `acme-theme::packs` module with 10 curated brand theme packs.
> Each implements the `ThemePack` trait (light + dark variants) built from real
> brand palettes via `Theme::from_colors`. No hardcoded colors in widgets —
> all flow through semantic ColorTokens. Registry: `theme_by_name(name, mode)`.

### Architecture
- [x] `Theme::from_colors(mode, colors, shadows)` constructor (shared structural tokens)
- [x] `ThemePack` trait (`name()` / `light()` / `dark()` / `theme(mode)`)
- [x] `theme_by_name(name, mode) -> Option<Theme>` dynamic registry
- [x] `available_themes() -> &[&str]` enumeration
- [x] `shadow_ladder()` helper for per-level shadow opacity

### Theme packs added (10)
- [x] Apple (`packs/apple.rs`) — macOS/iOS system blue #007AFF, rounded geometry
- [x] Windows 10 (`packs/windows10.rs`) — Fluent accent #0078D4, squared geometry
- [x] Windows 11 (`packs/windows11.rs`) — WinUI accent #0067C0, Mica rounded
- [x] Ubuntu (`packs/ubuntu.rs`) — Yaru orange #E95420 + aubergine accent
- [x] Material (`packs/material.rs`) — M3 baseline purple #6750A4, pill radii
- [x] Nord (`packs/nord.rs`) — arctic frost palette
- [x] Dracula (`packs/dracula.rs`) — vibrant dark + purple #BD93F9
- [x] Solarized (`packs/solarized.rs`) — precision light/dark #268BD2
- [x] Gruvbox (`packs/gruvbox.rs`) — retro groove warm palette
- [x] One Dark (`packs/one_dark.rs`) — Atom One Dark #61AFEF

### Validation
- [x] All 10 themes × 2 modes pass `Theme::validate()` (color range checks)
- [x] Light/dark backgrounds differ for every theme
- [x] Unknown theme names return `None`

### Remaining enhancements
- [x] Wire theme packs into Gallery theme switcher (live preview) — done in P12
- [x] Persist selected theme pack in app config — done in P12
- [ ] Add high-contrast accessibility variants per pack
- [ ] Theme pack JSON import/export (user-defined themes)
- [x] Contrast-ratio validation (WCAG AA/AAA) — done in P12 (`contrast_ratio`, `ContrastLevel`, `Theme::wcag_report`)

## P12 — Gallery theme integration, WCAG validation & design skill (2026-07-22)

### WCAG contrast validation (acme-theme)
- [x] `ThemeColor::relative_luminance()` — WCAG 2.1 luminance
- [x] `contrast_ratio(a, b) -> f32` — 1.0–21.0 contrast ratio
- [x] `ContrastLevel` enum (Aaa ≥7 / Aa ≥4.5 / AaLarge ≥3 / Fail) + `from_ratio` / `meets_aa`
- [x] `Theme::contrast()`, `Theme::wcag_report()` (6 key pairs), `Theme::meets_wcag_aa()`
- [x] Tests: black/white=21, identical=1, luminance extremes, level classification, built-in AA

### Gallery theme switcher (apps/acme-gallery)
- [x] Theme-pack cycle button in toolbar (🎨 <pack> ✓AA/⚠AA live WCAG indicator)
- [x] `current_theme()` resolves pack + light/dark via `theme_by_name`
- [x] `cycle_theme()` walks default → apple → windows10 → … (wraps)
- [x] Persistence: `load_theme_pref()` / `save_theme_pref()` → `acme-gallery-theme.conf` (gitignored)
- [x] Updated hardcoded render indices (toolbar_buttons [NodeId;3], tb_labels, content btn_idx 10)

### AcmeUI-Native design skill
- [x] `.qwen/skills/acmeui-native-design/SKILL.md` — crate architecture, component inventory,
      builder pattern, theme system + packs, WCAG, "how to add a component", AGENTS.md rules
- [x] `.gitignore` updated to commit `.qwen/skills/` (was fully ignored)

### Remaining enhancements
- [ ] High-contrast accessibility theme variants
- [ ] Theme pack JSON import/export (user-defined themes)
- [ ] Gallery: render theme pack swatches in Design System page
- [ ] Skill: add runnable copy-paste examples per component category
