# ROADMAP 004 — Keyboard focus and core control semantics (BATCH-004)

Base: branch `sdk/roadmap-004-focus-semantics` (on top of ROADMAP 003, which
makes Gallery layout and accessibility IDs stable retained `NodeId`s). Rust
1.88.0 pinned, Windows MSVC. Read first: `AGENTS.md`,
`docs/agent-team/ROADMAP.md` (row 004), `docs/team/roadmap-003/REPORT.md`.

## Problems (OBSERVED / REPRODUCIBLE)

F1 (P1, crash + drift) — Gallery keyboard focus is `focused: usize`, an index
into `button_info: Vec<HitRegion>` that is rebuilt every frame.
- `handle_enter_space_key` (`apps/gallery/src/events/dispatch.rs:150`) does
  `button_info[focused]` with no bounds check. Focus a late region on a page with
  many regions, switch to a page with fewer, press Enter → index-out-of-bounds
  panic. Also `button_info` can be empty.
- After a page switch, tree expand, or table sort, the same index points at a
  different control (focus drift).
- Accessibility focus (`AccessibilityAdapter::focus_widget`) is never told
  about keyboard focus.

F2 (P1, a11y semantics) — `crates/acme-accessibility/src/lib.rs` node build:
- `TextInput` exposes only `Role::TextInput`: no name (its `label` /
  `placeholder`), no value, no `Role::PasswordInput` for `password`, no
  disabled/readonly state, no Focus/SetValue actions.
- Disabled `Button` still advertises `Action::Click`.
- `TabBar` children are plain buttons: no `Role::Tab`, no selected state for
  `TabBar::selected`; `NavRail::selected` likewise not exposed.
- Check `Dialog` (modal), `Tree`/`Table`/`DataGrid` roles of children if the
  accesskit 0.21 API supports it cheaply; otherwise report as follow-up.

## Work items

| ID | Role | Write set |
|---|---|---|
| F1 | team-desktop-engineer | `apps/gallery/src/types.rs`, `apps/gallery/src/render/hit_test.rs`, `apps/gallery/src/events/dispatch.rs`, `apps/gallery/src/main.rs`, new `apps/gallery/src/focus.rs`, new `apps/gallery/src/focus_tests.rs` |
| F2 | team-accessibility-specialist (implementer) | `crates/acme-accessibility/src/lib.rs` |

### F1 — focus keyed by stable identity

- Add `pub node: NodeId` to `HitRegion` (it is a Gallery-internal type) and fill
  it with the layout node ID of the hit target at every construction site in
  `render/hit_test.rs` (synthetic tree/table rows get their synthetic layout ID).
- New `focus.rs`: a small pure module, e.g.
  `pub struct FocusTarget { pub node: NodeId, pub message: GalleryMessage }` and
  `pub fn resolve_focus(regions: &[HitRegion], target: Option<FocusTarget>, previous: usize) -> (usize, Option<FocusTarget>)`.
  Rules: if the target's node is non-synthetic
  (`acme_widgets::retained_binding::is_synthetic`) and a region has that node →
  that index; else a region with an equal `message` → that index; else clamp
  `previous` into range; empty regions → index 0 and target `None`.
- Gallery keeps `focused_target: Option<FocusTarget>` next to `focused`. After
  `button_info` is rebuilt in `frame`, resolve `focused` from it. Tab/Shift+Tab
  update both. Enter/Space uses `button_info.get(focused)` and does nothing when
  out of range (no panic). Pointer activation, if it changes focus today, keeps
  doing so and updates the target.
- When the focused region's node is non-synthetic, call
  `self.accessibility.focus_widget(node)`.
- Keep rendering code index-based (it counts regions in the same order); do not
  rewrite the render pipeline.
- Tests (`focus_tests.rs`, headless, use `Gallery::new()` + `frame()` and the
  existing event handler entry points; read `retained_ids_tests.rs` for the
  pattern): (1) regression: focus the last region on a large page, switch to the
  smallest page, press Enter via the real `event()` path → no panic; (2) focus a
  sidebar category button, switch category, the same button stays focused;
  (3) Tab/Shift+Tab wrap on a page; (4) `resolve_focus` unit tests for each rule
  incl. empty regions; (5) accessibility adapter's focused node equals the
  focused region's retained node after Tab.

### F2 — semantics of core controls

Implement in the node-build part of `walk`/`walk_node` (shared by `update` and
`update_with_binding`). Use only accesskit 0.21 APIs that exist (check
`~/.cargo/registry/src/*/accesskit-0.21*/src/lib.rs`). Expected:
- TextInput: name from `label`, else `placeholder`; value = `value` except for
  password inputs (never expose the secret; accesskit may need an empty value or
  no value); role `PasswordInput` when `password`; disabled → `set_disabled`;
  readonly → `set_read_only`; actions `Focus` and (when editable) `SetValue`;
  `invalid` → the accesskit invalid state if it exists.
- Button: keep `Focus`; add `Click` only when not disabled.
- TabBar: each child corresponding to a tab gets `Role::Tab` and
  `set_selected(index == selected)`; the TabBar stays `TabList`.
- NavRail: the selected item exposes selected state if the item maps to a
  child node.
- Tests in the crate's test module for each rule, plus: password value never
  appears in any node property.
- Do not change public signatures. No new dependencies.

## Hard rules

Same as `docs/team/roadmap-003/BRIEF.md` "Hard rules": only your write set; no
git add/commit/stash/checkout/reset; no cargo clean; `CARGO_TARGET_DIR=target/<id>`;
pinned toolchain, `--locked`; no GUI launches; honest PASS/FAIL/NOT_RUN.
F1 runs `cargo test --locked -p acme-gallery` + clippy for that package; F2
runs the same for `acme-accessibility`. If F1 sees acme-accessibility
temporarily broken by F2's in-progress edits, wait and retry.

## Report format

```
ID / role / files changed
Summary (≤10 lines)
Commands → status (+ output tails)
Patches requested for shared files
Risks / NOT_RUN / follow-ups
```
