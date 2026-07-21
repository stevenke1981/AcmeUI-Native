# AcmeUI-Native 優化待辦清單

> 建議在新增任何新 component 前，先完成 P0。  
> 狀態標記：`[ ]` 未開始、`[-]` 進行中、`[x]` 完成。

## P0 — Correctness / Architecture

### ACME-P0-001 Ordered Display List
- [ ] 建立 backend-neutral `DrawCommand` / `Scene`
- [ ] 保證 quad、text、clip、overlay 的原始順序
- [ ] renderer 只合併相鄰 compatible commands
- [ ] 移除以 HashMap iteration 決定 draw order
- [ ] 加入 overlay/text/clip golden regression

**驗收：** 同一 scene 連續 100 次產出相同像素與 batch order。

### ACME-P0-002 RuntimeTree Integration
- [ ] 建立 `RuntimeTree<M>`
- [ ] retained node 保存 key、kind、props、style、state、layout handle
- [ ] Gallery 改用 keyed reconciliation
- [ ] stable NodeId 貫穿 layout/hit-test/accessibility/render
- [ ] dirty flags 實際控制局部更新

**驗收：** keyed sibling reorder 後 identity 與 focus 不變。

### ACME-P0-003 Component Maturity Audit
- [ ] 定義 S0～S4 成熟度
- [ ] audit 全部 component files
- [ ] README 顯示各成熟度數量
- [ ] placeholder component 移到 experimental feature
- [ ] 暫停新增元件直到核心 15 個 controls 達 S3

### ACME-P0-004 Slider
- [ ] value/min/max/step normalize
- [ ] fill width 與 thumb position
- [ ] pointer drag
- [ ] keyboard Arrow/Home/End
- [ ] disabled
- [ ] on_change message
- [ ] accessible role/value/actions
- [ ] golden + behavior tests

### ACME-P0-005 DatePicker
- [ ] 42-cell calendar grid
- [ ] month 1～12 validation
- [ ] year underflow protection
- [ ] prev/next message
- [ ] day selection message
- [ ] keyboard grid navigation
- [ ] locale/week-start abstraction
- [ ] today provider
- [ ] accessibility semantics
- [ ] six-week-month regression tests

### ACME-P0-006 TextInput Undo/Redo
- [ ] 修正 redo 推回 undo stack 的 transaction direction
- [ ] 測試 insert→undo→redo→undo
- [ ] 測試 selection replacement sequence
- [ ] 測試 IME commit undo/redo
- [ ] 使用 property-based state machine tests

### ACME-P0-007 Unified Platform Events
- [ ] 停止同一 OS event 同時 dispatch legacy + detailed
- [ ] 統一 PointerEvent
- [ ] 統一 ImeEvent
- [ ] 統一 modifiers
- [ ] 提供明確 compatibility adapter
- [ ] deprecate legacy variants

### ACME-P0-008 Accessibility Truthfulness
- [ ] STATUS 從 Stable 調整為 Experimental
- [ ] AccessNodeId 使用 retained NodeId
- [ ] action 保留 target NodeId
- [ ] 移除 `(0,0)` click 模擬
- [ ] 移除 `node_id: 0` placeholder
- [ ] 接入 `accesskit_winit` lifecycle
- [ ] Windows Narrator manual test

### ACME-P0-009 Status Single Source
- [ ] 建立 `project-status.yaml`
- [ ] 自動產生 STATUS/README summary
- [ ] CI 檢查 todos/status/manual validation 一致性
- [ ] component maturity 由 manifest 產生
- [ ] release note 不可覆蓋 manual pending

## P1 — Runtime / Performance

### ACME-P1-001 Persistent Taffy
- [ ] runtime node 保存 Taffy handle
- [ ] mount/remove/update children 增量同步
- [ ] 只對 layout dirty subtree 計算
- [ ] text measurement cache
- [ ] 10k node benchmark

### ACME-P1-002 RetainedTree Hardening
- [ ] O(n²) `ids.contains` 改 HashSet
- [ ] key + kind identity policy
- [ ] props/style diff
- [ ] kind change state reset
- [ ] deep tree iterative path 或 depth limit
- [ ] fuzz duplicate/reorder/remove

### ACME-P1-003 Canonical Scene
- [ ] 合併 `acme-core::Scene` 與 renderer `Frame`
- [ ] 移除重複 PreparedGlyph/AtlasUpload model
- [ ] Renderer public API 只接受 backend-neutral scene
- [ ] clip stack validation

### ACME-P1-004 Component Conformance Suite
- [ ] pointer
- [ ] keyboard
- [ ] focus
- [ ] disabled
- [ ] controlled value
- [ ] event target
- [ ] accessibility
- [ ] DPI
- [ ] golden
- [ ] invalid input

### ACME-P1-005 Benchmarks
- [ ] Criterion/iai
- [ ] reconcile benchmark 使用正式 RuntimeTree
- [ ] layout cold/warm/partial
- [ ] text shaping cache hit/miss
- [ ] display-list batching
- [ ] renderer prepare
- [ ] machine-readable JSON
- [ ] nightly regression comparison

### ACME-P1-006 GPU Recovery
- [ ] async recovery state
- [ ] UI event loop 不 block
- [ ] retry/backoff
- [ ] multi-window device ownership policy
- [ ] recovery diagnostics
- [ ] real hardware sign-off

### ACME-P1-007 Input Model
- [ ] Up/Down/PageUp/PageDown
- [ ] physical key/code
- [ ] repeat
- [ ] alt/meta
- [ ] numpad
- [ ] dead keys
- [ ] scale factor event
- [ ] file drop redraw
- [ ] separate window focus/widget focus

## P2 — CI / Release / DX

### ACME-P2-001 CI
- [ ] `--locked`
- [ ] all-features
- [ ] feature powerset
- [ ] rustdoc/doctest
- [ ] pinned audit/deny tools
- [ ] timeout/concurrency
- [ ] test reports
- [ ] screenshot artifacts
- [ ] benchmark artifacts

### ACME-P2-002 Crate Metadata
- [ ] publish policy
- [ ] repository/homepage/docs/readme
- [ ] description/keywords/categories
- [ ] semver/API policy
- [ ] changelog
- [ ] release profile review

### ACME-P2-003 Error Handling
- [ ] component build validation
- [ ] layout errors 不直接 panic
- [ ] renderer recovery fallback
- [ ] diagnostics overlay
- [ ] structured error codes

### ACME-P2-004 Theme Tokens
- [ ] on_success
- [ ] on_warning
- [ ] on_danger
- [ ] on_info
- [ ] high contrast checks
- [ ] 禁止元件直接 RGB

## v0.1 Release Gate

- [ ] RuntimeTree 已接入正式 Gallery
- [ ] ordered display list
- [ ] 15 個 S3 control
- [ ] screenshot golden CI
- [ ] benchmark baseline
- [ ] Narrator basic PASS
- [ ] 注音 IME PASS
- [ ] 100/125/150/200% DPI PASS
- [ ] GPU recovery PASS 或正式 fallback policy
- [ ] docs/status sync gate
- [ ] fmt/check/clippy/test/doc/audit/deny 全過
