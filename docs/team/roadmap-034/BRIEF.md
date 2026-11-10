# ROADMAP 034 — SwipeActions acceptance

2026-11-11 Asia/Taipei. P2 SWIPE-034. Base
`343a3f55985655f7bbbaac1a6194657d46657e7b`, fetched master after Snackbar
PR #42 merged with all ten hosted checks PASS. Execution single-agent-role-passes
(Lead/UI/UX/Verification), sole writer, author self-review; independent review
NOT_RUN. Owner authorizes PR/merge after local tests, all hosted checks and
MERGEABLE. Third/final slice; no release or other-repository changes.

OBSERVED: P4 SwipeActions unchecked; no swipe/gesture implementation found in
UI/core/platform/Gallery. Introduce only a stateless controlled mobile builder
and backend-neutral gesture state, reusing existing Row/Column/Button widgets.

Acceptance: keyed child plus optional trailing Archive/Delete Buttons visible
only when revealed. Real 240/360/600 px layout, typed action messages, theme
variants, content identity across reveal/action options/reorder. State begins
in host hit bounds, ignores sub-threshold movement, locks a horizontal gesture
at 12 logical px and reveals left/closes right at 48 px on release. Vertical
movement, cancel, nonfinite coordinates and release without begin must not
reveal or invoke destructive actions. Host owns pointer identity/capture and
visibility; no platform types exposed. Gallery implements pointer reveal/close,
action/remove/restore, cancel/scroll/focus handling and F7/Enter/Delete/Escape
keyboard alternative, with tests and real native CPU captures.

Exact sole-writer leases: crates/acme-ui/src/mobile/swipe_actions.rs,
crates/acme-ui/src/mobile/swipe_actions_tests.rs,
crates/acme-ui/src/mobile/mod.rs,
crates/acme-ui/src/mobile/mobile_contract_tests.rs,
apps/acme-gallery/src/main.rs, apps/acme-gallery/src/swipe_actions.rs,
apps/acme-gallery/src/swipe_actions_tests.rs, README.md, README-zhtw.md,
docs/status/todos.md, docs/agent-team/ROADMAP.md,
docs/team/roadmap-034/{BRIEF.md,REPORT.md,evidence.json},
docs/team/roadmap-034/gallery-mobile-swipe-closed.png,
docs/team/roadmap-034/gallery-mobile-swipe-revealed.png,
docs/team/roadmap-034/gallery-mobile-swipe-archived.png,
docs/team/roadmap-034/gallery-mobile-swipe-deleted.png,
docs/team/roadmap-034/logs/* (generated). Shared Gallery/module exports serialized.
README inventory updates reflect public component declarations (200 including
this addition), excluding test/support files. No manifests/Scene/platform writes.

Checks: locked UI all-feature/mobile-only and Gallery tests; strict touched
Clippy, fmt, actual CPU verification profile; workspace gates at slice milestone.
Manual human Gallery, Windows 注音 IME, physical DPI/multitouch, GPU and screen
readers NOT_RUN. Non-goals: general gesture engine, continuous translation,
animation, velocity/inertia, global pointer capture or live-region semantics.
Rollback: revert this batch. No performance or production-readiness claim.
