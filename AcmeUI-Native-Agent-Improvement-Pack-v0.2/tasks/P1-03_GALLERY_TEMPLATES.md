# P1-03 Gallery and Visual Reference Templates

## Gallery 導航

1. Foundations
2. Inputs
3. Navigation
4. Overlay
5. Data
6. Patterns
7. Accessibility
8. Stress Tests

## 每個元件頁

- Anatomy
- Variants
- Sizes
- States
- Density
- Light/Dark
- Keyboard behavior
- Accessibility properties
- Long Traditional Chinese string

## 參考頁面模板

### A. Settings Template

```text
TitleBar
├── Sidebar 224px
└── Content
    ├── Page Header
    ├── Settings Section
    ├── Settings Section
    └── Danger Zone
```

### B. Dashboard Template

```text
Header + Primary Action
KPI Row (max 4)
Main insight region
Recent activity
```

### C. Desktop IDE Template

```text
TitleBar / Menu / Toolbar
NavigationRail
Sidebar
Editor/Canvas
Inspector or Terminal
StatusBar
```

### D. SpeakType Template

```text
NavigationRail
Recording Status
Provider Status
Hotkey Hint
Recent Transcript
Single Primary Record Action
```

## Screenshot Matrix

- 1280×800
- 1024×700
- 800×600
- Light/Dark
- Compact/Comfortable
- Focus visible
- Error/Loading/Empty

Gallery 不得依賴 `snapshot.get(<magic number>)`。
