# ROADMAP 037 — UI minimal feature dependencies

2026-11-11 Asia/Taipei. P2 UI-FEATURES-037. Base
`72c0f7914294b8634d14f4ead1e40b8c6e792964`, fetched master after PR #45 merged
with all ten hosted CI checks PASS. Resumed as the first slice of run 032;
preserve the existing source commit and uncommitted documentation. Execution
single-agent-role-passes (Lead/Architecture/Rust Core/Verification), sole writer,
author self-review; independent review NOT_RUN. Owner authorizes PR/merge after
local tests, all hosted CI PASS and MERGEABLE.

REPRODUCED in ROADMAP 036, including its pre-Paper baseline: layout-only fails
because Sidebar/BottomNavigation import foundation IconName and Form builds an
inputs::form_message. Inputs itself imports foundation IconName. P5 now records
this open automatable build issue. Prioritize it over DataList density.

Acceptance: inputs alone enables foundations; layout alone enables inputs and
therefore foundations, matching actual source dependencies. Existing public
builders work from an integration-test consumer with either flag and no
default features. Test icon activation and Form validation content. Existing
full/default behavior remains accepted. Add hosted minimal-feature tests
without weakening any current gate. No public API, widget behavior or version/
lockfile changes; document the dependency closure in SDK guide/changelog.

Exact sole-writer leases: crates/acme-ui/Cargo.toml,
crates/acme-ui/tests/minimal_features.rs, .github/workflows/ci.yml,
docs/guides/SDK_GUIDE.md, docs/CHANGELOG.md, docs/status/todos.md,
docs/agent-team/ROADMAP.md,
docs/team/roadmap-037/{BRIEF.md,REPORT.md,evidence.json,gallery-feature-smoke.png},
docs/team/roadmap-037/logs/* (generated). UI manifest and CI single-writer.
No lockfile, root manifest, Scene or platform changes.

Checks: initial inputs/layout RED; locked no-default inputs/layout/base and
all-feature UI tests, strict minimal and all-feature touched Clippy; Gallery
suite and actual native CPU smoke of unchanged Paper demo (no new UI flow);
actual CPU profile; workspace check/test at slice milestone. Reuse target;
no release/performance run, cargo clean or other repository. NOT_RUN: human
Gallery, Windows 注音 IME, physical DPI/touch, real GPU and screen readers.
Rollback: revert this batch. Other standalone optional features are not audited
or certified by the inputs/layout checks.
