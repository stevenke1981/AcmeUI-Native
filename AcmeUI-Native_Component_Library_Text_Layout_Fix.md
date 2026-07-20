# AcmeUI-Native 元件庫文字版面修正規格

**專案**：`stevenke1981/AcmeUI-Native`  
**文件用途**：提供 Codex、OpenCode 或其他 Agents 直接執行修正  
**優先級**：P0 / Blocker  
**主要問題**：Gallery 與未來使用 AcmeUI-Native 的應用程式，會因 Label 缺少 intrinsic text measurement 而發生文字重疊

---

# 1. 修正目標

本次修正不能只處理 `apps/gallery`。

必須修正 AcmeUI-Native 元件庫與 Layout 接合層，使所有文字型元件都能把實際文字尺寸提供給 Taffy。

完成後應達成：

- Label 不再取得 0 高度或過小高度
- 不同字級會產生不同 Layout 高度
- 長繁體中文可正常換行
- ScrollView 的 content height 正確
- TextInput 具有穩定控制項高度
- Button、Menu、Tree、Table、DataGrid 等文字內容不會重疊
- 100%、125%、150%、200% DPI 都能正常顯示
- Gallery 不再依賴人工 Y 座標或大量固定高度

---

# 2. 根因

目前 `Label` 具備：

```rust
pub struct Label {
    pub text: String,
    pub font_size: Option<f32>,
    pub cached: Option<ShapedText>,
}
```

但在 `WidgetNode::to_layout_alloc()` 中，Label 仍被轉成：

```rust
Self::Label(_) | Self::TextInput(_) => {
    LayoutNode::leaf(id, LayoutStyle::default())
}
```

這代表：

```text
width        = Auto
height       = Auto
measure data = None
```

Layout Engine 目前使用普通：

```rust
tree.compute_layout(...)
```

沒有使用 Taffy 的文字測量閉包，因此 Taffy 不知道 Label 的實際寬高。

Renderer 卻仍使用 14px、16px、24px 等字級繪圖，導致多個 Label 被安排在幾乎相同的 Y 座標。

---

# 3. 修正範圍

## 必須修改

```text
crates/acme-layout/src/lib.rs
crates/acme-widgets/src/lib.rs
crates/acme-widgets/src/foundations/label.rs
crates/acme-text/src/*
crates/acme-textinput/src/lib.rs
apps/gallery/src/main.rs
```

## 建議同步檢查

```text
crates/acme-widgets/src/foundations/button.rs
crates/acme-widgets/src/overlay/*
crates/acme-widgets/src/navigation/*
crates/acme-widgets/src/data/*
crates/acme-theme/src/lib.rs
```

## 本次不需要大改

```text
crates/acme-render-wgpu
```

目前文字可以被成功畫出，主要問題不在 Renderer，而在 Layout 尺寸計算。

---

# 4. 實作階段

# P0-1：立即止血修正

先讓 Gallery 與現有元件不再大面積重疊。

修改：

```text
crates/acme-widgets/src/lib.rs
```

將 Label 與 TextInput 分開處理。

```rust
Self::Label(label) => {
    let font_size = label.font_size.unwrap_or(16.0);
    let fallback_line_height = (font_size * 1.4).ceil();

    LayoutNode::leaf(
        id,
        LayoutStyle {
            min_height: Length::px(fallback_line_height),
            flex_shrink: 0.0,
            ..Default::default()
        },
    )
}

Self::TextInput(_) => {
    LayoutNode::leaf(
        id,
        LayoutStyle {
            min_height: Length::px(40.0),
            flex_shrink: 0.0,
            ..Default::default()
        },
    )
}
```

## 限制

這只是過渡修正。

不得把 `16.0` 與 `40.0` 當成最終 Theme 規格永久寫死。

---

# P0-2：建立文字 Measure Context

修改：

```text
crates/acme-layout/src/lib.rs
```

新增：

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
    pub max_lines: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextWrapMode {
    None,
    Word,
    Character,
}
```

擴充 `LayoutNode`：

```rust
pub struct LayoutNode {
    pub id: NodeId,
    pub style: LayoutStyle,
    pub measure: MeasureContent,
    pub children: Vec<LayoutNode>,
}
```

修改建構函式：

```rust
impl LayoutNode {
    pub fn leaf(id: NodeId, style: LayoutStyle) -> Self {
        Self {
            id,
            style,
            measure: MeasureContent::None,
            children: Vec::new(),
        }
    }

