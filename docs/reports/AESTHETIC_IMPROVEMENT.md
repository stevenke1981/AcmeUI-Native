# AcmeUI-Native 美感改善方案

> 目標：解決文字擠壓問題，提升整體排版視覺呼吸感
> 版本：v1.0 | 日期：2026-07-20

---

## 一、根因分析

### 🔴 主因：Label 佈局高度為 0（最高優先）

**位置**：`crates/acme-widgets/src/lib.rs`

```rust
// 現況：Label 使用完全 default style
Self::Label(_) | Self::TextInput(_) => LayoutNode::leaf(id, LayoutStyle::default()),
```

`LayoutStyle::default()` 所有尺寸皆為 `Length::Auto`。Taffy (flexbox 引擎) 在沒有 measure function 的情況下，會將 auto-height leaf node **壓縮為 0px 高度**。

影響鏈：
```
Label.height = 0
 → gap(4.0) 等於 4px 間距
 → 16px 文字 = 相鄰兩行只差 4px
 → 文字嚴重重疊，視覺上「全部擠在一起」
```

### 🟡 次因：間距 Token 偏小

**位置**：`crates/acme-theme/src/lib.rs`

| Token | 現況 | 建議 | 說明 |
|-------|------|------|------|
| `xs`  | 4px  | 4px  | 保持 |
| `sm`  | 8px  | 8px  | 保持 |
| `md`  | 12px | 16px | 一般內容間距偏緊 |
| `lg`  | 16px | 24px | Section 間距不足 |
| `xl`  | 24px | 40px | 大區塊間距過小 |

### 🟡 次因：Section 內 gap 寫死、不隨 Density 縮放

**位置**：`apps/gallery/src/main.rs` — 各 free-standing helper 函式

```rust
fn anatomy_diagram()   → .gap(4.0)   // ❌ 應為 8.0+
fn keyboard_behavior() → .gap(4.0)   // ❌
fn accessibility_props()→ .gap(4.0)  // ❌
fn sizes_demo()        → .gap(6.0)   // ❌
fn density_demo()      → .gap(4.0)   // ❌
```

### 🟡 次因：line_height 不一致

**位置**：`apps/gallery/src/main.rs` → `add_text()`

```rust
// Theme 定義：1.4
// add_text() 實際使用：
line_height: size * 1.35,  // ❌ 與 theme 不一致，文字底部可能被截
```

### 🟡 次因：文字垂直偏移寫死

**位置**：`apps/gallery/src/main.rs` → `render_content()`

```rust
// 現況：固定 2px offset，不依 rect.height 置中
add_text(..., ([rect.x + 4.0, y + 2.0], fs), ...);
```

---

## 二、解決方案

### Fix 1：給 Label 設定 `min_height`（**必做，最高優先**）

**檔案**：`crates/acme-widgets/src/lib.rs`

```rust
// ❌ 修改前
Self::Label(_) | Self::TextInput(_) => LayoutNode::leaf(id, LayoutStyle::default()),

// ✅ 修改後
Self::Label(l) => LayoutNode::leaf(
    id,
    LayoutStyle {
        // min_height = font_size × line_height，確保 Taffy 保留足夠行高
        // 使用 1.5 作為安全係數（體感留白更舒適）
        min_height: Length::px(l.font_size.unwrap_or(16.0) * 1.5),
        ..Default::default()
    },
),
Self::TextInput(_) => LayoutNode::leaf(id, LayoutStyle::default()),
```

**數學說明**：

設 $f$ 為 font size，$r$ 為 line-height ratio：

$$h_{min} = f \times r$$

以 body\_size = 16px，$r = 1.5$：

$$h_{min} = 16 \times 1.5 = 24\text{px}$$

配合 `gap(8.0)` ⟹ 每行有效間距 = 24 + 8 = **32px**，視覺上舒適。

---

### Fix 2：統一 line_height（必做）

**檔案**：`apps/gallery/src/main.rs` → `add_text()`

