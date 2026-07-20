# AcmeUI-Native Gallery 文字重疊修正方案

**Repository**：`stevenke1981/AcmeUI-Native`  
**問題範圍**：Gallery 一般 `Label`、Anatomy、Variants、Sizes、長繁體中文等內容垂直重疊。  
**優先級**：P0 / Blocker

## 1. 根因

問題不是 `cosmic-text` 畫錯字，而是 **文字在 Layout 階段沒有取得 intrinsic size**：

1. `Label` 保存了 `text`、`font_size`、`cached`。
2. `WidgetNode::to_layout_alloc()` 卻把 `Label` 轉成 `LayoutStyle::default()` 的 Leaf。
3. Leaf 的寬高都是 `Auto`，但 `LayoutEngine` 使用 `compute_layout()`，沒有提供文字 Measure Function。
4. Taffy 不知道文字實際高度，因此多數 Label 的 Layout Rect 高度接近 `0`。
5. Renderer 仍以 14–24px 字體在 `rect.y + 2` 畫字，下一個 Label 只移動 Column 的 4–8px gap，造成重疊。

Button 與 Toolbar 較正常，是因為它們被手動設定為 32px 或 36px 高度。

## 2. 程式證據

### `crates/acme-widgets/src/foundations/label.rs`

```rust
pub struct Label {
    pub text: String,
    pub font_size: Option<f32>,
    pub cached: Option<ShapedText>,
}
```

### `crates/acme-widgets/src/lib.rs`

目前：

```rust
Self::Label(_) | Self::TextInput(_) => {
    LayoutNode::leaf(id, LayoutStyle::default())
}
```

`font_size` 與 shaped text 沒有進入 Layout。

### `crates/acme-layout/src/lib.rs`

目前：

```rust
tree.compute_layout(root_node, available_space)?;
```

應改成使用 Taffy Leaf Measure 的：

```rust
tree.compute_layout_with_measure(...)?;
```

檔案後段雖已有 `measure_text()` 和 `ShapedText`，目前沒有接到 Taffy Layout。

### `apps/gallery/src/main.rs`

目前 Paint：

```rust
let y = rect.y - scroll_y;
let fs = label.font_size.unwrap_or(theme.typography.body_size);
add_text(..., ([rect.x + 4.0, y + 2.0], fs), ...);
```

當 `rect.height == 0` 時，文字仍以完整字體高度繪製，因此互相覆蓋。

## 3. 不可接受的假修法

不要只做：

```rust
column().gap(20.0)
```

也不要為 Gallery 每一行手動累加 Y 座標。

這些方式無法正確支援：

- 不同字級
- 繁體中文與英文不同 metrics
- 長文字換行
- 125%、150%、200% DPI
- TextInput、Table、Tree、Menu、Tooltip

## 4. 修正策略

分成兩階段：

- **階段 A：立即止血**：為 Label/TextInput 提供安全最小高度。
- **階段 B：正式修正**：接入 Taffy `compute_layout_with_measure()` 與 cosmic-text 實際文字測量。

---

## 5. 階段 A：立即止血

修改 `crates/acme-widgets/src/lib.rs`。

將：

```rust
Self::Label(_) | Self::TextInput(_) => {
    LayoutNode::leaf(id, LayoutStyle::default())
}
```

暫時改為：

```rust
Self::Label(label) => {
    let font_size = label.font_size.unwrap_or(16.0);
    let line_height = (font_size * 1.4).ceil();

    LayoutNode::leaf(
        id,
        LayoutStyle {
            min_height: Length::px(line_height),
            flex_shrink: 0.0,
            ..Default::default()
        },
    )
}

Self::TextInput(_) => LayoutNode::leaf(
    id,
    LayoutStyle {
        min_height: Length::px(40.0),
        flex_shrink: 0.0,
        ..Default::default()
    },
)
```

這只是 fallback。正式版本不得永久寫死 Theme body size。

### TextInput Marker

Gallery 使用特殊 Label 當 TextInput placeholder：

```rust
.child(label(TEXT_INPUT_MARKER))
```

必須確保該 Leaf 至少有 40px 高度，否則 `render_text_input()` 仍會取得過小 Rect。

### 階段 A 驗收

- Anatomy 每行不重疊。
- Variants Button 不碰到標題。
- Sizes、Light/Dark、Density 等區塊可讀。
- 長繁體中文不壓到下一節。
- ScrollView content height 能增加。

---

## 6. 階段 B：正式 Intrinsic Text Layout

### 6.1 為 LayoutNode 增加 Measure Context

