# ROADMAP 043 — AboutDialog Close message conversion

2026-11-11 Asia/Taipei. Base `bc7304d8c971083b343cba2b77314c0e7de75dca`,
after PR #51 merged with ten hosted checks PASS and MERGEABLE. Third and last
slice of this run. Execution single-agent-role-passes: Lead, Architecture,
UI/UX, Verification; sole writer, author self-review, independent review NOT_RUN.
Spec/plan/todos, role contracts and delivery workflow loaded during 041.

Select the next bounded P5 builder-conversion case. OBSERVED: AboutDialog
stores on_close but always constructs an unbound Close Button; existing tests
check its label only. Acceptance: bind the configured typed Close message after
conversion, preserve the passive fallback, keys/card/label structure and all
existing public signatures/fields. Verify current messages and Close identity
through optional description/credits changes. Add a Gallery inline about-card
with explicit host-owned hide/restore and stale-close rejection.

Exact serialized leases: crates/acme-ui/src/overlay/about_dialog.rs,
crates/acme-ui/src/overlay/about_dialog_tests.rs,
apps/acme-gallery/src/main.rs, apps/acme-gallery/src/about_dialog.rs,
apps/acme-gallery/src/about_dialog_tests.rs, docs/status/todos.md,
docs/agent-team/ROADMAP.md, docs/team/roadmap-043/BRIEF.md,
docs/team/roadmap-043/REPORT.md, docs/team/roadmap-043/evidence.json,
docs/team/roadmap-043/gallery-about-open.png,
docs/team/roadmap-043/gallery-about-closed.png and generated logs/*.

Checks: regression-first converted-message test, locked UI/Gallery tests,
strict touched Clippy/fmt, actual CPU profile, native CPU pointer captures;
workspace gates only at the slice milestone. No public API/dependency/manifest,
Scene or platform change. The legacy open field's rendering policy, real modal
placement/focus trapping, Escape handling and platform accessibility adapter
remain separate; this is a Close message repair, not full dialog acceptance.
Human Gallery, Windows 注音 IME, physical DPI/touch, GPU/screen readers NOT_RUN.
Owner authorizes merge commit only after local checks, all hosted CI PASS and
MERGEABLE. Rollback: revert this PR.
