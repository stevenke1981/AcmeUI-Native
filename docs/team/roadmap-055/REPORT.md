# ROADMAP 055 — Report: RadioCards typed selection and arrow-key navigation

## Execution (actual)
| Role | Model | Work |
|---|---|---|
| Lead / integration | Claude Opus 5.5 | Branching, the PR #70 landing fix, docs, commit/PR |
| Adviser + independent reviewer | Claude Fable 5.2 | Read-only audit and contract; read-only review of the finished diff |
| Component implementer | Claude Opus 5.5 (subagent) | `crates/acme-ui/src/inputs/radio_cards.rs` |
| Gallery worker (single writer of `main.rs`) | Claude Sonnet 5.5 (subagent) | `apps/acme-gallery/src/{radio_cards.rs,radio_cards_tests.rs,main.rs}` |
| Verification | Claude Haiku 5.5 (subagent) | Gates below and raw logs |

The component and Gallery workers ran in parallel on non-overlapping leases in one checkout. The reviewer is a different model from both authors.

## Problem (OBSERVED, reproduced RED)
`RadioCardsBuilder::on_select` was stored, but `From` never read it. Options were unkeyed passive Card→Label trees, so nothing could be clicked or reached with the keyboard. The RED log is `logs/component-red.txt`: the new tests failed to compile, with E0599 for the missing `navigate`.

## Change
- With `on_select` set, each option renders a Card keyed `"{id}:card:{value}"` holding a full-width Button keyed `"{id}:option:{value}"` that emits `on_select(value)`. The selected option is `ButtonVariant::Primary` and the others are `Ghost`, each with the ◉/○ marker, so state is not conveyed by colour alone.
- Without a handler, the tree is unchanged; this is pinned by an equality test against the previous tree.
- Additive API: `RadioCardsBuilder::navigate(&self, forward: bool) -> Option<M>`. It wraps; with no or unknown selection it returns the first or last option; with no options or no handler it returns `None`. Values must be unique, as documented.
- Gallery → Inputs: a controlled demo with 入門方案 / 專業方案 / 企業方案. Pointer click selects and confirms. F6 enters the group, arrows move the selection with wrap, Enter/Space confirm, and Esc, blur or a category change leave it. The host is scoped to category 2 and respects the preedit guard; the F6 host on the Mobile page (category 7) is unchanged.

## Tests added
- Component: `radio_cards_options_bind_typed_select_messages_to_instance_keyed_buttons`, `radio_cards_without_handler_stay_passive_and_keep_legacy_tree`, `radio_cards_navigate_wraps_and_handles_empty_unknown_and_unselected`, `radio_cards_retained_ids_survive_selection_change_and_keyed_reorder`, `radio_cards_duplicate_keys_absent_and_selected_variant_uses_theme_tokens`
- Gallery: `gallery_radio_cards_pointer_selects_each_option_with_real_events`, `gallery_radio_cards_f6_arrow_keys_wrap_and_enter_space_confirm`, `gallery_radio_cards_keys_ignored_off_page_and_after_escape_or_blur`, `gallery_radio_cards_hit_and_paint_slots_match_across_themes_and_scales`

## Verification
Tested source: HEAD `aeb0c8d` plus the uncommitted 055 diff (`logs/env.txt`) on Windows 11 with rustc/cargo 1.88.0.

| Command | Result |
|---|---|
| `cargo fmt -p acme-ui -p acme-ui-gallery -- --check` | PASS |
| `cargo test --locked -p acme-ui --all-features` | PASS (894 passed, 0 failed, 2 ignored) |
| `cargo test --locked -p acme-ui-gallery` | PASS (105 passed, 0 failed) |
| `cargo clippy --locked -p acme-ui --all-features -p acme-ui-gallery --all-targets --no-deps -- -D warnings` | PASS |
| `cargo check --locked --workspace --all-targets` | PASS |
| `cargo test --locked --workspace` | PASS (2050 passed, 0 failed, 11 ignored) |
| `python scripts/verify_sdk.py --profile cpu` | PASS. It covers only `acme-render-cpu`, so it is not evidence for this UI change. |

`logs/gallery-worker-run-before-ctxmenu-fix.txt` is the Gallery worker's earlier run. It shows the pre-existing `context_menu_tests` failure, which was fixed on PR #70 in `aeb0c8d` before the gates above were run.

## Independent review (Fable 5.2): no P0/P1
Open P2 follow-ups:
1. Duplicate option values produce duplicate sibling keys, which panic the host, and `navigate` sticks on the first duplicate. Uniqueness is documented but not enforced.
2. No test covers the pointer-off-card deactivation or the preedit arrow guard.
3. F6 does not scroll the group into view (compare `reveal_list_focus`).
4. There is no visible focus indicator beyond the status text.
5. Modified keys (Ctrl/Alt) are consumed; this matches the sibling hosts.
6. Only the title Button is a hit target; the description and padding are not.
7. The keyboard clear sets hover/press directly instead of calling `reset_hover_press()`.
8. A Snackbar press does not deactivate the group.

## NOT_RUN
Human Gallery interaction, Windows 注音 IME, physical DPI/touch, GPU, screen reader, and AccessKit RadioGroup semantics. The IME guard is tested only by setting `preedit` directly.

## Rollback
Revert the four source files listed in the write set.
