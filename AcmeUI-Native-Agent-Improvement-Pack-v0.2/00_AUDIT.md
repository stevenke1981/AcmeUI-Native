# AcmeUI Native 現況稽核

## 整體判斷

專案已經超過空骨架階段，具備 winit/wgpu 視窗、圖形與文字渲染、Taffy layout、簡易 retained tree、事件、Focus、TextInput、Overlay 資料模型、AccessKit tree builder、資料元件資料結構與 Gallery。

目前主要風險不是「沒有功能」，而是多個子系統尚未共享同一套 Runtime identity、狀態與生命週期。部分功能比較接近可測試的資料模型或展示程式，尚未達 production widget。

## P0：正確性與架構缺口

### 1. Retained NodeId 未貫穿 Layout、Hit Test、Accessibility

現況：

- `acme-core::RetainedTree` 產生穩定 `NodeId`。
- `WidgetNode::to_layout(&mut next)` 重新以 traversal counter 產生 `u64`。
- AccessKit tree 也重新依 DFS traversal 產生 ID。
- Gallery 直接依賴 `snapshot.get(2)`, `snapshot.get(7)` 等數字。

風險：

- sibling reorder 後 Layout ID 改變。
- Focus、hover、animation、accessibility focus 可能指向錯誤節點。
- keyed reconciliation 的價值無法傳到 layout 和 render。

修正：建立單一 `RuntimeNode` / `CompiledNode`，所有 LayoutSnapshot、HitTestEntry、SemanticNode、Paint cache 都以 `NodeId` 為 key。

### 2. Layout 每次重建完整 TaffyTree

現況：`LayoutEngine::compute()` 每次建立新的 `TaffyTree` 並遞迴 build。

風險：

- dirty flags 沒有帶來增量 layout。
- 大型 Tree/DataGrid 會重建整棵 layout tree。
- NodeId 到 Taffy NodeId 的映射無法持久化。

修正：建立 persistent layout arena 與 `HashMap<NodeId, taffy::NodeId>`，只同步變更節點。

### 3. Label 缺少 intrinsic text measurement

現況：Label 轉成 Auto leaf，但沒有 Taffy measure callback；Gallery 以手工尺寸、手工 text origin 補足。

風險：

- Auto-sized Label/Stack 不可靠。
- 長中文字、不同 DPI、字型 fallback 會破版。

修正：Layout leaf 保存 `MeasureKind::Text(TextMeasureKey)`，由 `acme-text` 提供 measure callback，並加入文字 layout cache。

### 4. AccessKit 尚未真正接入 Platform Runtime

現況：已有 `TreeUpdate` builder，但 `acme-platform` 沒有依賴或驅動 accessibility adapter，動作也沒有回送到 widget event system。

修正：建立 `AccessibilityBridge`，每視窗持有 adapter，送出 tree updates，接收 Focus/Click/SetValue/Scroll actions。

### 5. Platform event model 與 core event model 分裂

現況：

- `acme-core` 有較完整 PointerEvent、KeyboardEvent、ImeEvent。
- `acme-platform::PlatformEvent` 另有簡化版本。
- IME event 沒有 WindowId。
- PointerButton 沒有 button、position、pointer ID。
- Scroll 只有 Y 軸。

修正：Platform 僅負責轉換 OS event，統一輸出 `WindowEventEnvelope { window, event: UiEvent }`。

## P0：Renderer 效能缺口

### 6. 每幀建立大量 GPU buffers

現況：

- 每幀為 quads 建立 instance buffer。
- 每個 clipped quad 各自建立 buffer。
- 每個 text run/atlas format 各自建立 buffer。

風險：CPU/GPU allocation、driver overhead、draw call 過多。

修正：

- Persistent growable instance buffers。
- Frame ring buffer。
- 按 `(pipeline, texture, clip)` batch。
- 同 clip 的 quad 合併為一次 draw。

### 7. Glyph atlas 固定 2048 且缺乏生命週期

修正：增加 atlas page、LRU/epoch、dirty-region upload、容量與命中率 metrics。

### 8. Surface bridge 使用 `Any` downcast

修正：建立 renderer-private `SurfaceHost`/`WindowSurfaceHandle`，讓 platform crate 以明確私有橋接傳遞 raw-window-handle，不再依賴 Any。

## P1：Widget 與產品品質缺口

### 9. `acme-widgets/src/lib.rs` 單檔持續膨脹

按 foundations、inputs、navigation、overlay、data 拆分模組，保留相容 re-export 和 prelude。

### 10. 多個 widget 目前偏向資料描述，而不是完整 runtime 行為

例：

- Popover layout 只返回 anchor。
- Menu/Dialog 以單一 leaf 表示，缺少 overlay layer、anchor collision、focus trap。
- VirtualList layout 不真正掛入 visible children。
- Tree/Table/DataGrid 缺少完整 keyboard interaction、viewport virtualization、column resize 與 semantic row/cell tree。

### 11. 控制尺寸大量 hardcode

Button 36、Tree row 24、Menu width 200、indent 20 等都應改用 Theme control/density tokens。

## P1：美感與設計系統缺口

### 12. Theme token 太少

目前只有少量 colors、5 級 spacing、3 級 radius、body/label typography。缺少：

- Surface hierarchy
- Text hierarchy
- Semantic soft colors
- Pressed/selected/invalid
- Elevation/shadow
- Control sizes
- Density
- Motion
- Icon sizes
- z-layer

### 13. Gallery 是手工 smoke demo，不是 Design System Gallery

Gallery 直接使用數字 layout ID、手動畫文字位置和固定尺寸。需要改成由 widget/runtime 自己產生畫面，並新增 Tokens、States、Patterns、Stress、Accessibility 頁面。

## P2：TextInput 缺口

- Public cursor/selection 使用裸 `usize` byte offsets，可被外部建立成非 UTF-8 boundary。
- 尚缺完整 undo/redo、word navigation、mouse drag selection、double-click selection、horizontal scroll、candidate window caret positioning。
- Traditional Chinese 注音需真實 Windows 手動驗證，不可只靠單元測試宣稱完成。
- Model、controller 與 renderer 目前耦合在同一 crate。

## 文件與 CI 缺口

- `todos.md` 幾乎全勾選，但 `final.md` 同時保留早期限制與後續完成紀錄，狀態不易判讀。
- README、todos、final 對 IME、AccessKit、Tree/DataGrid 的描述需要版本化整理。
- CI 目前只跑 Windows，至少加入 Ubuntu check/test；macOS 可先 check。
- 缺少 accepted benchmark thresholds、dependency audit、MSRV job、visual regression gate。