    pub fn text_leaf(
        id: NodeId,
        style: LayoutStyle,
        measure: TextMeasureSpec,
    ) -> Self {
        Self {
            id,
            style,
            measure: MeasureContent::Text(measure),
            children: Vec::new(),
        }
    }

    pub fn text_input_leaf(
        id: NodeId,
        style: LayoutStyle,
        measure: TextMeasureSpec,
    ) -> Self {
        Self {
            id,
            style,
            measure: MeasureContent::TextInput(measure),
            children: Vec::new(),
        }
    }
}
```

---

# P0-3：讓 Taffy 使用文字測量

目前 Layout Engine 使用：

```rust
tree.compute_layout(root_node, available_space)
```

改為帶測量的 API：

```rust
tree.compute_layout_with_measure(
    root_node,
    available_space,
    |known_dimensions, available_space, node_id, node_context, style| {
        measure_node(
            known_dimensions,
            available_space,
            node_id,
            node_context,
            style,
            fonts,
            scale_factor,
        )
    },
)
```

具體閉包簽名需依專案鎖定的 Taffy 0.9 API 調整。

建立 Taffy Leaf 時：

```rust
match &node.measure {
    MeasureContent::None => {
        tree.new_leaf(style)
    }
    MeasureContent::Text(spec) => {
        tree.new_leaf_with_context(
            style,
            NodeMeasureContext::Text(spec.clone()),
        )
    }
    MeasureContent::TextInput(spec) => {
        tree.new_leaf_with_context(
            style,
            NodeMeasureContext::TextInput(spec.clone()),
        )
    }
}
```

新增：

```rust
#[derive(Clone, Debug)]
enum NodeMeasureContext {
    Text(TextMeasureSpec),
    TextInput(TextMeasureSpec),
}
```

---

# P0-4：提供正式 Layout API

建議保留目前 API：

```rust
pub fn compute(
    &mut self,
    root: &LayoutNode,
    viewport: (f32, f32),
) -> Result<LayoutSnapshot, LayoutError>
```

但新增正式文字版本：

```rust
pub fn compute_with_text(
    &mut self,
    root: &LayoutNode,
    viewport: (f32, f32),
    fonts: &mut FontSystem,
    scale_factor: f32,
) -> Result<LayoutSnapshot, LayoutError>
```

短期內：

- 沒有文字的單元測試可繼續使用 `compute`
- Gallery 與真實 App 必須改用 `compute_with_text`

長期可考慮將普通 `compute` 標記為低階 API。

---

# 5. 文字測量實作

建議建立：

```rust
fn measure_text_leaf(
    spec: &TextMeasureSpec,
    known_width: Option<f32>,
    known_height: Option<f32>,
    available_width: AvailableSpace,
    fonts: &mut FontSystem,
    scale_factor: f32,
) -> Size<f32>
```

概念：

```rust
fn measure_text_leaf(
    spec: &TextMeasureSpec,
    known_width: Option<f32>,
    known_height: Option<f32>,
    available_width: AvailableSpace,
    fonts: &mut FontSystem,
    scale_factor: f32,
) -> Size<f32> {
    let max_width = known_width.or_else(|| match available_width {
        AvailableSpace::Definite(value) => Some(value),
        AvailableSpace::MinContent => None,
        AvailableSpace::MaxContent => None,
    });

    let style = TextStyle {
        font_size: spec.font_size,
        line_height: spec.line_height,
        ..TextStyle::default()
    };

    let constraints = TextConstraints {
        max_width,
        wrap: match spec.wrap {
            TextWrapMode::None => TextWrap::None,
            TextWrapMode::Word => TextWrap::Word,
            TextWrapMode::Character => TextWrap::Character,
        },
    };

    let shaped = fonts.shape(
        &spec.text,
        &style,
        constraints,
        scale_factor,
    );

    Size {
        width: known_width.unwrap_or(shaped.width.max(1.0)),
        height: known_height.unwrap_or(
            shaped.height.max(spec.line_height)
        ),
    }
}
```

`TextWrap::Character` 名稱請依 `acme-text` 目前實際 Enum 調整。

---

# 6. Label API 修正

修改：

```text
crates/acme-widgets/src/foundations/label.rs
```

建議擴充：

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    pub text: String,
    pub font_size: Option<f32>,
    pub line_height: Option<f32>,
    pub wrap: LabelWrap,
    pub max_lines: Option<usize>,
}
```

新增：

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LabelWrap {
    #[default]
    None,
    Word,
    Character,
}
```

Builder API：

```rust
pub fn label<M>(text: impl Into<String>) -> LabelBuilder<M>;

