# AcmeUI-Native 更新版專案分析與優化建議

- 分析日期：2026-07-21
- Repository：`stevenke1981/AcmeUI-Native`
- 分支：`master`
- 最新檢視 commit：`fd0a87c0c2d30166179f3979982aa7e0767839dc`
- 上一版分析 commit：`70cc064cab945824e4243907fcb1281e7a4bdfba`
- 差異：最新分支比上一版分析前進 3 commits

---

## 1. 總結判斷

這次更新方向是正確的，確實處理了上一版報告中的多個問題：

- TextInput redo transaction direction 已修正。
- PlatformEvent 不再對同一 OS 事件同時派送 legacy 與 detailed 兩份事件。
- DatePicker 已加入 month/year normalize、42 格月份與 underflow 防護。
- Slider 已開始使用 value/min/max/step 計算視覺比例。
- RetainedTree orphan sweep 已由 `Vec::contains` 改為 `HashSet`。
- Accessibility 已誠實降級為 Experimental，Focus action 保留 target NodeId。
- `DrawCommand` 類型與 Ordered Display List 規劃已加入。
- 元件成熟度分級與 `project-status.yaml` 已建立。

但目前仍不能視為完成 Milestone A，原因包括：

1. **Slider 新增了嚴重的百分比單位錯誤。**
2. **元件成熟度 audit 將「有 on_change 欄位」誤認為「真正有事件派送」。**
3. **Ordered Display List 只有類型，renderer 與 Gallery 完全尚未接入。**
4. **Scene 同時保存兩套互不相關的 command vector，反而增加分裂風險。**
5. **Accessibility Click、SetValue 仍無法可靠命中 target。**
6. **RuntimeTree 仍只有計畫文件，Gallery 每 frame 全量重建。**
7. **文件重新分類提交未完整落實，且部分 Markdown 已出現亂碼／控制字元。**
8. **`project-status.yaml` 尚未驅動 README、STATUS、todos，仍不是實際 single source of truth。**

### 更新後成熟度判斷

| 領域 | 判斷 |
|---|---|
| 核心 crate 架構 | 良好 |
| correctness 修正速度 | 良好 |
| Runtime 整合 | 仍偏早期 |
| Renderer correctness | 高風險，尚未修正 |
| 元件互動成熟度 | 被文件高估 |
| Accessibility | Scaffold / Experimental |
| 文件可信度 | 目前偏低 |
| CI 可重現性 | 尚需補強 |
| 整體狀態 | Early Alpha |

---

## 2. 上一版問題處理狀態

| 上一版問題 | 最新狀態 | 判斷 |
|---|---|---|
| TextInput undo→redo→undo no-op | transaction direction 已修 | ✅ 已修正 |
| legacy + detailed 雙派送 | 已改成單一事件 variant | ✅ 已修正主要問題 |
| RetainedTree orphan sweep O(n²) | 使用 HashSet | ✅ 已修正 |
| DatePicker 固定 35 格 | 依月份使用 35/42 格 | ✅ 已修正 |
| DatePicker year underflow | normalize + checked_sub | ✅ 已修正 |
| Slider 完全忽略 value/range | 已加入視覺比例計算 | ⚠️ 部分修正，但出現新 bug |
| Accessibility 被標示 Stable | 已移到 Experimental | ✅ 文件局部修正 |
| Accessibility action target | Focus target 修正 | ⚠️ Click/SetValue 仍不正確 |
| Ordered display list | DrawCommand 類型加入 | ⚠️ Phase 1，不影響 live renderer |
| RuntimeTree | 新增完整計畫 | ❌ 尚未實作 |
| 狀態文件漂移 | 加入 project-status.yaml | ⚠️ 尚未自動同步 |
| 元件成熟度不透明 | 加入 audit 與 README 表格 | ⚠️ audit 方法有重大誤判 |
| Screenshot golden | 尚未接 CI | ❌ 未完成 |
| Benchmark threshold | 尚未接 CI | ❌ 未完成 |

---

# 3. P0 新發現與仍存在的問題

## P0-NEW-001：Slider 百分比單位錯誤

### 程式碼現況

`crates/acme-ui/src/inputs/slider.rs`：

