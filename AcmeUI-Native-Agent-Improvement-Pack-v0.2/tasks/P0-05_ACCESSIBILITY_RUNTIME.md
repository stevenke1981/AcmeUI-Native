# P0-05 Accessibility Runtime Integration

## 實作

- 每個 WindowState 建立 AccessKit adapter。
- semantic tree 使用同一 NodeId。
- layout/update/focus 改變時送出增量或完整 TreeUpdate。
- ActionRequest 回送到 UI event dispatch。
- TextInput 提供 value、selection、set value、replace selection。
- Tree/Table/DataGrid 建立 row/cell/column semantics，不只 root role。
- initial tree 使用真實 window bounds，不使用固定 1080×720。

## 驗收

- Windows Narrator 可讀 Label/Button/TextInput。
- Narrator Click/Focus action 可觸發。
- disabled/selected/expanded/checked/invalid state 正確。
- reorder 後 accessibility focus 不跳到別的 widget。
