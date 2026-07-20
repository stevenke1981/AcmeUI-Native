# Test and Release Plan

## CI Matrix

- Windows：fmt/check/clippy/test + Gallery smoke
- Ubuntu：check/test，具備 display runner 時加 smoke
- macOS：check，之後加入 smoke
- MSRV：Rust 1.85 check
- cargo-deny / cargo-audit

## 正確性測試

- keyed reorder identity
- focus/capture cleanup
- Layout ↔ NodeId mapping
- accessibility action routing
- multi-window event routing
- surface/device recreation
- zero-size suspend/resume

## 視覺測試

- screenshot golden files
- tolerance-based diff
- 1280×800、1024×700、800×600
- Light/Dark/High Contrast
- Compact/Comfortable
- CJK long text、emoji、200% DPI

## 效能 Gate

先建立 baseline，再設定 threshold。至少記錄：

- clean build / warm incremental build
- app startup
- 1k/10k node reconciliation
- clean/dirty subtree layout
- frame preparation
- draw calls
- bytes uploaded
- atlas hit rate
- idle CPU/GPU/memory

## Release 文件

建立單一 `STATUS.md`：

- Stable
- Experimental
- Architecture only
- Manually validated
- Not validated

清理 `final.md` 中過時段落，Milestone history 移到 `CHANGELOG.md`。
