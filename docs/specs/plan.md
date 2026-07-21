# Development Plan

## Phase 0 Foundation
Workspace, CI, tracing, dependency policy, compile-time baseline.

## Phase 1 Window + GPU
winit lifecycle, wgpu instance/adapter/device/queue/surface, resize, DPI, surface recovery.

## Phase 2 Renderer
Paint list, rectangle batching, rounded corners, borders, clips, texture abstraction, diagnostics.

## Phase 3 Retained Tree
Stable NodeId, parent/child graph, reconciliation, dirty flags, hit-test tree.

## Phase 4 Layout
Taffy integration, row/column/stack, constraints, text measurement, scrolling.

## Phase 5 Text
fontdb, cosmic-text shaping, glyph atlas, CJK/emoji fallback.

## Phase 6 Input
Pointer capture, hover path, capture-target-bubble events, focus manager, shortcuts.

## Phase 7 Widgets
Label, Button, IconButton, Card, Separator, ScrollView, Badge, Spinner.

## Phase 8 TextInput + IME
Grapheme cursor, selection, clipboard, preedit/commit, password, validation.

## Phase 9 Overlay
Tooltip, Popover, Menu, Dialog, focus trap, anchor placement.

## Phase 10 Accessibility + Hardening
AccessKit, reduced motion, high contrast, visual regression, benchmarks.
