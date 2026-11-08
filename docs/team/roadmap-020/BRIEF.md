# ROADMAP 020 — Mobile determinate progress

- Task: MOBILE-PROGRESS-020, priority P2.
- Base: `9b0645f3b208bdbf6e0e13832014a0968012f5ee` (fetched master).
- Branch: `sdk/roadmap-020-mobile-progress`.
- Execution: `single-agent-role-passes` (Lead, UI/UX, Verification); one writer,
  author self-review. Independent review NOT_RUN.

## Audit and acceptance

OBSERVED: the builder emits a 6-pixel Row containing percentage text, without
any painted track or proportional fill. NaN produces a non-finite fraction.
The Gallery has no progress example. Existing APIs and public builder fields
will remain available; no dependencies or platform contracts will change.

User journey: compare empty, partial and complete progress in the Mobile page,
including optional descriptive text; quantity changes through the existing
stepper also update a progress example. Theme switching and scrolling preserve
semantic colours and viewport clipping.

1. Reproduce missing fill/unsafe fraction before fixing production code.
2. Compose a muted 6-pixel track with primary proportional fill, responsive to
   available width. Keep percentage text outside the thin track and optional
   descriptive text above it. Preserve track/fill identity across optional
   label insertion/removal and value updates with instance-derived keys.
3. Return a finite fraction in [0, 1]. Preserve the nonpositive-total fallback
   of one; extend it to non-finite totals. NaN values mean zero; infinities
   saturate. Revalidate public fields at conversion as well as in `fraction()`.
4. Test real layout at multiple widths, empty/partial/full and numeric extremes,
   label omission/content/order, retained IDs and semantic colours. Gallery
   tests exercise page composition, stepper-driven updates, painting and clips.
5. Run locked targeted tests (all features and mobile only), strict Clippy and
   formatting on Rust 1.88.0; run the real CPU evidence runner. Use workspace
   gates only at completed-slice integration. Capture a native virtual-display
   CPU Gallery window if available, labelled automated evidence.
6. Record exact tested SHA, dirty state, command vectors/timestamps/log hashes,
   and truthful todos/report. Open a PR; the owner authorizes merging only after
   passing local targeted checks, hosted CI and GitHub MERGEABLE status.

## Exact file leases (serialized)

- `crates/acme-ui/src/mobile/mobile_progress.rs`
- `crates/acme-ui/src/mobile/mobile_progress_tests.rs` (new)
- `apps/acme-gallery/src/main.rs` (shared, sole writer)
- `apps/acme-gallery/src/mobile_progress_tests.rs` (new)
- `docs/status/todos.md`
- `docs/agent-team/ROADMAP.md`
- `docs/team/roadmap-020/BRIEF.md`
- `docs/team/roadmap-020/REPORT.md`
- `docs/team/roadmap-020/evidence.json`
- `docs/team/roadmap-020/logs/` (generated files enumerated in evidence)
- `docs/team/roadmap-020/gallery-mobile-progress.png`

Read dependencies: spec/plan/todos/roadmap, team contracts, 019 and 014 mobile
reports, primitives/style/layout/retained bridges, Gallery render and hit paths,
pinned toolchain, CI and verification runner. Graph: audit → regression →
implementation/Gallery → verification → self-review → PR/CI → authorized merge.

Non-goals: other P3 acceptance items, foundation progress rewrite, animation,
typed progress accessibility roles, renderer/platform/public API expansion,
performance claims, releases or production-readiness claims. Manual human
Gallery review, Windows 注音 IME, physical DPI/touch, GPU and screen-reader
acceptance stay NOT_RUN. CPU screenshots do not establish these results.

Acceptance refinement after the first native capture: percentage ink overlapped
the track despite non-overlapping layout rectangles. The existing Gallery Label
paint path supplied a top coordinate where prepared glyphs need a first-baseline
origin. Include that existing-path correction (normal and swatch labels share
the corrected shaping path) and a shaped-glyph RED/GREEN test at three scales.
Preserve the initial capture under `logs/gallery-overlap-before.png` and rerun
final checks on the corrected source commit before PR delivery.

Rollback: revert the bounded PR; manifests and lockfile remain unchanged.
