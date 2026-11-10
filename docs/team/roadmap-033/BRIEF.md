# ROADMAP 033 — Snackbar acceptance

2026-11-11 Asia/Taipei. P2 SNACKBAR-033. Base
`1e3817fc0ac7596221d82e357311f65d8b5e6e30`, fetched master after PR #41
merged with all ten hosted checks PASS. Single-agent-role-passes (Lead,
UI/UX, Verification), one writer, author self-review; independent review NOT_RUN.
Owner authorizes PR/merge after local targeted tests, all hosted CI and
MERGEABLE. Rollback: revert this batch. No release/other repository changes.

OBSERVED: P4 Snackbar is unchecked, while the design mapping points to existing
overlay::toast. That builder has duration metadata but no timer host, optional
action API or working dismiss message; text does not wrap and keys collide.
Reuse it instead of inventing a duplicate Snackbar API.

Acceptance: optional typed action/dismiss with instance-derived keys; wrapping
Unicode message with bounded real layout at 120/240/600 px, tone/default
compatibility and retained IDs across actions/messages/reordering. Stateless
widget: host owns bottom positioning, visibility and expiry. Gallery shows
with/without-action flows at the viewport bottom, replaces/restarts deadlines,
cancels stale presses on expiry, blocks click-through, and demonstrates action,
dismiss and automatic expiry. Cover default/Apple light/dark and scales 1/1.25/2,
locked UI/Gallery tests, overlay+foundations isolation, strict Clippy, actual CPU profile,
workspace gates at milestone and actual native CPU captures.

Exact sole-writer leases: crates/acme-ui/src/overlay/{toast.rs,toast_tests.rs},
apps/acme-gallery/src/{main.rs,snackbar.rs,snackbar_tests.rs},
docs/status/todos.md, docs/agent-team/ROADMAP.md,
docs/team/roadmap-033/{BRIEF.md,REPORT.md,evidence.json,gallery-mobile-snackbar*.png},
docs/team/roadmap-033/logs/* (generated). Shared Gallery serialized; no manifests,
Scene/platform changes. Dependencies: toast/Card/Button/Label/retained layout,
Gallery hit/render and notification timer paths; specs/plan/todos/team contracts.
Non-goals: SDK timer scheduler, global overlay manager, animation, live-region
semantics. Human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen
readers NOT_RUN. No performance or production-readiness claim.