```rust
let ratio = ((normalized - b.min) / range).clamp(0.0, 1.0);
let fill_pct = ratio * 100.0;
let track_pct = (1.0 - ratio) * 100.0;

.w(Length::Percent(fill_pct))
.w(Length::Percent(track_pct))
```

`acme-layout::Length::Percent` 直接傳給 Taffy：

```rust
Length::Percent(v) => percent(normalize(v))
```

Taffy 0.9 的 percentage 值使用 **0.0～1.0**，不是 0～100。

因此：

- 50% 應傳 `0.5`
- 目前卻傳 `50.0`
- 等價於 5000%

### 影響

Slider 的 fill/track 可能超出容器數十倍，造成：

- layout overflow
- 畫面不正確
- scroll metrics 異常
- hit region 錯位
- 元件測試仍通過，因為目前只檢查 tree 結構，沒有檢查實際 rect

### 修正

```rust
.w(Length::Percent(ratio))
.w(Length::Percent(1.0 - ratio))
```

並建議將 API 改得不容易誤用：

```rust
Length::Fraction(0.5)
Length::percent_100(50.0)
```

或至少為 `Length::Percent` 加上明確文件：

```rust
/// Ratio in the range 0.0..=1.0.
Percent(f32)
```

### 其他 Slider correctness 風險

```rust
b.value.clamp(b.min, b.max)
```

若：

- `min > max`
- bound 為 NaN

會 panic。還需要先 normalize finite range。

另外：

- `on_change` 只保存，完全沒有被建構輸出使用。
- 沒有 thumb。
- 沒有 pointer drag。
- 沒有 keyboard。
- 沒有 disabled。
- 沒有 accessibility value/action。

所以 Slider 目前仍應標為 **S1 Visual**，而不是 S2 Interactive。

---

## P0-NEW-002：Component Maturity Audit 有系統性誤判

`docs/component_maturity_audit.md` 使用 heuristic grep，把出現 `on_change`、`on_select` 等欄位視為 S2 證據。

但實際檢查：

### Slider

有：

```rust
pub on_change: Option<M>
```

但 `From<SliderBuilder<M>>` 完全沒有讀取 `b.on_change`。

### DatePicker

有：

```rust
pub on_change: Option<M>
```

但 day card、prev、next 按鈕都沒有接 message。

Audit 卻寫：

- Slider：S2，on_change
- DatePicker：S2，on_change、on_open
- DatePicker 實際上甚至沒有 `on_open` 欄位

### 其他 audit 不一致

- Header 寫 scope 為 106 files。
- 最終統計寫 111 components。
- README 寫 110。
- Foundations `mod.rs` 實際有 29 modules，README 寫 28。
- project-status 也使用 29，總數是 111。
- Audit 說「no benchmarks exist anywhere」，但 repository 有 `apps/benchmark`；若原意是「沒有 component benchmark」，應精確寫明。
- 文件包含多個異常控制字元，例如文字中的 backspace、bell、tab 字元。

### 建議重新定義 S2

S2 不能只看 builder 欄位，至少必須同時確認：

1. builder field 存在。
2. build/From 實際消費該 field。
3. 產出的 WidgetNode 具有 message/action。
4. runtime hit/keyboard path 能到達該 action。
5. 測試證明 action 恰好派送一次。

建議建立 machine-readable component manifest，而非 grep Markdown。

---

## P0-NEW-003：Ordered Scene 實作與計畫不一致

### 計畫目標

`docs/plans/P0-001_ordered_display_list.md` 的 target 是：

```rust
pub struct Scene {
    clear: Color,
    commands: Vec<DrawCommand>,
}
```

### 實際程式

`crates/acme-core/src/scene.rs` 現在是：

```rust
pub struct Scene {
    commands: Vec<PaintCommand>,
    draw_commands: Vec<DrawCommand>,
}
```

問題：

- 沒有 plan 中的 `clear`。
- 同時保存兩套 command list。
- `push()` 與 `push_draw()` 之間沒有相對順序。
- 若 caller 混合使用，無法重建 painter order。
- 沒有 PaintCommand → DrawCommand conversion helper。
- live renderer 完全沒有消費 `draw_commands`。

### 更嚴重的文字 atlas 型別問題

