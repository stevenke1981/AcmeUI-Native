# AcmeUI-Native 專案分析與優化建議

- 分析日期：2026-07-21
- Repository：`stevenke1981/AcmeUI-Native`
- 分析分支：`master`
- 分析快照：最新檢視 commit `70cc064cab945824e4243907fcb1281e7a4bdfba`
- 分析方式：GitHub repository metadata、主要文件、CI、核心 crate 與代表性元件的靜態檢視

> 限制：本次環境無法直接 clone repository，因此未能親自執行 `cargo check/test/clippy`、GPU 畫面、IME 或效能 benchmark。最新 commit 訊息宣稱 503 tests pass，但本報告未獨立重跑驗證。以下將「程式碼可直接確認」與「需要實機驗證」分開描述。

---

## 1. 結論摘要

AcmeUI-Native 已建立很有潛力的 Rust 原生 UI 框架骨架：

- crate 邊界清楚，涵蓋 core、layout、text、renderer、platform、widgets、accessibility、devtools。
- 對 Windows、wgpu、CJK、繁體中文 IME、GPU recovery 有明確產品方向。
- 多數核心模組禁止或限制 unsafe，並已有大量單元測試與手動驗證清單。
- Gallery 已從單一大型檔案拆成事件、渲染、頁面與 helper 層。

但目前最大的問題不是「元件還不夠多」，而是：

1. **retained-mode、stable identity、dirty propagation 尚未真正接入主要 runtime pipeline。**
2. **元件數量快速擴張，但相當一部分仍是視覺 scaffold，公開 API 與實際行為不一致。**
3. **renderer 的資料結構會破壞原始繪製順序，複雜畫面可能出現 z-order 錯誤。**
4. **Accessibility 目前偏向資料模型 scaffold，尚未形成可靠的作業系統整合。**
5. **文件、todos、STATUS 與程式碼已發生明顯漂移。**
6. **測試數量不少，但元件測試偏重「能建構、非零 layout」，不足以證明互動正確。**

### 建議立即改變開發策略

暫停新增元件 1～2 個里程碑，先進入 **Stabilization / Runtime Integration** 階段：

- 先修 correctness。
- 將核心 retained tree 接入 Gallery 與 runtime。
- 定義元件成熟度分級。
- 統一 ordered display list。
- 完成 AccessKit 真實平台接線。
- 建立可重現 benchmark 與 visual regression。

在上述工作完成前，專案較適合標示為 **early alpha / framework prototype**，不建議對外宣稱核心與 110 個元件皆已 stable。

---

## 2. 專案優點

### 2.1 架構方向正確

目前 workspace 已拆分為多個責任清楚的 crate：

- `acme-core`
- `acme-layout`
- `acme-text`
- `acme-render-wgpu`
- `acme-platform`
- `acme-widgets`
- `acme-ui`
- `acme-textinput`
- `acme-accessibility`
- `acme-animation`
- `acme-theme`
- `acme-style`
- `acme-devtools`

這種分層有利於：

- 對核心資料結構做純單元測試。
- 讓 renderer、platform 與 widget DSL 分離。
- 未來增加其他 renderer backend 或 headless testing。
- 將平台相關型別限制在 platform/render crate。

### 2.2 對高風險領域有誠實的手動驗證規則

`docs/MANUAL_VALIDATION.md` 對 GPU device loss 與繁體中文注音 IME 明確要求人工作業與簽核，這是很好的工程習慣。

應保留這種做法，並擴充到：

- 高 DPI。
- AccessKit + Narrator。
- 多螢幕、多 GPU。
- 睡眠/喚醒。
- 截圖 golden。
- 複雜 overlay/z-order。
- 鍵盤與滑鼠完整互動。

### 2.3 TextInput 已有較完整的狀態機雛形

`acme-textinput` 涵蓋：

- grapheme cursor。
- selection。
- clipboard。
- IME preedit/commit。
- password masking。
- undo/redo。
- readonly/invalid。
- caret geometry。

它是目前較接近「可持續硬化」的子系統，但仍需修補 undo/redo sequence 與加入 property-based tests。

### 2.4 Gallery 模組化已有進展

Gallery 從大型 `main.rs` 拆成：

