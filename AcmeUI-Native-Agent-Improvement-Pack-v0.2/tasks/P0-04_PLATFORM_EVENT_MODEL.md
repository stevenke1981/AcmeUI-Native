# P0-04 Unified Platform Event Model

## 目標 API

```rust
pub struct WindowEventEnvelope {
    pub window: WindowId,
    pub event: UiEvent,
}
```

## 實作

- IME Enabled/Disabled/Preedit/Commit 全部帶 WindowId。
- Pointer event 包含 PointerId、button、position。
- Scroll 包含 x/y 與 unit。
- Keyboard 統一 modifiers：shift/control/alt/meta。
- 保留 physical/logical key 和 text intent，不用字串猜 Ctrl+A/C/V/X。
- 加入 cursor entered/left、focus gained/lost、file drop 基礎事件。
- 提供 `set_ime_cursor_area(window, rect)` 讓候選窗跟隨 caret。

## 驗收

- 多視窗 IME 不會送到錯誤 TextInput。
- Right/Middle mouse 可辨識。
- Trackpad horizontal scroll 可辨識。
- shortcut 使用 key code + modifiers，不依賴輸入文字。
