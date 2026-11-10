# ROADMAP 036 — Paper square variant

2026-11-11 Asia/Taipei. P2 PAPER-SQUARE-036. Base
`2ad3e8841fff1c591433f92fd7590542cddc5f9f`, fetched master after PR #44 merged
with all ten hosted checks PASS. Execution single-agent-role-passes
(Lead/Architecture/UI/UX/Verification); sole writer, author self-review;
independent review NOT_RUN. Owner authorizes PR/merge after local tests,
all hosted CI PASS and MERGEABLE.

OBSERVED: open P8 square item; Paper maps elevation to corner radius.
E0 already has zero radius, but no independent square option exists for the
other six elevations. Add only square(bool), default false, preserving all
existing elevation/outline behavior. Public field compatibility is recorded
before implementation in ADR 0003.

Acceptance: all seven elevations x square x outline retain the Card variant,
children, key and padding; square forces radius zero, false restores the
existing elevation radius regardless of setter order/public field edits.
Real layout and retained IDs stay unchanged when corners change. Gallery
Layout page compares rounded, square elevated and square outlined surfaces;
theme/clip paint tests and native CPU capture. Non-goals: shadow redesign,
platform elevation, native MUI parity, new widget/Scene types or animation.

Exact sole-writer leases: crates/acme-ui/src/layout/{paper.rs,paper_tests.rs},
apps/acme-gallery/src/{main.rs,paper_examples.rs}, docs/adr/0003-paper-square.md,
docs/CHANGELOG.md, docs/status/todos.md, docs/agent-team/ROADMAP.md,
docs/team/roadmap-036/{BRIEF.md,REPORT.md,evidence.json,gallery-paper.png},
docs/team/roadmap-036/logs/* (generated). Shared Gallery entrypoint single-writer;
no manifests/Scene/platform writes.

Checks: locked UI all-feature and Gallery tests; extra layout-only probe
reproduced a pre-existing missing-feature dependency (baseline confirmed);
explicit layout,foundations,inputs combination is the supported scoped check; touched fmt and
strict all-target Clippy; actual CPU profile; workspace check/test at slice
milestone. Reuse target; no release/performance run or cargo clean. NOT_RUN:
human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen readers.
Rollback: revert this batch.
