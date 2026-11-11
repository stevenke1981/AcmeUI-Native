# ROADMAP 044 — BottomNavigation typed selection conversion

2026-11-11 Asia/Taipei. Base `aeeda7c57d5d697b1c39cedf3906730fbb23de83`.
Select the unfinished P5 layout-builder conversion coverage in todos.
OBSERVED: BottomNavigation stores on_select(fn(usize)->M), but converts every
action to a Label. Reproduce the lost message after actual conversion.
Priority P1: configured navigation cannot dispatch the host's selection.

Acceptance: configured actions become instance-keyed Buttons carrying their
zero-based selection messages. Preserve passive Labels without a callback,
existing icon/label/selected-marker text, fixed-height Row, public fields and
signatures, and host-controlled selection. Retain root/action IDs across
selection, label visibility and keyed instance reorders. Cover empty actions,
out-of-range selection, real layout, focus eligibility, theme/scale/clip paint
and Gallery pointer selection including repeat and invalid-index rejection.
Capture an actual automated X11 CPU Gallery window before/after selection.

Execution `single-agent-role-passes`: Lead, Architecture, UI/UX, Verification;
sole writer, author self-review, independent review NOT_RUN. Loaded spec/plan,
todos, team workflow, ownership and these role contracts. No parallel workers.

Exact single-writer leases:
- crates/acme-ui/src/layout/bottom_navigation.rs
- crates/acme-ui/src/layout/bottom_navigation_tests.rs
- apps/acme-gallery/src/main.rs
- apps/acme-gallery/src/bottom_navigation.rs
- apps/acme-gallery/src/bottom_navigation_tests.rs
- docs/status/todos.md
- docs/agent-team/ROADMAP.md
- docs/team/roadmap-044/BRIEF.md
- docs/team/roadmap-044/REPORT.md
- docs/team/roadmap-044/evidence.json
- docs/team/roadmap-044/gallery-bottom-navigation-home.png
- docs/team/roadmap-044/gallery-bottom-navigation-search.png
- docs/team/roadmap-044/logs/check.py
- docs/team/roadmap-044/logs/verify.py
- docs/team/roadmap-044/logs/capture.py
- Generated check logs, CPU profile report and native-window record under logs/.

Read dependencies: Button, retained binding/view bridge, Gallery layout/render/
hit collection and existing SpeedDial tests. No public API/dependency expansion,
manifest/lockfile, Scene or platform changes. Full navigation semantics, generic
keyboard host and global viewport anchoring are separate. Human Gallery,
Windows 注音 IME, physical DPI/touch, GPU and screen readers NOT_RUN.

Checks: regression-first locked focused tests; locked touched UI/Gallery suites,
formatting, strict Clippy and actual `python scripts/verify_sdk.py --profile cpu`.
Workspace check/tests and Gallery builds only at slice milestone. Reuse target/.
Owner authorizes merge commit after local acceptance, all hosted checks PASS
and GitHub MERGEABLE; do not wait for Codex review. Rollback: revert this PR.
