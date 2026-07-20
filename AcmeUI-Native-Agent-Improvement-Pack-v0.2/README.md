# AcmeUI Native Agent Improvement Pack v0.2

此修正包是給 Codex、OpenCode 或其他 coding agents 使用的專案改善規格。

目標不是繼續堆疊「名稱存在但尚未真正整合」的元件，而是把 AcmeUI Native 穩定為可持續發展的 Rust/wgpu Desktop UI Runtime，並加入一致且具有商業產品質感的視覺系統。

## 建議目標版本

`0.2.0 — Runtime Identity, Intrinsic Layout and Visual System`

## Agent 執行順序

1. `00_AUDIT.md`
2. `AGENT_MASTER_PROMPT.md`
3. `tasks/P0-01_NODE_IDENTITY.md`
4. `tasks/P0-02_INTRINSIC_TEXT_LAYOUT.md`
5. `tasks/P0-03_RENDERER_BUFFERING.md`
6. `tasks/P0-04_PLATFORM_EVENT_MODEL.md`
7. `tasks/P0-05_ACCESSIBILITY_RUNTIME.md`
8. `tasks/P1-01_THEME_V2.md`
9. `tasks/P1-02_WIDGET_VISUAL_STATES.md`
10. `tasks/P1-03_GALLERY_TEMPLATES.md`
11. `tasks/P2-01_TEXTINPUT_HARDENING.md`
12. `tasks/P2-02_DATA_WIDGETS.md`
13. `TEST_AND_RELEASE.md`

## 強制原則

- 不加入 GPUI。
- 不把 wgpu、winit、Taffy 型別暴露到上層公共 API。
- 不以 traversal index 作為長期 Node identity。
- 不在 widget 內硬編碼顏色、控制高度、圓角與間距。
- 不宣稱功能完成，除非 Runtime、Gallery、測試與手動驗證都完成。
- 不使用 `cargo clean` 作為一般修復方式。
- 每個 PR 只處理一個可回滾工作包。
