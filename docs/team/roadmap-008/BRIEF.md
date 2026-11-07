# ROADMAP 008 — Interaction and accessibility follow-ups (BATCH-008)

ROADMAP 001–007 are merged. This batch takes bounded, automatable follow-ups
recorded in `docs/status/todos.md` (batch-002 "Follow-ups" line and the
ROADMAP 007 NOT_RUN list). Base: `sdk/roadmap-008-followups` on master
`8df7212`. Rust 1.88.0 pinned, Windows MSVC host. Read `AGENTS.md` first.

Not in scope (owner decisions): version bump, `acme-core` rename, license text
in `.crate` files, `move_cursor_to_offset` byte-offset API, manual IME/DPI/
screen-reader/GPU acceptance.

## Hard rules (all agents)

- Edit only files in your write set. Do not run `git add/commit/stash/checkout`
  or switch branches. Do not add third-party dependencies.
- AGENTS.md: no GPUI; no hardcoded theme colors in widgets; no byte offsets
  for text cursors; no platform-specific types in public API; never claim
  注音 IME works.
- Test-first: write the failing test, show it fails (RED), then fix (GREEN).
- Run only your own crate's tests plus `cargo clippy --locked -p <crate>
  --all-targets -- -D warnings` on files you touched and `cargo fmt`.
  Never `cargo clean`.
- Report: files changed, RED output excerpt, GREEN command + result, anything
  NOT_RUN, open questions. Keep it under 60 lines.

## Work items

| ID | Role | Write set |
|---|---|---|
| P1 | team-rust-engineer | `crates/acme-accessibility/src/**` |
| P2 | team-desktop-engineer | `apps/gallery/src/**` |
| P3 | team-test-engineer | new `examples/sdk-consumer/tests/migration_snippets.rs`, `examples/sdk-consumer/Cargo.toml` (only `[dev-dependencies]` on existing workspace crates by path), `docs/guides/MIGRATION.md` (fix snippets only) |

Lead owns: `.github/workflows/*`, `docs/CHANGELOG.md`, `docs/status/*`,
`docs/agent-team/ROADMAP.md`, `Cargo.toml`, `Cargo.lock`, this folder.

### P1 — AccessKit semantics

1. `TextInput.validation_message` (`acme-widgets/src/inputs/text_input.rs`)
   is not exposed to AccessKit. When `Some`, mark the node invalid
   (AccessKit 0.21 `Invalid` / `set_invalid`) and expose the message as the
   node description (or the AccessKit error-message relation if the 0.21 API
   supports it without extra nodes — decide and justify). `None` → neither set.
2. Tree items, Table cells/rows, DataGrid cells: inspect how
   `acme-widgets` represents their children and how the accessibility
   builder walks them. Where children map to retained nodes, give them
   `TreeItem` / `Row` / `Cell` (or `GridCell`) roles. If the widget renders
   items as plain Labels without a per-item node, do not invent a new tree
   structure — report exactly what would be needed.
3. Keep legacy sequential APIs (`update`, `build_tree_update`) and the
   retained-binding path consistent; add tests for both where the role
   mapping is shared.

### P2 — Gallery hover/press drift

`apps/gallery` keeps `hovered: Option<usize>` / `pressed: Option<usize>`
hit indices (`main.rs`, `events/dispatch.rs`). After a page switch the hit
index list changes, so a stale index highlights/activates a different widget.
Reproduce with a headless test (existing `focus_tests.rs` / `dpi_tests.rs`
style), then fix: clear (or re-resolve) hover/press whenever the page or the
hit-index list changes. A release after page switch must not activate a
widget on the new page. No visual/manual claims.

### P3 — MIGRATION.md snippets compile

`docs/guides/MIGRATION.md` has 10 ```` ```rust ```` blocks; none is compiled.
Turn each *after*-form snippet into a compiled test (or `#[allow(dead_code)]`
fn) in `examples/sdk-consumer/tests/migration_snippets.rs`, one test per
M-item, named `m01_…`. Snippets showing the *old* (removed) API are not
compiled; mark them in MIGRATION.md as ```` ```rust,ignore ```` only if they
are labelled "before". If a snippet is wrong, fix the snippet in MIGRATION.md
(keep it minimal) and say so. Verify with
`cp Cargo.lock examples/sdk-consumer/Cargo.lock && cd examples/sdk-consumer &&
cargo test` (restore/remove the copied lockfile afterwards if it was not
tracked — check `git ls-files examples/sdk-consumer`).
