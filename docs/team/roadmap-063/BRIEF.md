# ROADMAP 063 — Left-aligned Sidebar / Sidenav item labels

- **Base:** master `c53bab5` (includes 062).
- **Severity:** P2 (UX, user request: 「Sidenav 左對齊」).
- **Execution mode:** Claude Code agent team.
  - Opus 5.5: component and renderer helper.
  - Sonnet 5.5: Gallery painters.
  - Fable 5.2: independent reviewer.
  - Opus 5.5 Lead: gates and real-window GUI.

## OBSERVED (master)
- Every button painter centres the label and ignores `style.text_align`:
  - `apps/acme-gallery/src/main.rs:2702` (toolbar), `:3143` and `:3487`;
  - `apps/gallery/src/render/content.rs:108` and `frame.rs:199`;
  - `apps/playground/src/main.rs:874`;
  - `crates/acme-widgets/src/painter_bridge.rs`.
- `Button` already implements `Styled`, so `.text_align(TextAlign::Left)` compiles but has no visible effect.
- Navigation lists centre their labels: Sidebar and Sidenav items in handler mode (`{id}:item:{value}`), and the acme-gallery shell category list (060 painted `▸` in a fixed left gutter, far from the centred label).

## Expected behavior
1. **Additive API (`acme-widgets`):**
   - `Button::label_x(&self, rect_x: f32, rect_width: f32, text_width: f32) -> f32` returns the label's left x.
   - When `style.text_align` is unset or `Center`, the label is centred, exactly as today.
   - With `Left`, it returns `rect_x + pad`, where `pad` is `style.padding` left if set, else 12.0 (document the constant).
   - With `Right`, it mirrors `Left`.
   - Text wider than the rect is clamped to `rect_x + pad` for Left and Right, and keeps today's `max(0)` centring for Center.
   - No other public API is added.
2. **All painters** listed above use `label_x`. Unaligned buttons must render byte-identical to master, proved by a Scene test in each shell.
3. **Sidebar** (`crates/acme-ui/src/layout/sidebar.rs`) and **Sidenav** (`crates/acme-ui/src/desktop/sidenav.rs`), handler mode only:
   - Expanded items set `TextAlign::Left`.
   - Collapsed Sidebar items stay centred (icon or glyph only).
   - Without a handler, the tree is unchanged.
   - With `▸ ` / two-space prefixes, label text starts in the same column whether or not the item is selected; check this with the measured glyph width. If `▸ ` and two spaces differ in width, document it and assert a ≤ 2px difference.
4. **acme-gallery shell categories:**
   - The label is left-aligned just after the `▸` gutter.
   - The label x is the same selected or unselected (keep the 060 test).
   - Hit rects are unchanged.

## Write set
- **Component (Opus):**
  - `crates/acme-widgets/src/inputs/button.rs`;
  - `crates/acme-widgets/src/painter_bridge.rs`;
  - `crates/acme-widgets/src/lib.rs` (tests);
  - `crates/acme-ui/src/layout/sidebar.rs` and `crates/acme-ui/src/desktop/sidenav.rs`.
- **Gallery (Sonnet):**
  - `apps/acme-gallery/src/main.rs` (single writer) and the related tests;
  - `apps/gallery/src/render/{content,frame}.rs` and tests;
  - `apps/playground/src/main.rs`.
- **Lead:** `docs/team/roadmap-063/*` and the status docs.

## Acceptance tests
- **Component:**
  - `button_label_x_centers_when_unset_or_center`
  - `button_label_x_left_and_right_use_padding_and_clamp`
  - `sidebar_and_sidenav_handler_items_are_left_aligned_collapsed_centered`
  - `sidebar_and_sidenav_without_handler_unchanged`
- **Gallery:**
  - `gallery_unaligned_buttons_paint_identically` (compare to the centred formula across pages)
  - `gallery_sidebar_and_sidenav_labels_start_at_left_padding`
  - `gallery_shell_category_labels_left_aligned_after_marker`
  - The `apps/gallery` shell equivalent.

## Non-goals
- RTL.
- Vertical alignment.
- Changing other widgets' alignment.
