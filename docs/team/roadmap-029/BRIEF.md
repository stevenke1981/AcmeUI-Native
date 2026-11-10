# ROADMAP 029 — Mobile sheet handle acceptance

Task MOBILE-SHEET-HANDLE-029, P2, 2026-11-10 Asia/Taipei. Base
`aad58f0889fb7b0f753694b91982e1538b369d57`, fetched clean master after PR #37
merged with ten hosted checks PASS. Branch sdk/roadmap-029-mobile-sheet-handle.
Execution **single-agent-role-passes** (Lead, UI/UX, Verification), sole writer;
author self-review, independent review NOT_RUN. Owner authorizes PR/merge after
local checks, hosted CI and MERGEABLE. Read AGENTS/team/roles, spec/plan/todos/
roadmap/latest 028 report; inspected handle/container/style/retained/Gallery.

OBSERVED: empty 36×4 Row has no background or radius, so no visible bar.
Acceptance: preserve builder/Row/key, default 36×4/custom width, semantic
MutedForeground, radius 2, no shrink/children/actions. Negative width clamps to
zero; nonfinite public widths fall back to 36. Test defaults/public edits,
real constrained geometry, retained width changes/reorder. Gallery sheet-like
card/default/custom examples, themed clips, no hits, following-button indices,
automated native CPU capture. Locked UI all-feature/mobile-only/Gallery tests,
strict Clippy/fmt, actual CPU profile and completed-slice workspace check/test.

Exact sole-writer leases: crates/acme-ui/src/mobile/mobile_sheet_handle.rs,
crates/acme-ui/src/mobile/mobile_sheet_handle_tests.rs,
apps/acme-gallery/src/main.rs, apps/acme-gallery/src/mobile_sheet_handle_tests.rs,
docs/status/todos.md, docs/agent-team/ROADMAP.md,
docs/team/roadmap-029/{BRIEF.md,REPORT.md,evidence.json,gallery-mobile-sheet-handles.png},
docs/team/roadmap-029/logs/* (generated evidence only, serialized).
Read dependencies: widgets/layout/style/theme/retained, prior capture harness.
No manifests/contracts/dependencies. Rollback: revert batch, no migration.
Non-goals: drag recognition, bottom_sheet rewrite, animation/performance/release
or mobile-runtime claims. Manual human Gallery, Windows 注音, physical DPI/touch,
GPU and screen readers NOT_RUN. Next slice: mobile builder contract tests.
