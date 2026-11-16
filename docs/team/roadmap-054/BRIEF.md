# ROADMAP 054 — Calendar range highlighting

2026-11-16 Asia/Taipei. Base `603730be870a86298c27fc9e2d2811a5bccfadaa`.
Branch `sdk/roadmap-054-calendar-range`. Execution: **single Codex agent,
sole writer and author self-review**; independent review NOT_RUN.

Selected from the incomplete P6 Calendar visual range item in
`docs/status/todos.md`. One bounded slice, one PR.

Acceptance:

- Add a host-controlled, inclusive same-month range. Normalize reversed endpoints;
  ignore ranges containing an invalid day rather than highlight phantom dates.
  The last single-day/range setter wins. Preserve single-day/today defaults.
- Use semantic colors, uniform weekday/day slots and passive cells; preserve
  retained day identity through range and optional-header changes.
- Add a Gallery preset/clear journey with real pointer, measured layout,
  light/dark, scale and clipping checks; capture automated native CPU evidence.
- Run locked targeted UI/Gallery tests, changed-file formatting, strict touched
  Clippy and actual `scripts/verify_sdk.py --profile cpu`, reusing `target/`.
  Run workspace gates at the slice milestone. Merge only after all hosted CI
  succeeds and GitHub reports MERGEABLE, using a merge commit.

Exact sole-writer lease: `crates/acme-ui/src/foundations/calendar.rs`,
`crates/acme-ui/src/foundations/calendar_range_tests.rs`,
`apps/acme-gallery/src/main.rs` (shared entrypoint, serialized),
`apps/acme-gallery/src/calendar_range.rs`, `docs/status/todos.md`, and
`docs/team/roadmap-054/BRIEF.md`, `REPORT.md`, `evidence.json`,
`gallery-calendar-range.png`, `gallery-calendar-range-dark.png`,
`gallery-calendar-clear.png`, plus exact check/capture logs under
`docs/team/roadmap-054/logs/` as recorded by `evidence.json`.
Read dependencies: widget Card/Label/Styled, retained binding, existing Calendar,
Gallery host/render/hit paths, CI and SDK verification runner. No parallel edits.

Only the range setter is new component API. No manifests, dependencies, lockfile,
Scene or platform edits. Cross-month selection, date-cell input, navigation,
calendar accessibility semantics and localized calendars remain separate work.
Rollback: revert this PR's merge commit through a normal reviewed revert.

Human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader
acceptance remain NOT_RUN. No project completion or release-readiness claim.