新的：

```rust
TextPrimitive {
    glyphs: Vec<GlyphDraw>,
    uploads: Vec<AtlasUpload>,
}
```

但 core 的 `AtlasUpload` 只有：

```rust
alpha: Vec<u8>
```

新的 `GlyphFormat` 卻允許：

- Alpha8
- Rgba8

因此目前新的 DrawCommand text model 無法完整表達 RGBA color glyph / emoji upload。

### 建議

不要保留兩個 vector，應改成：

```rust
pub struct Scene {
    pub clear: Color,
    commands: Vec<DrawCommand>,
}
```

舊 API 應立即轉換進同一 vector：

```rust
#[deprecated]
pub fn push_paint(&mut self, cmd: PaintCommand) {
    self.commands.extend(convert_paint(cmd));
}
```

Atlas upload 改成：

```rust
pub struct AtlasUpload {
    pub page: u32,
    pub origin: [u32; 2],
    pub size: [u32; 2],
    pub format: GlyphFormat,
    pub pixels: Vec<u8>,
}
```

---

## P0-NEW-004：Live Renderer 仍破壞 Painter Order

`acme-render-wgpu::Frame` 仍拆成：

- `quads`
- `clipped_quads`
- `text`

render 流程仍是：

1. regular quads
2. clipped quad HashMap groups
3. text HashMap groups

這表示新 `DrawCommand` 對目前畫面完全沒有影響。

仍存在：

- overlay quad 被文字蓋住
- tooltip/modal stacking 錯誤
- HashMap iteration 不確定
- screenshot golden 不穩
- clip/layer 順序不能完整表達

### 下一步不應再增加型別

應直接完成 P0-001 Phase 2～4：

1. `compile_scene(&Scene) -> Vec<RenderBatch>`
2. renderer consume Scene
3. Gallery emit Scene
4. 移除 Frame buckets

---

## P0-NEW-005：Accessibility Click 仍幾乎不能工作

目前 `route_action(Click(id))` 會產生：

1. `FocusChanged`，包含 target NodeId
2. `PointerButton { pressed: true, x: 0, y: 0 }`

問題：

- PointerButton 沒有 target NodeId。
- 座標是 (0,0)。
- 只有 pressed，沒有 released。
- Gallery 的 activation 主要發生在 pointer release。
- Gallery 沒有處理 `FocusChanged` target 來直接 activate widget。

所以即使 action 名稱是 Click，也很可能不會啟動指定元件。

`SetValue(id, value)`：

- 忽略 id。
- 轉成全域 `ImeCommit`。
- 可能把值送給目前 focused TextInput，而不是 accessibility target。

`Activate` 與 `ScrollIntoView` 仍回傳空事件。

### 正確方向

AccessKit action 不應轉成虛假的 pointer 座標，而應使用 target-based action：

```rust
pub enum UiAction {
    Focus { target: NodeId },
    Activate { target: NodeId },
    SetValue { target: NodeId, value: String },
    ScrollIntoView { target: NodeId },
}
```

Pointer、keyboard、Accessibility 都應進入同一 WidgetAction dispatcher。

---

## P0-NEW-006：文件重整未完整落實，且發生內容損壞

最新 commit 訊息宣稱：

- MANUAL_VALIDATION → `docs/guides/`
- project-status → `docs/status/`
- component_maturity_audit → `docs/reports/`

但最新 repository 中仍可讀到：

- `docs/MANUAL_VALIDATION.md`
- `docs/project-status.yaml`
- `docs/component_maturity_audit.md`

而宣稱的新路徑不存在。

### 內容亂碼

`docs/MANUAL_VALIDATION.md` 含有：

- `??`
- `�?`
- 破損引號
- 破損箭頭
- 部分繁體中文字損壞

`docs/component_maturity_audit.md` 含有控制字元：

- Backspace
- Bell
- Tab 被插入單字中
- UTF-8 BOM

這會影響：

- 人工驗證正確性
- Markdown 顯示
- LLM/agent 搜尋
- 文字 diff
- Windows PowerShell 寫檔流程

### 建議

新增 CI 文字品質 gate：

