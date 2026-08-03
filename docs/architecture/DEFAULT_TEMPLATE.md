# AcmeUI Native application templates v0.2.0

`acme-ui` is the application-facing component library. Its default feature set
ships the foundations, inputs, layout and overlay families. Optional chart,
desktop, mobile and browser families remain opt-in so a small application does
not pay for every category.

## Deterministic default template

Use `default_template(...)` when screenshots, examples, or products require the
same geometry on every target:

```rust
use acme_ui::prelude::*;

let view = default_template::<AppMessage>("Ledger")
    .subtitle("A calm, keyboard-friendly workspace")
    .child(button("save", "Save").primary().on_click(AppMessage::Save))
    .child(card::<AppMessage>().child(label("Recent activity")))
    .build();
```

The default template remains platform-neutral and keeps its stable
`acmeui-default-template` root key.

## Target-aware native template

Use `native_template(...)` when one component tree should adopt the target's
page rhythm, typography, control scale, corner geometry, theme pack, and
shortcut-label convention:

```rust
use acme_ui::prelude::*;

let profile = native_profile();
let theme = native_theme(ThemeMode::Light);
let layout_context = native_layout_context(1.0);

let view = native_template("Settings")
    .subtitle(format!("Save with {}", profile.shortcut_label("S")))
    .child(
        native_button("save", "Save")
            .primary()
            .on_click(AppMessage::Save),
    )
    .build();
```

For Gallery previews and deterministic tests, select a target explicitly:

```rust
let mac = native_template_for::<AppMessage>(NativePlatform::MacOs, "Settings");
let android = NativeProfile::for_platform(NativePlatform::Android);
let touch_button = native_button_for(android, "continue", "Continue");
```

`native_template`, `native_theme`, `native_layout_context`, and
`native_button` are designed to use the same `NativeProfile`. This prevents a
Windows shell from accidentally using macOS control heights or an Android
button from using desktop hit targets.

## Compatibility templates

The existing entry points remain available and keep their stable root keys:

- `apple_template(...)` uses the macOS profile;
- `windows11_template(...)` uses the Windows profile;
- `ubuntu25_template(...)` uses the Linux profile.

These are platform-inspired GPU-rendered components, not wrappers around WinUI,
AppKit/UIKit, GTK, or Android Views. Native OS-control backends can be added
later without leaking platform-specific types into the public component API.

The template system follows four reference ideas:

- semantic tokens and composition over a fixed visual skin;
- predictable builders, control sizes, and a discoverable component API;
- clear families for inputs, navigation, data display, and feedback;
- stable parts, focus/keyboard behavior, and accessibility boundaries.
