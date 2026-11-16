# ROADMAP 062 — Real-window GUI verification

- **Tested builds:** `acme-ui-gallery` debug builds.
  - Build A was taken after the first token pass.
  - Build B was taken after the review fixes (button painter, state tokens, chart series). Build B matches the committed tree.
- **Environment:** Windows 11 Pro, 1920×1080 at 150% scaling.
- **Operator:** Claude Opus 5.5 (Lead), using `skills/human-acceptance/scripts/human_input.py` with the foreground guard. No guard aborts occurred.

| # | Build | Theme / page | Expected | Observed | Screenshot |
|---|---|---|---|---|---|
| 1 | A | solarized light, Layout | Secondary buttons readable (060 finding 1) | **FAIL**: dark text on teal, still unreadable. Root cause: `acme-widgets` `Button::resolve_style` painted Secondary text with `foreground`, not `secondary_foreground`, and hardcoded white for Danger. The token gate passed because it measured a pair the button never paints. | `gui/01-solarized-light-layout.png` |
| 2 | A | solarized light, Charts | Bars distinct and ≥ 3:1; line dots visible | PASS | `gui/02-solarized-light-charts.png` |
| 3 | B | solarized light, Layout | Secondary buttons readable; toolbar shows "solarized ✓AA" | PASS (cream text on teal) | `gui/03-solarized-light-layout-fixed.png` |
| 4 | B | solarized light, hovered Secondary | Hover fill with readable text | PASS (dark text on the light hover fill) | `gui/04-solarized-light-secondary-hover.png` |
| 5 | B | solarized dark, Inputs and Design System | Body, muted and selected text readable | PASS | `gui/05-solarized-dark-inputs.png`, `gui/06-solarized-dark-design-system.png` |
| 6 | B | solarized dark, Semantic Colors | Primary, Secondary, Ghost and Danger labels readable; status dots distinct | PASS (dark ink on blue and red) | `gui/08-solarized-dark-semantic-buttons.png` |

## Findings (follow-ups)
- The Design System "w_full + h_full" demo box paints light text on `primary`. In solarized dark the primary ink is now dark, so the demo shows low contrast. The Gallery demo should use `primary_foreground`.

## NOT_RUN
- Real-window checks of dracula, nord, gruvbox, apple, ubuntu and one-dark in this pass; they are covered by the every-pack token, painted-button and chart tests.
- Hover and press on Primary and Danger in a real window.
- Screen reader.