- `events/`
- `render/`
- `pages/`
- `helpers.rs`
- `types.rs`

這降低了單一檔案負擔。下一步應從「把函式移到不同檔案」提升到「讓 runtime 本身承擔 reconciliation、state、hit testing、focus 與 rendering」。

---

## 3. P0：必須優先修正

## P0-01：Retained Tree 尚未接入主要 runtime

### 已確認現況

`apps/gallery/src/main.rs::frame()` 每個 frame 都執行：

1. `self.description()` 重建完整 `WidgetNode`。
2. `to_layout_with_context(NodeId::new(1), ...)` 依 DFS 重新配置 NodeId。
3. `LayoutEngine::compute_with_text()` 新建並計算完整 Taffy tree。
4. 重新 DFS 收集 hit regions。
5. 重新建立 accessibility tree。
6. 重新產生 Frame。

另一方面，`acme-core::RetainedTree`、`DirtyFlags`、`ReconcileReport` 並未成為 Gallery 主流程的一部分。

### 影響

- 所謂 stable NodeId 實際上只是「當前 DFS 順序 ID」，不是由 key 穩定維持。
- 插入一個前置節點會讓大量後續 ID 改變。
- dirty flags 無法驅動局部 layout/paint/semantics 更新。
- focus、hover、accessibility identity 容易隨樹結構改變而漂移。
- 每個 frame 重新配置 tree、HashMap、String、Taffy node 與 shaped text。
- benchmark 中的 reconciliation 與實際 app pipeline 脫節。

### 改善方向

建立真正的 `RuntimeTree<M>`：

```text
Widget description
  -> keyed reconciliation
  -> retained RuntimeNode
  -> ChangeSet
       - mounted
       - removed
       - props_changed
       - layout_dirty
       - paint_dirty
       - semantics_dirty
  -> persistent layout nodes
  -> ordered display list
```

每個 runtime node 至少需要：

- `NodeId`
- `WidgetKey`
- `WidgetKind`
- normalized props/style hash 或版本
- parent/children
- layout handle
- interaction state
- focus/hover/pressed/disabled
- accessibility metadata
- dirty flags
- cached text/layout/paint fragment

### 驗收條件

- keyed reorder 後 NodeId 穩定。
- 在第一個 sibling 前插入節點，不影響其他 keyed sibling ID。
- 修改 label 文字只標記該節點的 text/layout/paint。
- hover 改變只更新必要 paint fragment。
- Gallery 不再直接以 DFS index 當穩定 identity。
- benchmark 量測的 reconciliation 就是正式 runtime 使用的 reconciliation。

---

## P0-02：Renderer 會破壞原始繪製順序

### 已確認現況

`acme-render-wgpu::Frame` 將內容分成：

- `quads`
- `clipped_quads`
- `text`

render 時再分別處理：

1. 所有 regular quads。
2. 所有 clipped quads。
3. 所有 text。

另外 clipped quad 與 text 又用 `HashMap` 依 clip/format 分組後迭代。

### 影響

這無法保留 UI 原始 display order：

```text
背景 quad
文字
半透明 overlay quad
overlay 文字
```

可能被重排為：

```text
所有 regular quad
所有 clipped quad
所有文字
```

可能造成：

- tooltip、menu、modal、selection、caret 的 z-order 錯誤。
- 文字永遠在某些 quad 之上。
- HashMap iteration 導致跨執行或不同平台順序不穩定。
- 截圖 golden 難以穩定。
- overlay layer 雖在概念上存在，但 backend 無法完整表達順序。

### 改善方向

改為單一有序 display list：

```rust
enum DrawCommand {
    Quad(Quad),
    Text(TextRun),
    PushClip(Rect),
    PopClip,
    BeginLayer(LayerId),
    EndLayer,
}
```

renderer 只能合併「相鄰且相容」的 commands，不可跨越其他 command 重排。

可進一步編譯成：

```text
Ordered Display List
  -> stable adjacent batching
  -> RenderBatch[]
  -> wgpu passes
```

### 驗收條件

建立 golden scene：

1. 背景。
2. text A。
3. overlay quad。
4. text B。
5. clipped child。
6. floating tooltip。

