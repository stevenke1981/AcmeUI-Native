# AcmeUI-Native Release Guide

Status: **draft checklist for the release owner**. No version was changed. No tag was created. No package was published.
Date of this draft: 2026-11-07. Branch: `sdk/roadmap-007-packaging`.

Read this guide with [`MIGRATION.md`](MIGRATION.md), [`SDK_GUIDE.md`](SDK_GUIDE.md) and [`MANUAL_VALIDATION.md`](MANUAL_VALIDATION.md).

Each step has one owner decision or one command. Steps that need the owner are marked **(owner)**.
Steps that publish, push or log in are marked **(owner only)**. Do not run these steps as an agent.

## 1. Version facts

| Item | Location | Value now |
|------|----------|-----------|
| Workspace version | `Cargo.toml` line 12, `[workspace.package]` | `0.2.0` |
| Internal dependency versions | `Cargo.toml` lines 41-54, `[workspace.dependencies]`, 14 entries | `0.2.0` each |
| Lock file | `Cargo.lock` (workspace members) | `0.2.0` |
| Design-system constant | `crates/acme-ui/src/lib.rs` line 44, `DESIGN_SYSTEM_VERSION` | `"0.2.0"` (separate contract; owner decides) |
| Rust toolchain pin | `rust-toolchain.toml` | `1.88.0` |
| MSRV | `Cargo.toml` `[workspace.package]` `rust-version` | `1.88` |
| Git tags | `git tag` | none |
| Other version text | `README.md` line 29; `docs/guides/SDK_GUIDE.md` lines 3 and 211; `docs/architecture/DEFAULT_TEMPLATE.md` line 1; `docs/guides/BENCHMARKS.md` line 63 (example output) | `0.2.0` |
| Historic report | `docs/reports/retained-tree-invalidation.md` line 9 | do not edit; it is a record |

Facts about the bump:

1. The workspace version and the 14 internal dependency versions must change together. Both are in the root `Cargo.toml`.
2. `apps/*` packages are `publish = []` after P1 (verified with `cargo metadata` at the time of this draft).
3. `examples/sdk-consumer` is a standalone package with `publish = false` and its own `[workspace]` table.
4. Changing the version also changes `Cargo.lock`. CI runs `cargo check --locked`, so the lock file must be committed in the same change.
5. `DESIGN_SYSTEM_VERSION` is a separate contract constant. Do not change it by default. The owner decides.

## 2. Version drift (explained)

| Source | What it says |
|--------|--------------|
| Workspace (`Cargo.toml`) | `0.2.0` at commit `05ac86e` and at `HEAD` |
| CHANGELOG headings | `v0.2.3`, `v0.2.2`, `v0.2.1` (all dated 2026-07-20), `v0.2.0` (dated 2026-07-20), and "Unreleased" |
| Git tags | none |

Findings (verified):

- The `v0.2.1` entry says `Application::on_gpu_recovered` was added. That method already exists at the baseline commit `05ac86e`.
  So the CHANGELOG headings do not map to commits.
- The `v0.2.0` entry says `route_action` returns `Option<PlatformEvent>`. At `05ac86e` the code returns `Vec<PlatformEvent>` (see `MIGRATION.md` M13).
- The headings `v0.2.1` to `v0.2.3` were never published as packages. This project has published no crate. Note: the name `acme-core` on crates.io belongs to an unrelated project that already has versions `0.2.2`-`0.4.0` (see section 8).

Options for the owner **(owner)**:

- **Option A (proposed).** Keep the CHANGELOG history as written. Add a note that the `v0.2.x` headings were not released as versions.
  Release the next version as `0.3.0`.
- **Option B.** Tag the baseline commit `05ac86e` as `v0.2.0` before the next release. This tag records the state of the `0.2.0` version string.
  Do this only after the owner confirms that `05ac86e` is the intended `0.2.0` state. This was not verified.
- **Option C.** Renumber the CHANGELOG headings. This changes history and needs owner approval. Not recommended.

## 3. Proposed version: `0.3.0`

