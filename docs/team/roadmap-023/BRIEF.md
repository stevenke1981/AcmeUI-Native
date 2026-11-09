# ROADMAP 023 — Mobile chip acceptance

- Task MOBILE-CHIP-023, priority P2; base `d1bbe5d4fd901ceeaffe072a129ae5e6146f26c8`.
- Branch `sdk/roadmap-023-mobile-chip`; follows merged PR #31 (all hosted checks PASS).
- Execution **single-agent-role-passes** (Lead, UI/UX, Verification), serialized
  writes, author self-review; independent review NOT_RUN. Existing contracts and
  spec/plan/todos/roadmap plus 022 report read; no new model/worker claims.

OBSERVED: chip is an unstyled Row, uses a global `chip_close` key, and stores
on_tap only on its Row, which Gallery cannot activate. Users should compare a
static tag, toggle a filter, tap a removable tag, dismiss it, and restore it.

Acceptance:
1. Preserve constructor, public fields and chainable API. Add themed rounded
   surface and selection marker. Represent tap/dismiss as separate child Buttons
   so dismiss cannot bubble into an ancestor tap message; document that tree
   message placement changes, without adding public API.
2. Derive action keys from the instance. Cover all optional actions, Unicode/
   empty text, selection changes, real layout/hit geometry, and retained IDs
   across optional dismiss changes and keyed instance reorder.
3. Add static, selected/toggleable and removable Gallery examples plus restore,
   using real hit/activate/rebuild flow. Test theme/paint order/clips and retained
   binding; capture an automated native CPU Gallery window.
4. Locked all-feature/mobile-only acme-ui tests, Gallery tests, strict touched
   Clippy and formatting; actual verify_sdk.py CPU profile and workspace
   check/test at the completed-slice milestone. Record tested SHA, dirty state,
   argv, toolchain/OS, times, exits and raw-log hashes before PR/CI/authorized merge.

Exact sole-writer leases:
- `crates/acme-ui/src/mobile/mobile_chip.rs`
- `crates/acme-ui/src/mobile/mobile_chip_tests.rs` (new)
- `apps/acme-gallery/src/main.rs` (shared; sole writer)
- `apps/acme-gallery/src/mobile_chip_tests.rs` (new)
- `apps/acme-gallery/src/mobile_search_tests.rs` (viewport-independent pointer coordinates if needed)
- `docs/status/todos.md`
- `docs/agent-team/ROADMAP.md`
- `docs/team/roadmap-023/BRIEF.md`
- `docs/team/roadmap-023/REPORT.md`
- `docs/team/roadmap-023/evidence.json`
- `docs/team/roadmap-023/logs/` (generated files enumerated in evidence)
- `docs/team/roadmap-023/gallery-mobile-chips.png`

Read dependencies: chip/label/button/container/style, retained/layout bridges,
Gallery rendering/hits, team roles/workflow, current CI and verification runner.
Graph audit → regression → component/Gallery → checks → self-review → PR/CI → merge.
No dependencies, manifests, Scene/platform contracts, text engine, arbitrary-long
label truncation, new keyboard-navigation framework or performance rewrite.
Manual human Gallery, physical DPI/touch, Windows 注音 IME, GPU and screen-reader
acceptance remain NOT_RUN. No release or production-readiness claim.
Rollback: revert the bounded PR.
