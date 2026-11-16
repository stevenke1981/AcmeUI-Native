# ROADMAP 057 — Sidebar typed activation and roving keyboard navigation

- **Base:** master `8735e43`, which already includes 056.
- **Severity:** P1. Sidebar items are plain Labels (`crates/acme-ui/src/layout/sidebar.rs:138-148`), so there is no pointer or keyboard activation.
- **Execution mode:** Claude Code agent team.
  - Fable 5.2: adviser (contract) and independent reviewer.
  - Opus 5.5: component.
  - Sonnet 5.5: Gallery, single writer of `main.rs`.
  - Haiku 5.5: verification.
  - Opus 5.5 Lead: integration plus real-window GUI verification.

## OBSERVED (master)
- **No handler, no keys, no disabled state:** `SidebarItem` has only `label`, `icon` and `active` (`:26-30`), and the builder has no `on_select`.
- **Collapsed state hides the current item:** collapsed with an icon drops the `▸` marker (`:144`).
- **Key collision:** the default id `"sidebar"` collides with the Gallery shell key (`apps/acme-gallery/src/main.rs:461`).
- **Hit regions:** the Gallery collects them only for `activate()`=Some.

## Expected behavior
1. **Additive API:**
   - `SidebarBuilder::on_select(fn(&str) -> M)`.
   - `SidebarItem::value(impl Into<String>)`, which defaults to the label.
   - `SidebarItem::disabled(bool)`.
   - Values must be globally unique and must not contain `:` (documented, not enforced).
2. **No handler:** the tree is byte-identical to master, and the 4 existing tests pass unmodified.
3. **With a handler:**
   - Column `{id}`.
   - Each item is a Button `{id}:item:{value}` with `full_width(true)`.
   - Enabled items bind `on_select(value)`.
   - Disabled items have no message and no hit region.
   - Header, separators and footer are unchanged.
4. **Current item (handler mode):** `ButtonVariant::Primary` plus a `▸ ` marker; others are Ghost with two-space padding. Theme tokens only.
5. **Collapsed (handler mode):** shows `▸` plus the icon, or the icon alone. With no icon it shows the label's first `char` (no byte slicing). Keys do not depend on `state`.
6. **Additive `navigate(&self, forward) -> Option<M>`:**
   - Starts from the first `active` item, moves to the next enabled item, and wraps.
   - With no current item it returns the first or last enabled item.
   - Returns `None` with no handler or no enabled item.
7. **Gallery → Layout, a "Sidebar — typed activation" section appended last:**
   - Key `gallery_layout_sidebar`; four items, one disabled; status labels.
   - F7 enters and reveals the section, and Down/Up call `navigate` (selection follows focus).
   - Enter/Space confirm, Left/Right collapse and expand (state owned by the Gallery), and Esc leaves.
   - Blur, a category change, or a pointer press off the items deactivate the group.
   - The preedit guard applies, and every change calls `reset_hover_press()`.
8. **Page keyboard owner (category 3):** entering one group with F6 or F7 deactivates the other, through a single owner in the Gallery.
   - The sidebar hook runs before the nav-menu hook.
   - On F6 it deactivates itself and returns None.
   - While inactive it returns None for every key, and it always returns None for pointer and FocusChanged events.

## 056 review fixes folded in (Fable 5.2, confirmed by real-window GUI evidence `docs/team/roadmap-056/GUI.md`)
- **R1. Reveal only when needed:** reveal only when the group is out of view; never on the Esc that leaves, and never on a pointer click of an already-visible menu. This applies to both the NavigationMenu and Sidebar hosts (`apps/acme-gallery/src/navigation_menu.rs:139,218-219`).
- **R2. Preedit guard:** it returns None for keys the host does not handle (`navigation_menu.rs:173-175`), and the same applies to the Sidebar.
- **R3. Uniqueness doc:** the NavigationMenu doc says values are globally unique (`crates/acme-ui/src/layout/navigation_menu.rs:116-118`).

## Write set
- **Component (Opus):** `crates/acme-ui/src/layout/sidebar.rs`, plus the R3 doc-comment-only edit in `crates/acme-ui/src/layout/navigation_menu.rs`.
- **Gallery (Sonnet):** `apps/acme-gallery/src/sidebar.rs` (new), `apps/acme-gallery/src/sidebar_tests.rs` (new), `apps/acme-gallery/src/main.rs`, `apps/acme-gallery/src/navigation_menu.rs` (R1/R2 and owner integration), `apps/acme-gallery/src/navigation_menu_tests.rs` (only if R1 changes an asserted scroll value; justify each change).
- **Lead:** `docs/team/roadmap-057/*`, `docs/team/roadmap-056/GUI.md` and `gui/`, `docs/status/{todos,final}.md`, `docs/agent-team/ROADMAP.md`.

## Acceptance tests
**Component:**
- `sidebar_without_handler_matches_legacy_tree_expanded_collapsed_header_footer`
- `sidebar_on_select_builds_instance_keyed_buttons_bound_to_values`
- `sidebar_disabled_item_never_activates`
- `sidebar_current_item_is_primary_and_marked_expanded_and_collapsed`
- `sidebar_navigate_wraps_and_skips_disabled`
- `sidebar_navigate_without_current_returns_first_or_last_enabled`
- `sidebar_navigate_none_without_handler_or_enabled_item`
- `sidebar_item_keys_stable_across_collapse_and_distinct_per_instance`

**Gallery:**
- `gallery_sidebar_pointer_selects_each_enabled_item_with_real_events`
- `gallery_sidebar_disabled_item_has_no_hit_region`
- `gallery_sidebar_f7_reveals_group_into_view`
- `gallery_sidebar_up_down_wrap_skip_disabled_and_enter_space_confirm`
- `gallery_sidebar_left_right_collapse_expand_keep_selection_and_node_ids`
- `gallery_sidebar_pointer_press_off_group_deactivates`
- `gallery_sidebar_keys_ignored_off_page_and_after_escape_or_blur`
- `gallery_sidebar_keys_ignored_during_preedit`
- `gallery_sidebar_f7_and_navigation_menu_f6_are_mutually_exclusive`
- `gallery_sidebar_hit_and_paint_slots_match_across_themes_scales_and_states`

**R1/R2:**
- `gallery_navigation_menu_reveal_only_when_out_of_view_and_not_on_leaving_escape`
- `gallery_navigation_menu_pointer_click_on_visible_menu_does_not_scroll`
- `gallery_navigation_menu_preedit_passes_unhandled_keys_through`

**Must stay green, unmodified:** `anchor_tests`, `bottom_navigation_tests`, `page_header_tests`, `radio_cards_tests`, `mobile_list_item_tests`, `swipe_actions_tests`, `retained_tests`.

## Non-goals
- Sidenav or NavigationView; groups and nesting; Home/End and typeahead; Tab traversal; animation.
- `Role::Navigation`/`aria-current` export; enforcing uniqueness.
- The legacy collapsed quirks without a handler.
- The Gallery category-nav contrast (that is 060); manifests, lockfile, Scene and platform.

## Risks
- **Breaking change for external callers:** the new pub fields on `SidebarItem` break external struct literals. There are no in-repo literals.
- **Name shadowing:** in the Gallery, `use super::*` brings in `acme_widgets::sidebar` and `Gallery::sidebar()`, so import the component by path (`acme_ui::layout::sidebar`).
- **Layout shift:** the in-flow NavigationMenu dropdown shifts the Sidebar's Y position, so tests must locate targets through hit regions.
- **Gallery tests:** the 760 px viewport tests must use hit regions after the reveal.
