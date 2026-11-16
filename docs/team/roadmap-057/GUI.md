# ROADMAP 057 — Real-window GUI verification

- **Tested source:** `8735e43` plus the uncommitted 057 diff (4 tracked files, 581 insertions; new `apps/acme-gallery/src/sidebar.rs` and `sidebar_tests.rs`). Debug build of `acme-ui-gallery`, launched as a copy of that exact binary.
- **Environment:** Windows 11 Pro, 1920×1080 at 150% scaling (DPI 144), NumLock on, unlocked console session.
- **Operator:** Claude Opus 5.5 (Lead), using `skills/human-acceptance/scripts/human_input.py` for real pointer glides, clicks, wheel and scancode key presses, plus window screenshots.

## Results

| # | Input | Expected | Observed | Screenshot |
|---|---|---|---|---|
| 1 | Mouse click "Layout" | Layout page | PASS | `gui/01-layout.png` |
| 2 | F6 | NavigationMenu revealed, "Keyboard menu: active" | PASS | `gui/02-f6-nav.png` |
| 3 | → | 產品 selected and dropdown open; **no scroll jump** (R1) | PASS: menu row stays at y≈143 | `gui/03-right-no-jump.png` |
| 4 | F7 | Sidebar section revealed and takes the keyboard | PASS | `gui/04-f7-sidebar.png` |
| 5 | ↓ ↓ | 儀表板 → 專案 → (skip disabled 報表) → **▸ 設定** (Primary); status "目前項目：設定" | PASS | `gui/05-down-down.png` |
| 6 | ← | Collapsed: first-character glyphs 儀/專/報/▸設; current item kept; "狀態：收合" | PASS | `gui/06-left-collapse.png` |
| 7 | →, Enter | Expanded again; "已確認：設定" | PASS (confirmed by the step 9 frame) | `gui/07-right-enter.png`, `gui/09-wheel-sidebar-inactive.png` |
| 8 | F6 | Ownership switches to NavigationMenu ("Keyboard menu: active"), menu revealed | PASS | `gui/08-f6-switch-to-nav.png` |
| 9 | Mouse wheel down | Sidebar state kept: 已確認：設定, 狀態：展開 | PASS | `gui/09-wheel-sidebar-inactive.png` |
| 10 | Mouse click 儀表板 | Selected and confirmed; Sidebar takes ownership ("Keyboard group: active") | PASS | `gui/10-click-dashboard.png`, `gui/11-status-after-click.png` |

## Re-check after review fixes (same day, rebuilt binary)
The independent review (Fable 5.2) raised 2 P1s and several P2s. After the Gallery fixes, the rebuilt binary was driven again:

| # | Input | Expected | Observed | Screenshot |
|---|---|---|---|---|
| 11 | Make 設定 current, Esc, wheel to page top, F7 | Whole Sidebar group revealed: section title, instructions and 儀表板 all visible (review P2-4) | PASS | `gui/21-f7-late-current-whole-group.png` |
| 12 | Wheel so the menu row sits at the bottom edge, F6 | The menu is already visible, so **no scroll** (R1) | PASS | `gui/23-f6-visible-no-scroll.png` |
| 13 | →, ↓ | Dropdown opens below the fold; the focused child ▸ 桌面版 is revealed and visible (review P2-3) | PASS | `gui/25-down-child-revealed.png` |

Known limitation (review P1-2, not fixed): closing the in-flow 產品 dropdown shrinks the page. When the page is scrolled near its maximum, the scroll clamp moves content under the pointer. This is now recorded in a test comment; it is not claimed as fixed.

## Findings
1. **F6 reveal hides the section heading** (P2). When the NavigationMenu is off-screen, F6 scrolls it flush under the toolbar using margin 8. That hides the "NavigationMenu — typed activation" title and the instructions (step 2). The Sidebar uses margin 80 and keeps its heading visible (step 4). Follow-up: use a heading-preserving margin for the nav menu as well (ROADMAP 059).
2. **Sidebar labels are centred** (P2, cosmetic). Full-width sidebar item Buttons centre their text (steps 5 and 10); a navigation sidebar normally left-aligns. This is a Button alignment default, not a 057 regression. Follow-up: ROADMAP 063 (left-aligned Sidebar/Sidenav item labels, requested by the user).
3. **Child reveal is top-anchored** (P2). In step 13 the revealed child is placed at the top with margin 8, which hides its parent row 產品 under the toolbar. A minimal scroll that brings the child just into view at the bottom would keep the context. Follow-up: ROADMAP 059.
4. **Selected Gallery category contrast** ("Layout" in the left nav) is still nearly unreadable. This is already ROADMAP 060.

## NOT_RUN
Windows 注音 IME composition (the preedit guard is covered only by unit tests), screen reader, touch, other DPI or monitors, and the dark theme for this batch.
