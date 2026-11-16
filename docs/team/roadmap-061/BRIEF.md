# ROADMAP 061 — Sidenav typed activation and keyboard navigation

- **Base:** master `ecd81e3` (includes 060).
- **Severity:** P1. `crates/acme-ui/src/desktop/sidenav.rs` builds passive Cards (`sidenav_item_{value}`, `:154`), so no pointer or keyboard activation is possible, `SidenavItem::disabled` has no effect, and Sidenav has no Gallery demo.
- **Execution mode:** Claude Code agent team, following the 057 Sidebar contract (proven pattern).
  - Fable 5.2: independent reviewer.
  - Opus 5.5: component.
  - Sonnet 5.5: Gallery, single writer of `main.rs`.
  - Opus 5.5 Lead: gates and real-window GUI.

## OBSERVED (master)
- **No activation:** the builder has no `on_select`; the selected item is only a `CardVariant::Interactive` card.
- **Disabled is inert:** `disabled` is never read in `From<SidenavBuilder>`.
- **Unscoped keys:** item keys `sidenav_item_{value}` are not prefixed by the sidenav id, so two instances collide.
- **Section headers:** plain labels.

## Expected behavior
1. **Additive API:**
   - `SidenavBuilder::on_select(fn(&str) -> M)` and `SidenavBuilder::navigate(&self, forward: bool) -> Option<M>`.
   - Values must be unique across all sections and must not contain `:` (documented, not enforced).
2. **No handler:** the tree is byte-identical to master; existing tests pass unmodified.
3. **With a handler:**
   - Column keyed `{id}`; section headers stay labels.
   - Each item is a full-width Button `{id}:item:{value}`. Enabled items bind `on_select(value)`; disabled items get no message and therefore no hit region.
   - Selected item: `ButtonVariant::Primary` with a `▸ ` marker; others Ghost with two-space padding (same as Sidebar). Theme tokens only. An optional icon is rendered like Sidebar's (`icon` is a `String` here, so it is a text prefix).
4. **`navigate`** walks the flattened item list across sections: it starts from the selected item, moves to the next enabled item and wraps; with no selection it returns the first (forward) or last (backward) enabled item; `None` with no handler or no enabled item. Reuse the existing rove helper if it is crate-visible, else a private equivalent (no new public API).
5. **Gallery → Layout, a "Sidenav — typed sections" section appended after the Sidebar section:**
   - Key `gallery_layout_sidenav`; two sections (General: Profile, Appearance; Advanced: Privacy, Security disabled); status label "Sidenav selected: …".
   - zh-TW hint like the Sidebar's: `點選項目切換；F8 進入側邊導覽，↓/↑ 移動項目（跨區段），Enter / Space 確認，Esc 離開。`
   - F8 enters and reveals only when out of view (059 `reveal_minimal` rule). Down/Up call `navigate` (selection follows focus); Enter/Space confirm (status line "confirmed"); Esc leaves; repeat keys and Ctrl/Alt/Meta chords are ignored; preedit guard; `reset_hover_press()` on every change.
   - Blur, a category change, or a pointer press off the items deactivate the group.
6. **Layout keyboard owner:** extend the single owner from two groups (F6 NavigationMenu, F7 Sidebar) to three. Entering any group deactivates the others; `layout_keyboard_release()` releases all three; the owner frame / active indicator (059 `gallery_layout_sidebar_focus` pattern) gets a `gallery_layout_sidenav_focus` equivalent with zh-TW 鍵盤導覽：使用中／未使用.

## Write set
- **Component (Opus):** `crates/acme-ui/src/desktop/sidenav.rs` only.
- **Gallery (Sonnet):** `apps/acme-gallery/src/sidenav.rs` (new), `apps/acme-gallery/src/sidenav_tests.rs` (new), `apps/acme-gallery/src/main.rs`, `apps/acme-gallery/src/sidebar.rs` (owner integration only), `apps/acme-gallery/src/navigation_menu.rs` (owner integration only).
- **Lead:** `docs/team/roadmap-061/*`, `docs/status/{todos,final}.md`, `docs/agent-team/ROADMAP.md`.

## Acceptance tests
**Component:**
- `sidenav_without_handler_matches_legacy_tree`
- `sidenav_on_select_builds_instance_keyed_buttons_bound_to_values`
- `sidenav_disabled_item_never_activates`
- `sidenav_selected_item_is_primary_and_marked`
- `sidenav_navigate_crosses_sections_wraps_and_skips_disabled`
- `sidenav_navigate_without_selection_returns_first_or_last_enabled`
- `sidenav_navigate_none_without_handler_or_enabled_item`
- `sidenav_item_keys_distinct_per_instance`

**Gallery:**
- `gallery_sidenav_pointer_selects_each_enabled_item_with_real_events`
- `gallery_sidenav_disabled_item_has_no_hit_region`
- `gallery_sidenav_f8_reveals_only_when_hidden`
- `gallery_sidenav_up_down_cross_sections_wrap_skip_disabled_and_enter_space_confirm`
- `gallery_sidenav_repeat_and_modifier_chords_ignored`
- `gallery_sidenav_pointer_press_off_group_deactivates`
- `gallery_sidenav_keys_ignored_off_page_after_escape_blur_and_preedit`
- `gallery_layout_f6_f7_f8_groups_are_mutually_exclusive`
- `gallery_sidenav_hit_and_paint_slots_match_across_themes_and_scales`

## Non-goals
- Label left alignment (063), contrast retunes (062), collapse state.
