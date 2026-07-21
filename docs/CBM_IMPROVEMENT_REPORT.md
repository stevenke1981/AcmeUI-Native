# AcmeUI-Native 改善建議報告

> 產生日期：2026-07-21
> 分析工具：CBM Code Knowledge Graph (`cbm+AcmeUI-Native`)
> 分析範圍：14 crates + 4 apps，355 個 Rust 檔案，~86,000 行 Rust 程式碼

---

## 一、專案總覽

| 指標 | 數值 |
|------|------|
| Crates | 14 |
| Apps | 4 (gallery, acme-gallery, playground, benchmark) |
| Rust 檔案 | 355 |
| Rust 總行數 | ~85,968 |
| 符號總數 | 7,410 (5,113 函式 / 1,123 結構體 / 654 資料夾 / 517 檔案) |
| 邊總數 | 14,627 (7,407 CONTAINS / 5,735 CALLS / 1,432 IMPORTS / 53 IMPLEMENTS) |
| 測試函式 | 83 |

### Crate 符號分佈

| Crate | 符號數 | 佔比 |
|-------|--------|------|
| **acme-ui** | **3,555** | **48.0%** |
| acme-widgets | 1,146 | 15.5% |
| acme-textinput | 321 | 4.3% |
| acme-core | 320 | 4.3% |
| acme-render-wgpu | 210 | 2.8% |
| acme-style | 182 | 2.5% |
| acme-platform | 142 | 1.9% |
| acme-layout | 135 | 1.8% |
| acme-text | 117 | 1.6% |
| acme-accessibility | 113 | 1.5% |
| acme-animation | 102 | 1.4% |
| acme-devtools | 100 | 1.3% |
| acme-theme | 88 | 1.2% |

---

## 二、關鍵問題與改善建議

### 🔴 P0 — 測試覆蓋率嚴重不足

**現況：** 整個 workspace 僅有 **83 個測試函式**，且只有 3/14 個 crate 有測試：

| Crate | 測試數 | 符號數 | 覆蓋狀態 |
|-------|--------|--------|----------|
| acme-animation | 44 | 102 | ✅ 有測試 |
| acme-ui | 33 | 3,555 | ⚠️ 嚴重不足 (0.9%) |
| acme-widgets | 6 | 1,146 | ⚠️ 嚴重不足 (0.5%) |
| acme-core | 0 | 320 | ❌ 零測試 |
| acme-platform | 0 | 142 | ❌ 零測試 |
| acme-render-wgpu | 0 | 210 | ❌ 零測試 |
| acme-layout | 0 | 135 | ❌ 零測試 |
| acme-text | 0 | 117 | ❌ 零測試 |
| acme-accessibility | 0 | 113 | ❌ 零測試 |
| acme-style | 0 | 182 | ❌ 零測試 |
| acme-theme | 0 | 88 | ❌ 零測試 |
| acme-textinput | 0 | 321 | ❌ 零測試 |
| acme-devtools | 0 | 100 | ❌ 零測試 |

**建議：**
1. **優先為核心 crate 補測試：** `acme-core`（tree reconcile、geometry、scene）、`acme-layout`（布局演算法）、`acme-style`（樣式計算）是底層基礎，任何回歸都會波及全部上層。
2. **為 `acme-textinput` 補測試：** 321 個符號、2,555 行程式碼、零測試。文字輸入是使用者互動的核心路徑，`handle_key` 有 14 個 outgoing calls，需要鍵盤事件的單元測試。
3. **為 `acme-render-wgpu` 補測試：** 渲染管線至少需要 snapshot 測試或整合測試。
4. **目標：** 每個 crate 至少對公開 API 有基本測試覆蓋。

---

### 🔴 P0 — 巨型函式需要拆分

**現況：** 多個函式超過 250 行，可讀性和可維護性差：

| 函式 | 檔案 | 行數 |
|------|------|------|
| `window_event` | acme-platform/src/lib.rs | **339 行** |
| `to_layout_alloc_with_context` | acme-widgets/src/lib.rs | **316 行** |
| `render_scene` | acme-render-wgpu/src/lib.rs | **290 行** |
| `to_layout_alloc` | acme-widgets/src/lib.rs | **278 行** |
| `render_text_input` | acme-textinput/src/lib.rs | **275 行** |
| `render` | acme-render-wgpu/src/lib.rs | **266 行** |

**建議：**
1. **`window_event` (339 行)：** 將事件處理拆分為 `handle_keyboard_event`、`handle_mouse_event`、`handle_window_lifecycle` 等子函式，或使用 match arm 提取。
2. **`to_layout_alloc` / `to_layout_alloc_with_context`：** 這兩個函式合計近 600 行，是 widget → layout 的轉換核心。考慮按 widget 類型拆分為獨立的轉換函式，或使用 trait-based dispatch。
3. **`render_scene` / `render`：** 將渲染管線拆分為 pass 級別的子函式（clear pass、geometry pass、text pass、overlay pass）。

