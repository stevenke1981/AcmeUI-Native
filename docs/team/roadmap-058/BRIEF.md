# ROADMAP 058 — CheckboxCards typed toggling and roving keyboard focus

- **Base:** master `3a1814c` (includes 055 RadioCards, 056 NavigationMenu, 057 Sidebar).
- **Severity:** P1. `CheckboxCardsBuilder::on_toggle` exists but is never bound: every card renders a passive Label (`crates/acme-ui/src/inputs/checkbox_cards.rs:79-93`), so there is no pointer or keyboard toggling.
- **Execution mode:** Claude Code agent team. Fable 5.2 adviser (decisions, `058-decisions` D1–D7) and independent reviewer; Opus 5.5 component; Sonnet 5.5 Gallery (single writer of `main.rs`); Haiku 5.5 verification; Opus 5.5 Lead integration and real-window GUI.

## Expected behavior
1. **D1 Handler:** keep `on_toggle(fn(&str) -> M)`. The host owns checked state and passes it with `selected(..)`.
2. **D2 Keys (handler mode only):** Column `{id}`, Card `{id}:card:{v}`, title Button `{id}:option:{v}` with `full_width(true)` and `on_click(on_toggle(v))`. Checked → `ButtonVariant::Primary` with `☑`; unchecked → Ghost with `☐`. Theme tokens only. Without a handler the tree is byte-identical to master and the 4 existing tests pass unmodified.
3. **D3 Focus:** additive `focused(value)`. In handler mode the focused card is `CardVariant::Elevated` (others Outlined) and its title gets a `▸ ` prefix before the marker. Keys never depend on checked or focused state.
4. **Additive `navigate_focus(&self, current: Option<&str>, forward: bool) -> Option<&str>`:** returns the next/previous option value, wrapping; with `None` returns the first (forward) or last (backward). Returns `None` with no options. Never emits a message (focus moves, toggling does not). Uniqueness of values and no `:` are documented, not enforced.
5. **D6:** disabled options are a non-goal.
6. **Gallery → Inputs:** a "CheckboxCards — typed toggling" section directly after RadioCards. Key `gallery_checkbox_cards`; options 電子郵件 / 簡訊 / 推播通知 / 每週摘要 (values `email`, `sms`, `push`, `digest`); status labels for the checked set, the focused item and "Keyboard group: active/inactive".
   - Pointer: clicking a card's Button toggles it and activates the group (focus moves to it).
   - **D5/D4 keyboard:** F7 enters (focus = first option if none) and reveals the group only if hidden; ↓/→ and ↑/← move focus via `navigate_focus`; **Space toggles** the focused option; **Enter is unhandled (returns None)**; Esc leaves.
   - Blur, a category change, or a pointer press off the cards deactivate the group. Every keyboard change calls `reset_hover_press()`.
   - Preedit guard: handled keys are ignored during composition; unhandled keys pass through (same as 057 R2).
   - The value→message mapping must reject unknown values (no `unwrap_or(0)`).
7. **D5 Page owner:** `inputs_keyboard_event` in the Gallery owns category-2 (Inputs) keyboard hosts, mirroring 057 `layout_page_event`. F6 enters RadioCards and deactivates CheckboxCards; F7 enters CheckboxCards and deactivates RadioCards. Pointer and FocusChanged always return None.
8. **D7 055 P2 fold-ins:** #3 F6 reveals RadioCards only when hidden (reuse `reveal_if_hidden`); #7 RadioCards keyboard clears call `reset_hover_press()`; #2 tests for RadioCards pointer-off deactivation and the preedit arrow guard, placed in `checkbox_cards_tests.rs` (`radio_cards_tests.rs` stays frozen).

## Write set
- **Component (Opus):** `crates/acme-ui/src/inputs/checkbox_cards.rs` only.
- **Gallery (Sonnet):** `apps/acme-gallery/src/checkbox_cards.rs` (new), `apps/acme-gallery/src/checkbox_cards_tests.rs` (new), `apps/acme-gallery/src/radio_cards.rs`, `apps/acme-gallery/src/main.rs`. Reuse helpers in `navigation_menu.rs` without editing it.
- **Lead:** `docs/team/roadmap-058/*`, status docs.

## Acceptance tests
**Component:** `checkbox_cards_without_handler_matches_legacy_tree`, `checkbox_cards_on_toggle_binds_instance_keyed_buttons`, `checkbox_cards_checked_is_primary_unchecked_ghost`, `checkbox_cards_focused_is_elevated_and_marked`, `checkbox_cards_keys_stable_across_checked_and_focus`, `checkbox_cards_navigate_focus_wraps_and_handles_none_and_empty`, `checkbox_cards_navigate_focus_unknown_current_starts_at_edge`.

**Gallery:** `gallery_checkbox_cards_pointer_toggles_each_option_with_real_events`, `gallery_checkbox_cards_f7_enters_and_reveals_only_when_hidden`, `gallery_checkbox_cards_arrows_move_focus_without_toggling`, `gallery_checkbox_cards_space_toggles_enter_passes_through`, `gallery_checkbox_cards_escape_blur_category_and_pointer_off_deactivate`, `gallery_checkbox_cards_preedit_guard_passes_unhandled_keys`, `gallery_checkbox_cards_f6_f7_mutually_exclusive_with_radio_cards`, `gallery_checkbox_cards_unknown_value_is_rejected`, `gallery_checkbox_cards_hit_and_paint_slots_match_across_themes_and_scales`.

**D7:** `gallery_radio_cards_f6_reveals_only_when_hidden`, `gallery_radio_cards_pointer_off_card_deactivates`, `gallery_radio_cards_preedit_blocks_arrows`.

**Must stay green, unmodified:** `radio_cards_tests`, `sidebar_tests`, `navigation_menu_tests`, `anchor_tests`, `retained_tests`.

## Non-goals
Disabled options; Home/End and typeahead; Tab traversal; AccessKit checkbox semantics; the description and padding as hit targets (055 P2 #6); Snackbar deactivation (056 P2, scheduled in 059).
