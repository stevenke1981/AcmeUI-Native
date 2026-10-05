# Core invalidation and text editing: local patch report

## Baseline and scope

- Remote: `https://github.com/stevenke1981/AcmeUI-Native.git`
- Branch: `master`, baseline `2b9877087354f5e770274b7d6f036f981d5ad8cb`.
- Fresh isolated clone; working tree was clean. This report records local
  verification before publication; remote CI results are reported separately.
- Framework version remains 0.2.0. This patch is unreleased.
- Read AGENTS, README, spec/plan/todos, CI, runtime integration plan, component
  maturity report and repository design skill. No nested AGENTS or `.agents`
  files are tracked. This task changes core state management, not visual design.

The project is a Rust desktop UI framework built on wgpu, winit, Taffy,
cosmic-text and AccessKit. Runtime and Gallery implementations exist, but the
component catalog also contains placeholders and visual-only components. Manual
IME, DPI and device-recovery sign-off, screenshot regression and benchmark
thresholds remain open. Historical status documents disagree in some areas;
checked TODO boxes are not proof of end-to-end readiness.

## Problem and acceptance criteria

After clearing dirty flags, reconciling `[a, b]` to `[b, a]` kept the parent
clean. Inserting/removing children had the same defect. Changing a descendant's
kind, focusability or disabled state invalidated that descendant's layout but
left ancestors clean. Consumers using the documented dirty flags could reuse
stale layout or structure.

This patch marks changed child lists with CHILDREN, LAYOUT, PAINT and SEMANTICS.
Property changes keep their existing local invalidation flags. Both propagate
LAYOUT to ancestors, without marking unrelated siblings or changing public APIs.

Acceptance is covered by reorder, insertion, removal including the final child,
stable IDs, three property changes, multi-level propagation, unchanged repeated
frames and duplicate-key rejection without dirty-state mutation.

## Text editing and Gallery completion

Readonly inputs previously allowed undo/redo to mutate text and consume history.
Ctrl+Backspace/Delete with a selection skipped the undo transaction, so undo
could erase the preceding edit and an old redo branch could remain available.
Both paths now use the existing edit-history rules.

Gallery now routes Ctrl+Z, Ctrl+Y and Ctrl+Shift+Z into the existing history API,
normalizes uppercase shortcut text, and ignores key release. Its TextInput page
shows shortcut hints. No new public API, dependency or framework was introduced.

Three new text-input regressions failed before the fix (102 existing tests
passed). The Gallery handler regression also failed before shortcut wiring.
Afterward, all 105 text-input tests and the Gallery event-flow test passed.
That flow exercises select-all, deletion, undo/redo, key release, uppercase
Shift+Z, readonly rejection, loss of focus and recovery of editing history.

## Validation performed: initial resource-limited phase

Environment: Windows, installed Rust 1.94.1, GNU target. No dependencies or
toolchains installed; no GPU or foreground app used. CPU-only compilation was
serial through Cargo (`-j 1`).

- Before the fix: both new regression tests failed at the expected missing
  dirty flags; existing tree tests passed.
- After the fix: all 41 tests in the actual `acme-core/src/lib.rs` passed, first
  via rustc and then via the standalone Cargo manifest below. Doc-tests: 0.
- Standalone core Clippy, all targets, `-D warnings`: passed.
- Changed Rust files: rustfmt check passed with Rust 1.94.1.
- `git diff --check`: passed.
- Headless example passed and printed:
  `Reorder invalidated the list, preserved child IDs, and the repeated frame stayed clean.`

Initially, the standard core Cargo command could not resolve the workspace
because `taffy` was absent from the local registry cache. This was resolved in
the full validation phase below; it is no longer a blocker.

## Full workspace validation after resource release

Fetched the existing Cargo.lock dependencies from the official crates.io
registry with `cargo +1.94.1 fetch --locked`. Cargo.lock, Cargo.toml and
rust-toolchain.toml remain unchanged. No toolchain was installed. Rust 1.88 is
still pinned; the results below are Windows GNU / Rust 1.94.1 compatibility
results, not an MSRV claim. Package metadata declares no dependency MSRV above
1.88, but that does not replace compiling with 1.88.

GNU linking initially hit a sandbox access denial for existing `dlltool.exe`.
The authorized local test/build rerun outside that sandbox succeeded. No OS
security setting was changed.

