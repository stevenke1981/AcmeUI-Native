# P0-01 Unified Node Identity

## 問題

RetainedTree、Widget layout、Accessibility 使用不同 ID 生命週期。

## 實作

1. `LayoutNode.id` 改為 `NodeId`，或新增不暴露內部數字的 `LayoutNodeId(NodeId)`。
2. 移除 `WidgetNode::to_layout(&mut u64)`。
3. 新增 compile/reconcile 階段：

```rust
pub struct RuntimeNode<M> {
    pub id: NodeId,
    pub widget: WidgetNode<M>,
    pub children: Vec<NodeId>,
}
```

4. `LayoutSnapshot`、HitTest、Paint cache、AccessKit 全部以 NodeId 索引。
5. Gallery 禁止使用 `snapshot.get(7)`；必須以 WidgetKey → NodeId 或已保存 NodeId 取得。
6. sibling reorder 後 NodeId、focus、scroll state、animation state 保持。

## 驗收

- keyed reorder 不改變原節點 NodeId。
- reorder 後 layout rect 與 accessibility node 維持對應。
- focused node reorder 後仍 focused。
- remove node 後 focus、pointer capture、semantic node、layout node 同步清理。
- 不再存在公開的 traversal counter API。
