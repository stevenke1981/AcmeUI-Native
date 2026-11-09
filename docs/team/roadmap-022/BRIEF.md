# ROADMAP 022 — Mobile search acceptance

- Task: MOBILE-SEARCH-022, priority P2.
- Base: `a993b4e24dd274b159eb2c5536eada228506e484` (fetched master).
- Branch: `sdk/roadmap-022-mobile-search`.
- Execution: **single-agent-role-passes** (Lead, UI/UX, Rust Core, Verification).
  One author, serialized writes; author self-review, independent review NOT_RUN.
- Owner authorizes PR creation and merge after targeted tests, green CI and
  GitHub MERGEABLE status. No release or repository-setting changes.

## Observed problem and user journey

The current search builder renders a Label instead of an editable TextInput,
uses a global clear-button key, has no rounded themed surface, and Gallery has
no search demo or text-input host. TextInput layout currently ignores its Style,
so a flex input would collapse inside a Row. Users should focus the Mobile
search, edit a Unicode query, submit it, and clear it without losing input identity.

## Acceptance

1. Compose a themed rounded Row, search icon, real TextInput and optional clear
   button. Keep existing constructor/key/submit/clear methods. Add only the
   controlled value and input-message fields/methods needed for real editing;
   document this additive builder-struct compatibility change.
2. Derive input/action keys from the instance; input identity survives value,
   optional clear/submit changes and reordering keyed instances. Test defaults,
   Unicode/empty values, real layout at narrow/wide widths, optional actions,
   focusable semantics and themed styles. Clear emits only its own message.
3. Honour TextInput Style in both layout paths (existing style API), with a
   regression for responsive width. No widget kind, Scene, dependency or platform
   API additions. Use existing text-input state/editing APIs; no cursor arithmetic.
4. Gallery demonstrates editing, submit and clear through actual host events,
   bounded text paint, scroll clips and retained layout. Reuse TextInputState,
   render_text_input and keyboard/commit handlers. Include keyboard focus and
   composition guards; IME candidate/device acceptance stays NOT_RUN.
5. Preserve failing regression logs, then run locked touched-package tests,
   mobile-only/all-feature acme-ui tests, strict touched-package Clippy, actual
   CPU evidence runner, and workspace check/test at this slice's milestone.
   Capture and inspect an automated Xvfb native CPU window.
6. Record exact tested SHA, dirty state, toolchain/OS, argv, times, exit codes
   and raw log hashes; self-review before PR/CI/authorized merge.

## Exact leases (sole writer)

- `crates/acme-ui/src/mobile/mobile_search.rs`
- `crates/acme-ui/src/mobile/mobile_search_tests.rs` (new)
- `crates/acme-widgets/src/lib.rs` (only TextInput layout and regression)
- `apps/acme-gallery/src/main.rs` (shared; sole writer)
- `apps/acme-gallery/src/mobile_search.rs` (new private host)
- `apps/acme-gallery/src/mobile_search_tests.rs` (new)
- `docs/status/todos.md`
- `docs/agent-team/ROADMAP.md`
- `docs/team/roadmap-022/BRIEF.md`
- `docs/team/roadmap-022/REPORT.md`
- `docs/team/roadmap-022/evidence.json`
- `docs/team/roadmap-022/logs/` (generated, enumerated in evidence)
- `docs/team/roadmap-022/gallery-mobile-search.png`

Read dependencies: AGENTS/team contracts/workflow/ownership, spec/plan/todos,
roadmap and 014/019/020/021 reports, mobile primitives, style/layout/retained
bridges, Gallery paint/hit/event paths, TextInputState, CI and verify_sdk.py.
Graph: audit → regression → component/host → verification → review → PR/CI → merge.
Non-goals: remaining P3 widgets, spinner animation, text-input engine rewrite,
GPU/platform integration, performance claims and production readiness.
Manual human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader
checks are NOT_RUN. Rollback: revert this bounded PR; manifests remain unchanged.

## Scope refinement: text-input render acceptance

The new real-input paint path uses the same baseline-relative prepared glyphs
fixed for Labels in 020. Source inspection found TextInput still passes a top
coordinate, including preedit runs, and caches shaping by text alone across DPI
and typography changes. Add RED/GREEN renderer regressions before correcting
these observed defects; these are necessary for the search demo's real paint.
Additional sole-writer leases:

- `crates/acme-textinput/src/lib.rs` (private render-cache key and test module)
- `crates/acme-textinput/src/render.rs` (baseline and cache validation)
- `crates/acme-textinput/src/preedit.rs` (shared baseline for split runs)
- `crates/acme-textinput/src/render_acceptance_tests.rs` (new)

Run locked acme-textinput tests and strict Clippy in addition to original gates.
The existing Gallery stepper test's fixed action count becomes 10 after adding
Search/Clear; its limit-specific message assertions remain unchanged.

The first native CPU capture exposed Gallery content-button glyphs above their
44 px touch-target interiors. Preserve the rejected capture and failing glyph
regression. Correct the already-leased main.rs Button paint path to center its
shaped text and supply the baseline, then repeat the affected checks and capture.
