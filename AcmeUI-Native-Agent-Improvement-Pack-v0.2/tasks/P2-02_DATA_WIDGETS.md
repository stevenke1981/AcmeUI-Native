# P2-02 Data Widgets Productionization

## VirtualList

- visible children 真正進入 retained/layout/paint tree。
- overscan 可設定。
- 支援 variable height cache。
- scroll anchoring。

## Tree

- expand/collapse state keyed by NodeId/ItemKey。
- Arrow navigation、Home/End、typeahead。
- virtualize large trees。

## Table/DataGrid

- semantic header/row/cell nodes。
- column width constraints and resize。
- sort indicators。
- keyboard cell/row navigation。
- selection modes。
- sticky header。
- viewport row virtualization。
- empty/loading/error states。

不得只因資料型別與單元測試存在就標記為 production complete。
