# Todos

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
- [ ] Follow-ups: hovered/pressed indices drift after page switch; TextInput validation_message as AccessKit error text; Table/DataGrid cell and Tree item roles; preedit wider than the field does not scroll; CPU sharp-corner coverage (ignored test).

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
- [ ] mobile_card — title + subtitle + optional media slot + elevation
- [ ] mobile_toggle — switch track + thumb layout, checked state, on_change
- [ ] mobile_stepper — minus/plus buttons flanking a value label, min/max clamp
- [ ] mobile_progress — determinate bar (value/total) + optional label
- [ ] mobile_loader — spinner indicator with optional message
- [ ] mobile_search — rounded input + search icon + clear button
- [ ] mobile_segmented — N-segment row, selected highlight, on_select(usize)
- [ ] mobile_chip — compact tag with optional close icon, on_dismiss
- [ ] mobile_avatar — circular initials or image placeholder, size variants
- [ ] mobile_banner — full-width alert with icon + text + optional action
- [ ] mobile_notification — toast-style card with title + body + auto-dismiss hint
- [ ] mobile_list_item — title + detail + optional trailing icon, on_tap
- [ ] mobile_action — tappable action row with icon + label + destructive flag
- [ ] mobile_sheet_handle — drag indicator bar (fixed 36×4 rounded rect)
- [ ] Unit tests for all 16 modules (layout kind, child count, builder defaults)

## P4 — New mobile components (gap analysis)

- [ ] FloatingActionButton (FAB) — circular elevated button, mini/normal/extended
- [ ] Snackbar — bottom transient message with optional action button
- [ ] SwipeActions — swipe-to-reveal delete/archive on list items
- [ ] MobileTabBar — top scrollable tab strip with badge support
- [ ] MobileDrawer — side navigation drawer with overlay scrim
- [ ] MobileCarousel — horizontal paging with dot indicators
- [ ] MobileRefreshIndicator — pull-to-refresh spinner overlay (extend pull_to_refresh)

## P5 — Architecture & quality improvements (CBM audit)

- [ ] Decompose gallery `frame()` (244 lines in acme-gallery, 130 in gallery) into per-page helpers
- [ ] Increase trait abstraction (only 27 IMPLEMENTS edges across 4343 symbols)
- [ ] Add Gallery "Mobile" page showcasing all mobile components
- [ ] Add `#[cfg(test)]` coverage for overlay/layout/chart builder conversions
- [ ] Evaluate extracting `acme-gesture` crate for swipe/drag/long-press recognition
- [x] Document source-module inventory in both READMEs (199 modules; excludes `mod.rs`, does not imply interactive completeness)

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
- [ ] Paper square variant (no border radius)

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
