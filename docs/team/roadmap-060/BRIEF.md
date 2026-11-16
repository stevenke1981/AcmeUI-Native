# ROADMAP 060 — Readable selected category (Gallery shells)

- **Base:** master `a14c243` (includes 056–059).
- **Severity:** P1 accessibility. The selected category in the Gallery's left nav is nearly unreadable. 056–059 real-window evidence shows white text on a very light blue fill.
- **Root cause:** the selected category is painted with `bg = colors.accent` and `fg = colors.primary_foreground`:
  - `apps/acme-gallery/src/main.rs` around line 2606.
  - `apps/gallery/src/render/frame.rs:101-110`.
  - Contrast is 1.15:1 on default light, and the pair fails in 16 of 24 pack×mode combinations.
- **Execution mode:** Claude Code agent team.
  - Fable 5.2: adviser (decisions D1–D5) and independent reviewer.
  - Opus 5.5: `acme-theme`.
  - Sonnet 5.5: both Gallery shells.
  - Opus 5.5 Lead: gates and real-window GUI.

## Decisions
- **D1: additive `Theme::selected_item_colors(&self) -> (ThemeColor, ThemeColor)`** returning `(bg, fg)`.
  - `bg = colors.accent`.
  - `fg` = the first of `[accent_foreground, foreground, primary_foreground]` whose contrast against `bg` is ≥ 4.5:1. If none qualifies, the candidate with the highest ratio.
  - Document it as the token pair for "selected item in a list or nav".
- **Retune** so every pack and mode reaches ≥ 4.5:1 through the D1 rule.
  - Expected: solarized light `accent_foreground`, and gruvbox `accent` in light and dark (≈(175,58,3), verify).
  - Change only the minimum needed tokens and record before/after ratios.
  - Do not change `primary`, `ring` or other tokens; those are 062.
- **D1b: `wcag_report` gains a 7th row**, `"selected_fg/selected_bg"`, computed from `selected_item_colors()`.
  - Update `wcag_report_covers_key_pairs` (6 → 7) and say why in the report.
  - Check whether `meets_wcag_aa()` changes outcome for any pack, and report which.
- **D2:** additive public API; no breaking change.
- **D3:** the all-pack Primary/accent AA gate is 062, not here.
- **D4: the selected category gets a `▸ ` prefix**, so selection isn't shown by colour alone (WCAG 1.4.1). It applies in both Gallery shells. Keep the hit rects unchanged, and only change the label text if that doesn't shift layout.
- **D5:** fix `apps/gallery` (the second shell) too.

## Write sets (disjoint)
- **Theme (Opus):** `crates/acme-theme/src/lib.rs` and `crates/acme-theme/src/packs/*.rs`.
- **Gallery (Sonnet):**
  - `apps/acme-gallery/src/main.rs` (single writer).
  - A new `apps/acme-gallery/src/shell_contrast_tests.rs`.
  - `apps/gallery/src/render/frame.rs`, plus that crate's tests.
  - Use `theme.selected_item_colors()`, never a literal colour.
- **Lead:** `docs/team/roadmap-060/*`, status docs.

## Acceptance tests
**Theme:**
- `selected_item_colors_meet_aa_in_every_pack_and_mode`
- `selected_item_colors_use_theme_tokens_only` (the returned pair equals some token pair)
- `wcag_report_includes_selected_item_pair`
- A pack-retune test listing before/after ratios.

**Gallery:**
- `gallery_selected_category_text_contrast_meets_aa_all_packs`: render the shell for each pack and mode; the painted quad colour behind the selected category label vs the painted text colour is ≥ 4.5:1.
- `gallery_unselected_and_hovered_category_text_meets_aa_all_packs`
- `gallery_selected_category_has_marker_and_stable_hit_rects`
- An `apps/gallery` equivalent of the selected-contrast test.

## Non-goals
Focus ring for categories; the Primary/accent/ring all-pack gate (062); other widgets that use accent fills.
