# ROADMAP 058 — Real-window GUI verification

- **Tested source:** `3a1814c` plus the uncommitted 058 diff. Debug build of `acme-ui-gallery`, launched as a copy of that exact binary.
- **Environment:** Windows 11 Pro, 1920×1080 at 150% scaling, NumLock on, unlocked console session.
- **Operator:** Claude Opus 5.5 (Lead), using `skills/human-acceptance/scripts/human_input.py` (real pointer glides and clicks, wheel, scancode keys with the extended-key flag, window screenshots).

## Results

| # | Input | Expected | Observed | Screenshot |
|---|---|---|---|---|
| 1 | Mouse click "Inputs" | Inputs page | PASS | `gui/01-inputs.png` |
| 2 | F7 | CheckboxCards revealed (heading visible); focus ▸ 電子郵件; "Keyboard group: active" | PASS | `gui/02-f7-checkbox.png` |
| 3 | Space | 電子郵件 checked (Primary, ☑) | PASS | `gui/03-space-email.png` |
| 4 | ↓ ↓ | Focus moves to 推播通知; nothing toggles | PASS | `gui/04-down-down-no-toggle.png` |
| 5 | Space, Enter | 推播通知 checked; Enter changes nothing. "已勾選：電子郵件、推播通知" | PASS | `gui/05-space-push-enter-noop.png` |
| 6 | ↑ ↑ ↑ | Focus 簡訊 → 電子郵件 → wraps to 每週摘要 | PASS | `gui/06-up-wraps.png` |
| 7 | Esc | Group inactive | PASS | `gui/07-escape.png` |
| 8 | F7, F6 | RadioCards takes the keyboard and is revealed with its heading | PASS | `gui/08-f6-radio-takes-over.png` |
| 9 | F7 | Back to CheckboxCards; previous focus kept | PASS | `gui/09-f7-back-to-checkbox.png` |
| 10 | Mouse click 簡訊 | 簡訊 toggles on; focus follows the click; group active | PASS | `gui/10-click-sms.png` |
| 11 | F6, ↓, Space | RadioCards selects and confirms 入門方案; CheckboxCards loses its ▸ marker and no checkbox changes | PASS | `gui/11-radio-down-checkbox-inactive.png` |

## Re-check after review fixes (rebuilt binary)
| # | Input | Expected | Observed | Screenshot |
|---|---|---|---|---|
| 12 | F7, then Space held (1 press + 15 OS-style auto-repeats) | Exactly one toggle: 電子郵件 checked (16 toggles would leave it unchecked) | PASS | `gui/20-held-space-single-toggle.png` |
| 13 | Mouse held still on 簡訊, click, click | Each click flips: off, then on | PASS | `gui/22-still-click-1-off.png`, `gui/23-still-click-2-on.png` |
| 14 | Wheel the group out of view, Space | Focused 簡訊 toggles off **and** is scrolled into view | PASS | `gui/25-space-reveals-focus.png` |

## Findings
1. **Focus is hard to see on a checked card** (P2). The focused card is `Elevated`, but at the default theme the elevation shadow is barely visible; the only clear cue is the small `▸` inside a Primary button (steps 5, 10). Follow-up: a stronger focus ring token, tracked with the 056 P2 #8 focus-visual item in 059.
2. **☑ renders as a colour emoji-style glyph** from the font fallback, while ☐ is monochrome (steps 3–6). Readable, but inconsistent. Not a regression; noted for a future icon pass.
3. **Selected Gallery category contrast** ("Inputs") is still nearly unreadable — ROADMAP 060.

## NOT_RUN
Windows 注音 IME composition (preedit guard covered by unit tests only), screen reader, touch, other DPI/monitors, dark theme.