確認所有 backend 與多次執行皆保持完全相同 z-order。

---

## P0-03：元件 API 與實際功能不一致

### Slider 的已確認問題

`crates/acme-ui/src/inputs/slider.rs` 暴露：

- `value`
- `min`
- `max`
- `step`
- `show_value`
- `size`
- `on_change`

但 `From<SliderBuilder<M>> for WidgetNode<M>` 只建立三個 generic Card，沒有使用上述欄位計算：

- fill width。
- thumb position。
- step。
- label。
- interaction。
- event dispatch。

這不是可互動 Slider，只是 slider-shaped scaffold。

### DatePicker 的已確認問題

`crates/acme-ui/src/inputs/date_picker.rs`：

- `on_change` 被保存但未接到 day cell。
- prev/next button 沒有 message。
- 固定 `total_cells = 35`，會漏掉需要 6 週、42 格的月份。
- `month` 沒有限制為 1～12。
- `year = 0` 且 month < 3 時，`year - 1` 有 underflow 風險。
- 預設固定為 2025-01，沒有 calendar provider 或 today abstraction。
- day cell 使用 Card，缺少鍵盤、focus、selected semantics。

### 最新新增元件也包含 placeholder

最新 commit 的程式碼中，部分 chart 明確註明是 layout-level placeholder；ImageGallery 顯示文字 placeholder 而非 image resource，且 `columns` 欄位並未真正控制換行欄數。

### 改善方向：元件成熟度分級

每個元件必須標示：

| Level | 定義 |
|---|---|
| S0 Scaffold | 只有資料結構或視覺 placeholder |
| S1 Visual | 可正確顯示各狀態，但無完整互動 |
| S2 Interactive | pointer/keyboard/message/state 完整 |
| S3 Accessible | role/name/value/action/focus 正確 |
| S4 Production | edge cases、golden、性能、手動驗證完成 |

README 不應只計算「檔案數」，應列出每個 level 的數量。

### 建議短期策略

- 暫停新增元件。
- 將目前 110 個 component files 做 maturity audit。
- 未達 S2 的元件，不應宣稱為完整元件。
- charts、browser、mobile 類 placeholder 可改名為 `experimental-*` feature。
- 優先完成 15～20 個真正可用的 desktop core controls。

### 建議優先完成的元件

1. Button
2. TextInput
3. Checkbox
4. RadioGroup
5. Switch
6. Slider
7. Select
8. Combobox
9. ScrollView
10. Tabs
11. Menu
12. Dialog
13. Tooltip
14. Tree
15. Table

---

## P0-04：Accessibility 目前不是可靠的 OS 整合

### 已確認問題

`crates/acme-accessibility/src/lib.rs`：

- 每次 update 依 DFS 從 1 開始重新產生 AccessNodeId。
- `route_action(Focus(id))` 忽略 target，送出 `node_id: 0`。
- `route_action(Click(id))` 忽略 target，模擬 `(0,0)` click。
- `SetValue(id, value)` 忽略 target，轉成泛用 IME commit。
- `ScrollIntoView`、`Activate` 未接線。
- `initial_tree` 使用固定 1080×720。
- crate 宣告 `accesskit_winit` dependency，但目前主要 lib 並未形成真正 winit adapter lifecycle。
- Gallery 只呼叫內部 `AccessibilityAdapter::update`，沒有看到完整 OS event-loop adapter wiring。

### 影響

- Narrator/VoiceOver/Orca 無法可靠追蹤同一元件。
- accessibility action 可能作用到錯誤 widget。
- 目前應視為 scaffold，而不是 stable AccessKit support。

### 改善方向

- AccessNodeId 必須直接由 retained NodeId 轉換。
- action 必須路由為 `WidgetAction { target: NodeId, action }`，不可偽裝成座標 click。
- platform runtime 持有每個 window 的 AccessKit adapter。
- Window event loop 處理 accessibility action requests。
- 由 widget 提供 role、label、value、state、actions。
- 建立 Windows Narrator 手動測試矩陣。
- 在穩定前將 STATUS 改為 Experimental。

---

## P0-05：PlatformEvent 同時發送 legacy 與 detailed 事件

### 已確認現況

