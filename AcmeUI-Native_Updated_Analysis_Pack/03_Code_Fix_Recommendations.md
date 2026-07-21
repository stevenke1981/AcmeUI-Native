# AcmeUI-Native 具體修正建議

## 1. Slider

### 建議實作

```rust
fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

fn normalize_slider(min: f32, max: f32, value: f32, step: f32) -> (f32, f32, f32, f32) {
    let mut min = finite_or(min, 0.0);
    let mut max = finite_or(max, 100.0);

    if min > max {
        std::mem::swap(&mut min, &mut max);
    }

    if (max - min).abs() < f32::EPSILON {
        max = min + 1.0;
    }

    let value = finite_or(value, min).clamp(min, max);
    let step = if step.is_finite() && step > 0.0 { step } else { 1.0 };

    let stepped = (((value - min) / step).round() * step + min).clamp(min, max);
    (min, max, stepped, step)
}
```

```rust
let ratio = ((value - min) / (max - min)).clamp(0.0, 1.0);

row::<M>()
    .child(
        row::<M>()
            .w(Length::Percent(ratio))
            .h(Length::Px(track_h))
    )
    .child(
        row::<M>()
            .w(Length::Percent(1.0 - ratio))
            .h(Length::Px(track_h))
    );
```

### 必要測試

```rust
#[test]
fn slider_half_value_uses_half_width() {
    let node: WidgetNode<()> = slider("s")
        .min(0.0)
        .max(100.0)
        .value(50.0)
        .into();

    let mut root = node.to_layout(NodeId::new(1));
    root.style.width = Length::Px(200.0);

    let snapshot = LayoutEngine::new()
        .compute(&root, (200.0, 40.0))
        .unwrap();

    let fill_id = root.children[0].id;
    let fill = snapshot.get(fill_id).unwrap();
    assert!((fill.width - 100.0).abs() < 1.0);
}
```

再加入：

- min > max
- NaN
- infinity
- zero range
- step 0
- negative step

---

## 2. Scene

### 不建議

```rust
Scene {
    commands: Vec<PaintCommand>,
    draw_commands: Vec<DrawCommand>,
}
```

### 建議

```rust
pub struct Scene {
    clear: Color,
    commands: Vec<DrawCommand>,
}

impl Scene {
    pub fn push(&mut self, command: DrawCommand) {
        self.commands.push(command);
    }

    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }
}
```

Legacy：

```rust
#[deprecated(note = "Use DrawCommand")]
pub fn push_paint(&mut self, command: PaintCommand) {
    self.commands.extend(command.into_draw_commands());
}
```

### Atlas

```rust
pub struct AtlasUpload {
    pub page: u32,
    pub origin: [u32; 2],
    pub size: [u32; 2],
    pub format: GlyphFormat,
    pub pixels: Vec<u8>,
}
```

Validation：

```rust
pub fn validate(&self) -> Result<(), SceneError> {
    let mut clips = 0usize;
    let mut layers = 0usize;

    for cmd in &self.commands {
        match cmd {
            DrawCommand::PushClip(_) => clips += 1,
            DrawCommand::PopClip => {
                clips = clips.checked_sub(1).ok_or(SceneError::ClipUnderflow)?;
            }
            DrawCommand::BeginLayer(_) => layers += 1,
            DrawCommand::EndLayer => {
                layers = layers.checked_sub(1).ok_or(SceneError::LayerUnderflow)?;
            }
            _ => {}
        }
    }

    if clips != 0 { return Err(SceneError::UnbalancedClip); }
    if layers != 0 { return Err(SceneError::UnbalancedLayer); }
    Ok(())
}
```

---

## 3. Accessibility

### 問題模式

```rust
AccessibilityAction::Click(id)
    -> FocusChanged(id)
    -> PointerButton(0,0, pressed=true)
```

這不是 target click。

### 建議 action model

```rust
#[derive(Clone, Debug)]
pub enum UiAction {
    Focus {
        target: NodeId,
    },
    Activate {
        target: NodeId,
    },
    SetValue {
        target: NodeId,
        value: String,
    },
    ScrollIntoView {
        target: NodeId,
    },
}
```

Application 增加：

```rust
fn action(&mut self, action: UiAction) -> bool;
```

或統一到 Runtime：

```rust
runtime.dispatch_action(action);
```

Pointer hit-test 最後也轉為：

```rust
UiAction::Activate { target }
```

這樣 mouse、keyboard、Narrator 共用同一 action path。

---

## 4. DatePicker

### Builder 事件需要真正接入

建議不要只有：

```rust
on_change: Option<M>
```

而是分開：

```rust
on_open_change: Option<M>,
on_prev_month: Option<M>,
on_next_month: Option<M>,
on_day_select: Option<fn(u32) -> M>,
```

目前 message model 若無法依 day 生成不同 message，可改成 typed component action：

```rust
DatePickerAction::SelectDay(u32)
DatePickerAction::PreviousMonth
DatePickerAction::NextMonth
DatePickerAction::Open
DatePickerAction::Close
```

由 runtime 映射回 app message。

### 日期合法性

```rust
let dim = days_in_month(year, month);
let selected_day = selected_day.filter(|d| (1..=dim).contains(d));
```

或 build 回傳 error：

```rust
InvalidSelectedDay {
    year,
    month,
    day,
}
```

---

## 5. Component Maturity

### Manifest

```yaml
name: slider
module: inputs
level: S1
visual: true
pointer: false
keyboard: false
message_dispatch: false
accessibility: false
golden: false
benchmark: false
manual: false
evidence:
  - crates/acme-ui/src/inputs/slider.rs
```

### 自動驗證

S2 gate 不只 grep：

```text
on_* field exists
AND field is consumed during build
AND output has action/message
AND interaction test observes dispatch
```

S3 gate：

```text
stable NodeId
role
name
value/state
supported action
focus
platform adapter
```

---

## 6. Docs Integrity Script

可加入 `scripts/check_text_integrity.py`：

```python
from pathlib import Path

ALLOWED_CONTROL = {"\n", "\r", "\t"}
EXTS = {".md", ".yaml", ".yml", ".toml", ".rs"}

errors = []

for path in Path(".").rglob("*"):
    if not path.is_file() or path.suffix.lower() not in EXTS:
        continue

    text = path.read_text(encoding="utf-8")

    if "\ufffd" in text:
        errors.append(f"{path}: contains U+FFFD replacement character")

    for i, ch in enumerate(text):
        if ord(ch) < 32 and ch not in ALLOWED_CONTROL:
            errors.append(f"{path}: C0 control U+{ord(ch):04X} at offset {i}")

if errors:
    raise SystemExit("\n".join(errors))
```

另搭配 markdown link checker。

---

## 7. Status Generator

建議保留手寫區塊，但 status table 由 marker 生成：

```markdown
<!-- GENERATED:PROJECT_STATUS:BEGIN -->
...
<!-- GENERATED:PROJECT_STATUS:END -->
```

CI：

```powershell
python scripts/generate_status.py
git diff --exit-code
```

避免 README、STATUS、todos、project-status 四份互相矛盾。
