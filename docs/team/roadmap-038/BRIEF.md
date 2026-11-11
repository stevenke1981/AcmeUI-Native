# ROADMAP 038 — DataList density

2026-11-11 Asia/Taipei. P2 DATA-LIST-038. Base
`ac6171fa02787b9c84ff074c61ac6e723f87628d`, fetched master after PR #46
merged with all ten hosted CI checks PASS. Execution single-agent-role-passes
(Lead/Architecture/UI/UX/Verification), sole writer and author self-review;
independent review NOT_RUN. Owner authorizes PR/merge after local checks,
all hosted CI PASS and MERGEABLE.

OBSERVED: P9 DataList density is still unchecked. Existing builder has only
orientation and fixed spacing. User journey: compare identical key/value
content at compact/default/relaxed spacing, for vertical and horizontal items.
Acceptance: one density enum/setter, preserve all default spacing and content;
compact takes less space and relaxed more in real layout; root/content retained
IDs survive density changes and keyed instance reorders. Gallery shows all six
variants with live theme colors and preserved clips. No widget/Scene/platform
variant, runtime interaction, wrapping policy or semantic-role expansion.

Sole-writer exact leases: crates/acme-ui/src/foundations/data_list.rs,
crates/acme-ui/src/foundations/data_list_tests.rs,
apps/acme-gallery/src/main.rs, apps/acme-gallery/src/data_list_examples.rs,
docs/adr/0004-data-list-density.md, docs/CHANGELOG.md,
docs/status/todos.md, docs/agent-team/ROADMAP.md,
docs/team/roadmap-038/{BRIEF.md,REPORT.md,evidence.json,gallery-density.png},
docs/team/roadmap-038/logs/* (generated). Shared Gallery entrypoint serialized.
Read dependencies: container/layout/retained bridges, Gallery paint and theme
tokens, existing 037 acceptance. No manifest, lockfile, Scene/platform changes.

Checks: test-first API RED, focused real-layout/identity tests, locked UI
foundations-only and all-feature tests, Gallery tests, touched fmt and strict
Clippy, actual CPU profile, workspace check/tests at completed-slice milestone,
debug Gallery build and native Xvfb CPU capture. Reuse target; no release or
performance claims. NOT_RUN: human Gallery, Windows 注音 IME, physical DPI/touch,
real GPU and screen readers. Rollback: revert this batch.
