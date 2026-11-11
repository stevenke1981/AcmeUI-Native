# ROADMAP 045 — PageHeader action identity

2026-11-11 Asia/Taipei. Base `b8df270e1b124412f854df04f6c789b72ea036cf`,
after PR #53 merged with ten hosted checks PASS and MERGEABLE.
Select unfinished P5 layout-builder conversion coverage. HYPOTHESIS from source:
the unkeyed action Row changes its generated key when an optional description
is inserted/removed, replacing even explicitly keyed action Buttons and losing
focus. Reproduce on the real retained tree before assigning a fix.

Acceptance: preserve action Row/Button IDs and focused Button across optional
description changes and keyed header reorders; retain current typed messages,
primary/secondary variants, passive action behavior, no-action composition,
public fields/signatures and visual/typography policy. Add a Gallery Save/reset
and description-toggle scenario with pointer and theme/scale/clip regressions,
and actual X11 CPU-window evidence. No new public API or dependency.

Execution single-agent-role-passes: Lead, Architecture, UI/UX, Verification;
sole writer, author self-review, independent review NOT_RUN. Reuse loaded role
contracts, spec/plan/todos, ownership and workflow. Priority P1 if reproduced:
loading a subtitle invalidates a focused primary action.

Exact single-writer leases: crates/acme-ui/src/layout/page_header.rs,
crates/acme-ui/src/layout/page_header_tests.rs,
apps/acme-gallery/src/main.rs, apps/acme-gallery/src/page_header.rs,
apps/acme-gallery/src/page_header_tests.rs, docs/status/todos.md,
docs/agent-team/ROADMAP.md, docs/team/roadmap-045/BRIEF.md,
docs/team/roadmap-045/REPORT.md, docs/team/roadmap-045/evidence.json,
docs/team/roadmap-045/gallery-page-header-description.png,
docs/team/roadmap-045/gallery-page-header-no-description.png and generated
scripts/logs under docs/team/roadmap-045/logs/.

Read dependencies: PageHeader conversion, view bridge, retained binding,
FocusManager and Gallery layout/hit/render paths. Targeted locked UI/Gallery
tests, formatting/strict Clippy, actual CPU profile; workspace check/tests and
Gallery debug/release builds at slice milestone. Reuse target/. Existing
PageHeader typography/color policy, generic keyboard hosting, human Gallery,
Windows 注音 IME, physical DPI/touch, GPU and screen readers remain separate /
NOT_RUN. Owner authorizes merge commit only after local acceptance, all hosted
CI PASS and GitHub MERGEABLE, without waiting for Codex review. Rollback: revert.
