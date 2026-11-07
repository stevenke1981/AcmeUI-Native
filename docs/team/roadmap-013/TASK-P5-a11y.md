# ROADMAP 013 — TASK P5 (Antigravity / Claude Sonnet 5.5): TreeItem actions and content walk

Worktree: `D:\AcmeUI-013-a11y`, branch `sdk/roadmap-013-a11y` (based on master 1d81e52). Work only inside this worktree.
The lead (p3, Claude Opus) is splitting `crates/acme-textinput/src/lib.rs` in parallel in another worktree. You own **only** `crates/acme-accessibility/**`. Do not touch other crates.

## Open items (from docs/status/todos.md, ROADMAP 009)
- `[ ] Widgets inside tree-item content are not walked (name only); TreeItem has no Focus/Click action.`

## Current code (read first)
- `crates/acme-accessibility/src/structure.rs`:
  - `tree_item_ids` and `tree_item_node`, around lines 138–225.
  - A `TreeItem` node gets level, position/size of set, expanded, selected, label and bounds. It gets **no actions**. Its `content` is used only for the name (`name_of`).
- `crates/acme-accessibility/src/lib.rs`:
  - How ordinary interactive widgets get actions. Around line 835, `access_node.add_action(Action::Focus)` / `Action::Click`.
  - How `AccessibilityAction::Focus/Click/Activate(id)` maps to `PlatformEvent`s, around line 204.
- `crates/acme-accessibility/src/structure/tests.rs`: the existing structural tests and their style.
- The layout-ID invariant (ROADMAP 010): the accessibility walk must consume layout IDs in exactly the same order as layout does. Read the module docs and the tests about "mirrors layout ID consumption" **before** you change the walk.

## Required
1. **TreeItem actions.** Every TreeItem node supports `Action::Focus` and `Action::Click`. An item with children also supports expand/collapse, using AccessKit `Action::Expand` / `Action::Collapse` if this AccessKit version has them. Check the dependency source, and say which actions you used.
   Make sure that `AccessibilityAction::Click(id)` / `Focus(id)` on a TreeItem id routes to a sensible `PlatformEvent` that the app can act on. It must select (or focus) that row, consistent with how keyboard and mouse select tree rows today. If routing needs a new event or variant, keep it additive and document it.
2. **Walk tree-item content.** Interactive widgets inside a tree item's `content` become children of the TreeItem node, with correct roles, names, actions and bounds. Examples: a Checkbox, a Button, or a text label that today only contributes a name.
   Keep the TreeItem's own name rule unchanged unless the content has no name. The layout-ID order must stay identical to layout. Add a real-layout test that proves it, in the same style as the ROADMAP 010 tests.
3. No behaviour change for trees whose items contain only plain text, apart from the new actions. Prove this with a test or an explicit snapshot comparison.

## Hard rules
- TDD: write failing tests first (RED), keep the evidence, then implement (GREEN). Add at least one mutation check: break the new code on purpose, show that a test fails, then revert.
- Change only `crates/acme-accessibility/**`, plus the new task/report docs in `docs/team/roadmap-013/`. Do not git add, commit, stash, checkout or push. The lead commits.
- No new dependencies. Toolchain: Rust 1.88.0 (`rust-toolchain` in the repo). On Windows, run the Python helper scripts with `PYTHONUTF8=1`.
- Verify:
  - `cargo fmt -p acme-accessibility -- --check`
  - `cargo clippy -p acme-accessibility --all-targets -- -D warnings`
  - `cargo test -p acme-accessibility`
  - `cargo check --workspace --all-targets` (consumers such as Gallery must still compile).

## Report
Write `docs/team/roadmap-013/REPORT-P5-a11y.md` with:
- the design
- the files changed
- the AccessKit actions used
- the tests: RED, then GREEN, then mutation
- the commands run, with their results
- what is NOT_RUN (for example a real screen reader)

End your final message with a line containing only `P5 DONE`.
