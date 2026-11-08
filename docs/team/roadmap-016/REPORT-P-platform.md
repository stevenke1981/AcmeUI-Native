# ROADMAP 016 — Track P Report: Platform Input and Host Control

## Changed Files

- `crates/acme-platform/src/lib.rs`
- `crates/acme-platform/src/keys.rs` (new)
- `crates/acme-platform/src/waker.rs` (new)
- `crates/acme-accessibility/src/lib.rs` (minimal `PlatformEvent::Key` and `PlatformEvent::PointerButton` literal updates)
- `crates/acme-textinput/src/input.rs` (minimal `PlatformKey` match arm update)
- `apps/acme-gallery/src/main.rs` (minimal `WindowConfig` literal update)
- `apps/gallery/src/main.rs` (minimal `WindowConfig` literal updates)
- `apps/gallery/src/focus_tests.rs` (minimal `PlatformEvent::Key` literal update)
- `apps/gallery/src/hover_press_tests.rs` (minimal `PlatformEvent::Key` and `PlatformEvent::PointerButton` literal updates)
- `apps/gallery/src/dpi_tests.rs` (minimal `PlatformEvent::PointerButton` literal update)
- `apps/playground/src/main.rs` (minimal `WindowConfig` literal update)

## Public API Diff (`acme-platform`)

```rust
// 1. Extended PlatformKey variants
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformKey {
    Tab,
    Enter,
    Space,
    Escape,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    PageUp,
    PageDown,
    Insert,
    Backspace,
    Delete,
    Home,
    End,
    Function(u8), // F1..=F24 -> 1..=24
    Other,
}

// 2, 3, 8, 10. Extended PlatformEvent variants (PointerButton, Scroll, Key)
pub enum PlatformEvent {
    // ...
    PointerButton {
        window: WindowId,
        pressed: bool,
        x: f32,
        y: f32,
        button: u16,
        pointer: u64,
        shift: bool,
        ctrl: bool,
        alt: bool,
    },
    Scroll {
        window: WindowId,
        delta_x: f32,
        delta_y: f32,
        lines: bool,
        x: f32,
        y: f32,
    },
    Key {
        window: WindowId,
        key: PlatformKey,
        pressed: bool,
        shift: bool,
        ctrl: bool,
        alt: bool,
        meta: bool,
        repeat: bool,
        text: Option<String>,
        character: Option<String>,
    },
    // ...
}

// 4. Cross-thread wake-up handle
#[derive(Clone, Debug)]
pub struct Waker { /* EventLoopProxy<()> */ }

impl Waker {
    pub fn wake(&self);
}

// 5, 9. Host window requests
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowRequest {
    SetTitle(String),
    Focus,
    Minimize,
    Close,
    SetImeAllowed(bool),
}

// 4, 5. Application trait hooks
pub trait Application: 'static {
    // ...
    fn on_waker(&mut self, _waker: Waker) {}
    fn wake(&mut self) -> bool { false }
    fn take_window_requests(&mut self, _window: WindowId) -> Vec<WindowRequest> { Vec::new() }
}

// 6, 7. WindowConfig minimum size and initial outer position
#[derive(Clone, Debug)]
pub struct WindowConfig {
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub min_width: Option<f64>,
    pub min_height: Option<f64>,
    pub position: Option<(f64, f64)>,
}
```

## IME Behavior Confirmation (Addendum A, Item 9)

- In `Runtime::resumed` (`crates/acme-platform/src/lib.rs`), the runtime calls `window.set_ime_allowed(true)` unconditionally right after creating each window via `event_loop.create_window(attrs)`.
- `WindowRequest::SetImeAllowed(bool)` has been added and wired in `process_window_requests` (`state.window.set_ime_allowed(allowed)`) so terminal or host applications can explicitly enable, disable, or re-assert IME state at runtime.

## Verification & Test Counts

| Command | Status | Result |
| --- | --- | --- |
| `cargo test -p acme-platform` | PASS | 43 passed, 0 failed (unit tests); 0 failed, 1 ignored (doc-test) |
| `cargo clippy -p acme-platform --all-targets -- -D warnings` | PASS | 0 warnings, 0 errors |
| `cargo check --workspace --all-targets` | PASS | All workspace crates and app targets compiled cleanly |
| `rustfmt --edition 2024 --check <changed files>` | PASS | 0 formatting diffs |
| `PYTHONUTF8=1 python scripts/check_changed_rustfmt.py --base master` | NOT_RUN | Requires lead commit first (per brief: `# after lead commits`) |
