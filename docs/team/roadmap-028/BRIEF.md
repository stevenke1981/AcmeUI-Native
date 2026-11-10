# ROADMAP 028 — Mobile action acceptance

Task MOBILE-ACTION-028, P2, 2026-11-10 Asia/Taipei. Base
`d8204517821d70075356911fe6783c58699a951b`. Branch
`sdk/roadmap-028-mobile-action`. Execution: **single-agent-role-passes**
(Lead, UI/UX, Verification), sole writer, author self-review; independent review
NOT_RUN. Owner authorizes PR and merge after targeted checks, hosted CI and
MERGEABLE; no release. Read AGENTS, team/role contracts, spec/plan/todos/roadmap,
021 and latest 027 reports, component/layout/retained/Gallery paths.

OBSERVED: destructive/disabled fields are ignored; icon insertion changes the
unkeyed label slot; labels do not wrap and the row lacks a bounded surface.

Acceptance:
1. Preserve public builder and Row root/message contract; disabled rows emit no
   message. Use semantic surface/danger-soft/disabled colours, 44 px minimum
   target, full available width, bounded icon and wrapping Unicode label.
2. Stable instance-derived label key across icon/state changes and keyed reorder;
   test defaults/public fields, optional messages, narrow/wide real layout.
3. Gallery normal/destructive/disabled/passive/narrow demos, actual pointer
   dispatch, no disabled dispatch, themed text/surfaces, paint-index/clip checks
   and automated native CPU capture. Honour existing Label semantic text style.
4. Locked UI all-feature/mobile-only and Gallery tests, strict Clippy, fmt,
   actual CPU profile; workspace check/test at this completed-slice milestone.

Exact sole-writer leases: `crates/acme-ui/src/mobile/mobile_action.rs`,
`crates/acme-ui/src/mobile/mobile_action_tests.rs`,
`apps/acme-gallery/src/main.rs`, `apps/acme-gallery/src/mobile_action.rs`,
`apps/acme-gallery/src/mobile_action_tests.rs`, `docs/status/todos.md`,
`docs/agent-team/ROADMAP.md`, `docs/team/roadmap-028/BRIEF.md`,
`docs/team/roadmap-028/REPORT.md`, `docs/team/roadmap-028/evidence.json`,
`docs/team/roadmap-028/gallery-mobile-actions.png`,
`docs/team/roadmap-028/gallery-mobile-action-pressed.png`,
`docs/team/roadmap-028/logs/*` (generated evidence only, serialized).
Read dependencies: widgets/style/layout/text/retained/platform, existing Gallery
list host and capture harness. No manifests, Scene/platform contracts or deps.

Non-goals: generic retained Row focus/typed AccessKit actions, sheet dragging,
animation, performance/release claims. Manual human Gallery, Windows 注音,
physical DPI/touch, GPU and screen-reader scenarios remain NOT_RUN. Rollback:
revert this batch; no data migration. Reproduction: acceptance tests against
original conversion. Review: author bounds/state/identity/colour/clip audit.
