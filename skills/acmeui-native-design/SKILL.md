---
name: acmeui-native-design
description: 以 AcmeUI-Native 設計流程為核心的 UI 設計、元件開發與 App 模板路由 Skill。當 Agent 需要建立桌面 App、套用 voice-dictation/Typeless-like、dashboard、settings、media-studio 模板，新增元件、主題、畫面、Gallery 頁面、執行視覺驗收，或把模板選擇、semantic tokens、WCAG 與 Visual QA 流程應用到其他 Rust、GPUI、Web UI 專案時使用。先辨識目標技術棧；只有 AcmeUI-Native 目標套用 WidgetNode、wgpu 與 winit 實作規則，其他專案保留原有架構。
---

# AcmeUI-Native Design Director & App Template Skill

本 Skill 以 AcmeUI-Native 的**設計路由器、App 模板庫、元件參考與視覺驗收流程**為核心，也可用於其他 UI 技術棧。

Agent 不得只把它當成 API 文件。每次使用時必須先判斷任務模式、選擇模板或視覺方向、建立實作契約，再進入程式碼。

## 0. 技術棧路由

開始前先辨識目標為 `acmeui-native`、`other-rust`、`gpui` 或 `web`：

- `acmeui-native`：完整使用 WidgetNode、wgpu/winit、AcmeUI templates、semantic tokens 與 Cargo 驗證規則。
- 其他技術棧：沿用模式路由、資訊架構、模板工作流、Art Direction、Component Inventory、WCAG 與 Visual QA，並將元件、token 與驗證命令映射到目標原有架構。
- 啟用本 Skill 不代表同意遷移技術棧。是否採用 AcmeUI-Native 或 WidgetNode 必須由任務需求另行決定。

## 0.1 AcmeUI-Native 專案事實

- AcmeUI-Native 是從零建立的 Rust Native UI framework。
- 渲染與視窗層為 wgpu + winit，排版為 Taffy，文字為 cosmic-text。
- 核心 UI 使用 `WidgetNode<M>` 與 fluent builder API。
- `acme-ui` 是高階元件庫；`acme-widgets` 是 primitives。
- 顏色必須來自 `acme-theme` semantic tokens。
- 專案**不是 GPUI**，禁止加入 GPUI 依賴。

## 1. 啟動時必須先輸出的內容

Agent 在開始修改前，先用 4 至 8 行明確宣告：

```text
Mode: template-first | new-app | component | redesign | theme | review
Template: voice-dictation | dashboard | settings | media-studio | none
Visual recipe: calm-voice | focused-productivity | ...
Target files: ...
Validation: cargo check/test + screenshot/manual visual QA
```

不得在沒有宣告模式與範圍時直接大量寫 UI。

## 2. 工作模式路由

### `template-first`

適用：使用者說「做一個像 Typeless 的 App」、「做管理後台」、「做設定程式」、「做影音編輯器」。

流程：

1. 讀取 `templates/catalog.yaml`。
2. 根據產品工作流選模板，不可只看外觀名稱。
3. 讀取對應的 `templates/<id>.rs` starter。
4. 讀取 `templates/catalog.yaml` 的區域、必要狀態與 feature 要求。
5. 先建立 `.design/template-selection.md`，再實作。
6. 保留模板資訊架構，但依需求增減功能；不可直接複製第三方品牌或像素外觀。

### `new-app`

適用：建立新的 `apps/<name>`。

預設以 `apps/playground` 作為 runtime 參考，使用 `acme-ui` 組合畫面。Agent 必須：

1. 建立 app package 與 workspace member。
2. 選擇 `default_template`、`apple_template`、`windows11_template` 或 `ubuntu25_template`。
3. 將 state、message、view、runtime/event handling 分開。
4. 先讓 starter view 編譯，再加入完整互動。
5. 不得宣稱全域快捷鍵、語音辨識、IME 或 GPU recovery 已完成，除非有實際測試證據。

### `component`

適用：新增或修改 `crates/acme-ui`、`acme-widgets` 元件。

依本 Skill 的「新增元件」與 WidgetNode 規則執行。優先組合現有 primitives，避免新增重複元件。

### `redesign`