`acme-platform` 對同一個 OS 事件會連續呼叫 app 兩次：

- `PointerButton` + `PointerButtonDetailed`
- `ImePreedit` + `ImePreeditDetailed`
- `ImeCommit` + `ImeCommitDetailed`

Gallery 目前主要處理 legacy variant，但第三方 app 若同時 match 兩者，很容易重複點擊、重複 commit 或重複狀態變更。

### 改善方向

選擇一種做法：

**建議：在 0.x 階段直接統一事件 API。**

```rust
PlatformEvent::PointerButton {
    window,
    pointer,
    button,
    state,
    position,
    modifiers,
}
```

```rust
PlatformEvent::Ime {
    window,
    kind: ImeEvent,
}
```

若必須保留相容性：

- legacy event 透過 opt-in compatibility adapter 轉換。
- runtime 不可同時 dispatch 兩份事件。
- 加入 deprecation deadline。

---

## P0-06：TextInput redo 後再 undo 的 transaction 方向疑似錯誤

### 已確認程式邏輯

`redo()` 從 redo stack 取出 old→new transaction 後，將欄位反轉再推回 undo stack。

但 `undo()` 的語義是恢復 `old_text`。因此：

```text
insert
undo
redo
undo
```

最後一次 undo 很可能恢復到當前 new state，形成 no-op，而不是回到 old state。

現有測試只測到：

```text
insert a/b/c -> undo -> undo -> redo
```

沒有覆蓋 redo 後再次 undo。

### 立即加入的 regression test

```rust
#[test]
fn undo_after_redo_restores_previous_state() {
    let mut s = TextInputState::new();
    s.insert_char('a');
    assert!(s.undo());
    assert_eq!(s.text, "");
    assert!(s.redo());
    assert_eq!(s.text, "a");
    assert!(s.undo());
    assert_eq!(s.text, "");
}
```

### 建議修法

redo 後應將原 transaction 推回 undo stack，而不是將 old/new 反轉；或重新定義兩個 stack 都儲存 snapshot command，並以明確 direction 執行。

---

## P0-07：文件、STATUS、todos 與程式碼已漂移

### 已確認例子

`STATUS.md` 寫：

- 尚未有 cargo-deny / cargo-audit。
- 尚未有 Windows 以外 CI matrix。
- glyph atlas eviction/aging 只有 architecture。

但：

- `.github/workflows/ci.yml` 已有 Windows/macOS/Ubuntu matrix。
- CI 已執行 `cargo deny check` 與 `cargo audit`。
- `GlyphAtlas` 已有 `begin_frame`、`touch`、`stale_count`、`evict_stale`。

反過來，也有文件宣稱 Stable，但程式碼其實仍是 scaffold，例如 Accessibility。

### 改善方向

建立單一可信來源：

```yaml
# project-status.yaml
areas:
  accessibility:
    maturity: scaffold
    automated: partial
    manual: not_started
  ime:
    maturity: alpha
    automated: wired
    manual: pending
```

由 script 自動產生：

- STATUS.md
- README status table
- todos summary
- release checklist

CI 加入 `scripts/check_status_sync`，發現矛盾即失敗。

---

## 4. P1：高優先改善

## P1-01：LayoutEngine 每次重建完整 TaffyTree

`LayoutEngine::compute*()` 每次都：

- `TaffyTree::new()`
- 遞迴建立全部 nodes
- 建立 HashMap
- 全樹 layout
- 全樹 collect

這不符合 retained UI 的長期性能目標。

### 建議

- retained node 保存 `taffy::NodeId`。
- mounted/removed/children/style changes 才更新 Taffy tree。
- 只在 constraints 或 layout dirty 時 compute。
- 對 text measurement 建立 cache key：

```text
(text hash, font family, size, line height, width constraint, wrap, scale)
```

---

## P1-02：核心 Scene 與 renderer Frame 重複、架構分裂

`acme-core::Scene` 已有 ordered `PaintCommand`、clip stack、PreparedGlyph、AtlasUpload。

但實際 `Application::frame()` 回傳的是 `acme-render-wgpu::Frame`，又定義另一套：

- Quad
- ClippedQuad
- TextRun
- PreparedText

這造成：

