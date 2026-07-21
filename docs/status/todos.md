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
- [ ] Document component inventory in README (current count: 160+ UI components)

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