適用：保留功能與資料流，重新整理視覺層級、版面、配色與元件一致性。

依本 Skill 的 Visual Design Director Gates，先建立 brief、art direction、component inventory，再改程式。

### `theme`

適用：新增 theme pack、調整 semantic tokens 或 WCAG。

先查 `libraries/palettes.json` 與現有 `acme_theme::packs`。Hex 值只能用於 theme pack 定義或設計文件；Widget 內禁止硬編碼。

### `review`

適用：只檢查 UI、截圖、元件一致性、響應式與 accessibility，不直接改程式。

輸出 `.design/visual-qa.md`，依本 Skill 的 100 分 Visual QA 評分。

## 3. App 模板選擇器

依需求中的主要動詞與資料流選擇：

| 需求特徵 | 模板 | 核心區域 |
|---|---|---|
| 說話、錄音、轉文字、語氣、歷史紀錄 | `voice-dictation` | 錄音狀態、逐字稿、語氣、歷史、設定 |
| KPI、監控、報表、趨勢、資料表 | `dashboard` | 導航、指標、圖表、活動、篩選 |
| 偏好、帳號、快捷鍵、隱私、裝置 | `settings` | 分類導航、表單區、儲存狀態 |
| 素材、預覽、時間軸、屬性、匯出 | `media-studio` | 素材庫、畫布/預覽、Inspector、Timeline |
| 不符合以上 | `none` | 先做 Design Brief，再建立新模板 |

「像某 App」只代表工作流參考。必須轉譯為區域、狀態、元件與互動，不得複製商標、文案或專屬品牌造型。

## 4. Template-first 標準流程

### Gate A — 需求與模板匹配

建立 `.design/template-selection.md`：

- 使用者目標
- 選擇的 template ID
- 選擇理由
- 保留的區域
- 移除或新增的區域
- 必要狀態
- 技術風險

### Gate B — Art Direction

從 `libraries/style-recipes.yaml` 選一個 recipe，再從 `libraries/palettes.json` 選 palette 或現有 theme pack。

建立 `.design/art-direction.md`：

- 主視覺方向
- 主要與次要焦點
- 資訊密度
- 字體階層
- 容器模型
- 色彩與 theme pack
- 圖示與動態原則
- 禁止項目

### Gate C — Component Inventory

建立 `.design/component-inventory.md`，逐區列出：

- 可直接使用的 AcmeUI 元件
- 需要組合的 primitives
- 真正缺少、需要新增的元件
- Default/Hover/Pressed/Focus/Disabled/Loading/Error 狀態

禁止先新增元件，再尋找使用場景。

### Gate D — Starter Integration

1. 複製對應 `templates/<id>.rs` 到目標 app 的 view module。
2. 重新命名 message/state 型別。
3. 將 placeholder data 替換成真實 state。
4. 將訊息接到 update/event handling。
5. 以 semantic theme tokens 驗證 light/dark。
6. 加入功能時維持模板的區域責任，不把所有內容塞進 Card。

### Gate E — Visual QA

至少驗證：

- 預設狀態
- 核心操作中的 active/recording/editing 狀態
- Empty
- Error 或 permission denied
- Light/Dark
- 1280×720 與主要目標尺寸

沒有實際畫面或截圖證據，不得宣告設計完成。

## 5. Visual Design Director 規則

1. 複雜 App 必須先設計資訊架構，不得直接堆元件。
2. 一個畫面只能有一個主要焦點與一個主要動作。
3. 字體階層與間距優先於陰影、漸層和裝飾。
4. 預設使用開放式版面、列表、工具列、分割面板與畫布；不要把所有區域做成卡片。
5. 不使用與品牌無關的藍紫漸層、玻璃模糊、假 KPI、過量 Badge/Pill。
6. 同一類元件必須共享尺寸、圓角、間距與狀態規則。
7. 實作 Agent 無權只靠自評判定美觀；必須執行 Visual QA。
8. 視覺品質低於 90/100，繼續修正。

完整流程已整合在 Gate A 至 Gate E；不得跳過模板選擇、Art Direction、Component Inventory 與 Visual QA。

## 6. WidgetNode 最小正確模式

