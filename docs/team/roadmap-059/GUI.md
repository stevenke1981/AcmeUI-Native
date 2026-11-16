# ROADMAP 059 — Real-window GUI verification

- **Tested source:** `4b9e7ad` plus the uncommitted 059 diff. Debug builds of `acme-ui-gallery`, launched as copies of those exact binaries (first build before the review fixes, second after).
- **Environment:** Windows 11 Pro, 1920×1080 at 150% scaling, NumLock on.
- **Operator:** Claude Opus 5.5 (Lead), using `skills/human-acceptance/scripts/human_input.py`.

## Results

| # | Input | Expected | Observed | Screenshot |
|---|---|---|---|---|
| 1 | Click "Layout", F6 | NavigationMenu revealed with its **heading and instructions visible** (G, margin 80); label `鍵盤選單：使用中` (F) | PASS | `gui/01-f6-heading-visible.png` |
| 2 | Ctrl+F7 | Ignored: the menu keeps the keyboard, no scroll (B) | PASS | `gui/02-ctrl-f7-ignored.png` |
| 3 | F7 | Sidebar takes the keyboard; owner frame drawn (E1); `鍵盤群組：使用中` | PASS (but see finding 1) | `gui/03-f7-sidebar-frame.png` |
| 4 | Menu row parked at the viewport bottom; F6, →, ↓ | Minimal scroll: child ▸ 桌面版 just visible, parent 產品 and section heading still visible (H) | PASS | `gui/04-menu-near-bottom.png`, `gui/05-child-minimal-reveal.png` |
| 5 | Dark theme; F7, ↓, Enter held (1 press + 15 auto-repeats) | 專案 confirmed once | PASS (weak: repeat confirm is idempotent; unit tests prove repeat is ignored) | `gui/06-dark-f7-frame.png`, `gui/07-dark-held-enter.png` |
| 6 | After the fix: F7 in dark, then light | Owner frame is a clearly visible 2px `ring` border in both themes | PASS | `gui/08-dark-ring-frame.png`, `gui/09-light-ring-frame.png` |
| 7 | After the fix: browse Design System, Foundations, Inputs | No stray ring borders on the ~20 other components that use `CardVariant::Interactive` | PASS (visual) | not kept |

## Findings
1. **Owner frame invisible (P1, fixed).** The first build drew the `Interactive` border with `colors.accent`: measured (232,240,254) on (250,251,252) in light, about 1.1:1 against WCAG 1.4.11's 3:1. The unit tests passed because they asserted the variant, not the painted colour. Fixed by drawing `Interactive` with `colors.ring` at 2px in both the Gallery painter and the shared `painter_bridge`, plus a test on the painted border colour and contrast. Re-check: step 6.
2. **Ring contrast fails in two packs** (P2, to 062): dracula light 2.41/2.26 and gruvbox dark 2.74 against surface/background.
3. **Tooling incident (driver, fixed in this PR).** During step 7's first attempt, another window (CrewDesk, not launched by the test) came to the foreground mid-run. The driver only checked the foreground at start, so two clicks and two wheel scrolls went to CrewDesk at screen coordinates (the `place` step correctly moved the Gallery window). Afterwards, CrewDesk's geometry was unchanged and nothing was typed. Those screenshots were discarded. The driver now checks the foreground before every step and checks the window under every pointer target, and aborts with "no input sent" otherwise. Both guards were exercised.

## NOT_RUN
Snackbar press in the real window (unit-tested), Alt+Space system menu, short-window (≤300px) resize, 注音 IME, screen reader, touch.
