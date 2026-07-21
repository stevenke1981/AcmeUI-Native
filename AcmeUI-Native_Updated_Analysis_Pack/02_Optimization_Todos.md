# AcmeUI-Native 更新版優化待辦

> 建議凍結新元件功能，先完成本清單 P0。

## P0-A：立即 correctness 修正

### ACME-P0-A01 Slider Percentage
- [ ] `Length::Percent(fill_pct)` 改成 `Length::Percent(ratio)`
- [ ] track 使用 `1.0 - ratio`
- [ ] 為 Length::Percent 補 0..1 文件
- [ ] 搜尋全 repository 的 Percent usage
- [ ] 加 actual LayoutEngine width test

**驗收：**

- 200px Slider、value 50，fill rect 約 100px。
- value 0/100 分別為 0/200px。
- 不得只檢查 WidgetNode child count。

### ACME-P0-A02 Slider Range Safety
- [ ] reject/normalize NaN
- [ ] normalize min > max
- [ ] normalize step <= 0
- [ ] normalize infinite values
- [ ] actual value label test
- [ ] no panic property test

### ACME-P0-A03 DatePicker Valid Date
- [ ] selected_day clamp 或回傳 BuildError
- [ ] February 31 regression
- [ ] prev message
- [ ] next message
- [ ] day selection message
- [ ] closed card open message
- [ ] keyboard grid navigation

### ACME-P0-A04 Accessibility Action
- [ ] 新增 target-based UiAction
- [ ] Click 不使用 (0,0) pointer
- [ ] Click/Activate 直接命中 NodeId
- [ ] SetValue 保留 target
- [ ] ScrollIntoView 實作
- [ ] Gallery 處理 accessibility target
- [ ] action exactly-once tests

### ACME-P0-A05 Scene Model
- [ ] Scene 改為單一 `Vec<DrawCommand>`
- [ ] 加 `clear: Color`
- [ ] 移除獨立 `draw_commands`
- [ ] legacy PaintCommand 立即轉換到同一 vector
- [ ] AtlasUpload 加 format + pixels
- [ ] RGBA emoji upload test
- [ ] clip/layer balance validation

### ACME-P0-A06 Component Audit Repair
- [ ] S2 必須證明 build 消費 on_* field
- [ ] S2 必須證明 runtime 可 dispatch
- [ ] Slider 降為 S1
- [ ] DatePicker 降為 S1
- [ ] 修正不存在的 `on_open`
- [ ] 統一 component count 為實際值
- [ ] 由 source manifest 自動生成 audit

### ACME-P0-A07 Docs Integrity
- [ ] 修 `docs/MANUAL_VALIDATION.md` 亂碼
- [ ] 修 `docs/component_maturity_audit.md` 控制字元
- [ ] 決定並真正移動 docs 路徑
- [ ] 更新所有 relative links
- [ ] 更新 README repository tree
- [ ] 更新 AGENTS canonical paths
- [ ] CI 加 UTF-8/control char scan
- [ ] CI 加 markdown link checker

### ACME-P0-A08 Status Sync
- [ ] P0-009 改 partial
- [ ] generator 讀 project-status.yaml
- [ ] 產生 README status block
- [ ] 產生 STATUS table
- [ ] 產生 todos summary
- [ ] CI drift gate
- [ ] 禁止手動維護重複數字

## P0-B：完成已啟動架構

### ACME-P0-B01 Ordered Batcher
- [ ] `batch.rs`
- [ ] adjacent-only merge
- [ ] deterministic batches
- [ ] clip boundary
- [ ] layer boundary
- [ ] alpha/RGBA text pipeline
- [ ] 100-run deterministic test

### ACME-P0-B02 Renderer Scene Input
- [ ] `Renderer::render_scene`
- [ ] Frame bridge only for migration
- [ ] no HashMap ordering in hot path
- [ ] render order golden
- [ ] overlay/tooltip/modal test

### ACME-P0-B03 Gallery Scene Output
- [ ] RenderCtx 使用 Scene
- [ ] textinput 使用 Scene
- [ ] scroll 使用 PushClip/PopClip
- [ ] 移除 Frame buckets
- [ ] screenshot parity

### ACME-P0-B04 RuntimeTree Shadow
- [ ] `view_bridge.rs`
- [ ] Gallery RetainedTree field
- [ ] feature flag
- [ ] shadow reconcile logs
- [ ] duplicate key diagnostics
- [ ] synthetic data-node keys

### ACME-P0-B05 Stable ID Source
- [ ] layout 用 RuntimeTree NodeId
- [ ] hit-test 用 NodeId
- [ ] focus 用 NodeId
- [ ] accessibility 用 NodeId
- [ ] key lookup 取代 path/index
- [ ] reorder/insert stable ID tests

## P1：互動與可及性

### ACME-P1-01 Core Component S2
- [ ] Button
- [ ] TextInput
- [ ] Checkbox
- [ ] RadioGroup
- [ ] Switch
- [ ] Slider
- [ ] Select
- [ ] Combobox
- [ ] Tabs
- [ ] Menu
- [ ] Dialog
- [ ] Tooltip
- [ ] Tree
- [ ] Table
- [ ] DatePicker

每個元件需要：

- pointer
- keyboard
- disabled
- focus
- controlled value
- message dispatch
- actual behavior tests

### ACME-P1-02 Core Component S3
- [ ] role
- [ ] name
- [ ] value
- [ ] state
- [ ] actions
- [ ] focus
- [ ] Narrator smoke

### ACME-P1-03 Platform Input
- [ ] ArrowUp/Down
- [ ] PageUp/PageDown
- [ ] physical key
- [ ] repeat
- [ ] alt/meta
- [ ] unknown mouse button enum
- [ ] WindowFocused 與 WidgetFocused 分離
- [ ] FileDropped dirty/redraw
- [ ] scale change event

## P2：測試、CI、效能

### ACME-P2-01 Behavior Tests
- [ ] actual layout rect
- [ ] action dispatch
- [ ] stable identity
- [ ] focus stability
- [ ] clip/layer order
- [ ] accessibility target
- [ ] invalid input property tests

### ACME-P2-02 Visual Golden
- [ ] light
- [ ] dark
- [ ] high contrast
- [ ] CJK/emoji
- [ ] overlays
- [ ] 125/150/200% DPI
- [ ] artifact upload

### ACME-P2-03 Benchmark
- [ ] Criterion
- [ ] reconciliation
- [ ] layout cold/warm/partial
- [ ] text cache
- [ ] scene compile
- [ ] render prepare
- [ ] JSON output
- [ ] nightly regression

### ACME-P2-04 CI
- [ ] `--locked`
- [ ] `--all-features`
- [ ] docs/doctest
- [ ] feature matrix
- [ ] pinned audit/deny
- [ ] concurrency
- [ ] timeout
- [ ] status sync
- [ ] docs integrity
- [ ] screenshot/benchmark artifacts

## v0.1 Gate

- [ ] Ordered Scene live
- [ ] RuntimeTree live
- [ ] 15 core S2
- [ ] 10 core S3
- [ ] zero mojibake/broken docs
- [ ] CI visible on release commit
- [ ] golden baseline
- [ ] benchmark baseline
- [ ] DPI PASS
- [ ] 注音 PASS
- [ ] GPU recovery PASS/fallback
- [ ] Narrator PASS
