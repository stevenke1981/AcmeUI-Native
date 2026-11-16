# ROADMAP 060 — Real-window GUI verification

- **Tested source:** `a14c243` plus the uncommitted 060 diff. Debug builds of `acme-ui-gallery`, launched as copies of those exact binaries: the first before the review fixes, the second after them.
- **Environment:** Windows 11 Pro, 1920×1080 at 150% scaling.
- **Operator:** Claude Opus 5.5 (Lead), using `skills/human-acceptance/scripts/human_input.py`, which has the 059 foreground guard. No guard aborts occurred.

## Results

| # | Theme | Expected | Observed | Screenshot |
|---|---|---|---|---|
| 1 | default light | Selected "▸ Layout" readable (dark blue on pale blue, 5.85:1) | PASS (before this, white on pale blue, 1.15:1) | `gui/01-current-theme-layout.png` |
| 2 | default dark | Selected readable (light blue on navy, 7.00:1) | PASS | `gui/02-default-dark.png` |
| 3 | solarized dark | Selected readable | PASS | `gui/03-solarized-dark.png` |
| 4 | gruvbox dark (first build, `#AF3A03`) | Selected readable | Text readable; review P1 found the accent too dark for graphics (2.41:1 vs background) | `gui/04-gruvbox-dark.png` |
| 5 | gruvbox light | Cream on faded orange `#AF3A03`, 5.40:1 | PASS | `gui/05-gruvbox-light.png` |
| 6 | solarized light | Violet on lavender, 4.61:1 | PASS for the selected item; see finding 1 | `gui/06-solarized-light.png` |
| 7 | gruvbox dark (final, `#FE8019` + `#282828`) | Selected "▸ Charts" dark on bright orange, 5.84:1; line-chart dots and bars visible on the dark background | PASS | `gui/07-gruvbox-dark-charts.png`, `gui/08-gruvbox-dark-charts-2.png` |
| 8 | solarized light (final) | `▸` in a fixed left gutter; label does not move when selected | PASS (seen on "Mobile") | not kept |

## Findings
1. **Solarized light secondary buttons are unreadable (P1, pre-existing, → 062).** Text on the teal `secondary` buttons (toolbar Dark/Info/pack, Anchor links, BottomNavigation, Mobile actions) is nearly invisible (step 6). 060 did not change `secondary`; this needs the 062 palette gate.
2. **Accent as a graphic.** `accent` is a pale tint in most packs and falls below 3:1 against the background in 19 of 24 pack/mode combinations, yet charts use it for dots, a bar and the gauge hub. → 062.
3. In acme-gallery the `▸` sits in the left gutter while the label stays centred, so they are visually far apart on wide buttons. This is acceptable for now; it is the cost of the no-jump rule.

## NOT_RUN
`apps/gallery` (the second shell) in a real window; it is covered by Scene tests only. Hover and click-edge checks, screen reader.
