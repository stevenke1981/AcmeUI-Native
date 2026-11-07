# AcmeUI SDK consumer example

A standalone crate that consumes AcmeUI-Native the way an external application
would: through path dependencies on `crates/*` and the public API only. It is
not a member of the root workspace (it has its own `[workspace]` table), it is
`publish = false`, and it does not commit a `Cargo.lock`.

## What it shows

- `acme_platform::run` with an `Application` implementation.
- A declarative UI from `acme_ui::prelude` (`default_template`, `card`, `label`,
  `button`) converted to a `LayoutNode` tree and laid out by
  `LayoutEngine::compute_with_text`.
- Rendering through `acme_render_wgpu::Frame` and `scene_from_frame`, the same
  path the Gallery apps use.
- A headless `#[test]` that builds one frame `Scene` without opening a window.

The button is visual only; click messages are not routed in this example.

## Run

```powershell
cd examples/sdk-consumer
cargo run
```

Renderer selection belongs to the SDK: `ACME_RENDER_BACKEND=auto|gpu|cpu`
(see `docs/guides/SDK_GUIDE.md`).

## Test and build

```powershell
cd examples/sdk-consumer
cargo test
cargo build
```

When the crate sits inside the repository, the pinned toolchain comes from the
repository's `rust-toolchain.toml` (Rust 1.88.0).

## Notes

- Path dependencies inherit `version`, `edition`, `license`, `repository` and
  `rust-version` from the root `[workspace.package]`. Building this example
  therefore requires the root `repository` entry to exist.
- No `Cargo.lock` is committed (`.gitignore` excludes it), so dependencies are
  resolved on the first local build.
- To share the repository's `target/` directory, set
  `CARGO_TARGET_DIR=../../target/sdk-consumer` before building.