- 所有 `.md/.yaml/.toml/.rs` 必須 UTF-8。
- 禁止 U+FFFD。
- 禁止 C0 control chars，僅允許 `\n`、`\r`、`\t`。
- 對 Markdown 額外警告連續 `??` 與 mojibake pattern。
- 驗證所有 relative links。
- 驗證 commit 中宣稱的新路徑存在。

---

## P0-NEW-007：Single Source of Truth 尚未成立

`docs/project-status.yaml` 是很好的開始，但目前：

- README 仍說 Core framework Stable，包含 accessibility。
- README 說 Text input + IME Stable，但注音仍未人工驗證。
- STATUS 說 renderer stable。
- STATUS 的 Not Validated 仍列出沒有 cargo-audit、cargo-deny、macOS/Ubuntu CI；但 workflow 已經存在。
- todos 把 AccessKit、Animation、Stable NodeId、Dirty propagation 全部打勾，卻未區分「crate primitive 存在」與「正式 runtime 已接入」。
- `project-status.yaml` 把 P0-009 標 completed，但 note 明確說 sync script/CI gate pending。
- README 元件數與 project-status/audit 不一致。

### 建議狀態

P0-009 應為：

```yaml
status: partial
```

直到完成：

```text
project-status.yaml
  -> generator
  -> README status block
  -> STATUS.md
  -> todos summary
  -> CI drift check
```

---

## P0-NEW-008：DatePicker 仍不是 Interactive Component

已改善：

- month/year normalize
- 35/42 cells
- year underflow
- weekday tests

仍缺：

- `on_change` 未使用
- prev/next 按鈕未接 message
- closed card 無 open message
- `selected_day` 未依月份 clamp
- 2025-02-31 在 closed state 仍可顯示成合法外觀
- keyboard grid navigation
- locale/week start
- accessibility role/state/action

因此 DatePicker 目前應是 **S1 Visual with correct calendar math**，不是 S2。

---

# 4. Runtime 與架構問題

## 4.1 RuntimeTree 仍只有計畫

Gallery 每 frame 仍執行：

```text
description()
to_layout_with_context(NodeId::new(1))
new TaffyTree
accessibility full rebuild
hit regions full rebuild
Frame full rebuild
```

所以：

- stable NodeId 仍是 frame-local DFS。
- RetainedTree 尚未進正式 app。
- dirty flags 尚未節省任何工作。
- Accessibility ID 每 frame 重配。
- focus/hover 仍使用 index。
- plan 中的 RuntimeTree 沒有 production evidence。

### 建議立即實作 Phase 1 shadow

比起再寫更多計畫，下一個 commit 應直接加入：

- `view_bridge.rs`
- Gallery `RetainedTree` field
- shadow reconcile feature
- duplicate key diagnostics
- parity logs/tests

---

## 4.2 RuntimeTree 計畫中的 Kind Change 規則需重審

計畫寫：

> same key、kind change 視為 props update 並保留 NodeId

這可能讓：

- Button state 留給 TextInput
- input selection/focus 留給不同 kind
- accessibility role 改變但 local state 未清除

建議 identity 至少是：

```text
(parent, key, kind)
```

或在 kind change 時：

- 保留外部 NodeId 但明確 reset runtime local state
- 重新建立 layout/semantics/action state
- 發出 replace event

不能只設 dirty flags。

---

## 4.3 Auto-key 只能是 fallback

計畫用 path/index 產生 Label/Separator auto-key。

這種 key 在前方插入 sibling 時仍會改變，所以不是穩定 identity。

建議：

- 靜態 decorative nodes 可用 auto-key。
- 動態 list、focusable、semantic nodes 強制 explicit key。
- debug build 對動態 unkeyed sibling 發 warning。
- 不應把 auto-key 宣稱為完全 stable。

---

# 5. Platform/Input 仍需改善

雙派送已修正，但事件模型仍不完整：

- PlatformKey 缺 ArrowUp/Down。
- 缺 PageUp/PageDown。
- 缺 physical key/code。
- 缺 repeat。
- Runtime 儲存 alt/meta，但 Key event 不提供 alt/meta。
- ScaleFactorChanged 沒有向 app 派送 Resized/Scale 事件。
- FileDropped 忽略 app 回傳的 dirty。
- unknown mouse button 被映射為 0，與 Left 衝突。
- Window focus 與 Widget focus 共用 FocusChanged/node_id=0，語義混雜。