- documented architecture 與真正 pipeline 不一致。
- ordered Scene 沒有成為 renderer input。
- text/atlas types 重複。
- core 與 backend 邊界不清。

### 建議

將 `acme-core::Scene` 演進為 canonical backend-neutral display list，renderer 只接受：

```rust
fn render(&mut self, scene: &Scene) -> Result<PresentResult, RenderError>
```

---

## P1-03：RetainedTree reconciliation 需要硬化

`acme-core/src/tree.rs` 的風險：

1. 移除判斷使用 `ids.contains(id)`，大量 sibling 時可能形成 O(n²)。
2. 同 key、不同 kind 仍沿用 NodeId，可能把舊 widget state 留給新 widget kind。
3. 只比較 kind/focusable/disabled，沒有比較 props/style/content。
4. 深樹使用 recursive reconcile/remove，極端情況有 stack depth 風險。
5. `ViewNode.kind` 使用 String，容易產生配置與拼字錯誤。

### 建議

- `new_ids: HashSet<NodeId>` 做 O(1) membership。
- identity 定義為 `(key, widget_type_id)`。
- widget kind 改為 enum 或 stable type tag。
- props 使用 typed payload/version/hash。
- 明確定義 kind change 是 replace 或 state reset。
- 加入 10k sibling benchmark 與深度 fuzz test。

---

## P1-04：Benchmark 不是真正 headless，也不具統計可信度

`apps/benchmark`：

- 仍啟動 `acme_platform::run()`、window 與 GPU。
- 在 `frame()` 內 `std::process::exit(0)`。
- 使用單次 `Instant`。
- 沒有 Criterion、warm sample、variance、black_box。
- frame build 的 text run 使用空 glyph，不能反映實際 text cost。
- 不產出 machine-readable baseline。
- 無 CI threshold。

### 建議

拆成：

```text
benches/reconcile.rs
benches/layout.rs
benches/text.rs
benches/display_list.rs
benches/render_prepare.rs
```

使用 Criterion 或 iai-callgrind，另保留獨立 GPU smoke benchmark。

CI：

- PR：只跑短 benchmark smoke。
- nightly：跑完整 benchmark，輸出 JSON。
- 與基準分支比較，超過 10～15% 回歸才警告或失敗。

---

## P1-05：測試數量高，但元件測試品質不均

代表性元件測試大量集中在：

- builder default 是否保存。
- variant 是否正確。
- layout rect 是否 > 0。
- child count 是否符合。

這類測試可以防止 compile-level regression，但不能證明：

- pointer interaction。
- keyboard interaction。
- message dispatch。
- controlled state。
- disabled/read-only。
- focus。
- accessibility。
- visual state。
- edge cases。

### 建議元件契約測試

每個 interactive component 至少共用以下 conformance suite：

1. default render。
2. every visual state。
3. pointer activation。
4. keyboard activation。
5. disabled blocks action。
6. controlled value update。
7. event target 正確。
8. focus order。
9. accessibility role/name/value/action。
10. DPI 100/125/150/200。
11. golden image。
12. invalid input normalization。

---

## P1-06：Renderer 統計語義不一致

`RenderStats` 中：

- `bytes_uploaded`、`buffer_grows` 看起來累積。
- `draw_calls`、`quad_count`、`glyph_count` 每 frame 覆寫。
- `atlas_hit_rate` 是每 frame。

同一 struct 混合 lifetime cumulative 與 per-frame metric，容易讓 devtools 解讀錯誤。

### 建議

拆成：

```rust
FrameRenderStats
RendererLifetimeStats
```

並加入 frame number、CPU prepare time、GPU submit time、atlas occupancy、batch count。

---

## P1-07：GPU recovery 在 event thread 同步 block

`Renderer::on_device_lost()` 使用 `pollster::block_on` 重新 request adapter/device。

可能造成：

- UI event loop 長時間卡住。
- driver recovery 慢時視窗無回應。
- 多 window 各自 recovery 時更嚴重。

### 建議

- runtime state：Ready / Recovering / Failed。
- recovery 非同步執行。
- recovery 期間顯示 fallback frame 或暫停 redraw。
- 支援 retry/backoff。
- device 共享策略需明確：per-window device 或 per-adapter shared device。

