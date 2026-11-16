# ROADMAP 059 — Layout keyboard hosts polish (NavigationMenu + Sidebar)

- **Base:** master `4b9e7ad` (includes 056–058).
- **Severity:** P2 bundle — the remaining 056 review P2s, 057 GUI findings 1 and 3, and 058 GUI finding 1.
- **Execution mode:** Claude Code agent team. Fable 5.2 adviser (this contract) and independent reviewer; Opus 5.5 component; Sonnet 5.5 Gallery (single writer of `main.rs`); Lead runs gates and real-window GUI.
- **No new public API.**

## Decisions
| Item | Decision |
|---|---|
| A Snackbar press | IN: a press over the Snackbar deactivates the Layout hosts |
| B Modifier chords and auto-repeat | IN: mirror 058 |
| C Child of a disabled parent | IN: guard plus a demo entry to make it reachable |
| D Missing tests | IN: pin tests, each with a mutation check |
| E Keyboard-ownership visual | IN: Gallery-only frame (E1), plus CheckboxCards focused variant swap (E2) |
| F zh-TW status labels | IN: all four hosts |
| G F6 reveal margin | IN: 80, same as Sidebar |
| H Child reveal | IN: minimal scroll that keeps the parent row visible |
| I Dropdown-close scroll clamp | OUT: stays a known limitation (needs an overlay dropdown) |

## Behaviour
- **A.** In the existing `overlay_pointer && PointerButton{pressed:true}` block in `main.rs`, call a new `layout_keyboard_release()` (in `sidebar.rs`). It sets `nav_menu.active` and `side_nav.active` to false and changes nothing else.
- **B.** Every `Key` arm in `nav_menu_event` and `sidebar_event` (including the Sidebar's F6 arm) matches only when `ctrl`, `alt` and `meta` are all false; otherwise it returns `None` and changes nothing. Shift is allowed. `repeat: true` returns `Some(false)` with no state change for Enter/Space in both hosts, and for Esc in the NavigationMenu. Arrow repeat still roves.
- **C.** Append `("priority", "優先支援", Some("support"), false)` to `ENTRIES` as index 7 (indices 0–6 stay stable). `nav_menu_select` returns false when the parent entry is disabled.
- **E1.** Wrap each component in an always-present Card, `gallery_nav_menu_focus` and `gallery_layout_sidebar_focus`. Padding and border are constant; the variant is `Interactive` while that host is active and `Plain` otherwise. The Gallery painter already maps `Interactive` to the theme accent border. The wrapper is never conditional, so IDs and layout don't shift.
- **E2.** `crates/acme-ui/src/inputs/checkbox_cards.rs`: the focused card uses `CardVariant::Interactive` instead of `Elevated`. Do not use `foundations::focus_ring`, which bakes in a light-theme colour.
- **F.** NavigationMenu shows `鍵盤選單：使用中` / `鍵盤選單：未使用`. Sidebar, RadioCards and CheckboxCards show `鍵盤群組：使用中` / `鍵盤群組：未使用`.
- **G.** `GROUP_REVEAL_MARGIN = 80.0` in `navigation_menu.rs`, used for every top-level item reveal (still only when hidden).
- **H.** With a focused child that has a hit region:
  - Child hidden below: scroll down by `child_bottom − viewport_bottom + 8`, clamped.
  - Child hidden above: `reveal_at(parent, GROUP_REVEAL_MARGIN)`.
  - Child still hidden after that (short viewport): minimal reveal of the child; the child wins.
  - Otherwise: the G rule.
  - The leaving Esc and pointer clicks never scroll.

## Write sets (disjoint)
- **Component (Opus):** `crates/acme-ui/src/inputs/checkbox_cards.rs` (E2 and its test).
- **Gallery (Sonnet):** `apps/acme-gallery/src/navigation_menu.rs`, `sidebar.rs` and `main.rs` (the A call only); append-only to `navigation_menu_tests.rs` and `sidebar_tests.rs`; `radio_cards.rs` and `checkbox_cards.rs` (the label strings only).
- **Lead:** `docs/team/roadmap-059/*`, status docs.

## Acceptance tests
**Component (RED on master):** `checkbox_cards_focused_card_uses_interactive_variant_and_marker` replaces `checkbox_cards_focused_is_elevated_and_marked`.

**Gallery (RED on master):**
- `gallery_layout_hosts_snackbar_press_deactivates_menu_and_sidebar`
- `gallery_navigation_menu_ignores_ctrl_alt_meta_chords`
- `gallery_sidebar_ignores_ctrl_alt_meta_chords`
- `gallery_navigation_menu_enter_space_escape_ignore_os_auto_repeat`
- `gallery_sidebar_enter_space_ignore_os_auto_repeat`
- `gallery_navigation_menu_rejects_child_of_disabled_parent`
- `gallery_layout_keyboard_owner_frame_is_interactive_only_while_active`
- `gallery_keyboard_status_labels_are_zh_tw`
- `gallery_navigation_menu_f6_reveal_keeps_section_heading_visible`
- `gallery_navigation_menu_child_reveal_is_minimal_and_keeps_parent_visible`
- `gallery_navigation_menu_child_hidden_above_reveals_parent_row`

**Pins (pass today; mutation check each):**
- `gallery_navigation_menu_arrow_up_without_child_enters_last_enabled_child`
- `gallery_navigation_menu_unhandled_key_passes_through_and_keeps_menu_active`
- `gallery_navigation_menu_category_change_by_real_click_deactivates`

## Frozen
- Existing test bodies in `navigation_menu_tests.rs` and `sidebar_tests.rs`.
- `radio_cards_tests.rs`, `checkbox_cards_tests.rs`, `snackbar_tests.rs`, and the inline tests in `main.rs`.
- `crates/acme-ui/src/layout/{navigation_menu,sidebar}.rs`.

## Non-goals
Overlay dropdown (I); Snackbar deactivation and focus frames for the Inputs hosts; left-aligned labels (063); category contrast (060); the ☑ glyph; fixing `focus_ring.rs`.

## Risks
- Margin 80 can still clip the title in a narrow window where the instructions wrap.
- E2 changes what other consumers of a focused CheckboxCards see.
- C adds a disabled ` ▾` item to the demo.
- H uses last-frame hit regions.
