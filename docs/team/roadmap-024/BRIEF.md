# ROADMAP 024 — Mobile avatar acceptance

- Task MOBILE-AVATAR-024, priority P2.
- Base `7171d4b2d492f44075cbe029f3ee037a2d5188d4`, fetched merge of PR #32;
  all ten hosted PR checks PASS. Branch `sdk/roadmap-024-mobile-avatar`.
- Execution **single-agent-role-passes** (Lead, UI/UX, Verification), serialized
  writes, author self-review; independent review NOT_RUN. Existing workflow,
  role contracts, spec/plan/todos/P3 contract and latest 023 report read.

OBSERVED: avatar is a fixed Stack with a plain Label, no circular surface,
centering or empty-content placeholder. Label layout ignores its existing Style;
Gallery ignores text alignment and only paints styled Row backgrounds.

Acceptance:
1. Preserve constructor/public fields/size variants (32/40/56 px). Add a themed
   circular surface, centered initials and a neutral `?` placeholder for empty
   or whitespace initials. No image loader/API expansion; preserve Unicode text.
2. Use an inset text square inside the circle, existing Label dimensions,
   line-height and TextAlign::Center; bound long text paint. Honour Label Style
   in both layout paths and existing single-line alignment in Gallery. Paint
   styled Stack backgrounds with the same themed/viewport-clipped path as Rows.
3. Test defaults/public edits, all sizes, true layout, theme inheritance,
   noninteractive semantics, retained IDs across text/size changes and keyed
   reordering. Add Gallery sizes/placeholder examples and tests for circles,
   aligned/clipped glyph geometry at multiple scales/themes and complete binding.
4. Run locked acme-widgets/acme-ui/Gallery tests, all-feature/mobile-only UI,
   strict touched/mobile Clippy and formatting, actual verify_sdk.py CPU profile,
   workspace check/test at slice milestone, then capture/inspect native CPU window.
5. Record exact tested SHA/dirty state, argv, toolchain/OS, timestamps, exits and
   raw logs/hashes. Open PR, await hosted CI, and merge only when MERGEABLE as
   explicitly authorized. This is the third and final slice for this session.

Exact sole-writer leases:
- `crates/acme-ui/src/mobile/mobile_avatar.rs`
- `crates/acme-ui/src/mobile/mobile_avatar_tests.rs` (new)
- `crates/acme-widgets/src/lib.rs` (only Label layout and regression)
- `apps/acme-gallery/src/main.rs` (shared; sole writer)
- `apps/acme-gallery/src/mobile_avatar_tests.rs` (new)
- `docs/status/todos.md`
- `docs/agent-team/ROADMAP.md`
- `docs/team/roadmap-024/BRIEF.md`
- `docs/team/roadmap-024/REPORT.md`
- `docs/team/roadmap-024/evidence.json`
- `docs/team/roadmap-024/logs/` (generated files enumerated in evidence)
- `docs/team/roadmap-024/gallery-mobile-avatars.png`

Read dependencies: avatar/Label/container/style/layout/retained, Gallery render
and hit paths, CI and verification runner. Audit → regressions → implementation/
Gallery → checks → self-review → PR/CI → authorized merge. No dependencies,
manifests, Scene/platform contracts, actual image decoding, accessibility-role
expansion, general multiline text alignment, performance rewrite or release.
Manual human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader
acceptance remain NOT_RUN. Rollback: revert this bounded PR.
