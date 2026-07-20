# 建議修改檔案對照

## 必改

- `crates/acme-core/src/tree.rs`
- `crates/acme-core/src/event.rs`
- `crates/acme-layout/src/lib.rs`（後續拆模組）
- `crates/acme-widgets/src/lib.rs`（拆模組並保留 re-export）
- `crates/acme-accessibility/src/lib.rs`
- `crates/acme-platform/src/lib.rs`
- `crates/acme-render-wgpu/src/lib.rs`
- `crates/acme-textinput/src/lib.rs`
- `crates/acme-theme/src/lib.rs`
- `apps/gallery/src/main.rs`（重構成 pages/shell/runtime）

## 新增

```text
crates/acme-runtime/
crates/acme-widgets/src/foundations/
crates/acme-widgets/src/inputs/
crates/acme-widgets/src/navigation/
crates/acme-widgets/src/overlay/
crates/acme-widgets/src/data/
apps/gallery/src/pages/
docs/design/
docs/status/STATUS.md
```

`acme-runtime` 負責將 retained tree、layout、events、focus、semantics、paint compilation 串成單一生命週期，避免 Gallery 或每個 app 自己手工拼接。
