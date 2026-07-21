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
- [x] Manual validation checklists documented (`docs/MANUAL_VALIDATION.md`) — **human sign-off still open**
- [ ] Manual Traditional Chinese 注音 IME validation on Windows (checklist B)
- [ ] Manual GPU device-loss recovery on Windows (checklist A)
- [ ] Manual DPI interaction at 125/150/200% Windows scaling
- [x] Glyph atlas eviction/aging when full
- [ ] Benchmark headless CI thresholds
- [ ] Screenshot golden / pixel regression
