# ROADMAP 047 — Gallery toolbar paint investigation

2026-11-12 Asia/Taipei. Base `5f3c13af3bfc942357753ce658d36560750b88d9`.
Select the unresolved ROADMAP 046 toolbar observation in docs/status/todos.md.
Priority P2; HYPOTHESIS: scrolled Tour frames can displace or clip toolbar text
while the toolbar container remains 40 px. Reproduce before choosing a fix.

Acceptance: toolbar controls and text remain within the fixed toolbar through
scrolling and Tour first/middle/finished transitions. Cover actual frame layout,
paint and pointer interaction, light/dark and scale changes, and capture an actual
X11 CPU Gallery window. Preserve themes, messages and public APIs. No hardware
acceptance or full overlay/IME claims. Rollback: revert this PR.

Execution: single-agent-role-passes (Lead, UI/UX, Rendering, Verification), sole
writer; author self-review, independent review NOT_RUN. Loaded role contracts,
spec/plan/todos, workflow and ownership. Exact single-writer leases:
- apps/acme-gallery/src/main.rs
- apps/acme-gallery/src/toolbar_tests.rs
- docs/status/todos.md
- docs/status/final.md
- docs/agent-team/ROADMAP.md
- docs/team/roadmap-047/BRIEF.md
- docs/team/roadmap-047/REPORT.md
- docs/team/roadmap-047/evidence.json
- docs/team/roadmap-047/gallery-toolbar-light-first.png
- docs/team/roadmap-047/gallery-toolbar-light-finished.png
- docs/team/roadmap-047/gallery-toolbar-dark-first.png
- docs/team/roadmap-047/gallery-toolbar-dark-finished.png
- docs/team/roadmap-047/gallery-toolbar-light-detail.png
- docs/team/roadmap-047/gallery-toolbar-dark-detail.png
- generated scripts, JSON records and raw logs under docs/team/roadmap-047/logs/

Read dependencies: Gallery retained layout/render/hit paths, text shaping,
Frame-to-Scene bridge and CPU renderer. Targeted locked Gallery tests and strict
Clippy, actual CPU profile; workspace check/tests and release Gallery build at
the slice milestone. Reuse target/. Human Gallery, Windows 注音 IME, physical
DPI/touch, GPU and screen-reader validation NOT_RUN. Owner authorizes merge
commit after local acceptance, all hosted CI PASS and GitHub MERGEABLE.

Investigation result: NOT_REPRODUCED. The stored ROADMAP 046 first/finished
PNG toolbar regions (x=200..1100, y=0..40) are byte-identical. New native
captures and expanded layout/glyph probes agree. Earlier visual descriptions
of differing toolbar clipping are unsupported by those pixel data. Scope is
now tests/evidence/documentation only; do not claim a runtime fix. Initial
probe checks PASS. The first invariance assertion incorrectly included the
sidebar selection highlight; its FAIL is test-authoring evidence, not a
product failure. Exclude sidebar paint from the toolbar comparison.