label("一般文字")
    .font_size(14.0)
    .line_height(20.0)
    .wrap(LabelWrap::Word)
    .build();

label("這是一段很長的繁體中文文字")
    .font_size(14.0)
    .wrap(LabelWrap::Character)
    .build();
```

為了減少破壞性改動，也可保留：

```rust
pub fn label<M>(text: impl Into<String>) -> WidgetNode<M>
```

另外新增：

```rust
pub fn text_label<M>(text: impl Into<String>) -> LabelBuilder<M>
```

---

# 7. Widget → Layout 需要 Typography Context

不要在 `acme-widgets` 永久硬編碼 Theme 字級。

新增：

```rust
pub struct WidgetLayoutContext {
    pub body_font_size: f32,
    pub body_line_height: f32,
    pub label_font_size: f32,
    pub control_height: f32,
    pub scale_factor: f32,
}
```

新增 API：

```rust
pub fn to_layout_with_context(
    &self,
    id: NodeId,
    context: &WidgetLayoutContext,
) -> LayoutNode
```

Label 分支：

```rust
Self::Label(label) => {
    let font_size = label
        .font_size
        .unwrap_or(context.body_font_size);

    let line_height = label
        .line_height
        .unwrap_or(context.body_line_height);

    LayoutNode::text_leaf(
        id,
        LayoutStyle {
            flex_shrink: 1.0,
            min_height: Length::px(line_height),
            ..Default::default()
        },
        TextMeasureSpec {
            text: label.text.clone(),
            font_size,
            line_height,
            wrap: match label.wrap {
                LabelWrap::None => TextWrapMode::None,
                LabelWrap::Word => TextWrapMode::Word,
                LabelWrap::Character => TextWrapMode::Character,
            },
            max_lines: label.max_lines,
        },
    )
}
```

TextInput：

```rust
Self::TextInput(input) => {
    let font_size = input
        .font_size
        .unwrap_or(context.body_font_size);

    LayoutNode::text_input_leaf(
        id,
        LayoutStyle {
            min_height: Length::px(context.control_height),
            flex_shrink: 0.0,
            ..Default::default()
        },
        TextMeasureSpec {
            text: input.value.clone(),
            font_size,
            line_height: context.body_line_height,
            wrap: TextWrapMode::None,
            max_lines: Some(1),
        },
    )
}
```

---

# 8. Theme Typography 必須參與 Layout

修改：

```text
apps/gallery/src/main.rs
```

目前 Theme 是在 Layout 後才建立，需改為在 Layout 前建立。

正確順序：

```rust
fn frame(&mut self, context: FrameContext) -> Frame {
    let theme = if self.dark {
        Theme::dark()
    } else {
        Theme::light()
    };

    let description = self.description();

    let layout_context = WidgetLayoutContext {
        body_font_size: theme.typography.body_size,
        body_line_height: theme.typography.body_line_height_px(),
        label_font_size: theme.typography.label_size,
        control_height: theme.controls.medium_height,
        scale_factor: context.scale_factor,
    };

    let mut root = description.to_layout_with_context(
        NodeId::new(1),
        &layout_context,
    );

    apply_gallery_styles(
        &mut root,
        context.logical_width,
        context.logical_height,
    );

    let snapshot = self
        .layout
        .compute_with_text(
            &root,
            (context.logical_width, context.logical_height),
            &mut self.fonts,
            context.scale_factor,
        )
        .expect("finite Gallery viewport");

    // accessibility
    // hit testing
    // paint
}
```

---

# 9. Theme Token 建議修正

目前 Typography Token 若使用倍數：

```rust
pub line_height: f32
```

容易混淆它是倍率還是像素。

建議改為：

```rust
pub struct TypographyTokens {
    pub body_size: f32,
    pub body_line_height: f32,
    pub label_size: f32,
    pub label_line_height: f32,
    pub title_size: f32,
    pub title_line_height: f32,
}
```

全部使用實際 logical pixels。

或明確命名：

```rust
pub body_line_height_multiplier: f32
```

不可讓同一欄位有時被當成像素、有時被當成倍率。

---

# 10. TextInput 修正

修改：

```text
crates/acme-textinput/src/lib.rs
```

TextInput Layout 尺寸至少包含：

```text
text line height
+ vertical padding × 2
+ border × 2
```

例如：

```rust
let intrinsic_height =
    line_height
    + padding_top
    + padding_bottom
    + border_width * 2.0;