Reason. [`SDK_GUIDE.md`](SDK_GUIDE.md) section 7 says that a breaking change bumps the minor version (`0.2.x` to `0.3.0`).
The "Unreleased" sections contain breaking changes. Each one is listed in `MIGRATION.md`:

| ID | Breaking change | Source |
|----|-----------------|--------|
| M1 | `PlatformError` has new variants `NoWindows` and `Renderer`. `run()` returns `Err` in new cases. | `crates/acme-platform/src/lib.rs` lines 238-248 |
| M11 | `RenderBatch.clip` type changed to `Option<[f32; 4]>` (not in CHANGELOG). | `crates/acme-render-wgpu/src/batch.rs` line 35 |
| M8 | Read-only text fields ignore undo and redo. | `crates/acme-textinput/src/lib.rs` lines 131-133 |
| M3 | Glyph atlas eviction moved to `begin_frame`. Glyphs can be dropped for one frame. | `crates/acme-text/src/lib.rs` lines 442-450 |
| M6 | Accessibility roles, states and actions changed. | `crates/acme-accessibility/src/lib.rs` lines 575-629 |

**(owner)** Approve `0.3.0`, or choose another version. This role did not change the version.

## 4. crates.io publish order (derived from the dependency graph)

Method: `cargo metadata --format-version 1 --no-deps --locked` with `CARGO_TARGET_DIR=target/p3`, run at `HEAD` `fa223d9` with the P1 manifest edits in the working tree.
The workspace has 18 packages. 14 are libraries under `crates/`. The 4 app packages are `publish = []`.

Edges: 69 workspace dependency edges. All are `normal` kind. There are no build dependencies and no internal dev dependencies.
One internal dependency is optional: `acme-platform` depends on `acme-render-cpu` through the `cpu-renderer` feature.
Cargo requires optional dependencies to be on the registry when a crate that names them is packaged, so the order below applies even when the feature is off. This Cargo behaviour is **not verified** by a dry run.

Publish in this order. Crates in the same layer do not depend on each other, so their order inside a layer does not matter.

| Order | Layer | Crates |
|-------|-------|--------|
| 1 | 0 | `acme-animation`, `acme-core`, `acme-text` |
| 2 | 1 | `acme-layout`, `acme-render-cpu`, `acme-render-wgpu`, `acme-theme` |
| 3 | 2 | `acme-platform`, `acme-style` |
| 4 | 3 | `acme-widgets` |
| 5 | 4 | `acme-accessibility`, `acme-devtools`, `acme-textinput`, `acme-ui` |

Direct dependencies per crate (verified):

- `acme-core`, `acme-animation`, `acme-text`: none.
- `acme-theme`: `acme-core`.
- `acme-render-cpu`: `acme-core`.
- `acme-render-wgpu`: `acme-core`, `acme-text`.
- `acme-layout`: `acme-core`, `acme-text`.
- `acme-platform`: `acme-core`, `acme-render-wgpu`, `acme-render-cpu` (optional).
- `acme-style`: `acme-core`, `acme-layout`, `acme-theme`.
- `acme-widgets`: `acme-core`, `acme-layout`, `acme-style`, `acme-theme`.
- `acme-accessibility`: `acme-core`, `acme-layout`, `acme-platform`, `acme-theme`, `acme-widgets`.
- `acme-devtools`: `acme-core`, `acme-layout`, `acme-render-wgpu`, `acme-theme`, `acme-widgets`.
- `acme-textinput`: `acme-core`, `acme-layout`, `acme-platform`, `acme-text`, `acme-theme`, `acme-widgets`.
- `acme-ui`: `acme-core`, `acme-layout`, `acme-text`, `acme-theme`, `acme-widgets`.

Publish commands **(owner only)**, one crate at a time. Wait until each version is visible on the registry before the next one:

```text
cargo publish --locked -p <crate>
```

Do not run `cargo publish --dry-run` against crates.io with credentials. Do not store a crates.io token in this repository.
If a release is wrong after publish, the only Cargo action is `cargo yank --version <v> -p <crate>`. It does not delete the crate. `cargo yank` was **not verified** in this environment.

## 5. Gates before tagging

