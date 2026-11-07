# AcmeUI Native SDK Guide

Status: **0.x pre-release** (workspace version `0.2.0`). The API may change between
minor versions; see [Versioning and compatibility](#versioning-and-compatibility).

This guide describes what the repository code does today. Each claim is either
backed by code or CI configuration named here, or is marked as **not verified**.

## 1. Getting started

### 1.1 Path dependency (verified)

`examples/sdk-consumer/` is a standalone crate that depends on the SDK through
relative path dependencies. Copy the pattern for a sibling checkout:

```toml
[dependencies]
acme-platform = { path = "../AcmeUI-Native/crates/acme-platform" }
acme-ui = { path = "../AcmeUI-Native/crates/acme-ui" }
acme-core = { path = "../AcmeUI-Native/crates/acme-core" }
acme-render-wgpu = { path = "../AcmeUI-Native/crates/acme-render-wgpu" }
```

Path dependencies inherit `version`, `edition`, `license`, `repository`, and
`rust-version` from the repository's `[workspace.package]`, so the repository
root must be present (a standalone copy of a single crate is not supported).

### 1.2 Git dependency (not verified)

```toml
[dependencies]
acme-ui = { git = "https://github.com/stevenke1981/AcmeUI-Native", rev = "<commit-sha>" }
```

Pin a `rev` or tag. This form has **not been verified** by the current batch.
No crate is published to crates.io.

### 1.3 Minimal application

The consumer example implements `acme_platform::Application`, builds a
`WidgetNode` tree with `acme_ui::prelude`, lays it out, and returns a `Scene`
from `frame`. `acme_platform::run` opens the window and returns a `Result`:

Sketch only (`ignore`): see `examples/sdk-consumer/src/main.rs` for a compiling app.

```rust,ignore
use acme_platform::{run, Application, FrameContext};
use acme_core::Scene;

struct App;

impl Application for App {
    fn frame(&mut self, _context: FrameContext) -> Scene {
        // Build the widget tree, lay it out, and return a Scene.
        // See examples/sdk-consumer/src/main.rs for a complete implementation.
        unimplemented!("see examples/sdk-consumer")
    }
}

fn main() {
    if let Err(error) = run(App) {
        eprintln!("acme app: {error}");
        std::process::exit(1);
    }
}
```

Run the example and its headless test:

```powershell
cd examples/sdk-consumer
cargo test
cargo run
```

## 2. Crate map

| Crate | Role | Typical use |
|-------|------|-------------|
| `acme-ui` | High-level components and templates (`default_template`, `card`, `button`, …) | Application screens |
| `acme-widgets` | `WidgetNode` enum and builder DSL | Lower-level widget trees |
| `acme-layout` | Taffy-based layout facade (`LayoutEngine`, `LayoutSnapshot`) | Layout pass |
| `acme-platform` | `Application` trait, `run`, window events, renderer selection | Application entry point |
| `acme-core` | Platform-independent tree, geometry, events, `Scene` | Shared data model |
| `acme-render-wgpu` | GPU surface lifecycle and batched renderer; `Frame`, `scene_from_frame` | GPU path |
| `acme-render-cpu` | Headless software rasterizer for `Scene` | CPU path (through `acme-platform`) |
| `acme-text` | Text shaping and glyph atlas (`FontSystem`, `GlyphAtlas`) | Text drawing |
| `acme-textinput` | Text input state: cursor, selection, clipboard, IME preedit/commit, undo/redo | Editable fields |
| `acme-theme` | Semantic tokens (`Theme`, `ThemeColor`) and theme packs | Colors and typography |
| `acme-animation` | Tween engine and easing | Motion |
| `acme-style` | Styling helpers (`Style`, `Styled`) | Styling layer |
| `acme-accessibility` | AccessKit bridge (tree builder, adapter) | Accessibility tree |
| `acme-devtools` | Frame metrics, widget-tree dumps, diagnostics | Debugging |

Each crate's `description` in `Cargo.toml` is one line taken from its role above.

## 3. Renderer backend selection

The window renderer is chosen per window when it is created. The policy is in
`crates/acme-platform/src/backend.rs`.

Environment variable: `ACME_RENDER_BACKEND`

| Value | Meaning |
|-------|---------|
| unset, empty, or `auto` | GPU first. If GPU initialisation fails and the `cpu-renderer` feature is compiled in, use CPU. |
| `gpu` | Same as `auto` in the current policy: prefers GPU and falls back to CPU when the feature is compiled in. |
| `cpu` | Force the CPU software renderer. |
| anything else | Ignored with a `tracing::warn!`; behaves as `auto`. Matching is case-insensitive and trims whitespace. |

Plan mapping (`plan_renderer`), with `cpu-renderer` compiled in or not:

| Requested | `cpu-renderer` on | `cpu-renderer` off |
|-----------|-------------------|--------------------|
| `auto` / `gpu` | GPU, then CPU on init failure | GPU only |
| `cpu` | CPU only | Window creation fails with `CPU backend requires the acme-platform/cpu-renderer feature`, reported through `PlatformError::NoWindows` |

The `cpu-renderer` Cargo feature is **enabled by default** on `acme-platform`.
To omit it:

```toml
acme-platform = { path = "...", default-features = false }
```

Windows forced software smoke test (from the repository README):

```powershell
$env:ACME_RENDER_BACKEND = 'cpu'
cargo run -p acme-gallery
Remove-Item Env:ACME_RENDER_BACKEND
```

Runtime fallback: when GPU device-loss recovery fails, the affected window may
switch once to CPU (per the CPU renderer plan). This path is covered by the
manual GPU device-loss checklist and is **not manually validated**.

## 4. Error handling for `run()`

`acme_platform::run(app)` returns `Result<(), PlatformError>`:

| Result | Meaning |
|--------|---------|
| `Ok(())` | The last window closed normally. |
| `Err(PlatformError::EventLoop(String))` | The event loop could not be created or failed while running. |
| `Err(PlatformError::NoWindows { reasons: Vec<String> })` | No window could be created. `reasons` lists one entry per failed window (window creation or renderer initialisation). |
| `Err(PlatformError::Renderer(String))` | The loop stopped because of an unrecoverable renderer failure (for example, GPU recovery and CPU fallback both failed, or a renderer requested exit; the message carries the root cause). If both a renderer failure and an event-loop error occur, the renderer/`NoWindows` error is returned. |

`NoWindows` and `Renderer` were added in batch-002 for the SDK hardening work.
`PlatformError` is **not** `#[non_exhaustive]`, so an exhaustive `match` in
application code will need new arms when upgrading. Treat this as a breaking
change in 0.x (see the compatibility policy).

```rust
match acme_platform::run(app) {
    Ok(()) => {}
    Err(acme_platform::PlatformError::NoWindows { reasons }) => {
        eprintln!("no window could be opened: {}", reasons.join("; "));
        std::process::exit(1);
    }
    Err(error) => {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
```

## 5. `acme-ui` feature flags

From `crates/acme-ui/Cargo.toml`:

| Feature | Enables | Notes |
|---------|---------|-------|
| `default` | `foundations`, `inputs`, `layout`, `overlay` | |
| `foundations` | `acme_ui::foundations` module | |
| `inputs` | `acme_ui::inputs` module | |
| `layout` | `acme_ui::layout` module | |
| `overlay` | `acme_ui::overlay` module | |
| `charts` | `acme_ui::charts` module | |
| `desktop` | `acme_ui::desktop` module | |
| `mobile` | `acme_ui::mobile` module | Implies `foundations` |
| `browser` | `acme_ui::browser` module | |
| `full` | All of the above | |

Example:

```toml
acme-ui = { path = "crates/acme-ui", default-features = false, features = ["foundations", "inputs"] }
```

## 6. Supported targets and verification status

| Target | Configured in CI | Local evidence (batch-002, 2026-11-07) | Notes |
|--------|------------------|------------------------|-------|
| Windows 10, MSVC (authoring host) | Yes: `cargo check` and `cargo test --workspace` on `windows-latest` | `cargo check --locked --workspace` PASS; `examples/sdk-consumer` `cargo test` PASS and `cargo build` PASS | Native window run: **NOT_RUN** |
| macOS desktop | Yes: same CI matrix (`macos-latest`) | Not run here | CI result must be checked in the workflow run |
| Linux desktop | Yes: same CI matrix (`ubuntu-latest`) | Not run here | CI result must be checked in the workflow run |
| `wasm32-unknown-unknown` | Cross-target `cargo check -p acme-ui --lib --all-features` | Not run here | Library compile only; no browser runtime claim |
| `aarch64-linux-android` | Cross-target `cargo check -p acme-ui --lib --all-features` | Not run here | Library compile only; no Android runtime claim |
| `aarch64-apple-ios` | Cross-target `cargo check -p acme-ui --lib --all-features` | Not run here | Library compile only; no iOS runtime claim |

Rules for claiming support:

- A cross-target `cargo check` is a **library compile check**, not runtime support.
- Runtime windowing on Windows, macOS, and Linux is only proven by the manual
  checklists and an actual run of the application. Automated CI does not open
  a window.
- The toolchain is pinned to Rust **1.88.0** by `rust-toolchain.toml`.

## 7. Versioning and compatibility

- The current version is `0.2.0` (`[workspace.package]`).
- Pre-1.0 policy for this SDK: a breaking change bumps the **minor** version
  (for example `0.2.x` to `0.3.0`) and is recorded in [`docs/CHANGELOG.md`](../CHANGELOG.md). Patch
  versions carry fixes only.
- Adding enum variants, such as the `PlatformError` variants above, counts as a
  breaking change in 0.x.
- Public API changes should not add platform-specific types. Platform
  implementation details stay private.
- Unreleased changes, including the new `PlatformError` variants, are listed
  under "Unreleased" in [`docs/CHANGELOG.md`](../CHANGELOG.md).

## 8. Known limitations

Manual validation is still pending. No claim below is verified by automated tests alone.

- **Traditional Chinese 注音 IME: NOT YET MANUALLY VALIDATED.** Preedit, commit,
  and candidate-window placement have automated coverage only. See
  `docs/guides/MANUAL_VALIDATION.md`, checklist B.
- **GPU device-loss recovery: manual validation pending.** See checklist A.
- **DPI scaling:** visual glyph quality and layout at 125%, 150%, and 200% have not
  been signed off.
- **Screen reader (AccessKit):** no manual validation record exists in the
  repository. Do not claim screen-reader support until a record is added.
- **CPU renderer:** output is opaque XRGB8888 (no transparent windows); image
  scaling is nearest-neighbour; the glyph atlas is page 0 only; anti-aliasing is
  not identical to wgpu; no performance parity is claimed.
- **Pixel goldens:** deterministic, text-free CPU-renderer goldens
  (`crates/acme-render-cpu/tests/golden.rs`, regenerate with
  `ACME_BLESS_GOLDEN=1`) run in CI through `cargo test --workspace`.
  Gallery screenshot and text goldens are not yet in CI.
- **Font availability** varies by OS, so text snapshots are not portable.
- **Mobile and web runtime** integration is not proven. Only library compile
  checks are configured.
- **Click routing:** `examples/sdk-consumer` renders a button without handling its
  click. Message routing is application-defined.
