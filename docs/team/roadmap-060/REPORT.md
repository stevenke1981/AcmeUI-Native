# ROADMAP 060 — Report: Readable selected category (Gallery shells)

## Execution (actual)
| Role | Model | Work |
|---|---|---|
| Lead / gates / real-window GUI | Claude Opus 5.5 | Branch, brief, gate runs, GUI verification, docs, PR |
| Adviser + independent reviewer | Claude Fable 5.2 | Decisions D1–D5; read-only review (1 P1, 5 P2) |
| Theme | Claude Opus 5.5 (subagents ×2) | `acme-theme`: `selected_item_colors`, wcag row, retunes, review P1 |
| Gallery shells | Claude Sonnet 5.5 (subagents ×2) | `apps/acme-gallery`, `apps/gallery`: painting, marker, tests, review P2s |

## Problem
The selected category was painted `accent` behind `primary_foreground`. That pair is 1.15:1 on default light, and master painted as low as 1.09:1 (solarized light). It failed in 17 of 24 pack/mode combinations (`logs/gallery-red.txt`).

## Change
- **`acme-theme` (additive):**
  - New `Theme::selected_item_colors() -> (bg, fg)`. `bg` is `accent`; `fg` is the first of `accent_foreground`, `foreground`, `primary_foreground` that reaches 4.5:1 against it, otherwise the one with the highest ratio.
  - `wcag_report` gains a 7th row, `selected_fg/selected_bg`. `meets_wcag_aa()` gives the same result as before for every pack.
- **Retunes (minimum tokens):**
  - solarized light `accent_foreground` `#6C71C4` → `#5F63AC`.
  - gruvbox light `accent` `#D65D0E` → `#AF3A03` (gruvbox faded orange).
  - gruvbox dark `accent` → `#FE8019` and `accent_foreground` → `#282828` (gruvbox bright orange; review P1 option B, so the accent also works as a graphic: 5.84:1 vs background, 4.59:1 vs surface).
- **Both Gallery shells:**
  - The selected category is painted from `selected_item_colors()`.
  - A `▸` marker is painted in a fixed left gutter, so the label never moves and hit rects are unchanged.

## Results
Selected-category contrast is ≥ 4.5:1 in all 24 pack/mode combinations. The lowest are windows11 light (4.55) and solarized light (4.61; the D1 rule on the old tokens gave 3.78, while master actually painted 1.09).

## Tests
- **Theme:** `selected_item_colors_meet_aa_in_every_pack_and_mode`, `..._use_theme_tokens_only`, `..._prefer_first_qualifying_candidate`, `wcag_report_includes_selected_item_pair`, `roadmap_060_pack_retune_reaches_aa`, `gruvbox_accent_meets_non_text_contrast`, `accent_vs_background_non_text_guard` (keeps a self-checking list of the other packs below 3:1, for 062).
- **Gallery:** the shell tests read the painted quad and text colours from the Scene, per pack and mode. They cover the selected text, unselected and hovered text, the marker and label positions, hit-rect stability, and the longest label fitting inside its button. They were RED against master's paint code.
- **Solarized unselected text:** this pair is pinned at its current floor (light ≥ 3.6, dark ≥ 4.1) instead of being exempted.

## Verification
Tested source: HEAD `a14c243` plus the uncommitted 060 diff, Windows 11, rustc/cargo 1.88.0. CPU profile on the commit is in `cpu-profile-*`.

| Command | Result |
|---|---|
| `cargo fmt -p acme-theme -p acme-ui-gallery -p acme-gallery -- --check` | PASS |
| `cargo test --locked -p acme-theme` | PASS (38 passed) |
| `cargo test --locked -p acme-ui-gallery -p acme-gallery` | PASS (215 passed, 0 failed, 1 ignored) |
| `cargo clippy --locked -p acme-theme -p acme-ui-gallery -p acme-gallery --all-targets --no-deps -- -D warnings` | PASS |
| `cargo check --locked --workspace --all-targets` | PASS |
| `cargo test --locked --workspace` | PASS (2150 passed, 0 failed, 11 ignored) |

Real-window GUI: [GUI.md](GUI.md).

## Independent review (Fable 5.2)
- **P1, gruvbox dark accent too dark as a graphic:** fixed with option B and verified in the real window on the Charts page.
- **P2, label jump:** fixed with the fixed-gutter marker.
- **P2, marker test:** strengthened with ink extents inside the rect.
- **P2, solarized exemption:** narrowed to pinned floors.
- **P2, stale docs:** the todos "6 key pairs" line is updated.
- **P2, evidence:** gates re-run on the final tree; CPU profile recorded on the commit.

## Follow-ups (062)
- solarized light `secondary` button text is nearly invisible.
- solarized unselected text is 3.64 / 4.11.
- `accent` is below 3:1 vs background in 19 of 24 pack/mode combinations, but charts use it as a graphic.
- `ring` is below 3:1 in dracula light and gruvbox dark (from 059).

## Rollback
Revert the commit. No manifest or lockfile change.