Run each gate on a clean checkout of the release commit. Use the pinned toolchain `1.88.0`.
The table maps each gate to its CI source. Status is for this draft only.

| # | Gate | Command | Source | Status in this draft |
|---|------|---------|--------|----------------------|
| G1 | Pinned toolchain | `cargo --version` shows `1.88.0` | `rust-toolchain.toml` | PASS (`cargo 1.88.0`, this machine) |
| G2 | Format of changed files | `python scripts/check_changed_rustfmt.py` | `ci.yml` job `rust` | NOT_RUN |
| G3 | Workspace check | `cargo check --locked --workspace --all-targets` | `ci.yml` job `rust` | NOT_RUN |
| G4 | Lock file in sync | `git diff --exit-code -- Cargo.lock` | `ci.yml` job `rust` | NOT_RUN |
| G5 | `acme-ui` all features | `cargo check --locked -p acme-ui --all-targets --all-features` | `ci.yml` job `rust` | NOT_RUN |
| G6 | Clippy on changed files | `python scripts/check_changed_clippy.py` | `ci.yml` job `rust` | NOT_RUN |
| G7 | Workspace tests | `cargo test --locked --workspace` | `ci.yml` job `rust` | NOT_RUN |
| G8 | `acme-ui` tests | `cargo test --locked -p acme-ui --all-features` | `ci.yml` job `rust` | NOT_RUN |
| G9 | Milestone evidence | `python scripts/verify_sdk.py --profile milestone --output <new-directory>` | `scripts/verify_sdk.py` (profile `milestone`) | NOT_RUN |
| G10 | CPU evidence | `python scripts/verify_sdk.py --profile cpu --output <new-directory>` | `.github/workflows/cpu-renderer.yml` | NOT_RUN |
| G11 | GPU-free platform build | `cargo check --locked -p acme-platform --no-default-features` | `cpu-renderer.yml` | NOT_RUN |
| G12 | External consumer | copy `Cargo.lock` to `examples/sdk-consumer`, then `cargo test` and `cargo build` there | `ci.yml` job `sdk-consumer` | NOT_RUN |
| G13 | Cross-target compile | `cargo check --locked -p acme-ui --lib --all-features --target <t>` for `wasm32-unknown-unknown`, `aarch64-linux-android`, `aarch64-apple-ios` | `ci.yml` job `cross-target` | NOT_RUN |
| G14 | Dependency bans and sources | `cargo deny check bans sources` | `ci.yml` job `audit` | NOT_RUN |
| G15 | Licence policy | `cargo deny check licenses` | `ci.yml` job `audit` (`cargo deny check bans licenses sources`) | NOT_RUN |
| G16 | Advisory audit | `cargo audit --ignore RUSTSEC-2026-0194 --ignore RUSTSEC-2026-0195` | `ci.yml` job `audit` | NOT_RUN |
| G17 | Packageable crates | `python scripts/check_package.py` | `ci.yml` job `audit` | NOT_RUN |
| G18 | Git dependency from outside the repo | scratch crate in a temporary directory with `acme-ui = { git = "file:///...", rev = "<sha>" }` and `cargo check` | Manual. **Not in CI.** PASS locally in ROADMAP 007. | NOT_RUN |
| G20 | crates.io name availability | `curl https://index.crates.io/ac/me/<name>` for each of the 14 names (404 = free) | Manual | NOT_RUN |
| G19 | CI result on the release commit | `gh run watch` on the pushed branch; all jobs green | `.github/workflows/ci.yml` | NOT_RUN (no push) |

Notes:

- G9 and G10 need a new output directory each time. `verify_sdk.py` does not overwrite an existing directory.
- G14 and G16 use the same ignore list as `ci.yml`. The ignore entries are for Wayland and `quick-xml` (see the comment in `ci.yml`).
- The CHANGELOG core-invalidation entry says that its validation ran on Rust 1.94.1. The pinned toolchain is 1.88.0. The release gates must run on 1.88.0. The CHANGELOG says the pinned gates were still pending.

## 6. Manual validation status

Release notes must state this status. Source: [`MANUAL_VALIDATION.md`](MANUAL_VALIDATION.md) and [`SDK_GUIDE.md`](SDK_GUIDE.md) section 8.

