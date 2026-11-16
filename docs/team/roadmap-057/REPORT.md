# ROADMAP 057 — Report: Sidebar typed activation and roving keyboard navigation

## Execution (actual)
| Role | Model | Work |
|---|---|---|
| Lead / integration / real-window GUI | Claude Opus 5.5 | Branch, brief, GUI verification, docs, commit and PR |
| Adviser + independent reviewer | Claude Fable 5.2 | Read-only contract; read-only review of the finished diff |
| Component implementer | Claude Opus 5.5 (subagent) | `crates/acme-ui/src/layout/sidebar.rs`, R3 doc in `navigation_menu.rs` |
| Gallery worker (single writer of `main.rs`) | Claude Sonnet 5.5 (subagent) | `apps/acme-gallery/src/{sidebar.rs,sidebar_tests.rs,navigation_menu.rs,main.rs}` |
| Verification | Claude Haiku 5.5 (subagent) | Gates and raw logs (run twice: before and after review fixes) |

The component and Gallery workers ran in parallel on non-overlapping leases. The reviewer is a different model from both authors.

## Problem (OBSERVED)
Sidebar items were plain Labels with no handler, no keys and no disabled state; collapsed mode with an icon dropped the current-item marker. RED: `logs/component-red.txt`.

## Change
- **Component (additive):** `SidebarItem::value` (defaults to the label) and `disabled`; `SidebarBuilder::on_select(fn(&str) -> M)` and `navigate(forward) -> Option<M>`. With a handler, items are full-width Buttons keyed `{id}:item:{value}`; the current item is Primary with `▸ `, others Ghost; collapsed mode shows the icon or the label's first `char`. Disabled items never emit and have no hit region. Without a handler the tree is unchanged (legacy tests unmodified).
- **Gallery → Layout, "Sidebar — typed activation":** pointer selection; F7 enters and reveals the whole group; ↓/↑ rove (skip disabled, wrap); Enter/Space confirm; ←/→ collapse/expand; Esc leaves. Blur, category change or a press off the group deactivate. One page keyboard owner (`layout_page_event`) makes F6 (NavigationMenu) and F7 (Sidebar) mutually exclusive.
- **056 follow-ups folded in:** R1 reveal only when the target is hidden (and reveal the focused child, not just the parent); R2 preedit guard passes unhandled keys through; R3 uniqueness doc.

## Tests
Component: 8 new tests (12 sidebar tests pass). Gallery: 17 new tests in `sidebar_tests.rs` (130 Gallery tests pass). A mutation check (forcing reveal on every key) fails two R1 tests, so the "only when hidden" guard is now actually covered.

## Verification
Tested source: HEAD `8735e43` plus the uncommitted 057 diff (`logs/env.txt`), Windows 11, rustc/cargo 1.88.0.

| Command | Result |
|---|---|
| `cargo fmt -p acme-ui -p acme-ui-gallery -- --check` | PASS |
| `cargo test --locked -p acme-ui --all-features` | PASS (909 passed, 0 failed, 2 ignored) |
| `cargo test --locked -p acme-ui-gallery` | PASS (130 passed, 0 failed) |
| `cargo clippy --locked -p acme-ui --all-features -p acme-ui-gallery --all-targets --no-deps -- -D warnings` | PASS |
| `cargo check --locked --workspace --all-targets` | PASS |
| `cargo test --locked --workspace` | PASS (2090 passed, 0 failed, 11 ignored) |

Real-window GUI: 13/13 steps PASS with real pointer and keyboard input at 150% DPI; see [GUI.md](GUI.md).

## Independent review (Fable 5.2): 2 P1, 5 P2 — resolved as follows
- P1-1 R1 guard untested → tests now start off-anchor; mutation check proves coverage. **Fixed.**
- P1-2 pointer-click jump when the in-flow dropdown closes near max scroll → **known limitation**, documented in the test; not claimed fixed.
- P2-3 reveal focused child → fixed. P2-4 F7 reveals whole group → fixed. P2-6 symmetric exclusion → fixed. P2-7 F6-during-preedit test → added.
- P2-5 click on disabled item deactivates the group → accepted (same as 056).
- Compat: new pub fields on `SidebarItem` break external struct literals (none in repo).

## Follow-ups
- 059: nav-menu reveal margin hides the section heading; child reveal is top-anchored; remaining 056 P2s.
- 060: selected Gallery category contrast.
- 063: left-align Sidebar/Sidenav item labels.

## NOT_RUN
Windows 注音 IME composition, screen reader, touch, other DPI/monitors, GPU path, dark theme in the real window.

## Rollback
Revert the four tracked source files and delete `apps/acme-gallery/src/sidebar{,_tests}.rs`. No manifest or lockfile change.
