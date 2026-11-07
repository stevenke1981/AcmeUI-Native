# ROADMAP 004 Report — Keyboard focus and core control semantics (2026-11-07)

Brief: [BRIEF.md](BRIEF.md). Base: ROADMAP 003 head. Rust 1.88.0 MSVC, Windows 10.

## Team

| Item | Owner | Result |
|---|---|---|
| F1 | team-desktop-engineer / Haiku | Focus keyed by retained node + message; red-before-green panic reproduced |
| F2 | team-accessibility-specialist / Haiku | TextInput/Button/Tab/NavRail/Dialog semantics; 3 mutation checks |
| Lead | Opus 5.5 | `clear_widget_focus`, review fixes, commits |
| Review | team-independent-reviewer / Sonnet | APPROVE_WITH_FIXES (1 MEDIUM, 5 LOW) |

## Reproduced bug

`handle_enter_space_key` indexed `button_info[focused]` without a bounds check.
Focusing region 14 on a large page and switching to an 11-region page made
Enter panic: `index out of bounds: the len is 11 but the index is 14`.

## Review fixes applied

- MEDIUM-1: on the sequential fallback path a focused tree/table row could
  export an AccessKit focus ID absent from the tree (AccessKit consumers panic
  on that). Focus is now reported only for nodes the tree contains; test walks
  every region of every page.
- LOW-1: `label_override` again renames label-only tabs.
- LOW-2: tab/nav-item semantics are dropped when `tabs`/`items` and `children`
  diverge (no mislabelling).
- LOW-3: direct test for Enter/Space with a stale index and no regions.
- LOW-5a: duplicated doc paragraph removed.
- Deferred: LOW-4 hovered/pressed index drift; LOW-5b validation message as
  AccessKit error text.

## Evidence

acme-accessibility 65 tests, acme-gallery focus tests, clippy `-D warnings`,
rustfmt — PASS at the time of the commit. Final workspace gates: see PR.

## NOT_RUN

Narrator/NVDA/UIA sessions against the new semantics; native window focus.