| Area | Status | Release note rule |
|------|--------|-------------------|
| A. GPU device-loss recovery | NOT YET MANUALLY VALIDATED | State it as pending. |
| B. Traditional Chinese IME (注音) | NOT YET MANUALLY VALIDATED | State it as pending. Do not claim IME support. |
| DPI scaling (125 %, 150 %, 200 %) | Not signed off (automated evidence only at 1.0 to 2.0 scale) | State it as pending. |
| Screen reader (AccessKit) | No manual record in the repository | State it as pending. |

A status changes only when a dated sign-off exists in `docs/guides/MANUAL_VALIDATION.md`.

## 7. Release steps

1. **(owner)** Decide the version (proposed `0.3.0`, section 3) and the version-history option (section 2).
2. **(owner)** Confirm that P1 (manifests and `check_package.py`) and P2 (licence files and `deny.toml`) are merged.
3. Set `[workspace.package] version` in `Cargo.toml` line 12 to the new version.
4. Set the 14 internal dependency versions in `Cargo.toml` lines 41-54 to the same version.
5. Refresh the lock file: `cargo check --workspace --all-targets` (without `--locked`). Review `git diff -- Cargo.lock`. Only workspace-member version lines should change.
6. **(owner)** Decide whether `DESIGN_SYSTEM_VERSION` (`crates/acme-ui/src/lib.rs` line 44) and the version text in section 1 change too.
7. In `docs/CHANGELOG.md` **(Lead)**, rename "Unreleased" to `vX.Y.Z — YYYY-MM-DD`. Add a link to `MIGRATION.md`. Add an empty "Unreleased" heading above it.
8. Run gates G1 to G20 (section 5). Record the exit code and the output path of each gate.
9. Merge the release change to `master` through a pull request. Keep the lock file in the same commit.
10. **(owner)** Create an annotated tag on the merge commit: `git tag -a vX.Y.Z -m "AcmeUI-Native vX.Y.Z"`. Push the tag only after the owner says so.
11. **(owner only)** Publish the 14 library crates in the order in section 4.
12. **(owner)** Check that each crate is visible on the registry. Then run a consumer build from the registry. This is a follow-up. `examples/sdk-consumer` currently uses path dependencies. The registry consumer build is **not verified**.
13. Write release notes. Use the CHANGELOG section, the link to `MIGRATION.md` and the manual validation status (section 6).
14. Start the next "Unreleased" section.

## 8. Risks and open items

- **(owner) Name collision:** `acme-core` is already taken on crates.io by an unrelated project (checked 2026-11-07; the other 13 names were free). `cargo publish -p acme-core` will fail, and the whole chain depends on it. Rename the crate (for example `acmeui-core`) or move all crates to a common unused prefix before the first publish. `check_package.py` cannot detect this, because it patches crates.io to local paths.
- **(owner) License text in packages:** the `.crate` files contain no `LICENSE-*` file (see `LICENSES.md` open item 7).
- The git dependency form (`SDK_GUIDE.md` section 1.2) passed locally with a `file://` URL. The GitHub URL form is not verified.
- `RenderBatch.clip` (M11) and the `route_action` note (M13) are recorded in the CHANGELOG ROADMAP 007 entry.
- `AGENTS.md` says "Never use byte offsets for text cursors". The public `TextInputState::move_cursor_to_offset(byte_offset: usize)` still uses byte offsets. The owner decides whether to change the API before `0.3.0`.
- No crate is on crates.io. Consumers must use path or git dependencies until the owner publishes.
- The benchmark JSON writes `package_version` from `CARGO_PKG_VERSION` (`apps/benchmark/src/report.rs` line 37). The example in `BENCHMARKS.md` is only an example. Regenerate it after the bump if the owner wants the example to match.

## 9. Change history

| Version | Date | Content | Scope |
|---------|------|---------|-------|
| draft 1 | 2026-11-07 | First draft from ROADMAP 007 P3. Version facts, drift analysis, derived publish order, gates mapped to CI. | `docs/guides/RELEASE.md` |
