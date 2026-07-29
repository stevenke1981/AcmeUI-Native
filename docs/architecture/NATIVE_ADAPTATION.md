# Native cross-platform adaptation

AcmeUI Native renders its own component tree through `wgpu`. That gives the
framework one renderer across DirectX, Metal, Vulkan, and WebGPU, but a single
set of hard-coded dimensions can still make every operating system feel like
the same web design wearing a different color theme.

The `acme_ui::native` module provides a backend-neutral adaptation contract.
It changes semantic defaults without exposing Win32, AppKit, UIKit, GTK,
Android, browser, `winit`, or renderer types.

## Platform contract

`NativePlatform` represents Windows, macOS, Linux, Android, iOS, Web, and an
unknown fallback. `NativeProfile` combines that target with:

- density (`Compact`, `Comfortable`, or `Touch`);
- page and content spacing;
- title, body, label, and line-height defaults;
- compact, regular, and large control heights;
- minimum hit-target size and horizontal control padding;
- control, surface, and dialog corner radii;
- a semantic theme-pack selection;
- the native primary shortcut-label convention (`Ctrl+S` or `⌘S`).

Automatic selection uses Rust target configuration:

```rust
let profile = NativeProfile::current();
```

Design tools, screenshots, and tests should use an explicit profile:

```rust
let windows = NativeProfile::for_platform(NativePlatform::Windows);
let macos = NativeProfile::for_platform(NativePlatform::MacOs);
let android = NativeProfile::for_platform(NativePlatform::Android);
```

## Keep theme, layout, and components aligned

Use one profile for all three layers:

```rust
let profile = native_profile();
let theme = profile.theme(ThemeMode::Dark);
let layout_context = profile.layout_context(window_scale_factor);
let save = native_button_for(profile, "save", "Save");
```

The profile applies platform geometry to existing semantic theme tokens. It
does not insert literal colors into widgets. `WidgetLayoutContext` receives the
same body, label, line-height, control-height, and scale-factor contract.

## Rendering boundary

The current implementation is a native-feeling, platform-adaptive component
library backed by AcmeUI's retained tree and GPU renderer. It is intentionally
not an OS-widget wrapper. A future hybrid backend could map selected controls to
WinUI, AppKit/UIKit, GTK, or Android primitives, but that backend must stay
behind AcmeUI-owned traits so application code remains portable.

## Cross-platform verification

CI runs formatting, workspace checks, Clippy, and tests on Windows, macOS, and
Ubuntu. The component crate is additionally built and tested with all optional
families enabled so browser, mobile, chart, desktop, and overlay APIs cannot
silently drift apart.

Tests in `native.rs` verify that every profile produces valid light and dark
themes, touch targets remain large enough, invalid scale factors normalize to
1.0, shortcut labels follow platform conventions, and native buttons receive
the selected profile geometry.