建議拆分：

```text
WindowEvent
PointerEvent
KeyboardEvent
TextInputEvent
ImeEvent
WidgetAction
AccessibilityAction
```

---

# 6. CI 與驗證

目前 CI 有：

- Windows/macOS/Ubuntu
- fmt
- check
- clippy
- test
- cargo-deny
- cargo-audit

這比 STATUS 文件寫的狀態更完整。

仍建議補：

```powershell
--locked
--all-features
cargo doc --workspace --all-features --no-deps
cargo test --doc
cargo hack check --feature-powerset
```

並加入：

- concurrency / cancel-in-progress
- timeout
- pinned cargo-audit/cargo-deny version
- `cargo install --locked`
- Markdown link check
- UTF-8/control-character scan
- status sync check
- screenshot artifact
- benchmark JSON

### 最新 commit 驗證限制

GitHub 連接器未回傳 latest commit 的 combined status。執行環境也無法 clone，所以本次無法獨立確認 commit message 所稱的 tests pass。

---

# 7. 建議優先順序

## 第一順位：修 correctness

1. Slider percent 0～100 → 0～1。
2. Slider finite/min/max normalize。
3. DatePicker selected day normalize。
4. 修 component audit false-positive。
5. 修 docs encoding/control chars。
6. 修 docs path/link drift。
7. Accessibility 改 target-based action。

## 第二順位：完成已開始的架構

1. Ordered Display List Phase 2～4。
2. RuntimeTree shadow reconcile。
3. stable NodeId source。
4. AccessKit IDs 改用 retained NodeId。
5. key-based Gallery lookup。

## 第三順位：驗證與品質

1. actual layout behavior tests。
2. event dispatch tests。
3. visual golden。
4. Criterion benchmarks。
5. CI status/doc sync gates。
6. Windows manual DPI/IME/GPU/Narrator。

---

# 8. 修正版成熟度建議

| Component / Area | README 現況 | 建議 |
|---|---|---|
| Slider | S2 | S1，直到 action/drag/keyboard 完成 |
| DatePicker | S2 | S1，直到 on_change 真正接線 |
| Accessibility | Experimental | 維持 S0 |
| Renderer | Stable/S1 混用 | S1 experimental |
| Runtime tree | 架構圖中像已接入 | Planned / not integrated |
| TextInput state machine | S2 | S2，自動化良好，IME manual pending |
| Platform dual dispatch | Completed | 該子問題 Completed |
| Component audit | Completed | Partial，需要語義驗證 |
| Status single source | Completed | Partial，需要 generator + CI |

---

# 9. v0.1 Release Gate

至少完成：

- [ ] Slider percent correctness regression。
- [ ] 15 個核心元件真正 S2。
- [ ] 至少 10 個核心元件 S3。
- [ ] Ordered Scene 成為 live renderer input。
- [ ] RuntimeTree 成為 Gallery NodeId source。
- [ ] Accessibility action target 正確。
- [ ] docs 無亂碼與 broken links。
- [ ] project-status 自動生成 README/STATUS。
- [ ] screenshot golden CI。
- [ ] benchmark baseline。
- [ ] Windows DPI manual PASS。
- [ ] 注音 IME manual PASS。
- [ ] GPU recovery manual PASS 或正式 restart fallback。
- [ ] Narrator basic PASS。
- [ ] 最新 release commit 有可見 CI checks。

---

# 10. 最終結論

這次更新顯示專案已開始從「大量增加元件」轉向「correctness 與架構收斂」，方向值得肯定。

目前最關鍵的不是再建立新計畫或新元件，而是把已開始的三件事真正完成：

1. **讓 Ordered Scene 進入 live renderer。**
2. **讓 RuntimeTree 成為正式 identity source。**
3. **讓元件成熟度由行為測試證明，而不是由 builder 欄位名稱推測。**

尤其 Slider percent bug 說明目前測試仍偏向「tree 能建立」，沒有驗證「layout 結果是否正確」。下一階段應把測試重心由結構數量提升為 actual rect、action dispatch、focus、accessibility 與 pixel output。
