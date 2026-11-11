# ROADMAP 048 — Fullscreen Close conversion

2026-11-12 Asia/Taipei. Base `fdae1b9b599b970eb7bf609ecb842cfd28cee310`, after PR #56 merged
with ten hosted checks PASS and GitHub MERGEABLE. Select the unfinished P5
overlay/layout/chart conversion coverage todo. OBSERVED: Fullscreen build emits
only a close Label and ignores on_close. HYPOTHESIS: removing its optional
unkeyed header also changes the body identity/focus. Priority P1 for the ignored
configured close action; reproduce conversion and retained behavior first.

Acceptance: configured visible close emits the current typed message through
an instance-keyed Button; missing callback retains the passive ✕ Label and
close_button(false) exposes no close action. Preserve Plain Card composition,
content, public signatures and existing opacity metadata. Keep body identity
and focus when the optional close header changes. Add a controlled inline
Gallery close/reopen flow with stale-close rejection, themed/clipped real
layout/paint/pointer tests and actual X11 CPU-window captures. Full-viewport
placement, backdrop opacity painting, Escape/focus trap and hardware acceptance
remain separate. No public API or dependency expansion. Rollback: revert PR.

Execution single-agent-role-passes: Lead, Architecture, UI/UX, Verification;
sole writer, author self-review; independent review NOT_RUN. Loaded contracts,
spec/plan/todos, workflow and ownership. Exact single-writer leases:
- crates/acme-ui/src/overlay/fullscreen.rs
- crates/acme-ui/src/overlay/fullscreen_tests.rs
- apps/acme-gallery/src/main.rs
- apps/acme-gallery/src/fullscreen.rs
- apps/acme-gallery/src/fullscreen_tests.rs
- docs/status/todos.md
- docs/status/final.md
- docs/agent-team/ROADMAP.md
- docs/team/roadmap-048/BRIEF.md
- docs/team/roadmap-048/REPORT.md
- docs/team/roadmap-048/evidence.json
- docs/team/roadmap-048/gallery-fullscreen-open.png
- docs/team/roadmap-048/gallery-fullscreen-closed.png
- generated scripts, records and raw logs under docs/team/roadmap-048/logs/

Read dependencies: Fullscreen/Card/Button conversion, retained binding/view
bridge, FocusManager and Gallery events/render/hit paths. Regression-first
locked UI/Gallery checks, touched strict Clippy, actual CPU profile. Workspace
checks/tests and Gallery debug/release builds at slice milestone; reuse target/.
Human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen readers
NOT_RUN. Owner authorizes merge commit after local acceptance, all hosted CI
PASS and GitHub MERGEABLE, without waiting for Codex review.
