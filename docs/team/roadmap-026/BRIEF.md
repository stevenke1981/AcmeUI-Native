# ROADMAP 026 — Mobile notification acceptance

2026-11-10, Asia/Taipei. Task MOBILE-NOTIFICATION-026, priority P2. Base
`9728f967df4526b3e275d30a9bd629e6bb86116c`, fetched merge of PR #34 after
all ten hosted checks PASS. Branch `sdk/roadmap-026-mobile-notification`.
Execution **single-agent-role-passes** (Lead, UI/UX, Verification), sole writer,
author self-review, independent review NOT_RUN. Owner explicitly authorizes
merge only after local targeted checks, all hosted CI PASS and MERGEABLE.

OBSERVED: notification already uses an elevated Card with optional body/action/
dismiss. Text is unwrapped; button keys are constants; no Gallery scenarios or
auto-dismiss hint/lifecycle demonstration. The builder has no duration property.
Existing platform background polling supports host-owned deadlines without
platform changes. No new public timer API is necessary.

Observed during real-layout verification: Card ignores its existing Style in
both layout paths (180 px available but root sizes to 179 px intrinsic width).
Add a bounded Card layout-style fix and independent no-text/context regressions;
include locked widgets tests/strict Clippy in the touched-package gates.

Acceptance:
1. Preserve constructor/public fields; wrap title/body and fill available width
   using existing semantic elevated Card. Key content/footer/action/dismiss by
   instance; independent 44 px action targets in a footer below text.
2. Test empty/Unicode/public-field/optional combinations, narrow/wide real layout,
   no overlap and retained title/content/dismiss identity across optional changes
   and keyed instance reorder.
3. Gallery: title-only, narrow long body and interactive action/dismiss examples.
   Show-for-five-seconds control and explicit lifetime hint; host owns deadline
   and removes the card. Test exact deadline, restart, cancellation and actual
   background callback/pointer dispatch. Poll only while a deadline is pending.
4. Capture/inspect actual CPU window and real timed dismissal on isolated Xvfb.
   Run locked all-feature/mobile-only UI/Gallery tests, strict touched/mobile
   Clippy, formatting, actual CPU profile and workspace check/test at milestone.
   Preserve exact SHA/dirty state, argv, timestamps, OS/toolchain, exits/log hashes.

Exact sole-writer leases:
- `crates/acme-ui/src/mobile/mobile_notification.rs`
- `crates/acme-ui/src/mobile/mobile_notification_tests.rs` (new)
- `crates/acme-widgets/src/foundations/card.rs`
- `crates/acme-widgets/src/lib.rs` (only the two Card layout arms)
- `apps/acme-gallery/src/main.rs` (shared, serialized)
- `apps/acme-gallery/src/mobile_notification.rs` (new private helper)
- `apps/acme-gallery/src/mobile_notification_tests.rs` (new)
- `docs/status/todos.md`
- `docs/agent-team/ROADMAP.md`
- `docs/team/roadmap-026/BRIEF.md`
- `docs/team/roadmap-026/REPORT.md`
- `docs/team/roadmap-026/evidence.json`
- `docs/team/roadmap-026/logs/` (generated files enumerated in evidence)
- `docs/team/roadmap-026/gallery-mobile-notifications.png`
- `docs/team/roadmap-026/gallery-mobile-notification-expired.png`

Read dependencies: AGENTS/team/roles/ownership/spec/plan/todos/P3/roadmap and
latest 025 report; notification/Card/Label/style/retained/layout, Gallery render/
event paths, Application background polling and hosted CI/verification runner.
No manifest, dependency, Scene/platform changes, public API expansion, typed
live-region semantics, native toast service, release or performance rewrite.
The component remains stateless; lifetime belongs to its host. Manual human
Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader checks are
NOT_RUN. Rollback: revert this bounded PR.
