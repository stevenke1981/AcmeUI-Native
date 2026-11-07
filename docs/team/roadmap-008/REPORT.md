# ROADMAP 008 — Report

Branch `sdk/roadmap-008-followups` (on master `8df7212`). Windows 10, Rust 1.88.0 MSVC.

## Execution mode (actual)

| ID | Role / model | Outcome |
|---|---|---|
| Lead | Opus 5.5 | brief, CI benchmark smoke step, M2 table fix, review fixes, CHANGELOG/todos/ROADMAP |
| P1 | team-rust-engineer / Haiku (high) | validation text → Invalid + description; Table/DataGrid `Role::Cell` |
| P2 | team-desktop-engineer / Haiku (high) | Gallery stale hover/press fix + 3 headless tests |
| P3 | team-test-engineer / Haiku (high) | 11 compiled MIGRATION snippets in `examples/sdk-consumer/tests` |
| Review | team-independent-reviewer / Sonnet, read-only | APPROVE_WITH_FIXES (F1, F2 MED doc text: fixed; F3 LOW: recorded in todos) |

## Evidence

- RED before fix: P1 5 new tests failed; P2 2 tests failed (release activated `DpiInfo` from a NavRailSelect press; hover `Some(11)` vs `Some(12)`); P3 original M5 snippet failed `E0425 snapshot`.
- Reviewer reproduced P2 RED by removing each half of the fix on a scratch copy.
- `cargo test --locked --workspace`: 1442 passed, 0 failed, 10 ignored.
- `cargo check --locked --workspace --all-targets`: 0 warnings.
- `examples/sdk-consumer` `cargo test`: 1 + 11 passed.
- `acme-benchmark --quick --json` (release, local): completes, JSON written.

## Open / NOT_RUN

- Tree item roles: `Tree::children()` is empty and layout emits synthetic rows; needs per-item widgets and IDs in `acme-widgets` first.
- No `Row` nodes or column headers for tables; TextInput `description` field not exposed.
- Screen-reader confirmation of the new roles, Gallery visual check: NOT_RUN (manual).
