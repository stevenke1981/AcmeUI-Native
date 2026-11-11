# ROADMAP 042 — SpeedDial typed action conversion

2026-11-11 Asia/Taipei. Base `097fb7de73ce04d97d2f993ecaa69aaf06fea416`
(PR #50 merged after ten hosted checks PASS and MERGEABLE).
Execution single-agent-role-passes: Lead, Architecture, UI/UX, Verification;
sole writer, author self-review, independent review NOT_RUN.

Select P5 builder-conversion coverage from docs/status/todos.md. OBSERVED:
SpeedDial accepts on_action(fn(usize)->M), but its open actions always become
Labels and the existing test only checks the stored function. Reproduce after
conversion. Acceptance: an open dial with an action callback produces distinct
keyed Buttons carrying the callback's indexed messages and optional icons;
closed dials expose only the existing toggle; no callback preserves passive
labels. Preserve existing builder signatures/fields, root and toggle identity,
index order and host-controlled open state. Test actual conversion, finite
layout, retained IDs through close/open and independent keyed instances.

Exact single-writer leases: crates/acme-ui/src/overlay/speed_dial.rs,
crates/acme-ui/src/overlay/speed_dial_tests.rs,
apps/acme-gallery/src/main.rs, apps/acme-gallery/src/speed_dial.rs,
apps/acme-gallery/src/speed_dial_tests.rs, docs/status/todos.md,
docs/agent-team/ROADMAP.md, docs/team/roadmap-042/BRIEF.md,
docs/team/roadmap-042/REPORT.md, docs/team/roadmap-042/evidence.json,
docs/team/roadmap-042/gallery-speed-dial-open.png,
docs/team/roadmap-042/gallery-speed-dial-action.png and generated logs/*.

Read spec/plan/todos and role contracts already loaded in 041; read existing
SpeedDial/FAB and newer Gallery host. Verify regression-first, locked UI and
Gallery tests, strict touched Clippy/fmt, actual CPU profile; workspace gates
only at this slice milestone. Add an actual native CPU Gallery input capture.
No new public API, dependency, manifest, Scene or platform change. Directional
expansion, animations, global overlay placement and automatic close policy stay
separate. Human Gallery, IME, physical DPI/touch, GPU and screen readers NOT_RUN.
Owner authorizes merge commit only after local tests, all hosted CI PASS and
MERGEABLE. Rollback: revert this PR.
