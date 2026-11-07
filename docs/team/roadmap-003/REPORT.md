# ROADMAP 003 Report — Retained IDs in the real app path (2026-11-07)

Brief: [BRIEF.md](BRIEF.md). Plan: `docs/plans/P0-002_runtimetree_integration.md`
(Phase 2). Base: master `896f4d6`. Rust 1.88.0 MSVC, Windows 10.

## Team

| Item | Owner | Result |
|---|---|---|
| Contract | Lead (Opus 5.5) | `retained_binding.rs`, `view_child_widgets`, brief |
| A | team-rust-engineer / Haiku | `to_layout_with_binding` via one generic layout body; 12 tests incl. mutation check |
| B | team-rust-engineer / Haiku | `update_with_binding`; 6 tests; Tooltip/Popover read child rect (accepted deviation) |
| C | Lead | Gallery wiring, sidebar key fix, 8 Gallery tests (a sigma attempt was stopped at the user's request after ~20 min of reading, no file written) |
| Review | team-independent-reviewer / Sonnet | APPROVE_WITH_FIXES; byte-identical sequential layout vs HEAD proven by dump `cmp` |

## Key finding (REPRODUCED)

All eight sidebar category buttons used the key `""`. `reconcile_roots` rejected
every Gallery frame with `DuplicateSiblingKey("")`, so the Phase 1 shadow tree
was never populated. The new Gallery tests failed with exactly that error before
the key fix and pass after it.

## Review fixes applied

- MEDIUM-1: parity now compares computed snapshot rects (after
  `apply_gallery_styles`) for every page, not only styles.
- MEDIUM-2: new test asserts accessibility bounds equal layout rects through the
  real `frame()` path for every page.
- LOW-1: a partial binding (mismatches > 0) now drops to sequential IDs for the
  frame and logs once, in release builds too.
- Not changed: LOW-2 (Table/DataGrid cell and Dialog content a11y nodes have no
  layout rect in bound mode; previously they read unrelated rects). Follow-up.

## Evidence (Lead, head of branch)

- `cargo test --locked --workspace`: 1284 passed before the review-fix tests
  were added; `acme-gallery` 8 passed after.
- `cargo clippy -p acme-gallery -p acme-widgets -p acme-accessibility --all-targets -D warnings`: PASS.
- rustfmt, `cargo doc -D warnings` (widgets, accessibility): PASS. Cargo.lock unchanged.

## NOT_RUN

Native Gallery window, screen reader run against the stable IDs, DPI. Other apps
(`apps/acme-gallery`, `apps/playground`) still use sequential IDs.

## Next

ROADMAP 004: Gallery keyboard focus is an index into the hit-region list; it
drifts on page changes and `Enter` indexes out of bounds when the new page has
fewer regions. Key focus by the now-stable NodeId.
