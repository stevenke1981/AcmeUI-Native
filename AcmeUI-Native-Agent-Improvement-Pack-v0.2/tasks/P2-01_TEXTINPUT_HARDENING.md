# P2-01 TextInput Hardening

## 資料安全

- cursor/selection fields 改 private。
- 對外使用 typed `TextOffset` 或 validated byte boundary。
- 所有 mutation 維持 UTF-8 boundary 與 selection invariant。

## 功能

- mouse click caret positioning
- drag selection
- double click word selection
- Ctrl+Left/Right
- Ctrl+Backspace/Delete
- Shift+Home/End
- undo/redo transaction stack
- horizontal scrolling
- placeholder/readonly/invalid
- IME caret area
- composition underline and selection

## 分層

- model：text/selection/composition/history
- controller：keyboard/pointer/clipboard/IME
- view adapter：paint commands and semantic data

## 手動 Gate

Windows 10/11 注音：

- preedit
- candidate window position
- select candidate
- commit
- cancel
- mixed zh-TW/English
- emoji
- selection replacement