---

## P1-08：Widget-to-layout conversion 存在 panic 與 identity 風險

`WidgetNode::to_layout_alloc()` 中 Popover 直接取 `v.children[0]`。

如果 builder 可產生空 children，會 panic。

此外 tooltip/popover 透過回退 `next` 重用 child ID，這種 layout-transparent 特例會讓 widget tree、layout tree、accessibility tree 的 walk 規則更難保持一致。

### 建議

- 所有 component build 先 validate。
- layout conversion 回傳 `Result<LayoutNode, WidgetBuildError>`。
- 建立 canonical compiled tree，一次決定 identity、layout transparency、semantics。
- 不要讓 layout、accessibility、hit-test 各自重新猜測 DFS mapping。

---

## P1-09：Platform input model 不完整

目前 PlatformKey 主要只有：

- Tab、Enter、Space、Escape
- Left/Right
- Backspace/Delete
- Home/End
- Other

需要補：

- ArrowUp/Down
- PageUp/PageDown
- Insert
- Function keys
- physical key/code
- repeat
- alt/meta
- numpad
- dead keys
- composition-aware text input separation

其他問題：

- `ScaleFactorChanged` 沒有將完整 resize/scale event送給 app。
- FileDropped 的 dirty return 被忽略。
- FocusChanged 的 `node_id` 固定 0，混淆 window focus 與 widget focus。

### 建議

分開：

```text
WindowEvent
PointerEvent
KeyboardEvent
TextInputEvent
ImeEvent
AccessibilityAction
```

---

## 5. P2：中期改善

### 5.1 CI 可重現性

目前 CI 建議加入：

- `cargo check/test --locked`
- `cargo test --workspace --all-features`
- `cargo hack check --feature-powerset` 或合理 feature matrix
- `cargo doc --workspace --no-deps`
- doctest
- 固定 cargo-audit/cargo-deny 版本或使用預建 action
- `cargo install --locked`
- CI concurrency / cancel-in-progress
- job timeout
- artifact：test report、benchmark JSON、screenshots
- Windows GPU/IME manual workflow dispatch

### 5.2 Public crate metadata

若未準備發布 crates.io：

```toml
publish = false
```

若準備發布，workspace package 應補：

- repository
- homepage
- documentation
- readme
- description
- keywords
- categories
- authors（依專案政策）
- license files

並建立每個 crate 的 public API policy。

### 5.3 錯誤處理

Gallery 中 `expect("finite Gallery viewport")` 適合 demo，但 framework runtime 應：

- 將 layout/render error 交給 app error hook。
- dev build 顯示 diagnostics overlay。
- release build 可 graceful fallback。
- 不因一個非法元件值整個 panic abort。

### 5.4 Theme token discipline

`acme-ui::resolve_tone()` 仍直接建立白色：

```rust
ThemeColor::rgb(255, 255, 255)
```

這與「所有顏色由 semantic tokens」的設計原則不完全一致。

應補：

- `on_success`
- `on_warning`
- `on_danger`
- `on_info`

並由 theme/high-contrast theme 決定。

### 5.5 API 限制與 builder validation

Builder 目前常保存無效值，或只做局部 clamp。

建議統一：

```rust
build() -> Result<WidgetNode<M>, BuildError>
```

或：

```rust
build_checked()
build_unchecked() // internal only
```

對日期、range、size、ratio、columns、selection、index 做一致驗證。

---

## 6. 建議目標架構

```text
Application state
  |
  v
View / Widget description
  |
  v
Keyed reconciler
  |
  +--> Retained RuntimeTree<NodeId>
  |      - widget props
  |      - interaction state
  |      - semantic state
  |      - layout handle
  |      - dirty flags
  |
  v
ChangeSet
  |
  +--> Persistent Taffy tree updates
  +--> Focus/hit-test index updates
  +--> AccessKit tree diff
  +--> Text/layout cache invalidation
  |
  v
Ordered backend-neutral Scene / DisplayList
  |
  v
Stable adjacent batching
  |
  v
wgpu Renderer
```

### 核心原則

