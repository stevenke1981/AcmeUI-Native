# Agent Master Prompt

你正在維護 `stevenke1981/AcmeUI-Native`。

## 任務目標

把專案提升到 `0.2.0 — Runtime Identity, Intrinsic Layout and Visual System`。

## 執行規則

1. 先閱讀 `00_AUDIT.md` 與當前 task 文件。
2. 一次只執行一個 task，不把下一階段順便混入。
3. 修改前列出：現況、根因、變更邊界、驗收標準。
4. 先跑目標 crate 的 check/test，再跑 workspace gate。
5. 所有 UI 修改必須補 Gallery 畫面與 Light/Dark evidence。
6. 對外 API 變更必須提供 migration note；能以相容 builder method 完成時，不做破壞性修改。
7. 不得以 traversal number 作為 persistent identity。
8. 禁止 widget 內出現任意 RGB、固定 control height、固定 radius；改用 tokens。
9. 不得用 placeholder demo 宣稱 runtime feature 完成。
10. IME、DPI、AccessKit、surface loss 等需要人工或平台驗證的項目，未驗證就寫成「未驗證」。

## 快速迴圈

```powershell
cargo fmt --all
cargo check -p <changed-crate>
cargo test -p <changed-crate>
```

## 里程碑 Gate

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo tree | Select-String gpui
```

`cargo tree` 不得出現 GPUI。

## 每次交付

更新：

- `todos.md`
- `final.md`
- 對應 docs
- tests
- Gallery evidence

`final.md` 必須分成：Delivered、Validated、Not Validated、Known Limitations、Next Task。
