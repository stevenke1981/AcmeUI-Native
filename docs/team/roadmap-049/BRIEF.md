# ROADMAP 049 — Anchor typed navigation conversion

2026-11-12 Asia/Taipei. Base `09d00899ea1fe1ecbeff89999e1b705c9835e18f` after PR #57.
Select unfinished P5 overlay/layout/chart builder conversion coverage.
OBSERVED: Anchor ignores on_navigate during conversion; rendered links remain
passive Labels. P1: configured navigation cannot emit its typed message.

Acceptance: currently rendered parent and immediate-child links emit their
current href through instance/path-keyed Buttons when on_navigate is configured.
Preserve passive Label text, active marker, indentation, Column composition,
public fields/signatures, and host-owned active_href. Retain focused action
identity across active href/callback updates and keyed instance reorder.
Add Gallery controlled selection/count demo, pointer and themed/scaled/clipped
real-layout checks, plus actual native CPU-window evidence.

Non-goals: automatic scrolling, URL handling, deeper nesting (currently omitted),
semantic navigation roles, generic keyboard host, or new public APIs/dependencies.
Human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen readers NOT_RUN.
Rollback: revert the PR. User authorizes merge commit after local acceptance,
all hosted CI PASS and GitHub MERGEABLE, without waiting for Codex review.

Execution: single-agent-role-passes (Lead, Architecture, UI/UX, Verification),
sole writer and author self-review; independent review NOT_RUN. Read spec/plan/
todos, team workflow/ownership and four role contracts. Exact sole-writer leases:
- crates/acme-ui/src/layout/anchor.rs
- crates/acme-ui/src/layout/anchor_tests.rs
- apps/acme-gallery/src/main.rs
- apps/acme-gallery/src/anchor.rs
- apps/acme-gallery/src/anchor_tests.rs
- docs/status/todos.md
- docs/status/final.md
- docs/agent-team/ROADMAP.md
- docs/team/roadmap-049/BRIEF.md
- docs/team/roadmap-049/REPORT.md
- docs/team/roadmap-049/evidence.json
- docs/team/roadmap-049/gallery-anchor-before.png
- docs/team/roadmap-049/gallery-anchor-after.png
- generated scripts/records/raw logs under docs/team/roadmap-049/logs/

Read dependencies: Button/Column conversion, retained binding/view bridge,
FocusManager, Gallery event/layout/render/hit paths. Regression-first locked
UI/Gallery checks, strict touched Clippy, actual CPU profile; workspace
check/tests and release compilation at slice milestone. Reuse target/.