```

控制項最終高度：

```rust
height = intrinsic_height.max(theme.controls.medium_height)
```

TextInput 不應依靠 Gallery 的 Marker Label 來決定尺寸。

建議直接讓：

```rust
WidgetNode::TextInput
```

參與 Layout、Hit Test、Accessibility 與 Paint。

---

# 11. Button 與其他文字元件

Button 目前有固定高度，所以看起來較正常，但仍須檢查：

- 長文字
- 中文
- 大字級
- High DPI
- 左右 padding
- Disabled/Loading icon

建議 Button Measure：

```text
width =
leading icon
+ icon gap
+ shaped label width
+ trailing icon
+ horizontal padding × 2

height =
max(icon height, text line height)
+ vertical padding × 2
```

以下元件也應改為共用文字測量：

```text
MenuItem
Tooltip
Popover content
Tab item
Breadcrumb segment
Tree cell
Table cell
DataGrid cell
Sidebar item
NavRail item
Badge
Toast
Dialog title
```

不得各自實作不同的估算公式。

---

# 12. CJK 換行

繁體中文長文字預設不一定包含空白。

因此：

```rust
LabelWrap::Word
```

不足以處理所有中文內容。

中文說明、文件內容與 Stress Test 建議使用：

```rust
LabelWrap::Character
```

例如：

```rust
text_label(LONG_CHINESE_TEXT)
    .font_size(14.0)
    .wrap(LabelWrap::Character)
    .build()
```

同時要提供最大可用寬度給文字 shaping。

---

# 13. Text Measure Cache

目前 `Label.cached` 放在每幀重建的 Widget Tree 中，可能無法真正跨幀保存。

建議將 Cache 移到 Runtime 或 Layout Engine：

```rust
pub struct TextMeasureCache {
    entries: HashMap<TextMeasureKey, ShapedText>,
}
```

Key：

```rust
pub struct TextMeasureKey {
    pub node_id: NodeId,
    pub text_hash: u64,
    pub font_size_bits: u32,
    pub line_height_bits: u32,
    pub max_width_bits: u32,
    pub scale_factor_bits: u32,
    pub wrap: TextWrapMode,
}
```

失效條件：

- 文字改變
- 字體改變
- 字級改變
- 行高改變
- 可用寬度改變
- DPI 改變
- Theme 改變
- Font database 更新

P0 先確保正確性，Cache 可列入 P1。

---

# 14. 必須新增的測試

## 14.1 Label 有實際高度

```rust
#[test]
fn label_has_non_zero_intrinsic_height() {
    let measured = measure_label("Typography", 24.0, 800.0);

    assert!(measured.height >= 24.0);
}
```

## 14.2 Column 中 Label 不重疊

```rust
#[test]
fn labels_in_column_do_not_overlap() {
    let tree = column::<Msg>()
        .gap(4.0)
        .child(label("Line 1"))
        .child(label("Line 2"))
        .child(label("Line 3"))
        .build();

    let snapshot = compute_widget_layout(tree);

    let rects = label_rects(&snapshot);

    for pair in rects.windows(2) {
        assert!(
            pair[1].y >= pair[0].y + pair[0].height + 4.0 - 0.5
        );
    }
}
```

## 14.3 字級影響尺寸

```rust
#[test]
fn larger_font_has_larger_intrinsic_height() {
    let small = measure_label("Text", 12.0, 800.0);
    let large = measure_label("Text", 24.0, 800.0);

    assert!(large.height > small.height);
}
```

## 14.4 中文換行

```rust
#[test]
fn narrow_cjk_text_wraps_to_more_lines() {
    let wide = measure_cjk(LONG_TEXT, 600.0);
    let narrow = measure_cjk(LONG_TEXT, 240.0);

    assert!(narrow.height > wide.height);
}
```

## 14.5 ScrollView content height

```rust
#[test]
fn scroll_content_height_includes_measured_text() {
    let content = many_labels(50);
    let snapshot = compute_scroll(content, 320.0);

    let metrics = snapshot.scroll_metrics(scroll_id).unwrap();

    assert!(metrics.content_height > metrics.viewport_height);
}
```

## 14.6 TextInput 高度

```rust
#[test]
fn text_input_respects_control_height() {
    let rect = measure_text_input();

    assert!(rect.height >= 36.0);
}
```

---

# 15. Gallery 視覺驗收

必須檢查：

```text
Foundations / Typography
Inputs / Button
Inputs / TextInput
Navigation / Sidebar
Navigation / TabBar
Overlay / Tooltip
Data / Tree
Data / Table
Data / DataGrid
Patterns / Settings Page
Patterns / Dashboard
Stress Tests / Long Text
```

截圖矩陣：

```text
1280×800  Light  Comfortable
1280×800  Dark   Comfortable
1024×700  Light  Compact
1024×700  Dark   Compact
800×600   Light  Compact
800×600   Dark   Compact
```

人工 DPI：

```text
Windows 100%
Windows 125%
Windows 150%
Windows 200%
```

---

# 16. Agent 執行順序

## 任務 1：立即止血

修改：

```text
crates/acme-widgets/src/lib.rs
```

加入 Label/TextInput `min_height`。

完成後先執行：

```powershell
cargo check -p acme-widgets
cargo check -p acme-gallery
cargo run -p acme-gallery
```

## 任務 2：正式文字 Measure Context

修改：

```text
crates/acme-layout/src/lib.rs
crates/acme-widgets/src/lib.rs
crates/acme-widgets/src/foundations/label.rs
```

完成：

- `MeasureContent`
- `TextMeasureSpec`
- `TextWrapMode`
- `compute_with_text`
- Taffy measure callback

## 任務 3：Gallery 接線

修改：

```text
apps/gallery/src/main.rs
```

完成：

- Theme 在 Layout 前建立
- Typography Context 傳入
- 改用 `compute_with_text`

## 任務 4：TextInput

移除 Marker Label 作為主要尺寸來源，改用正式 TextInput Widget Layout。

## 任務 5：其他元件

逐項修正：

```text
Button
Menu
Tooltip
TabBar
Breadcrumb
Tree
Table
DataGrid
Sidebar
NavRail
Dialog
```

---

# 17. Agent Master Prompt

```text
請修正 stevenke1981/AcmeUI-Native 元件庫的文字版面問題。