---

### 🟠 P1 — `acme-ui` 是 God Crate

**現況：** `acme-ui` 包含 **3,555 個符號（佔全專案 48%）**、120 個檔案、8 個模組：

- `browser/` — carousel, image_gallery, lightbox, zoom_view
- `charts/` — area, bar, donut, line, pie, scatter, sparkline
- `desktop/` — command_bar, dock, menubar, navigation_view, property_grid, sidenav, taskbar, title_bar, window_controls...
- `foundations/` — alert, avatar, badge, banner, calendar, collapsible, diff_viewer, drop_zone, empty_state, flex, hero, link, list, progress, skeleton, spinner, statistic, tag, timeline, watermark...
- `inputs/` — autocomplete, button_group, cascader, color_picker, date_picker, mentions, multi_select, number_input, password_input, pin_input, radio, range_slider, rating, slider, switch, tag_input, time_picker, toggle_button, transfer, tree_select...
- `layout/` — breadcrumb, form, navigation_menu, page_header, pagination, scroll_area, section, tabs
- `mobile/` — action_sheet, bottom_nav, bottom_sheet, pull_to_refresh, search_bar
- `overlay/` — about_dialog, confirm_dialog, context_menu, dropdown_menu, fullscreen, hover_card, modal, toast

**建議：**
1. **將 `acme-ui` 拆分為多個 crate：**
   - `acme-ui-foundations` — 基礎元件（alert, badge, tag, progress, spinner...）
   - `acme-ui-inputs` — 輸入元件（date_picker, slider, cascader, tree_select...）
   - `acme-ui-charts` — 圖表元件（獨立性高，無 UI 框架依賴）
   - `acme-ui-desktop` — 桌面專用元件（title_bar, menubar, dock, taskbar...）
   - `acme-ui-mobile` — 行動裝置元件
   - `acme-ui-overlay` — 覆蓋層元件（modal, toast, dropdown...）
2. **好處：** 編譯時間改善（增量編譯粒度更細）、依賴管理更清晰、使用者可選擇性引入。

---

### 🟠 P1 — 單檔案巨型 Crate

**現況：** 多個 crate 的核心邏輯集中在單一 `lib.rs`：

| 檔案 | 行數 |
|------|------|
| `crates/acme-textinput/src/lib.rs` | **2,555** |
| `crates/acme-widgets/src/lib.rs` | **1,753** |
| `crates/acme-render-wgpu/src/lib.rs` | **1,360** |
| `crates/acme-accessibility/src/lib.rs` | **1,069** |
| `crates/acme-platform/src/lib.rs` | **844** |
| `crates/acme-text/src/lib.rs` | **831** |
| `crates/acme-layout/src/lib.rs` | **759** |

**建議：**
1. **`acme-textinput` (2,555 行)：** 拆分為 `input.rs`（核心狀態）、`keys.rs`（鍵盤處理）、`render.rs`（渲染）、`selection.rs`（選取邏輯）、`clipboard.rs`（剪貼簿）。
2. **`acme-widgets` (1,753 行)：** 已有 `data/`、`inputs/`、`navigation/`、`overlay/` 子模組，但 `lib.rs` 仍包含核心的 `to_layout` 轉換邏輯（fan-in 147！）。應將 `to_layout` 拆分為獨立的 `layout_bridge.rs`。
3. **`acme-render-wgpu` (1,360 行)：** 拆分為 `pipeline.rs`、`scene.rs`、`resources.rs`、`batch.rs`（已有 batch.rs，但主 lib.rs 仍過大）。

---

### 🟠 P1 — `to_layout` 是全域瓶頸

**現況：** `to_layout` 函式（acme-widgets/src/lib.rs）有 **147 個傳入呼叫（fan-in）**，是全專案最高。這代表幾乎所有 widget 都直接依賴這個單一函式。

**建議：**
1. 引入 `ToLayout` trait，讓每個 widget 類型實作自己的布局轉換。
2. 將 `to_layout` 改為 trait dispatch，降低單點耦合。
3. 這也能改善編譯時間——目前任何 widget 的布局變更都可能觸及這個核心函式。

---

### 🟡 P2 — Gallery App 重複

**現況：** 存在兩個 Gallery 應用：
- `apps/gallery/src/main.rs` — 2,388 行
- `apps/acme-gallery/src/main.rs` — 1,509 行

兩者都實作 `Application` trait，功能似乎重疊。

**建議：**
1. 確認兩者的定位差異（開發測試 vs. 展示用途？）。
2. 如果功能重疊，合併為一個，或明確分離職責。
3. `gallery/src/main.rs` 的 `frame` 函式有 28 個 outgoing calls（全專案最高），需要拆分。

