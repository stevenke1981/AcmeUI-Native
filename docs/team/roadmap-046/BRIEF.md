# ROADMAP 046 — Tour conversion bounds

2026-11-11 Asia/Taipei. Base `9b340b3880796aa7b982b2023f3dc0aef5bb0214`,
after PR #54 merged with ten hosted checks PASS and MERGEABLE. Third and final
slice of this run. Select unfinished P5 overlay-builder conversion coverage.
HYPOTHESIS from source: current=usize::MAX overflows in last-step arithmetic;
empty/out-of-range steps can still emit Previous/Finish despite having no
current step. Reproduce actual conversion before implementation. Priority P1:
public builder input can panic or expose actions for nonexistent content.

Acceptance: empty or invalid current yields the existing keyed Elevated Card
without content or navigation, and never overflows. Valid first/middle/last
steps preserve counter/title/optional description, Previous/Next/Finish labels,
current typed messages, no-callback behavior and host-owned state. Cover real
layout and root identity through valid/invalid transitions. Add a Gallery
controlled inline tour, restart/stale-message guards, pointer and theme/scale/
clip regressions, and actual X11 CPU-window input evidence. No public API or
dependency expansion. Target spotlight/scrolling, full overlay semantics,
navigation focus continuity and human hardware validation remain separate.

Execution single-agent-role-passes: Lead, Architecture, UI/UX, Verification;
sole writer, author self-review, independent review NOT_RUN. Reuse loaded role
contracts, spec/plan/todos, workflow and ownership. Exact single-writer leases:
crates/acme-ui/src/overlay/tour.rs,
crates/acme-ui/src/overlay/tour_tests.rs,
apps/acme-gallery/src/main.rs, apps/acme-gallery/src/tour.rs,
apps/acme-gallery/src/tour_tests.rs, docs/status/todos.md,
docs/agent-team/ROADMAP.md, docs/team/roadmap-046/BRIEF.md,
docs/team/roadmap-046/REPORT.md, docs/team/roadmap-046/evidence.json,
docs/team/roadmap-046/gallery-tour-first.png,
docs/team/roadmap-046/gallery-tour-finished.png and generated scripts/logs
under docs/team/roadmap-046/logs/.

Read dependencies: Tour/Card/Button conversion, retained binding/view bridge,
Gallery event/hit/render paths. Regression-first locked UI/Gallery tests,
formatting and strict Clippy; actual CPU profile. Workspace check/tests and
Gallery debug/release builds at slice milestone only. Reuse target/. Human
Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen readers NOT_RUN.
Owner authorizes merge commit after local acceptance, all hosted CI PASS and
GitHub MERGEABLE, without waiting for Codex review. Rollback: revert the PR.

Native inspection follow-up: the scrolled first-step capture shows clipped
Gallery toolbar text, while the finished-state capture shows normal toolbar
text. The layout-shrink hypothesis was NOT_REPRODUCED: a real-frame geometry
probe passes with the toolbar still 40 px at both heights and Tour states.
Keep that geometry guard, without a toolbar layout change. The paint difference
remains unresolved and is recorded as a follow-up, not visual acceptance.
Preserve the initial capture under logs/initial-tour-first.png,
logs/initial-tour-finished.png, logs/native-window-initial.json and
logs/native-window-initial.txt. The check named toolbar-red is a passing
hypothesis probe, not a reproduced regression.
