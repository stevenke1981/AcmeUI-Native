# ROADMAP 027 — Mobile list item acceptance

2026-11-10, Asia/Taipei. Task MOBILE-LIST-ITEM-027, priority P2. Base
`60b3d7e91094e40ff9ed09b1c234d021b07160c1`, fetched merge of PR #35 after all
ten hosted checks PASS. Branch `sdk/roadmap-027-mobile-list-item`. Execution
**single-agent-role-passes** (Lead, UI/UX, Verification), sole writer, author
self-review; independent review NOT_RUN. Owner authorizes merge after local
checks, all hosted CI PASS and MERGEABLE. Third/final slice this session.

OBSERVED: existing Row has title/detail/icon and on_tap message, but lacks a
full-width semantic surface, wrapping, bounded trailing icon and content key.
Gallery only collects Button hit areas, so no list-item on_tap example runs.
Rows retain generic Group/nonfocusable framework semantics; this slice does not
claim typed list accessibility or general focus integration. Those remain an
explicit follow-up, alongside manual screen-reader validation.

Acceptance:
1. Preserve constructor/public fields/Row root and optional on_tap message. Use
   semantic Surface, full available width, minimum 44 px height, wrapping Unicode
   title/detail in an instance-keyed flexible column and a fixed 20 px icon box.
2. Cover public/optional/empty/Unicode combinations, 180/320/640 px real layout,
   no overlap, inherited colours and retained root/text/title identity through
   detail/icon/message changes and keyed reorder.
3. Gallery shows two actionable rows, passive/narrow long-text rows, detail
   toggling and reorder controls. Scope hit collection/paint-index accounting to
   this demo's row messages; do not reinterpret the search host's parent marker.
   Test actual pointer dispatch and stable identity after reorder/detail edits.
4. Add an explicitly documented Gallery keyboard host: F6 enters the list,
   Tab/Shift+Tab or arrows navigate, Enter/Space activates, Escape leaves; reveal
   focus on scroll and preserve active search composition. Test real events.
5. Check themed surfaces, hover/press states, following-button paint indices,
   viewport clips and scales/themes. Capture/inspect actual native CPU pointer
   and keyboard flows. Run locked UI/Gallery all-feature/mobile-only tests,
   strict touched/mobile Clippy, formatting, actual CPU profile and workspace
   milestone check/test. Preserve exact SHA/dirty state/argv/OS/toolchain/logs.

Exact sole-writer leases:
- `crates/acme-ui/src/mobile/mobile_list_item.rs`
- `crates/acme-ui/src/mobile/mobile_list_item_tests.rs` (new)
- `apps/acme-gallery/src/main.rs` (shared, serialized)
- `apps/acme-gallery/src/mobile_list_item.rs` (new private helper)
- `apps/acme-gallery/src/mobile_list_item_tests.rs` (new)
- `docs/status/todos.md`
- `docs/agent-team/ROADMAP.md`
- `docs/team/roadmap-027/BRIEF.md`
- `docs/team/roadmap-027/REPORT.md`
- `docs/team/roadmap-027/evidence.json`
- `docs/team/roadmap-027/logs/` (generated files enumerated in evidence)
- `docs/team/roadmap-027/gallery-mobile-list-items.png`
- `docs/team/roadmap-027/gallery-mobile-list-opened.png`
- `docs/team/roadmap-027/gallery-mobile-list-keyboard.png`

Read dependencies: AGENTS/team/roles/ownership/spec/plan/todos/P3/roadmap, 026
report, list item/Row/Column/Label/Icon/style/retained/layout, Gallery rendering,
hits/search host, view bridge/accessibility semantics and CI/verification runner.
No manifest/dependency, Scene/platform changes, new public API, framework-wide
container routing/focus/accessibility rewrite, release or performance claim.
Manual human Gallery, Windows 注音 IME, physical DPI/touch, GPU/recovery and
screen readers are NOT_RUN. Rollback: revert this bounded PR.
