# ROADMAP 050 — Collapsible typed toggle conversion

2026-11-12 Asia/Taipei. Base `6d29095a8d934e1256f10be19755a9e1cb94b482` after PR #58 merged with
10 hosted checks PASS and GitHub MERGEABLE. Second slice, selected from the
unfinished P5 builder conversion coverage todo. OBSERVED: Collapsible drops
on_toggle and disabled when converting its header Row. P1: the configured
header cannot dispatch a toggle message.

Acceptance: an enabled configured header Row carries the current typed message;
disabled or unconfigured headers are passive and excluded from retained focus.
Preserve existing Column/Row/chevron/title/content composition and public API;
open remains host-owned. Header identity/focus survives open/title/message
updates and keyed instance reorder; disabling clears its focus eligibility.
Add a controlled Foundations Gallery open/close and disable/enable preview,
with pointer/stale-action, real layout, theme/scale/clip/paint-hit checks and
actual native CPU-window evidence. No public API/dependency expansion.

Non-goals: animation, generic keyboard hosting, disclosure accessibility roles,
OS motion policy or hardware acceptance. Human Gallery, Windows 注音 IME,
physical DPI/touch, GPU and screen readers NOT_RUN. Rollback: revert PR.
User authorizes merge commit after local acceptance, all hosted CI PASS and
GitHub MERGEABLE, without waiting for Codex review.

Execution: single-agent-role-passes (Lead, Architecture, UI/UX, Verification),
sole writer, author self-review; independent review NOT_RUN. Reuse previously
loaded spec/plan/todos, workflow, ownership and role contracts. Exact leases:
- crates/acme-ui/src/foundations/collapsible.rs
- crates/acme-ui/src/foundations/collapsible_tests.rs
- apps/acme-gallery/src/main.rs
- apps/acme-gallery/src/collapsible.rs
- apps/acme-gallery/src/collapsible_tests.rs
- apps/acme-gallery/src/retained_tests.rs (lease added after the milestone
  exposed its background-Quad assumption for every action; retain button
  guards and verify transparent Collapsible header glyph bounds instead)
- docs/status/todos.md
- docs/status/final.md
- docs/agent-team/ROADMAP.md
- docs/team/roadmap-050/BRIEF.md
- docs/team/roadmap-050/REPORT.md
- docs/team/roadmap-050/evidence.json
- docs/team/roadmap-050/gallery-collapsible-closed.png
- docs/team/roadmap-050/gallery-collapsible-disabled.png
- generated scripts/records/raw logs under docs/team/roadmap-050/logs/

Read dependencies: Container handlers, retained binding/view bridge and focus,
Gallery input/render/hit routing. Preserve existing Mobile action routing when
adding the private Collapsible Gallery handler. Regression-first locked touched
checks and strict Clippy; workspace check/tests, debug/release Gallery and actual
CPU profile only at slice milestone. Reuse target/.
