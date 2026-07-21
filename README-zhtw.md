# AcmeUI Native

**Rust 原生宣告式桌面 UI 執行環境 — 以 wgpu 驅動。**

> **🌐 語言：** [English](README.md) · [繁體中文](README-zhtw.md)

---

## 什麼是 AcmeUI Native？

AcmeUI Native 是一套 **Rust 原生桌面 UI 框架**，透過 **wgpu**（DirectX 12 / Vulkan / Metal）進行 GPU 渲染。它是 AcmeUIKit 的獨立執行時期後繼方案 —— **完全不依賴 GPUI**。

整合了：

- **宣告式 Widget 樹** — 使用 Builder DSL 建構 UI（column、row、button、label…）
- **Flexbox 排版** — 透過 Taffy 引擎
- **GPU 加速渲染** — 批次矩形、圓角、邊框、裁剪、紋理
- **中日韓 + 表情符號文字** — cosmic-text 塑形 + 字形圖集
- **繁體中文 IME** — 預編輯/提交、游標幾何、候選視窗支援
- **語意化設計 Token** — 淺色/深色/高對比主題
- **AccessKit 無障礙** — 螢幕閱讀器支援
- **動畫引擎** — 補間/緩動/彈跳/迴圈
- **80+ 高階元件** — 經由 `acme-ui`（受 shadcn/ui + Material UI + Ant Design 啟發）

### 架構

```text
App → WidgetNode DSL → Retained Tree → Taffy Layout → Scene → wgpu → OS Surface
```

| 圖層 | Crate | 角色 |
|------|-------|------|
| UI 元件 | `acme-ui` | 80+ 高階 Widget（Slider、Switch、DatePicker、Toast、Dock…） |
| Widget 基礎 | `acme-widgets` | WidgetNode 列舉、Builder DSL、Overlay 管理、視覺狀態 |
| 文字編輯 | `acme-textinput` | 游標、選取、剪貼簿、IME 預編輯/提交、復原/重做（✓ 100 項測試） |
| 排版 | `acme-layout` | 基於 Taffy 的 Flexbox 排版引擎 |
| 文字 | `acme-text` | cosmic-text 塑形、字形圖集、中日韓 + 表情符號後備 |
| 主題 | `acme-theme` | 語意色彩 Token V1 + V2、淺色/深色/高對比 |
| 動畫 | `acme-animation` | 補間引擎、緩動、迴圈、延遲 |
| 渲染 | `acme-render-wgpu` | GPU Surface 生命週期、批次矩形/路徑渲染、裁剪堆疊 |
| 平台 | `acme-platform` | winit 事件迴圈、Application Trait、WindowId、IME、GPU 復原鉤子 |
| 無障礙 | `acme-accessibility` | AccessKit 橋接、焦點管理、動作路由 |
| 核心 | `acme-core` | 樹、幾何、事件、場景模型 —— 平台無關 |
| 開發工具 | `acme-devtools` | Widget 檢查器、排版除錯器、幀指標、Surface 狀態 |

### Gallery 應用程式

| 應用 | 套件 | 用途 |
|------|------|------|
| `apps/gallery` | `acme-gallery` | 主要展示 —— 8 類別導覽、即時 Data/Nav 展示（Tree、Table、DataGrid、VirtualList）、截圖模式 |
| `apps/acme-gallery` | `acme-ui-gallery` | V2 元件展示 —— 80+ 高階 `acme-ui` 元件 |
| `apps/playground` | `playground` | 最小開發沙盒，快速實驗 |
| `apps/benchmark` | `benchmark` | 無頭排版/調和/幀建置基準測試 |

---

## 快速開始

### 前置需求

- **Rust** 1.85+（MSRV，edition 2024）
- **Windows 10/11**（主要目標；Linux/macOS 次要）
- 支援 **DirectX 12** 或 **Vulkan** 的 GPU

### 執行 Gallery

```powershell
cargo run -p acme-gallery
```

Gallery 展示：矩形、圓角邊框、裁剪滾動內容、cosmic-text 渲染（英文、繁體中文、表情符號）、淺色/深色主題切換、IME 輸入，以及互動式 Tree/Table/DataGrid/VirtualList 展示。

### 執行 V2 展示

```powershell
cargo run -p acme-ui-gallery
```

### 驗證

