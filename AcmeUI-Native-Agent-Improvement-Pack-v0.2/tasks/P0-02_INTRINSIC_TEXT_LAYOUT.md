# P0-02 Intrinsic Text Measurement and Persistent Layout

## 實作

- 建立 persistent `LayoutEngine`，保存 NodeId → Taffy NodeId。
- 根據 DirtyFlags 增量更新 style/children。
- Label/TextInput/Button 可註冊 measure function。
- 文字 cache key 至少包含 text、font family、size、weight、line height、wrap、max width、scale factor。
- Layout 不直接依賴具體 cosmic-text 型別，使用 framework-owned measure trait。

```rust
pub trait IntrinsicMeasure {
    fn measure(&mut self, request: MeasureRequest) -> MeasuredSize;
}
```

## 驗收

- Auto Label 寬高由文字決定。
- 中英混排、emoji、font fallback 正確。
- 125/150/200% scale layout 不裁切。
- 修改一個 leaf 不重建整棵 Taffy tree。
- 10k static nodes warm layout 有數字基線。
