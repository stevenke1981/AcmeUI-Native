# ROADMAP 040 — Spinner rotation and reduced motion

2026-11-11 Asia/Taipei. Base `b881ecbde71cc870b257883ceb912a31ebdb3f52`;
branch `sdk/roadmap-040-spinner-motion`. Execution single-agent-role-passes
(Lead, Architecture, UI/UX, Accessibility, Verification), sole writer and author
self-review. Independent review NOT_RUN. Owner authorizes PR and merge only
after local tests, all hosted CI PASS and GitHub MERGEABLE.

OBSERVED: foundation spinner is a static glyph; mobile_loader forwards no
clock/policy. Application already supports bounded background polling. Scope:
one clock-driven, backend-neutral spinner composition and Gallery scheduler.

Acceptance: themed rotating seven-dot indicator with fixed bounds, one-second
period and deterministic reduced-motion pose; host-owned Duration/policy API
on spinner and loader, unchanged existing signatures; retained identities across
phase/policy/message/reorder changes; Gallery live toggle, stop spinner polling
off Mobile or in reduced motion without stopping notification/Snackbar expiry;
real layout, clips and light/dark paint regressions. No Scene/platform/dependency
changes, OS preference detection, animation framework rewrite or perf claims.

Exact serialized leases: `crates/acme-ui/src/foundations/spinner.rs`,
`crates/acme-ui/src/foundations/spinner_tests.rs`,
`crates/acme-ui/src/mobile/mobile_loader.rs`,
`crates/acme-ui/src/mobile/mobile_loader_tests.rs`,
`apps/acme-gallery/src/main.rs`, `apps/acme-gallery/src/spinner_motion.rs`,
`apps/acme-gallery/src/mobile_loader_tests.rs`, `docs/adr/0005-spinner-motion.md`,
`docs/status/todos.md`, `docs/agent-team/ROADMAP.md`,
`docs/team/roadmap-040/{BRIEF.md,REPORT.md,evidence.json,gallery-spinner.png,gallery-spinner-reduced.png}`
and generated `logs/*` enumerated in evidence.

Read dependencies: spec/plan/todos, 014 mobile and 019–021/037–039 reports,
team contracts, primitives/style/layout/retained/paint paths, polling and CI.
Checks: test-first regression, locked full/mobile UI and Gallery suites, touched
fmt/strict Clippy, actual CPU profile, workspace check/tests at slice milestone,
debug Gallery build and automated native CPU captures. Existing target reused.
Manual human Gallery, Windows 注音/physical DPI/touch/GPU/screen readers NOT_RUN.
Rollback: revert this PR. No release or production-readiness claim.
