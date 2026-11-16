# ROADMAP 061 — Report: Sidenav typed activation and keyboard navigation

## Execution (actual)
| Role | Model | Work |
|---|---|---|
| Lead / brief / gates / GUI | Claude Opus 5.5 | Brief (from the 057 contract), feature-manifest fix, gates, real-window GUI, docs, PR |
| Component | Claude Opus 5.5 (subagents ×2) | `crates/acme-ui/src/desktop/sidenav.rs` |
| Gallery | Claude Sonnet 5.5 (subagents ×2) | F8 host, three-way Layout owner, tests |
| Independent reviewer | Claude Fable 5.2 | Read-only review: 1 P1, 8 P2 |

## Change
- **Component (`crates/acme-ui/src/desktop/sidenav.rs`):**
  - `SidenavBuilder::on_select(fn(&str) -> M)` and `navigate(&self, forward) -> Option<M>`.
  - With a handler, items are full-width Buttons keyed `{id}:item:{value}`: Primary with `▸ ` when selected, Ghost with two-space padding otherwise. Disabled items get no message.
  - Without a handler the tree is identical to master (tested against a replica of the legacy construction).
  - `navigate` crosses sections, wraps and skips disabled items, using a private rove helper.
  - **Note:** `SidenavBuilder` gains a private field, so external struct literals no longer compile. Nothing in the workspace uses one; the same happened to `SidebarBuilder` in 057.
- **Gallery → Layout, "Sidenav — typed sections":**
  - F8 enters, revealing only when hidden (falling back to the current item on short viewports).
  - ↓/↑ rove; repeat roves, matching the Sidebar.
  - Enter/Space confirm, ignoring repeat. Esc leaves.
  - Repeat F8 and Ctrl/Alt/Meta chords are ignored, and the preedit guard applies.
  - Blur, a category change, or a press off the group deactivates it.
  - Owner frame `gallery_layout_sidenav_focus` with ring, plus status 鍵盤導覽：使用中／未使用.
  - **Single Layout owner:** F6, F7 and F8 are mutually exclusive, and `layout_keyboard_release` releases all three.
- **Pre-existing manifest bug fixed:** `overlay` and `desktop` used the `foundations` `IconName` without depending on it, so each failed to build alone (`--no-default-features --features desktop|overlay`). They now enable `foundations`. CI's minimal-feature step now checks foundations, overlay, charts, desktop, mobile and browser individually.

## Verification
Base `ecd81e3`; tested on the final uncommitted tree; CPU profile on the commit is in `cpu-profile-*`.

| Command | Result |
|---|---|
| `cargo fmt -p acme-ui -p acme-ui-gallery -- --check` | PASS |
| `cargo test --locked -p acme-ui --all-features` | PASS (925 passed, 0 failed, 2 ignored) |
| `cargo test --locked -p acme-ui-gallery` | PASS (187 passed, 0 failed) |
| clippy `-p acme-ui --all-features -p acme-ui-gallery --all-targets -D warnings` | PASS |
| `cargo check --locked --workspace --all-targets` | PASS |
| `cargo test --locked --workspace` | PASS (2174 passed, 0 failed, 11 ignored) |
| `cargo check -p acme-ui --no-default-features --features <each of 6>` | PASS (desktop and overlay failed before the fix) |
| `cargo test -p acme-ui --no-default-features --features desktop sidenav` | PASS (19) |

Real-window GUI: [GUI.md](GUI.md).

## Independent review (Fable 5.2)
- **P1, `desktop` reaching into the `layout` rove:** fixed with a private rove in `sidenav.rs`; `sidebar.rs` is unchanged from master.
- **P2-1, private field:** documented.
- **P2-2, red evidence is compile-only:** accepted. Mutation checks were added for the review fixes.
- **P2-3, exclusivity lines:** a new test, which mutation-checks fail as expected.
- **P2-4, tests through real events:** the disabled-row press, the category click and the chords now go through `g.event`.
- **P2-5, F8 repeat:** ignored, with a test. Arrows keep repeat-roving, matching the Sidebar (Lead decision).
- **P2-6, short-viewport fallback:** added, with a test.
- **P2-7, stale docs:** fixed.
- **P2-8, Shift not guarded:** accepted, matching 057.

## Rollback
Revert the commit. The manifest change only adds `foundations` to two feature lists, and the lockfile is unchanged.