```rust
// ❌ 修改前
fn add_text(... size: f32 ...) {
    let style = TextStyle {
        font_size: size,
        line_height: size * 1.35,  // ❌ 與 theme 不一致
        ..TextStyle::default()
    };
```

```rust
// ✅ 修改後：改為傳入 line_height_ratio 參數
fn add_text(
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
    frame: &mut Frame,
    text: &str,
    geometry: ([f32; 2], f32),
    color: ThemeColor,
    scale: f32,
    clip: Option<[f32; 4]>,
    line_height_ratio: f32,   // ← 新增參數
) {
    let (origin, size) = geometry;
    let style = TextStyle {
        font_size: size,
        line_height: size * line_height_ratio,  // ✅ 由呼叫端決定
        ..TextStyle::default()
    };
    // ... 其餘不變
}

// 所有呼叫端傳入 theme.typography.line_height（預設 1.4 → 修改後 1.5）
add_text(
    fonts, atlas, frame, text,
    ([rect.x + 4.0, y_centered], fs),
    colors.text, scale, Some(clip),
    theme.typography.line_height,  // ← 統一來源
);
```

---

### Fix 3：文字垂直置中（建議）

**檔案**：`apps/gallery/src/main.rs` → `render_content()` 的 `WidgetNode::Label` 分支

```rust
// ❌ 修改前：固定 2px offset
let y_text = rect.y - scroll_y + 2.0;

// ✅ 修改後：在 rect 內垂直置中
let line_h = fs * 1.5;                               // 使用相同 ratio
let y_text = rect.y - scroll_y + (rect.height - line_h).max(0.0) * 0.5;
```

---

### Fix 4：調整 SpacingTokens（建議）

**檔案**：`crates/acme-theme/src/lib.rs`

```rust
// ❌ 修改前
spacing: SpacingTokens {
    xs: 4.0,
    sm: 8.0,
    md: 12.0,
    lg: 16.0,
    xl: 24.0,
},

// ✅ 修改後
spacing: SpacingTokens {
    xs:  4.0,   // 不變：微小間距（icon gap 等）
    sm:  8.0,   // 不變：緊湊元素間距
    md: 16.0,   // +4   ：一般文字/元件間距
    lg: 24.0,   // +8   ：Section 間距
    xl: 40.0,   // +16  ：Page 大區塊間距
},

// 同步調整 TypographyTokens
typography: TypographyTokens {
    body_size:   16.0,
    label_size:  14.0,
    line_height:  1.5,  // 1.4 → 1.5：增加行高舒適度
},
```

**注意**：`validate()` 已有 spacing 驗證，無需改測試邏輯；但現有 snapshot test 若寫死舊數值需更新。

---

### Fix 5：Section 內 gap 改為 8.0（建議）

**檔案**：`apps/gallery/src/main.rs`

```rust
// ✅ 所有 free-standing section builder 統一提升 gap
fn anatomy_diagram() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)   // was 4.0
        // ...
}

fn keyboard_behavior() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)   // was 4.0
        // ...
}

fn accessibility_props() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)   // was 4.0
        // ...
}

fn density_demo() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)   // was 4.0
        // ...
}

fn long_text_section() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)   // was 4.0
        // ...
}

fn screenshot_info() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)   // was 4.0
        // ...
}

fn sizes_demo() -> WidgetNode<GalleryMessage> {
    column()
        .gap(10.0)  // was 6.0
        // ...
}
```

---

### Fix 6：Section-level 間距提升（建議）

**檔案**：`apps/gallery/src/main.rs`

```rust
// build_component_page：Section 間距
fn build_component_page(&self, title, sections) {
    let mut page = column::<GalleryMessage>()
        .gap(spacing(self.density, 28.0))    // was 20.0：讓各 section 更分明
        .padding(spacing(self.density, 24.0)); // 保持
    // ...
}

// page_section：標題與分隔線之間的 gap
fn page_section(&self, title, content) {
    column()
        .gap(spacing(self.density, 10.0))    // was 8.0：輕微增加
        // ...
}
```

---

### Fix 7：Sidebar 按鈕間距（建議）