1. Identity 只決定一次。
2. Widget、layout、hit-test、semantics 共用同一 NodeId。
3. Rendering 不可改變 display order。
4. Component event 以 target NodeId/message 路由，不依賴 Gallery 自訂 magic state。
5. Cache 與 invalidation 必須由 ChangeSet 驅動。
6. Scaffold 與 production component 必須明確區分。

---

## 7. 建議實施順序

### Milestone A：Correctness Freeze

- 暫停新增元件。
- Slider、DatePicker、undo/redo 修正。
- ordered display list。
- 移除 legacy + detailed 雙 dispatch。
- Accessibility 降級標示為 experimental。
- STATUS 自動同步。
- 為所有 P0 問題加 regression test。

### Milestone B：Runtime Integration

- `RuntimeTree<M>`。
- keyed reconciliation 接 Gallery。
- stable NodeId 貫穿 layout/hit/accessibility/render。
- persistent Taffy tree。
- dirty-driven frame build。
- text measurement cache。

### Milestone C：Interaction + Accessibility

- 統一 WidgetAction。
- focus manager 接 runtime。
- pointer capture。
- keyboard navigation。
- AccessKit winit adapter。
- Narrator validation。
- 15 個核心 control 達到 S3。

### Milestone D：Performance + Release Hardening

- Criterion benchmarks。
- visual golden。
- feature matrix CI。
- benchmark thresholds。
- release metadata。
- semver/API review。
- Windows high-DPI/IME/GPU sign-off。

---

## 8. 建議重新定義 v0.1 完成條件

v0.1 不應以 component file 數量定義，而應至少滿足：

- retained tree 真正接入正式 app。
- stable keyed identity。
- ordered rendering。
- 15 個 S3 controls。
- Windows Narrator 基本可用。
- 繁中 IME 人工 PASS。
- 100/125/150/200% DPI 人工 PASS。
- GPU recovery 人工 PASS 或有明確 restart fallback。
- screenshot golden 進 CI。
- benchmark 有 baseline。
- docs/status 由單一資料來源生成。
- `fmt/check/clippy/test/doc/audit/deny` 全部穩定。
- 至少一個非 Gallery 的完整示範 app。

---

## 9. 主要證據位置

| 類別 | 檔案 / Symbol |
|---|---|
| Workspace/依賴 | `Cargo.toml` |
| 專案宣稱 | `README.md`, `STATUS.md`, `todos.md` |
| CI | `.github/workflows/ci.yml` |
| Retained tree | `crates/acme-core/src/tree.rs` |
| 重複 Scene abstraction | `crates/acme-core/src/scene.rs` |
| 每 frame layout rebuild | `crates/acme-layout/src/lib.rs::LayoutEngine::compute*` |
| Renderer ordering | `crates/acme-render-wgpu/src/lib.rs::Frame`, `Renderer::render` |
| Widget DFS ID | `crates/acme-widgets/src/lib.rs::to_layout_alloc` |
| 主 pipeline | `apps/gallery/src/main.rs::frame` |
| Platform 雙事件 | `crates/acme-platform/src/lib.rs::window_event` |
| Accessibility scaffold | `crates/acme-accessibility/src/lib.rs` |
| Slider scaffold | `crates/acme-ui/src/inputs/slider.rs` |
| DatePicker correctness | `crates/acme-ui/src/inputs/date_picker.rs` |
| TextInput undo/redo | `crates/acme-textinput/src/lib.rs::{undo,redo}` |
| Atlas aging | `crates/acme-text/src/lib.rs::GlyphAtlas` |
| Benchmark | `apps/benchmark/src/main.rs` |
| Manual validation | `docs/MANUAL_VALIDATION.md` |

---

## 10. 最終判斷

AcmeUI-Native 不是需要「再多加 50 個元件」，而是需要把已存在的好骨架收斂成一條真正一致的 runtime。

目前最有價值的投資是：

1. **整合 retained tree。**
2. **保證 display order。**
3. **讓 component API 真正可互動。**
4. **讓 accessibility action 精確命中 target。**
5. **把測試從結構測試提升為行為契約測試。**
6. **讓專案狀態文件自動反映實作。**

完成這些工作後，AcmeUI-Native 才會從「功能廣泛的 prototype」跨入「可供實際桌面 app 使用的 alpha framework」。
