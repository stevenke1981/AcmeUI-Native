# Validation Record

## 2026-07-23 — Mobile button and Gallery hardening

### Automated checks

| Check | Result | Evidence |
|---|---|---|
| `cargo test -p acme-ui --features mobile mobile_button` | PASS | 4 passed; size mapping, touch height, full width, disabled activation, message dispatch |
| `cargo test -p acme-ui-gallery` | PASS | 3 passed; dynamic category IDs, disabled render index, interactive render index |
| `cargo check -p acme-ui-gallery` | PASS | Gallery and full `acme-ui` feature set compile |
| `cargo build -p acme-ui-gallery` | PASS | Native Windows debug executable produced and launched |
| `cargo fmt --all -- --check` | Known baseline failure | Rust 1.97 would reformat pre-existing unrelated files; this delivery does not include that mechanical churn |
| `cargo check --workspace --all-targets` | PASS | All workspace targets compile |
| `cargo test --workspace` | PASS | 31 test binaries; 1,162 passed, 0 failed, 7 ignored |
| `cargo clippy -p acme-ui-gallery --all-targets --no-deps -- -D warnings` | PASS | Delivery target is warning-free |
| `cargo check -p acme-ui --no-default-features --features mobile` | PASS | Confirms the feature now declares its real dependency |
| `cargo clippy -p acme-ui --no-default-features --features mobile --lib --no-deps -- -D warnings -A clippy::manual_range_patterns` | PASS | Delivery surface is warning-free; one pre-existing `progress_ring` lint is explicitly isolated |
| `cargo clippy --workspace --all-targets -- -D warnings` | Known baseline failure | Rust 1.97 reports 15 pre-existing lints in unrelated chart/browser/foundation/input/overlay files |

### Visual QA

- Target window: `AcmeUI Gallery`, 1100×700 logical pixels, Mobile page selected by default.
- The real wgpu window launched and remained responsive.
- Windows Computer Use could enumerate the exact Gallery window but could not capture or focus it (`GetCursorPos: access denied`; Windows Graphics Capture interface unsupported).
- The standard screenshot helper captured the occluding Codex window; `PrintWindow` captured a black GPU surface. Both invalid captures were excluded from version control.
- Result: **manual Light/Dark screenshot and pointer click sign-off remain open**. Automated layout and message wiring passed; no visual score is claimed.

### Environment note

The active PowerShell session did not include Cargo on `PATH`; validation used
`C:\Users\steven\.cargo\bin\cargo.exe` without changing system configuration.
