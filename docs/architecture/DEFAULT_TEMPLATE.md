# AcmeUI Native Default Template v0.2.0

`acme-ui` is the application-facing component library. Its default feature set
ships the foundations, inputs, layout and overlay families. Optional chart,
desktop, mobile and browser families remain opt-in so a small desktop app does
not pay for every category.

```rust
use acme_ui::prelude::*;

let view = default_template::<AppMessage>("Ledger")
    .subtitle("A calm, keyboard-friendly workspace")
    .child(button("save", "Save").primary().on_click(AppMessage::Save))
    .child(card::<AppMessage>().child(label("Recent activity")))
    .build();
```

The template follows four reference ideas:

- shadcn/ui: semantic tokens and composition over a fixed visual skin;
- MUI: predictable builders, control sizes and a discoverable component API;
- Ant Design: clear families for inputs, navigation, data display and feedback;
- Radix: stable parts, focus/keyboard behavior and accessibility boundaries.

The public version constants are `acme_ui::VERSION` and
`acme_ui::DESIGN_SYSTEM_VERSION`; both are `0.2.0` for this release.

## Apple-inspired template

Use `apple_template("Dashboard")` for a quieter hierarchy with a tighter
12px rhythm, 20px shell inset, and stable `acmeui-apple-template` root key.
Colors remain semantic and platform-neutral; materials and motion are left to
the active theme and renderer.
