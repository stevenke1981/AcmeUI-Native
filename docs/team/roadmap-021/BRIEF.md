# ROADMAP 021 — Mobile loader acceptance

- Task: MOBILE-LOADER-021, priority P2.
- Base: `58874a32c7509de11d647de1dcab290287eb7ac2` (fetched master).
- Branch: `sdk/roadmap-021-mobile-loader`.
- Execution: `single-agent-role-passes` (Lead, UI/UX, Verification), serialized
  by one author. Independent review NOT_RUN. No worker/model-routing claims.

## Audit, user journey and acceptance

OBSERVED: the builder already emits a Column with a foundation spinner followed
by an optional message; it is not a passthrough. The foundation spinner is a
static `⟳` Label, with no animation scheduling. The loader message defaults to
no wrapping, and Gallery shapes all content labels without width constraints.
The Gallery has no loader example and existing tests check only child counts.

Users can compare indicator-only and descriptive loaders in the Mobile page,
read long loading messages at narrow widths, and scroll or change themes.

1. Preserve builder signatures, defaults, root keys, padding/gap and existing
   foundation indicator. Explicitly document the static-glyph limitation;
   animation/runtime scheduling is a separate unchecked follow-up.
2. Reproduce long-message overflow before changing behavior. Wrap messages at
   character boundaries (including unbroken strings and Traditional Chinese).
   Keep message after indicator with no overlap at 120/240/400 logical pixels.
3. Test omission, empty/custom messages, public field edits, theme inheritance,
   noninteractive semantics and retained IDs across message changes, omission
   and multiple keyed loaders reordered in the same tree.
4. Add distinct-key indicator-only, short-message and long-message Gallery
   examples. Test real page layout, shaped multiline text, foreground colours,
   scales and viewport clips. Reuse existing Label wrap modes in Gallery paint;
   do not add widget kinds or renderer/platform/dependency contracts.
5. Run locked touched-package tests (all features and mobile only), strict
   Clippy, formatting, the actual CPU evidence runner, and workspace checks/tests
   once this slice is complete. Capture an automated Xvfb native CPU Gallery
   window; do not equate this with human/device acceptance.
6. Preserve command vectors, exact SHA/dirty state, OS/toolchain, timestamps,
   exit codes and raw log hashes. Author self-review, then open a PR and merge
   only with passing targeted checks, hosted CI and GitHub MERGEABLE status,
   as explicitly authorized by the owner.

## Exact file leases (one writer)

- `crates/acme-ui/src/mobile/mobile_loader.rs`
- `crates/acme-ui/src/mobile/mobile_loader_tests.rs` (new)
- `crates/acme-text/src/lib.rs` (scope refinement below)
- `apps/acme-gallery/src/main.rs` (shared, sole writer)
- `apps/acme-gallery/src/mobile_loader_tests.rs` (new)
- `docs/status/todos.md`
- `docs/agent-team/ROADMAP.md`
- `docs/team/roadmap-021/BRIEF.md`
- `docs/team/roadmap-021/REPORT.md`
- `docs/team/roadmap-021/evidence.json`
- `docs/team/roadmap-021/logs/` (generated files enumerated in evidence)
- `docs/team/roadmap-021/gallery-mobile-loader.png`

Read dependencies: AGENTS/team workflow and Lead/UI/UX/Rust Core/Verification
contracts, spec/plan/todos/roadmap, ROADMAP 020/019 and 014 mobile reports,
spinner/label/container primitives, layout/text/retained bridges, Gallery
render/hit paths, pinned toolchain, CI and verification runner.

Graph: audit → regression → fix/Gallery → verification → self-review → PR/CI →
authorized merge. Non-goals: other P3 components, animation scheduling, typed
status/live-region accessibility, performance rewrites/claims, releases,
production-readiness certification. Human Gallery, Windows 注音 IME, physical
DPI/touch, GPU and screen readers remain NOT_RUN. Rollback: revert the bounded
PR; manifests and lockfile are unchanged.

Acceptance refinement after the first Gallery GREEN attempt: even with correct
width/wrap constraints, prepared glyphs from all lines overlap the first line.
OBSERVED in `FontSystem::shape`: `ShapedGlyph.y` includes `run.line_y`, but
`RasterGlyph.y` omits each run's baseline delta. Include the narrow correction
in `crates/acme-text/src/lib.rs` and a RED/GREEN test for explicit and automatic
line breaks at multiple scales. Preserve first-line baseline-relative offsets,
and run locked text tests/Clippy plus dependent workspace/CPU gates.