修改 `crates/acme-layout/src/lib.rs`：

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum MeasureContent {
    None,
    Text(TextMeasureSpec),
    TextInput(TextMeasureSpec),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextMeasureSpec {
    pub text: String,
    pub font_size: f32,
    pub line_height: f32,
    pub wrap: TextWrapMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextWrapMode {
    None,
    Word,
    Character,
}
```

擴充：

```rust
pub struct LayoutNode {
    pub id: NodeId,
    pub style: LayoutStyle,
    pub measure: MeasureContent,
    pub children: Vec<LayoutNode>,
}
```

新增：

```rust
pub fn text_leaf(
    id: NodeId,
    style: LayoutStyle,
    measure: TextMeasureSpec,
) -> Self
```

### 6.2 Label 將文字資訊傳入 Layout

修改 `crates/acme-widgets/src/lib.rs`：

```rust
Self::Label(label) => LayoutNode::text_leaf(
    id,
    LayoutStyle {
        flex_shrink: 1.0,
        ..Default::default()
    },
    TextMeasureSpec {
        text: label.text.clone(),
        font_size,
        line_height,
        wrap: TextWrapMode::Word,
    },
)
```

更好的 API 是讓 Widget-to-Layout 接受 Theme typography：

```rust
pub struct WidgetLayoutContext {
    pub body_font_size: f32,
    pub line_height: f32,
    pub scale_factor: f32,
}

pub fn to_layout_with_context(
    &self,
    id: NodeId,
    context: &WidgetLayoutContext,
) -> LayoutNode
```

### 6.3 接入 Taffy Measure Function

將 `TaffyTree<u64>` 改成帶 Node Context 的 Tree，例如：

```rust
enum NodeContext {
    Empty,
    Text(TextMeasureSpec),
    TextInput(TextMeasureSpec),
}
```

文字 Leaf 使用：

```rust
tree.new_leaf_with_context(style, NodeContext::Text(spec))
```

Layout 使用：

```rust
tree.compute_layout_with_measure(
    root_node,
    available_space,
    |known_dimensions, available_space, _node_id, node_context, _style| {
        measure_leaf(
            known_dimensions,
            available_space,
            node_context,
            fonts,
            scale_factor,
        )
    },
)?;
```

閉包參數型別依目前鎖定的 Taffy 0.9.x API 調整。

### 6.4 新增正式 API

```rust
pub fn compute_with_text(
    &mut self,
    root: &LayoutNode,
    viewport: (f32, f32),
    fonts: &mut FontSystem,
    scale_factor: f32,
) -> Result<LayoutSnapshot, LayoutError>
```

Measure Function 必須：

1. 取得 known width/height。
2. 從 available width 產生 `TextConstraints.max_width`。
3. 用 `FontSystem::shape()` 測量。
4. 回傳 shaped width/height。
5. 高度不得小於 line height。

概念：

```rust
let style = TextStyle {
    font_size: spec.font_size,
    line_height: spec.line_height,
    ..TextStyle::default()
};

let shaped = fonts.shape(
    &spec.text,
    &style,
    TextConstraints {
        max_width,
        wrap,
    },
    scale_factor,
);

Size {
    width: known_width.unwrap_or(shaped.width),
    height: known_height.unwrap_or(shaped.height.max(spec.line_height)),
}
```

---

## 7. Gallery 接線

修改 `apps/gallery/src/main.rs`。

目前 Theme 在 Layout 之後才建立。正式修正後，Typography 是 Layout Input，因此 Theme 必須提前：

```rust
let theme = if self.dark {
    Theme::dark()
} else {
    Theme::light()
};

let description = self.description();

let layout_context = WidgetLayoutContext {
    body_font_size: theme.typography.body_size,
    line_height: theme.typography.line_height,
    scale_factor: context.scale_factor,
};

let mut root = description.to_layout_with_context(
    NodeId::new(1),
    &layout_context,
);

apply_gallery_styles(&mut root, width, height);

let snapshot = self.layout.compute_with_text(
    &root,
    (width, height),
    &mut self.fonts,
    context.scale_factor,
)?;
```

Paint 可以繼續使用 Layout Rect，但不得再假設 Auto Leaf 自動具有文字高度。

---

## 8. 長繁體中文換行

中文不一定有空白，`Word Wrap` 不足以處理所有 CJK 文本。

建議 Label API：

```rust
label(LONG_CHINESE_TEXT)
    .font_size(14.0)
    .wrap(TextWrapMode::Character)
```

並確保內容區：

```rust
content.style.width = Length::Percent(1.0);
content.style.max_width = Length::Percent(1.0);
content.style.flex_shrink = 1.0;
```

長文字在窄視窗下應增加高度，而不是向右無限延伸或覆蓋下一節。

---

## 9. Cache 修正

`Label.cached` 放在 Widget 裡不理想，因為 Gallery 每幀重新建立：

```rust
let description = self.description();
```

建議把快取放到 Runtime：

```rust
pub struct TextMeasureCache {
    entries: HashMap<TextCacheKey, ShapedText>,
}
```

Key 至少包含：

- NodeId
- text hash
- font size / weight / family
- max width
- DPI scale
- line height
- wrap mode

失效條件：Text、Theme、Width、DPI、Font 任何一項改變。

先修正 Layout，再做 Cache。

---

## 10. 渲染位置

一般 Label 建議 Top Align：

```rust
let text_y = rect.y;
```

Button、TextInput 才垂直置中：

```rust
let text_y = rect.y + ((rect.height - measured.height) * 0.5).max(0.0);
```

不要所有文字都固定 `y + 2.0`。

---

## 11. 必須新增的測試

### Label 高度不為零

```rust
#[test]
fn label_has_non_zero_intrinsic_height() {
    let rect = measure_label("Typography", 24.0, 800.0);
    assert!(rect.height >= 24.0);
}
```

### Column 文字不重疊

```rust
#[test]
fn labels_in_column_do_not_overlap() {
    let a = rect(first_label);
    let b = rect(second_label);
    assert!(b.y >= a.y + a.height + gap - 0.5);
}
```

### 字級影響高度

```rust
assert!(measure_label("Text", 24.0).height > measure_label("Text", 12.0).height);
```

### CJK 換行

```rust
assert!(measure_cjk(240.0).height > measure_cjk(600.0).height);
```

### Scroll Metrics

完整 Typography Page 的：

```rust
content_height > viewport_height
```

且最後一節 Screenshot Mode 可捲動到可見。

---

## 12. 視覺回歸矩陣

產生：

```text
1280×800  Light  Comfortable
1280×800  Dark   Comfortable
1024×700  Light  Compact
800×600   Dark   Compact
```

手動 DPI：

```text
Windows 100%
Windows 125%
Windows 150%
Windows 200%
```

必查頁面：

- Foundations / Typography
- Inputs / TextInput
- Navigation / Sidebar
- Data / Table
- Patterns / Settings Page
- Stress Tests / Long Text

---

## 13. Agent 執行順序

### P0-1：立即止血

- Label min height
- TextInput min height
- Gallery 可讀

### P0-2：正式 Measure Context

- `MeasureContent`
- `TextMeasureSpec`
- `new_leaf_with_context`
- `compute_layout_with_measure`

### P0-3：Gallery 接線

- Theme 在 Layout 前建立
- 呼叫 `compute_with_text`
- CJK wrapping

### P1：效能與 Cache

- Text Measure Cache
- Resize/DPI invalidation
- Visual regression

---

## 14. Agent Master Prompt

```text
請修復 stevenke1981/AcmeUI-Native 的 Gallery 文字重疊問題。

已確認根因：
1. WidgetNode::Label 在 to_layout_alloc 中使用 LayoutStyle::default()。
2. Label 的 Auto width/height 沒有 intrinsic measurement。
3. LayoutEngine 使用 Taffy compute_layout，而不是 compute_layout_with_measure。
4. Renderer 仍按照實際字體大小在零高度或過小 Rect 中畫字。

執行要求：

P0：
- 先為 Label 與 TextInput 加入安全 min_height，讓 Gallery 立即恢復可讀。
- 不得只增加 Column gap 或手動 y offset。

正式修正：
- 在 acme-layout 增加文字 Measure Context。
- 使用 Taffy 0.9 的 new_leaf_with_context 與 compute_layout_with_measure。
- 使用 acme-text / cosmic-text 的 shaped width、height 回傳 Leaf size。
- Label font_size、line_height、wrap 必須參與 Layout。
- Gallery 必須在建立 Layout Tree 前先取得 Theme Typography。
- 長繁體中文必須支援 Character Wrap。
- 不得破壞 Button、ScrollView、Tree、Table、DataGrid。

測試：
- Label intrinsic height 不得為 0。
- Column 中相鄰 Label 不得重疊。
- 24px Label 高度必須大於 12px Label。
- 長繁體中文在窄寬度下必須增加高度。
- 產生 1280×800、1024×700、800×600 的 Light/Dark 截圖。

驗證：
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p acme-gallery

最後更新 todos.md、test.md、final.md。
final.md 必須誠實列出根因、修改檔案、自動測試、人工 DPI 驗證狀態與未完成項目。
```

---

## 15. Definition of Done

- Typography 頁不再有文字重疊。
- 每個 Label Layout Rect 高度大於 0。
- 字級變化會改變 Layout 高度。
- 長繁體中文在窄內容區換行。
- ScrollView `content_height` 包含完整文字高度。
- TextInput Rect 高度符合控制項 token。
- Light、Dark、Compact、Comfortable 全部可讀。
- 100%、125%、150%、200% DPI 不重疊。
- 不使用人工硬編碼每一行 Y 座標。
- fmt、check、clippy、test 全部通過。