All commands below use `cargo +1.94.1`; build/check/test/Clippy used
`--locked --offline -j 4` after fetching dependencies.

| Gate | Result |
| --- | --- |
| `check --workspace --all-targets` | Passed |
| `check --workspace --all-targets --all-features` | Passed |
| `test --workspace` | Passed: 1184 tests, 0 failures, 7 ignored |
| `test --workspace --all-features` | Passed: 1184 tests, 0 failures, 7 ignored |
| `build --workspace --all-targets --all-features` | Passed; Gallery and other app binaries linked |
| `clippy --workspace --all-targets --all-features` | Completed; 29 warnings outside changed files, 0 in changed files |
| `fmt --all -- --check` | Passed |
| `run -p acme-core --example reconcile_dirty` | Passed |
| `git diff --check` | Passed |

The seven ignored cases are one GPU adapter smoke test and six explicitly
ignored documentation examples. They were not run. Clippy warnings comprise
21 dead-code warnings, four unnecessary-lazy-evaluation warnings, two range
patterns and two collapsible-if warnings in unchanged files. The repository CI
uses a changed-file warning gate; this result is not a global `-D warnings` pass.
The committed-range CI scripts would skip uncommitted changes, so validation
used complete workspace commands and compared JSON primary-span paths against
the actual working-tree changes and new files. Separate stdout/stderr logs
prevented diagnostic JSON from being interleaved.

Local logs are under ignored `target/`: `workspace-check-all.log`,
`workspace-tests.log`, `workspace-tests-all.log`, `workspace-build-all.log`,
`workspace-clippy-json.log`, `workspace-clippy-stderr.log`,
`workspace-format.log`, `reconcile-example.log`. The initial failing regressions
are recorded in `textinput-before.log` and `gallery-before.log`.

### Reproduce the dependency-free checks

With normal dependencies/toolchain available:

```powershell
cargo test -p acme-core
cargo clippy -p acme-core --all-targets -- -D warnings
cargo run -p acme-core --example reconcile_dirty
```

To avoid unrelated workspace dependency resolution, the following ignored
manifest points directly to the same source and example (no mocks or copied
implementation). Run from the repository root:

```powershell
New-Item -ItemType Directory -Force target/core-standalone | Out-Null
@'
[package]
name = "acme-core"
version = "0.2.0"
edition = "2024"
[workspace]
[lib]
path = "../../crates/acme-core/src/lib.rs"
[[example]]
name = "reconcile_dirty"
path = "../../crates/acme-core/examples/reconcile_dirty.rs"
'@ | Set-Content -Encoding utf8 target/core-standalone/Cargo.toml
cargo +1.94.1 test --manifest-path target/core-standalone/Cargo.toml --offline -j 1
cargo +1.94.1 clippy --manifest-path target/core-standalone/Cargo.toml --offline --all-targets -j 1 -- -D warnings
cargo +1.94.1 run --manifest-path target/core-standalone/Cargo.toml --offline --example reconcile_dirty -j 1
rustfmt +1.94.1 --edition 2024 --check crates/acme-core/src/tree.rs crates/acme-core/examples/reconcile_dirty.rs
git diff --check
```

## Gallery evidence and remaining gates

Source inspection of `apps/gallery/src/rt_shadow.rs` confirms Gallery's retained
tree is diagnostics-only. The active layout, paint, hit testing and accessibility
pipeline is not switched to retained-tree incremental updates by this patch.
The headless example proves the retained-tree update cycle. The separate Gallery
event test proves the editing handler flow. Neither proves Gallery visual
usability. No screenshot or manual visual sign-off was produced.

Remaining: compilation with pinned Rust 1.88, cross-platform CI, dependency
audits, GPU device-loss validation and manual native GUI checks. GUI resize, DPI
125/150/200%, visible focus/keyboard behavior, repeated interaction, close/cancel,
error states and Traditional Chinese IME remain untested. No native automation
workaround was used. Existing Rust 1.94.1 was sufficient for explicitly labeled
compatibility verification, so no additional toolchain installation is requested.

This is a correctness fix. No performance improvement is claimed or benchmarked;
layout propagation walks the ancestor chain for each changed node.

Remaining product gaps include label-placeholder mobile components, broader
component accessibility, active retained-tree integration, visual regression
and benchmark thresholds. They need separate bounded work and are not claimed
complete by this patch.
