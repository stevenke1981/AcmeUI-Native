# AcmeUI-Native 驗證方案

## 1. 自動化 Gates

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo doc --workspace --all-features --no-deps
cargo audit
cargo deny check
```

另建議：

```powershell
cargo hack check --workspace --feature-powerset
cargo nextest run --workspace --all-features
```

## 2. P0 Regression Tests

### Rendering order

- quad → text → overlay quad → overlay text
- nested clip
- tooltip above modal content
- selection behind glyph but above field background
- caret above text
- deterministic hash across 100 runs

### Runtime identity

- keyed reorder
- insert before keyed siblings
- remove/re-add
- kind change
- nested keyed list
- focus remains on same key
- accessibility ID remains stable

### Slider

- min/max normalization
- value clamp
- step rounding
- pointer drag
- keyboard
- disabled
- on_change exactly once

### DatePicker

- 31-day month starting Friday/Saturday
- leap year
- month 0/13
- year 0
- prev/next year rollover
- keyboard selection
- on_change target

### TextInput

- undo after redo
- redo after multiple undo
- IME commit undo
- grapheme replacement
- selection replacement
- readonly history
- property-based command sequence

### Platform events

- one OS input → one framework event
- modifiers
- repeat
- scale factor
- file drop redraw
- IME target window

### Accessibility

- stable IDs
- focus action target
- click action target
- set-value target
- disabled state
- value/range
- tree/table relationships

## 3. Property-based / Fuzz

建議使用 `proptest`：

- random widget tree reconciliation
- duplicate keys
- random reorder/remove/insert
- random text edit command sequences
- random Unicode grapheme
- random date/month/year
- random layout values including NaN/Inf
- clip stack balance

Invariants：

- no duplicate NodeId
- parent/child reciprocal
- focus target exists
- no invalid UTF-8 boundary
- undo/redo round-trip
- scene clip stack balanced
- layout rect finite/non-negative

## 4. Visual Golden

最少建立以下 scenes：

1. Basic controls light。
2. Basic controls dark。
3. High contrast。
4. CJK + emoji。
5. IME preedit。
6. Overlay stack。
7. Long text wrapping。
8. Tree/Table/DataGrid。
9. 125% DPI。
10. 150% DPI。
11. 200% DPI。
12. RTL 預留測試。

比較：

- pixel diff
- perceptual diff
- font-platform tolerance mask
- artifact upload

## 5. Benchmark Matrix

| Benchmark | Size |
|---|---|
| Reconcile unchanged | 100 / 1k / 10k |
| Reconcile reorder | 100 / 1k / 10k |
| Insert/remove | 1 / 10 / 100 nodes |
| Layout cold | 100 / 1k / 10k |
| Layout partial | 1 dirty leaf / subtree |
| Text shape | ASCII / CJK / emoji / mixed |
| Text cache | hit / miss |
| Display list | 1k / 10k primitives |
| Clip groups | 1 / 10 / 100 |
| Accessibility update | unchanged / partial / full |
| Component build | core controls |

輸出：

- median
- p95
- allocations
- bytes uploaded
- draw calls
- atlas occupancy
- cache hit rate

## 6. Windows Manual Matrix

### DPI

- 100%
- 125%
- 150%
- 200%
- 跨螢幕移動
- scale change 後 focus/hit/caret

### IME

- Microsoft 注音
- 中英切換
- preedit
- candidate placement
- commit/cancel
- mixed CJK/ASCII
- password
- refocus
- second window

### Accessibility

- Narrator browse
- Tab traversal
- button activation
- input value
- slider value
- tree expand/collapse
- dialog focus trap
- disabled announcements

### GPU

- resize/minimize/restore
- sleep/resume
- adapter reset
- hybrid GPU
- device lost
- atlas re-upload

## 7. Release Evidence

每個 release artifact 應附：

- commit SHA
- Rust version
- dependency lock hash
- test summary
- golden result
- benchmark delta
- manual validation sign-off
- known limitations
