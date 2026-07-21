# Todos

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
- [x] Glyph atlas eviction/aging when full
- [ ] Benchmark headless CI thresholds
- [ ] Screenshot golden / pixel regression

## P3 — Mobile component real implementations (CBM audit 2026-07-21)

> 16 mobile_* modules are currently stubs (`crate::label()` passthrough).
> Each needs a builder-pattern struct + `From<Builder> for WidgetNode<M>` + unit tests,
> following the `bottom_nav` / `action_sheet` reference pattern.

- [ ] mobile_button — sized variants (sm/md/lg), disabled state, on_press message
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
- [ ] Document component inventory in README (current count: 152+ UI components)
