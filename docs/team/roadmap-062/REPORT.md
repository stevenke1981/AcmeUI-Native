# ROADMAP 062 — Report: WCAG AA gate across all theme packs

## Execution (actual)
| Role | Model | Work |
|---|---|---|
| Lead / brief / gates / GUI | Claude Opus 5.5 | Audit, brief, scope decisions, gates, real-window GUI (found the button-painter root cause), docs, PR |
| Theme | Claude Opus 5.5 (subagents ×2) | First token pass (59 tokens); review pass (state tokens) — 104 tokens in total |
| Charts and Gallery tests | Claude Sonnet 5.5 (subagents ×2) | Chart marks off `accent`; distinct series palette; Gallery ≥ 4.5 and toolbar ✓AA tests |
| Button painter | Claude Sonnet 5.5 | `acme-widgets` `Button::resolve_style` text follows the painted fill |
| Independent reviewer | Claude Fable 5.2 | Read-only review: 1 P0, 4 P1, about 10 P2 |

## Problem
- `meets_wcag_aa()` was false in 15 of 24 pack/mode combinations. Before: `logs/wcag-audit-before.txt`.
- Rings were below 3:1 in dracula light and gruvbox dark.
- Charts painted the `accent` tint as a graphic.
- In the real window, solarized light Secondary buttons were unreadable.

## Change
- **Theme gate (`crates/acme-theme`).** These hold for every pack × mode, with no exemptions:
  - **Text:** all 7 `wcag_report` rows ≥ 4.5; `foreground` ≥ 4.5 on `surface`, `ghost_hover` and `ghost_pressed`; `muted_foreground` ≥ 4.5 on `background`; `primary_foreground` ≥ 4.5 on `primary`, `primary_hover` and `primary_pressed`.
  - **Danger fill:** in all three states, at least one existing ink reaches ≥ 4.5 against it.
  - **Graphics:** `ring` ≥ 3:1 vs `background` and `surface`; `primary` ≥ 3:1 vs `background`; `success`, `warning`, `danger` and `info` ≥ 3:1 vs `surface` and `background`.
  - **State feedback:** for the packs 062 retuned, hover and pressed each differ from rest by ≥ 1.15:1 and move away from the ink. The untouched packs that miss this are listed exactly in the test.
- **Retune.** 104 token values across apple, ubuntu, nord, dracula, solarized, gruvbox and one-dark, plus a `primary_pressed` step in default dark, material dark and windows11 dark.
  - Every change keeps the hue and uses the smallest lightness step that passes, or a colour from the pack's own palette.
  - Each change is recorded old → new with its ratios in `ROADMAP_062_CHANGES`, which a test checks, and in `logs/token-changes.md`.
- **Button painter (`crates/acme-widgets/src/inputs/button.rs`).** The text colour now follows the fill actually painted:
  - Secondary at rest uses `secondary_foreground`.
  - Hovered and pressed states put `foreground` on the ghost fills.
  - Danger picks the first token that reaches 4.5:1, so the hardcoded white is gone (AGENTS.md).
  - A painted-pair test covers every pack × mode × variant × state.
- **Charts (`crates/acme-ui/src/charts`).**
  - Marks use `primary` instead of `accent`.
  - The bar palette is primary, success, info, warning, danger, and is distinct at the wrap point.
  - Tests check the painted quads (≥ 3:1).
- **Gallery.** The 060 solarized floors are replaced by ≥ 4.5 for every pack, and a new test checks that the toolbar shows ✓AA for every pack × mode.

## Verification
Base `ffd7e51`, tested on the final uncommitted tree; CPU profile on the commit.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo test --locked -p acme-ui --all-features` | PASS (931 passed, 0 failed) |
| `cargo test --locked -p acme-theme -p acme-widgets -p acme-ui-gallery -p acme-gallery` | PASS (474 passed, 0 failed, 1 ignored) |
| clippy (theme, widgets, ui all-features, both galleries) `--all-targets -D warnings` | PASS |
| `cargo check --locked --workspace --all-targets` | PASS |
| `cargo test --locked --workspace` | PASS (2190 passed, 0 failed, 11 ignored) |

Real-window GUI: [GUI.md](GUI.md). The first build failed the solarized Secondary check, which led to the button-painter fix. The final build passes.

## Independent review (Fable 5.2)
- **P0, Secondary text painted with the wrong token:** fixed in the button painter and verified in the real window.
- **P1-1, Danger white text:** picked by contrast now.
- **P1-2, pressed-text regressions:** gated and fixed.
- **P1-3, lost hover and pressed feedback:** gated and fixed for retuned packs; untouched packs are listed.
- **P1-4, chart series collisions:** palette changed and pairwise distinctness tested, including the wrap.
- **P2 items:**
  - **Fixed:** gruvbox teals unified; one-dark caption contrast; solarized muted hierarchy; test renamed; chart marks moved to `primary`; RED evidence re-established.
  - **Accepted:** some identity shifts in dracula light and nord light status colours, documented in `token-changes.md`.

## Follow-ups
- **Soft badges:** status text on `*_soft` fills stays below 4.5 in several light packs. It improved but still needs a gate.
- **Hover/pressed feedback in untouched packs:** the six listed pack/modes miss the hover and pressed step rule (1.09–1.14).
- **primary and info:** identical in 9 pack/modes; not adjacent in the bar palette.
- **Design System demo box:** "w_full + h_full" paints `foreground` on `primary`.

## Rollback
Revert the commit. Token values only; no API, manifest or lockfile change.