本節只適用於目標已採用或明確選擇 AcmeUI-Native 的任務。其他技術棧保留其既有 view/component model。

```rust
use acme_ui::prelude::*;

#[derive(Clone, Debug, PartialEq)]
enum AppMessage {
    Save,
    Cancel,
}

fn view() -> WidgetNode<AppMessage> {
    default_template("Settings")
        .subtitle("A calm, keyboard-friendly workspace")
        .child(
            card::<AppMessage>()
                .gap(8.0)
                .padding(16.0)
                .child(label("Preferences"))
                .build(),
        )
        .child(
            row::<AppMessage>()
                .gap(8.0)
                .child(button("cancel", "Cancel").on_click(AppMessage::Cancel))
                .child(button("save", "Save").primary().on_click(AppMessage::Save))
                .build(),
        )
        .build()
}
```

規則：

- 動態列表與互動控制使用 stable key。
- View 只由 state 推導；狀態修改放在 update/event layer。
- Container builders 使用 `.build()` 或明確 `Into<WidgetNode<M>>`。
- 不要在 app view 直接操作 wgpu/winit public types。

## 7. Theme 與配色

- Widget 內只能使用 `theme.colors.<semantic_field>` 或由元件解析的 semantic tone。
- Palette library 是 theme pack 的設計輸入，不是 Widget 的 Hex 值來源。
- Light/Dark 都要定義 background、surface、foreground、muted、border、primary、focus、success、warning、danger、info。
- 新 theme pack 必須執行 `meets_wcag_aa()` 與 `wcag_report()`。
- 優先使用現有 packs：apple、windows10、windows11、ubuntu、material、nord、dracula、solarized、gruvbox、one-dark。

配色資料見 `libraries/palettes.json`；視覺方向見 `libraries/style-recipes.yaml`。

## 8. 新增元件

1. 確認 catalog 中不存在等價元件。
2. 建立 `crates/acme-ui/src/<family>/<name>.rs`。
3. 定義 `XxxBuilder<M>` 與 fluent setters。
4. 若 `M` 未出現在欄位，加 `PhantomData<M>`。
5. `From<XxxBuilder<M>> for WidgetNode<M>` 優先組合現有 primitives。
6. 在 `mod.rs` 同時加入 `pub mod xxx;` 與 `pub use xxx::*;`。
7. 測試 stable key、children、defaults、variant/state。
8. 在 Gallery 加入可見示例；注意 `apps/acme-gallery` 的手動 button index 同步問題。

## 9. 硬性規則

- 在 AcmeUI-Native 目標中禁止加入 GPUI；其他專案沿用其既有技術棧。
- Widget 禁止硬編碼 theme colors。
- Text cursor 禁止使用 byte offsets；使用 char/grapheme indices。
- 禁止公開 platform-specific winit/wgpu types。
- 未手動驗證，不得聲稱繁體中文 IME 正常。
- 不以 `cargo clean` 當例行修復。
- Build 成功不等於 Visual QA 通過。
- 不得把第三方產品的商標、文案與專屬視覺直接做成模板。

## 10. 驗證命令

以下命令適用於 AcmeUI-Native 目標；其他專案改用其既有 formatter、lint、typecheck、test、build 與視覺驗收命令。

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p acme-ui --all-features
cargo test -p acme-theme
cargo check -p acme-ui-gallery
```

新 App 至少另外執行：

```bash
cargo check -p <new-package>
cargo run -p <new-package>
```

## 11. 完成回報格式

```text
Mode / Template / Recipe:
Changed files:
State and message wiring:
Theme and WCAG result:
Functional checks:
Visual QA sizes and states:
Visual score:
Known limitations:
```

沒有證據不得使用「專業級」、「像素級一致」、「完整支援」等描述。

## 12. 資源索引

- `README.md`：人類與 Agent 快速使用說明
- `templates/catalog.yaml`：可機器讀取的模板索引、區域與必要狀態
- `templates/*.rs`：可複製的 WidgetNode starter views
- `libraries/palettes.json`：semantic palette library
- `libraries/style-recipes.yaml`：視覺方向 recipes
- `assets/layout-gallery.svg`：App layout 圖庫
- `assets/palette-gallery.svg`：配色圖庫
