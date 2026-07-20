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
- Traditional Chinese IME is architecture-only and has not received manual 注音
  preedit/candidate/commit/cancel validation.
- AccessKit, TextInput/clipboard, multi-window and cross-platform support are deferred.
- Manual interaction at physical 125/150/200% Windows display scaling remains to be
  performed; automated DPI conversion and glyph-scale tests do not replace it.

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

## Next milestone

Finish deterministic surface/device recreation, retained NodeId layout identity,
intrinsic text measurement for auto-sized Label/Stack, manual DPI validation, then
begin TextInput and manually validated Traditional Chinese IME work.
