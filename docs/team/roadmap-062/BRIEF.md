# ROADMAP 062 — WCAG AA gate across all theme packs

- **Base:** master `ffd7e51` (includes 061).
- **Severity:** P1. In the real window, solarized light secondary buttons are nearly unreadable (060 GUI, finding 1). `meets_wcag_aa()` is false for 14 of 24 pack/mode combinations.
- **Execution mode:** Claude Code agent team.
  - Opus 5.5: theme retune.
  - Sonnet 5.5: chart tokens and Gallery tests.
  - Fable 5.2: independent reviewer.
  - Opus 5.5 Lead: gates and real-window GUI.

## OBSERVED (master after 060, audit `logs/wcag-audit-before.txt`)
| Pack | Mode | Failing pairs (ratio) |
|---|---|---|
| default | light | muted_fg/muted 4.43 |
| apple | light | primary_fg/primary 4.02, muted_fg/muted 2.92, accent_fg/accent 3.41 |
| apple | dark | primary_fg/primary 3.65, accent_fg/accent 4.06 |
| ubuntu | light | primary_fg/primary 3.65 |
| ubuntu | dark | primary_fg/primary 3.65 |
| nord | light | primary_fg/primary 3.50, accent_fg/accent 2.98 |
| nord | dark | accent_fg/accent 4.31 |
| dracula | light | muted_fg/muted 3.93, accent_fg/accent 4.49, ring vs bg 2.26, ring vs surface 2.41, primary vs bg 2.26 |
| dracula | dark | muted_fg/muted 1.94, accent_fg/accent 3.84 |
| solarized | light | foreground/background 4.13, surface_fg/surface 3.64, primary_fg/primary 3.41, secondary_fg/secondary 2.93, muted_fg/muted 2.18 |
| solarized | dark | surface_fg/surface 4.11, primary_fg/primary 3.41, secondary_fg/secondary 4.11, muted_fg/muted 2.42, accent_fg/accent 4.12 |
| gruvbox | light | primary_fg/primary 3.73, muted_fg/muted 3.55 |
| gruvbox | dark | primary_fg/primary 3.73, muted_fg/muted 4.17, ring vs surface 2.74 |
| one-dark | light | primary_fg/primary 4.05, muted_fg/muted 2.26 |
| one-dark | dark | muted_fg/muted 2.55 |

`accent` vs background is below 3:1 in 19 of 24 combinations. `accent` is a soft tint by design, but charts paint it as a graphic: the gauge hub (`crates/acme-ui/src/charts/gauge.rs:116`), the fifth bar (`bar_chart.rs:118`), line-chart dots (`line_chart.rs:124`) and the sparkline end dot (`sparkline.rs:103`).

## Expected behavior
1. **Theme gate (acme-theme tests):** for every pack and mode:
   - `meets_wcag_aa()` is true (all 7 report rows ≥ 4.5);
   - `ring` reaches ≥ 3:1 vs `background` and vs `surface`;
   - `primary` reaches ≥ 3:1 vs `background` (graphic use);
   - `foreground` reaches ≥ 4.5:1 vs `surface`.

   Replace the 060 `KNOWN_BELOW_3` list and any other exemptions with these assertions. Keep the `accent_vs_background` guard only as documentation that accent is a tint, not a graphic token.
2. **Retune rules (minimal, brand-preserving):**
   - Prefer changing the foreground side of a pair (`*_foreground`, `muted_foreground`) before the fill.
   - Keep hue; move lightness (in HSL or OKLCH) by the smallest step that reaches **≥ 4.6** (≥ 3.1 for 3:1 rules), a 0.1 margin.
   - Prefer the pack's own palette colours when one passes; for example, solarized base01/base03 and nord polar night.
   - Don't touch passing packs (windows10, windows11, material, high-contrast, and default dark) unless a shared token forces it.
   - A test records each changed token old → new together with its before and after ratio.
3. **Charts:** non-text chart marks must not use `accent`. Use a token that the gate above guarantees at ≥ 3:1 vs background (`primary`, or `ring` where emphasis is meant). Add a chart test that, for every pack and mode, every chart mark colour reaches ≥ 3:1 vs the chart's own fill or background.
4. **Gallery tests:** the 060 solarized pinned floors (3.6 and 4.1) become a plain ≥ 4.5 assertion across every pack, and the Gallery toolbar AA badge reads ✓ for every pack (a test).

## Write set
- **Theme (Opus):** `crates/acme-theme/src/lib.rs` (tests) and `crates/acme-theme/src/packs/*.rs`.
- **Charts and Gallery (Sonnet):** `crates/acme-ui/src/charts/*.rs`, `apps/acme-gallery/src/shell_contrast_tests.rs`, `apps/gallery/src/shell_contrast_tests.rs`, and Gallery toolbar tests.
- **Lead:** `docs/team/roadmap-062/*` and the status docs.

## Non-goals
- New tokens or a new public API.
- Changing the accent tint semantics.
- Changing `ContrastLevel` thresholds.
