# AcmeUI-Native 目標架構草案

## 1. Pipeline

```text
Application::view(state)
        |
        v
WidgetDescription<M>
        |
        v
Reconciler
  input: previous RuntimeTree + new description
  output: ChangeSet
        |
        v
RuntimeTree<M>
  NodeId
  WidgetKey
  WidgetKind
  Props
  Style
  InteractionState
  LayoutHandle
  SemanticState
  DirtyFlags
        |
        +--------------------+
        |                    |
        v                    v
Persistent Layout       Interaction Index
Taffy handles           hit/focus/capture
        |                    |
        +----------+---------+
                   |
                   v
             Ordered Scene
                   |
                   v
          Adjacent Batch Compiler
                   |
                   v
              wgpu Renderer
```

## 2. RuntimeNode 建議

```rust
pub struct RuntimeNode<M> {
    pub id: NodeId,
    pub key: Option<WidgetKey>,
    pub kind: WidgetKind,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,

    pub props: WidgetProps<M>,
    pub style: ComputedStyle,

    pub interaction: InteractionState,
    pub semantics: Semantics,
    pub dirty: DirtyFlags,

    pub layout_handle: Option<LayoutHandle>,
    pub cached_measure: Option<MeasureCache>,
    pub cached_paint: Option<PaintFragment>,
}
```

## 3. ChangeSet 建議

```rust
pub struct ChangeSet {
    pub mounted: Vec<NodeId>,
    pub removed: Vec<NodeId>,
    pub moved: Vec<NodeId>,
    pub props_changed: Vec<NodeId>,
    pub style_changed: Vec<NodeId>,
    pub layout_dirty: Vec<NodeId>,
    pub paint_dirty: Vec<NodeId>,
    pub semantics_dirty: Vec<NodeId>,
}
```

## 4. Identity 規則

1. keyed node identity = `(parent NodeId, WidgetKey, WidgetKind)`.
2. 相同 key 但 kind 改變，預設 replace 並清除 local state。
3. unkeyed sibling 只能以位置 identity，debug mode 應警告動態 list 未加 key。
4. layout、hit-test、focus、semantics、render 全部共用 Runtime NodeId。
5. 不允許 accessibility 自行重新配置 DFS ID。

## 5. Ordered Scene

```rust
pub enum DrawCommand {
    Quad(QuadPrimitive),
    Text(TextPrimitive),
    Image(ImagePrimitive),
    Path(PathPrimitive),
    PushClip(ClipPrimitive),
    PopClip,
    BeginLayer(Layer),
    EndLayer,
}
```

Batch compiler：

- 只合併相鄰 command。
- 不跨 clip boundary。
- 不跨 layer boundary。
- 不跨 blend/pipeline/texture state。
- 保留 deterministic order。

## 6. Widget Action

```rust
pub struct WidgetAction<M> {
    pub target: NodeId,
    pub source: ActionSource,
    pub message: Option<M>,
    pub kind: WidgetActionKind,
}
```

```rust
pub enum ActionSource {
    Pointer,
    Keyboard,
    Accessibility,
    Programmatic,
}
```

所有 pointer、keyboard、AccessKit action 進入同一 target-based pipeline。

## 7. Component Maturity Manifest

建議每個元件旁建立 metadata：

```toml
name = "slider"
maturity = "S1"
pointer = false
keyboard = false
accessibility = false
golden = true
manual = false
```

由工具自動產生 README component matrix。

## 8. Cache

### Text cache key

```text
text hash
font family
font size
line height
weight/style
width constraint
wrap mode
scale factor
font database generation
```

### Paint cache key

```text
node props version
computed style version
layout rect version
theme version
interaction state version
atlas generation
```

## 9. Error Boundary

```rust
pub trait Application {
    fn view(&self, window: WindowId) -> WidgetNode<Message>;
    fn update(&mut self, action: WidgetAction<Message>);
    fn on_error(&mut self, error: RuntimeError) -> ErrorPolicy;
}
```

可選 ErrorPolicy：

- ContinueWithFallback
- RebuildWindow
- RestartRenderer
- Exit

## 10. 漸進遷移

1. 先讓 Gallery 同時建立 description 與 RuntimeTree，結果對照。
2. layout ID 改由 RuntimeTree 提供。
3. hit-test/accessibility 改用 RuntimeTree。
4. Scene 改為 ordered。
5. 移除舊 DFS ID pipeline。
6. 啟用 dirty-driven partial rebuild。
