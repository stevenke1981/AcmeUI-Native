# AcmeUI-Native 建議里程碑

## Milestone A2 — Correctness Closure

目標：把本次更新造成或揭露的 correctness 問題關閉。

### Scope

- Slider percentage unit。
- Slider invalid range。
- DatePicker valid selected day。
- Component audit semantic validation。
- Scene single command vector。
- RGBA atlas upload model。
- Docs encoding/path repair。
- project-status sync status 改為 truthful。
- Accessibility target action skeleton。

### Exit Gate

- 所有 P0-A tests 通過。
- README/STATUS/project-status 數字一致。
- 所有 Markdown UTF-8 且無異常 control chars。
- Slider actual rect tests。
- DatePicker action test。
- Accessibility target action unit tests。

---

## Milestone B1 — Ordered Rendering

目標：修正 painter order，讓 Scene 成為 live backend input。

### Phase

1. canonical Scene
2. adjacent batch compiler
3. render_scene
4. Frame bridge
5. Gallery Scene output
6. remove Frame

### Exit Gate

- no HashMap iteration determines draw order
- overlay/text/clip golden
- deterministic batch hash
- Gallery visual parity
- TextInput caret/selection layering correct

---

## Milestone B2 — Stable Runtime Identity

目標：RetainedTree 正式接入。

### Phase

1. view bridge
2. shadow reconcile
3. stable layout NodeId
4. key lookup
5. accessibility stable ID
6. focus/hit NodeId
7. dirty-gated layout

### Exit Gate

- reorder/insert keeps NodeId
- focus survives sibling insertion
- accessibility IDs stable
- unchanged frame skips layout
- synthetic Tree/Table row IDs stable

---

## Milestone C — Interaction + Accessibility

目標：核心元件從視覺 shell 變成真正桌面 UI controls。

### Core 15

- Button
- TextInput
- Checkbox
- RadioGroup
- Switch
- Slider
- Select
- Combobox
- Tabs
- Menu
- Dialog
- Tooltip
- Tree
- Table
- DatePicker

### Exit Gate

每個元件：

- pointer
- keyboard
- disabled
- focus
- controlled state
- exactly-once message
- role/name/value/action
- Narrator smoke
- golden

---

## Milestone D — Performance + Release

### Scope

- persistent Taffy
- text measurement cache
- Criterion
- screenshot CI
- feature matrix
- semver/API review
- crate metadata
- DPI/IME/GPU manual validation

### v0.1 Exit Gate

- Early-alpha status可提升為 alpha
- release commit CI visible
- no status drift
- no known P0 correctness issue
- Windows primary flows manual PASS