這不是單純 Gallery 問題。根因在 acme-layout 與 acme-widgets：

1. Label 在 WidgetNode::to_layout_alloc 中只是 LayoutStyle::default() Leaf。
2. Label 的 text/font_size/line_height 沒有傳入 Layout Engine。
3. LayoutEngine 使用 Taffy compute_layout，沒有使用 intrinsic measure callback。
4. Renderer 依實際字級繪圖，但 Layout Rect 高度可能接近 0。
5. Gallery 因此出現大量文字重疊。

執行順序：

P0：
- 先為 Label 與 TextInput 加入安全 min_height，立即恢復可讀。
- 不得只增加 Column gap。
- 不得手動寫死每行 Y 座標。

正式修正：
- 在 acme-layout 增加 MeasureContent、TextMeasureSpec、TextWrapMode。
- 使用 Taffy 0.9 的 leaf context 與 compute_layout_with_measure。
- 使用 acme-text / cosmic-text 回傳實際 shaped width、height。
- 建立 WidgetLayoutContext，讓 Theme Typography 參與 Layout。
- Label 必須支援 font_size、line_height、wrap、max_lines。
- 長繁體中文必須支援 Character Wrap。
- TextInput 必須使用正式 intrinsic height。
- Button、Menu、Tree、Table、DataGrid 等文字元件必須共用同一測量系統。

測試：
- Label intrinsic height 不得為 0。
- 相鄰 Label 不得重疊。
- 大字級高度必須大於小字級。
- 長繁體中文窄寬度時高度必須增加。
- ScrollView content_height 必須包含完整文字高度。
- TextInput 高度不得小於 control token。

驗證：
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p acme-gallery

人工驗收：
Windows 100%、125%、150%、200% DPI。

最後更新：
todos.md
test.md
final.md

final.md 必須誠實列出：
- 根因
- 修改檔案
- 自動測試結果
- 人工 DPI 測試狀態
- 已知限制
- 下一個里程碑
```

---

# 18. Definition of Done

只有全部符合才算完成：

- Gallery 不再有文字重疊。
- 所有 Label Layout Rect 高度大於 0。
- 不同字級能影響 Layout 高度。
- 長繁體中文可依內容寬度換行。
- ScrollView 高度正確。
- TextInput 有正確控制項尺寸。
- Button、Menu、Tree、Table、DataGrid 文字正常。
- Theme Typography 參與 Layout。
- Light/Dark、Compact/Comfortable 都正常。
- Windows 100%–200% DPI 可讀。
- 不依賴人工 Y 座標修補。
- fmt、check、clippy、test 全部通過。
