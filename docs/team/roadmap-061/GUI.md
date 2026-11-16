# ROADMAP 061 — Real-window GUI verification

- **Tested build:** `acme-ui-gallery` debug build of the 061 tree before the review fixes. The fixes change only test-covered edges: F8 repeat, the short-viewport fallback, and the feature manifest.
- **Environment:** Windows 11 Pro, 1920×1080 at 150% scaling, gruvbox dark (persisted).
- **Operator:** Claude Opus 5.5 (Lead), using `skills/human-acceptance/scripts/human_input.py` with the foreground guard. No guard aborts.

| # | Step | Expected | Observed | Screenshot |
|---|---|---|---|---|
| 1 | Layout, F8 | Scrolls to "Sidenav — typed sections", ring frame, 鍵盤導覽：使用中 | PASS | `gui/01-f8-enter.png` |
| 2 | ↓ (Appearance → Privacy) | Crosses the section boundary | PASS | `gui/02-down-crosses-section.png` |
| 3 | ↓ from Privacy | Skips the disabled Security item and wraps to Profile | PASS | `gui/03-down-skips-disabled-wraps.png` |
| 4 | ↑ then Enter | Privacy selected; 已確認：Privacy | PASS | `gui/04-up-enter-confirm.png` |
| 5 | Esc, then ↓ | Group 未使用; ↓ ignored | PASS | `gui/05-escape-leaves.png`, `gui/06-down-after-escape-ignored.png` |
| 6 | Click Appearance | Selected | PASS | `gui/07-pointer-click-appearance.png` |
| 7 | Cold start, then F8, ↓ | Works on a fresh launch | PASS | `gui/08-cold-start-f8-down.png` |
| 8 | Click disabled Security | No change; group released | PASS | `gui/09-click-disabled-security.png` |
| 9 | F8 then F7 | Sidebar takes over; Sidenav frame off | PASS | `gui/10-f7-takes-over.png` |
| 10 | F8, ← | Sidebar not collapsed (← is not Sidenav's key) | PASS | `gui/11-f8-left-ignored.png` |

## Observation (not reproduced)
On the very first run, ↓ ×3 after F8 had no visible effect (`gui/00-first-run-down-no-effect-not-reproduced.png`). The workspace gates were compiling in the background at the time. The same sequence, replayed twice and from a cold start, worked. It is recorded as not reproduced, not as a pass.

## NOT_RUN
Light theme and high-contrast, 100% scaling, held keys, Alt+Tab blur, screen reader.