```powershell
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

---

## 專案狀態

| 領域 | 狀態 |
|------|------|
| 核心框架 | ✅ **穩定** — 樹、排版、渲染、文字、主題、動畫、無障礙、Widget |
| 文字輸入 + IME | ✅ **穩定** — 100 項測試、游標幾何、繁體中文預編輯/提交 |
| 資料元件 | 🧪 **實驗性** — Tree、Table、DataGrid、VirtualList 附 Gallery 即時展示 |
| UI 元件庫 | 🧪 **實驗性** — 80+ 元件（acme-ui），於 acme-ui-gallery 展示 |
| GPU 裝置遺失復原 | 🧪 **已接線** — 純測試狀態機 + `on_gpu_recovered` 鉤子；**手動驗證待完成** |
| 繁體中文注音 IME | 🧪 **架構完成** — **手動驗證待完成** |
| 螢幕截圖黃金測試 | 📋 **已搭建骨架** — 尚未納入 CI |
| CI 基準測試 | 📋 **尚未** — 無效能門檻 |

> **完整狀態：**[`STATUS.md`](status/STATUS.md) · **手動檢查清單：**[`MANUAL_VALIDATION.md`](guides/MANUAL_VALIDATION.md)

---

## 儲存庫結構

```
AcmeUI-Native/
├── crates/
│   ├── acme-core/          # 樹、幾何、事件、場景
│   ├── acme-platform/      # winit 迴圈、Application Trait、IME
│   ├── acme-render-wgpu/   # GPU Surface + 批次渲染器
│   ├── acme-layout/        # Taffy 排版封裝
│   ├── acme-text/          # cosmic-text 塑形 + 字形圖集
│   ├── acme-textinput/     # 文字編輯狀態機
│   ├── acme-theme/         # 設計 Token（淺色/深色/高對比）
│   ├── acme-animation/     # 補間引擎
│   ├── acme-style/         # 樣式抽象層
│   ├── acme-widgets/       # WidgetNode 列舉 + Builder DSL
│   ├── acme-ui/            # 80+ 高階元件
│   ├── acme-accessibility/ # AccessKit 橋接
│   └── acme-devtools/      # 檢查器、指標、除錯器
├── apps/
│   ├── gallery/            # 主要展示應用
│   ├── acme-gallery/       # V2 元件展示
│   ├── playground/         # 開發沙盒
│   └── benchmark/          # 無頭基準測試
├── docs/                   # 架構、手動驗證、ADR
├── scripts/                # CI / 開發腳本
├── spec.md                 # 專案規格
├── plan.md                 # 開發計畫
├── todos.md                # 任務追蹤
├── docs/architecture/      # 詳細架構說明
└── AGENTS.md               # Agent 工作流程規則
```

---

## 設計原則

- **語意優先**：所有顏色、間距、排版經由設計 Token —— 無硬編碼值
- **前景/背景配對**：每個表面 Token 都有對應的文字 Token
- **桌面最佳化**：比 Web 預設更緊湊，比經典 Win32 更多呼吸空間
- **WCAG AA**：所有文字/背景配對滿足最低對比度；可見焦點環
- **函數式管線**：事件處理分層為 `hit → activate → dispatch → match`（無單體 `&mut self`）
- **GPU 友善**：批次處理一切；最小化狀態變更；持久緩衝區搭配 epoch 失效

> **完整設計系統：**[`DESIGN_SYSTEM.md`](architecture/DESIGN_SYSTEM.md)

---

---

## 函數式與分層架構 — 轉換程度

主要展示應用 `apps/gallery` 已完成從 **2593 行單體**到分層函數式架構的完整轉換：

### 事件管線（第 1 → 4 層）

| 圖層 | 檔案 | 角色 | 函式數 | `Gallery` 方法數 |
|------|------|------|--------|-----------------|
| **第 4 層** | `main.rs`（event match） | 純比對派發 | 0 | `event()`（每行 3 行） |
| **第 3 層** | `events/dispatch.rs` | 各事件類型處理器 | **10 個 pub fn** | 0 |
| **第 2 層** | `events/activate.rs` + `events/ime.rs` | 狀態轉換 | **2 個 pub fn** | 0 |
| **第 1 層** | `events/hit.rs` | 純查詢 | **1 個 pub fn** | 0 |

**全部 13 個事件處理器皆為自由函式** — 零個 `impl Gallery` 方法，零個 `&mut self`。

### 渲染管線（第 1 → 4 層）

| 圖層 | 模組 | 角色 | 函式數 |
|------|------|------|--------|
| **第 4 層** | `render/frame.rs` | 管線編排 | **8 個 pub fn** |
| **第 3 層** | `render/content.rs` + `render/hit_test.rs` | Widget 渲染 | **6 個 pub fn** |
| **第 2 層** | `render/style.rs` + `render/text.rs` | 樣式與文字輔助 | **3 個 pub fn** |
| **第 1 層** | `render/geometry.rs` + `render/layout.rs` | 基礎圖元 | **6 個 pub fn** |

**全部 23 個渲染函式皆為自由函式** — 零個 `impl Gallery` 方法。

### main.rs 縮減

| 指標 | 轉換前 | 轉換後 |
|------|--------|--------|
| 總行數 | 2,593 | **538**（-79%） |
| `impl Gallery` 方法 | ~25 | 2（new, window_config）+ 3（Application trait） |
| 自由函式 | 0 | **40+**（分散於 events/、render/、pages/、helpers/） |
| 事件處理 | 內聯於 `event()` | **13 個自由函式**於 `events/dispatch.rs` |
| 渲染步驟 | 內聯於 `frame()` | **23 個自由函式**於 `render/`（8 個檔案） |
| 頁面建構器 | 內聯 | **9 個檔案**於 `pages/` |

### 資料元件（acme-widgets）

| 元件 | 行數 | Builder API | 狀態管理 | 測試數 |
|------|------|-------------|----------|--------|
| `data/tree.rs` | 563 | ✅ TreeNode, Tree | ✅ 展開/收合、選取、即時輸入 | 12 |
| `data/table.rs` | 824 | ✅ TableColumn, TableRow | ✅ 排序、選取、調整大小、鍵盤導航 | 24 |
| `data/datagrid.rs` | 663 | ✅ DataGridColumn, DataGridRow | ✅ 凍結儲存格、合併、雙向虛擬化 | 14 |
| `data/virtual_list.rs` | 562 | ✅ 項目高度、overscan | ✅ 可視範圍、錨點、高度快取 | 15 |
| **總計** | **2,612** | — | — | **65 項測試** |

### UI 元件庫（acme-ui）

| 模組 | 元件數 | 功能閘門 | 預設啟用 |
|------|--------|----------|---------|
| `foundations/` | 26（Alert、Badge、Calendar、Icon、Link、Progress、Skeleton、Tag…） | `foundations` | ✅ |
| `inputs/` | 28（ButtonGroup、Checkbox、Combobox、DatePicker、Radio、Slider、Switch、Select…） | `inputs` | ✅ |
| `layout/` | 12（Form、Grid、Pagination、Tabs、Toolbar、SplitPanel、Stepper…） | `layout` | ✅ |
| `overlay/` | 8（Drawer、Toast、ConfirmDialog、ContextMenu、HoverCard…） | `overlay` | ✅ |
| `desktop/` | 11（TitleBar、Dock、Sidenav、Menubar、CommandBar、PropertyGrid…） | `desktop` | — |
| `charts/` | 6（LineChart、PieChart、BarChart、Sparkline、AreaChart、Gauge） | `charts` | — |
| `mobile/` | 3（BottomNav、BottomSheet、PullToRefresh） | `mobile` | — |
| `browser/` | 3（Carousel、Lightbox、ZoomView） | `browser` | — |
| **總計** | **97 個元件檔案** | 8 個功能閘門 | 4 個預設 |

每個元件遵循 **Builder 模式**：`Component::new() → .option(value) → .on_event(message) → .build() → WidgetNode<M>`。

---

## 關鍵設計決策

| 決策 | 理由 |
|------|------|
| **無 GPUI** | 獨立執行時期路徑，無 Zed 依賴 |
| **wgpu** | 跨平台 GPU 抽象層（D3D12/Vulkan/Metal） |
| **Taffy** | 經過實戰考驗的純 Rust Flexbox 排版 |
| **cosmic-text** | 成熟的塑形 + 字形快取，支援中日韓 |
| **訊息驅動** | Widget 透過 `Message<M>` 列舉溝通 —— 無回呼 |
| **分層事件** | `hit → activate → dispatch → match` 實現欄位層級借用檢查 |
| **持久緩衝區** | 僅在 `gpu_epoch` 增加時重新上傳 |
| **不使用 `cargo clean`** | 從不作為常規修復手段 |

---

## 給 AI Agent

**本專案使用 Controlled Workflow。** 請參閱 [`AGENTS.md`](AGENTS.md) 了解：

- 範圍閘門與邊界合約
- 任務分類（T0–T3）
- 驗證優先工作流程
- 子 Agent 交接規則
- 持久記憶（經由 `.opencode/memory/`）

Agent 關鍵文件：

| 文件 | 用途 |
|------|------|
| [`spec.md`](spec.md) | 專案規格與閘門 |
| [`plan.md`](plan.md) | 階段計畫（P0–P10） |
| [`todos.md`](todos.md) | 任務追蹤含完成狀態 |
| [`ARCHITECTURE.md`](architecture/ARCHITECTURE.md) | Crate 階層與資料流 |
| [`STATUS.md`](status/STATUS.md) | 元件成熟度（穩定/實驗性/僅架構） |
| [`AGENTS.md`](AGENTS.md) | Agent 工作流程規則 |
| [`docs/adr/`](docs/adr/) | 架構決策紀錄 |

---

## 授權

MIT OR Apache-2.0