---

### 🟡 P2 — Trait 使用率偏低

**現況：** 1,123 個結構體中僅有 **53 個 IMPLEMENTS 關係**。主要的 trait 實作集中在：
- `Application` trait（4 個 app + 幾個測試 app）
- `Default`、`Display`、`Error`、`From`、`BitOr` 等標準 trait

**建議：**
1. 在核心 crate 中引入更多 trait 抽象：
   - `Widget` trait 統一 widget 行為（目前 widget 是函式建構，缺乏統一介面）
   - `Renderable` trait 抽象渲染行為
   - `Layoutable` trait 抽象布局計算
2. 這能提升可測試性（mock 實作）和可擴展性。

---

### 🟡 P2 — 高 Fan-Out 熱點

**現況：** 以下函式有過多的 outgoing calls，承擔了過多職責：

| 函式 | 檔案 | Fan-Out |
|------|------|---------|
| `frame` | apps/acme-gallery/src/main.rs | 28 |
| `frame` | apps/playground/src/main.rs | 22 |
| `reconcile_children` | crates/acme-core/src/tree.rs | 20 |
| `frame` | apps/gallery/src/main.rs | 16 |
| `style_page` | apps/gallery/src/pages/foundations.rs | 16 |
| `handle_key` | crates/acme-textinput/src/lib.rs | 14 |
| `window_event` | crates/acme-platform/src/lib.rs | 14 |

**建議：**
1. `frame` 函式應拆分為 `update_state` → `build_scene` → `submit_render` 等階段。
2. `reconcile_children` (fan-out 20) 是 tree reconciliation 的核心，應確保有完整的測試覆蓋。
3. `handle_key` 應使用查表法或策略模式取代深層 match。

---

### 🟢 P3 — 其他觀察

#### 3.1 技術債追蹤
- 整個程式碼庫僅有 **1 個 TODO**（`acme-accessibility/src/lib.rs:91`）。
- 建議：對於已知但暫不處理的問題，使用 `// TODO(P0-xxx):` 格式標記，方便追蹤。

#### 3.2 CI 管線
- CI 已涵蓋 `fmt`、`check`、`clippy`、`test`、`cargo-deny`、`cargo-audit`，跨 Windows/macOS/Linux。
- 建議：加入 **coverage 報告**（`cargo-llvm-cov`）以量化測試覆蓋率。

#### 3.3 編譯設定
- `[profile.dev]` 使用 `codegen-units = 256` 和 `debug = 1`，適合開發迭代。
- `[profile.release]` 使用 `lto = "thin"` + `codegen-units = 1` + `panic = "abort"`，適合發佈。
- 設定合理，無需調整。

#### 3.4 CBM 索引重複
- CBM 索引中每個檔案出現兩次（來自两次索引），導致統計數據膨脹。
- 建議：重新索引時使用 `incremental: true` 或先刪除舊索引。

---

## 三、建議優先順序

| 優先級 | 項目 | 影響 | 工作量 |
|--------|------|------|--------|
| **P0** | 為核心 crate 補測試 (acme-core, acme-layout, acme-style) | 防止回歸 | 中 |
| **P0** | 為 acme-textinput 補測試 | 核心互動路徑 | 中 |
| **P0** | 拆分巨型函式 (window_event, to_layout_alloc) | 可維護性 | 中 |
| **P1** | 拆分 acme-ui God Crate | 編譯時間、依賴管理 | 大 |
| **P1** | 拆分單檔案 crate (textinput, widgets, render-wgpu) | 可維護性 | 中 |
| **P1** | 重構 to_layout 為 trait dispatch | 降低耦合 | 中 |
| **P2** | 合併/分離 Gallery apps | 減少混淆 | 小 |
| **P2** | 增加 trait 抽象 | 可測試性、擴展性 | 大 |
| **P2** | 拆分高 fan-out 函式 | 可讀性 | 中 |
| **P3** | 加入 CI coverage 報告 | 可見性 | 小 |
| **P3** | 清理 CBM 重複索引 | 分析準確性 | 小 |

---

## 四、架構亮點（值得保持）

- ✅ **清晰的 crate 分層：** core → layout/style/text → widgets → platform/render → apps，依賴方向正確。
- ✅ **CI 完善：** 跨三平台、包含 fmt/clippy/test/audit/deny。
- ✅ **技術債極少：** 僅 1 個 TODO，程式碼庫乾淨。
- ✅ **acme-animation 測試典範：** 44 個測試函式覆蓋 tween、loop、easing、engine，可作為其他 crate 的參考。
- ✅ **acme-ui 模組化良好：** 8 個模組、120 個檔案，元件按功能分類（雖然應進一步拆 crate）。
- ✅ **Rust Edition 2024 + MSRV 1.85：** 使用最新穩定版。
