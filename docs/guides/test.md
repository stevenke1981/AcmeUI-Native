# Test Strategy

## v0.1 required gates

## Unit
Geometry, NodeId, reconciliation, dirty propagation, event order, focus traversal, and theme validation.

## Layout snapshots
Nested row/column, min/max, flex grow/shrink, text measurement, scroll content, CJK, 100-200% DPI.

## Renderer
Rect, rounded rect, border, clip, alpha, glyph positioning, off-screen image comparison.

## Integration
Startup, resize, simulated surface loss, pointer Button, keyboard Button, focus restore, theme switching.

## Performance
10k static nodes, dirty-subtree layout, and warm incremental build per crate.

## Deferred gates

These become required only when their corresponding P1/P2 feature is delivered.

- TextInput and IME: text cursor, 注音 preedit, candidate commit, cancellation,
  mixed Chinese/English, emoji, and grapheme-safe cursor. Traditional Chinese IME
  requires manual validation and must not be inferred from automated event tests.
- VirtualList: virtualized 1k rows.
- Accessibility: AccessKit tree and platform integration.