**檔案**：`apps/gallery/src/main.rs` → `sidebar()`

```rust
fn sidebar(&self) -> WidgetNode<GalleryMessage> {
    let mut col = column::<GalleryMessage>()
        .key("sidebar")
        .gap(4.0)         // was 2.0：按鈕間需要更多呼吸空間
        .padding(12.0);
    // ...
}
```

**檔案**：`apps/gallery/src/main.rs` → `apply_gallery_styles()`

```rust
// Sidebar category buttons — 高度從 36px 提升至 40px
for i in 2..=9 {
    sb.children[i].style.width = Length::px(SIDEBAR_WIDTH - 24.0);
    sb.children[i].style.height = Length::px(40.0);   // was 36.0
}
```

---

### Fix 8：KPI Card 改善（建議）

**檔案**：`apps/gallery/src/main.rs`

```rust
fn kpi_card(&self, value: &str, title: &str) -> WidgetNode<GalleryMessage> {
    column()
        .gap(6.0)        // was 4.0
        .padding(16.0)   // was 12.0：增加卡片內距
        .child(label_with_size(value, 22.0))
        .child(label(title))
        .build()
}
```

---

## 三、改動優先級彙整

| 優先級 | Fix | 檔案 | 影響 | 複雜度 |
|--------|-----|------|------|--------|
| 🔴 P0 | Fix 1：Label min_height | `acme-widgets/src/lib.rs` | **消除文字重疊根因** | 低（3行） |
| 🔴 P0 | Fix 2：統一 line_height | `apps/gallery/src/main.rs` | 防止文字底部截切 | 低（+1 param） |
| 🟡 P1 | Fix 4：SpacingTokens | `acme-theme/src/lib.rs` | 全局呼吸感提升 | 低（改數值） |
| 🟡 P1 | Fix 5：Section gap | `apps/gallery/src/main.rs` | 多處小型改善 | 低（批量替換） |
| 🟡 P1 | Fix 7：Sidebar 間距 | `apps/gallery/src/main.rs` | 導覽列觀感 | 低（2行） |
| 🟢 P2 | Fix 3：垂直置中 | `apps/gallery/src/main.rs` | 文字在格子內置中 | 中 |
| 🟢 P2 | Fix 6：Section-level | `apps/gallery/src/main.rs` | 頁面整體層次感 | 低 |
| 🟢 P2 | Fix 8：KPI Card | `apps/gallery/src/main.rs` | 儀表板頁面 | 低（2行） |

---

## 四、預期效果

套用 P0 + P1 修正後：

```
修改前（Label h=0, gap=4）：
  ┌────────────────────┐
  │ Text A              │← y=24, h=0
  │ Text B              │← y=28, h=0  (僅差 4px → 重疊)
  │ Text C              │← y=32, h=0  (嚴重擠壓)
  └────────────────────┘

修改後（Label h=24, gap=8）：
  ┌────────────────────┐
  │                    │
  │  Text A            │← y=0,  h=24
  │                    │
  │                    │ (gap=8)
  │  Text B            │← y=32, h=24
  │                    │
  └────────────────────┘
```

---

## 五、執行建議順序

```bash
# Step 1：修改 Label min_height（核心修復）
# 檔案：crates/acme-widgets/src/lib.rs，約第 203 行

# Step 2：修改 SpacingTokens + line_height
# 檔案：crates/acme-theme/src/lib.rs，約最後 15 行

# Step 3：add_text 加入 line_height_ratio 參數
# 檔案：apps/gallery/src/main.rs，約第 2780 行

# Step 4：批量替換 section gap(4.0) → gap(8.0)
# 使用 sed 或 IDE 全域替換

# Step 5：確認 unit tests 通過
cargo test --workspace

# Step 6：執行 gallery 目視確認
cargo run -p gallery
```

---

> **備注**：若未來 `acme-text` 的 `FontSystem` 支援 measure callback 並回傳 shaped text 實際高度，
> 可將 Fix 1 的 `font_size * 1.5` 替換為精確的 `ShapedText::height`，達到 pixel-perfect layout。
