# ROADMAP 056 — Report: NavigationMenu typed activation and roving keyboard navigation

## Execution (actual)
| Role | Model | Work |
|---|---|---|
| Lead / integration | Claude Opus 5.5 | Branch, brief, docs, commit and PR |
| Adviser + independent reviewer | Claude Fable 5.2 | Read-only contract; read-only review of the finished diff |
| Component implementer | Claude Opus 5.5 (subagent) | `crates/acme-ui/src/layout/navigation_menu.rs` |
| Gallery worker (single writer of `main.rs`) | Claude Sonnet 5.5 (subagent) | `apps/acme-gallery/src/{navigation_menu.rs,navigation_menu_tests.rs,main.rs}` |
| Verification | Claude Haiku 5.5 (subagent) | Gates and raw logs |

The component and Gallery workers ran in parallel on non-overlapping leases. The reviewer is a different model from both authors.

## Problem (OBSERVED, reproduced RED)
Top-level and child Buttons were built without `on_click`, so the bar could not navigate. Selection gated only the dropdown. The legacy keys are ambiguous: item `a_b` collides with item `a` plus child `b`. RED: `logs/component-red.txt` fails to compile with 26 missing-method errors.

## Change
- New additive builder methods: `on_select(fn(&str) -> M)`, `active_child(value)`, `navigate(forward) -> Option<M>` and `navigate_child(current, forward) -> Option<&str>`.
- Handler-mode keys are Row `{id}`, `{id}:group:{v}`, `{id}:item:{v}`, `{id}:menu:{v}` and `{id}:sub:{parent}:{child}`.
- Without a handler, the tree and the legacy `_` keys are byte-identical, pinned by a copy of the old build.
- The selected item and the active child are Primary, and the rest are Ghost. Items with children show ` ▾`, and the active child shows `▸ `. No hardcoded colours.
- Disabled items, disabled children and children of a disabled parent never emit and have no hit region.
- `navigate` and `navigate_child` wrap and skip disabled entries. `navigate_child(None, false)` enters at the last enabled child.
- Gallery → Layout gets a section after PageHeader: 首頁 / 產品 (桌面版, 行動版, 企業版 disabled) / 文件 / 支援 (disabled). It supports pointer activation and this keyboard model:
  - F6 enters the menu and reveals it.
  - Left/Right rove items; Down/Up rove children.
  - Enter/Space activate.
  - Esc first clears child focus, then leaves.
  - Blur, a category change, or a pointer press off the menu deactivate it.
  - The host is scoped to category 3 and `!overlay_pointer`, and respects the preedit guard.

## Tests added
Component: 7 new tests; the 7 existing tests are unchanged.
- `navigation_menu_items_bind_typed_select_messages_to_instance_keyed_buttons`
- `navigation_menu_without_handler_keeps_legacy_tree_and_keys`
- `navigation_menu_disabled_items_and_children_never_emit`
- `navigation_menu_navigate_wraps_skips_disabled_and_handles_empty_unknown`
- `navigation_menu_navigate_child_wraps_skips_disabled_and_requires_open_parent`
- `navigation_menu_retained_ids_survive_selection_change_and_dropdown_toggle`
- `navigation_menu_keys_unique_and_variants_use_theme_tokens`

Gallery: 8 new tests.
- `gallery_navigation_menu_pointer_activates_items_and_children_with_real_events`
- `gallery_navigation_menu_disabled_items_have_no_hit_region`
- `gallery_navigation_menu_f6_reveals_offscreen_menu_and_arrows_rove_skipping_disabled`
- `gallery_navigation_menu_down_enter_activates_child_and_escape_steps_out`
- `gallery_navigation_menu_pointer_press_off_menu_deactivates_group`
- `gallery_navigation_menu_preedit_blocks_f6_and_arrow_keys`
- `gallery_navigation_menu_keys_ignored_off_page_and_after_blur_or_category_change`
- `gallery_navigation_menu_hit_and_paint_slots_match_across_themes_and_scales`

The retained-ID test was narrowed. Closing the dropdown removes the Card, so child IDs are fresh on reopen. Row, group and item IDs stay stable throughout.

## Verification
Tested source: HEAD `a84af74` plus the uncommitted 056 diff (`logs/env.txt`), on Windows 11 with rustc/cargo 1.88.0.

| Command | Result |
|---|---|
| `cargo fmt -p acme-ui -p acme-ui-gallery -- --check` | PASS |
| `cargo test --locked -p acme-ui --all-features` | PASS (901 passed, 0 failed, 2 ignored) |
| `cargo test --locked -p acme-ui-gallery` | PASS (113 passed, 0 failed) |
| `cargo clippy --locked -p acme-ui --all-features -p acme-ui-gallery --all-targets --no-deps -- -D warnings` | PASS |
| `cargo check --locked --workspace --all-targets` | PASS |
| `cargo test --locked --workspace` | PASS (2065 passed, 0 failed, 11 ignored) |

## Independent review (Fable 5.2): no P0/P1
Scheduled into ROADMAP 057, because they affect the Sidebar host on the same page:
1. Reveal snaps the scroll on every key. It should reveal only when the menu is out of view, and not on the leaving Esc.
2. The preedit guard swallows unhandled keys.
3. The doc should say values are globally unique.

057 must also make the F6 and F7 hosts mutually exclusive.

Remaining P2s:
4. A Snackbar press does not deactivate the menu.
5. Modified keys still act.
6. The Gallery handler does not reject a child of a disabled parent (unreachable today).
7. Untested paths: ArrowUp with no child focused, an unhandled key while active, and a category change via a real click.
8. There is no focus visual. The "Keyboard menu" label is in English, and glyph rendering is not verified.

## NOT_RUN
Human Gallery interaction, Windows 注音 IME (the guard is tested only by setting `preedit`), DPI/touch, GPU and screen reader.

## Rollback
Revert the four source files. There is no manifest or lockfile change.
