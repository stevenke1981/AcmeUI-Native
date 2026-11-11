# ROADMAP 041 — Retained container focus and typed activation

2026-11-11 Asia/Taipei. Base `6b8f0f61daa34987d1fb3e3809b401fcb31bafee`,
after PR #49 merged with all ten hosted checks PASS. Branch
`sdk/roadmap-041-container-actions`. Execution single-agent-role-passes (Lead,
Architecture, Rust Core, Accessibility, UI/UX, Verification), sole writer,
author self-review; independent review NOT_RUN. Owner authorizes merge only
after local targeted tests, all hosted CI PASS and MERGEABLE.

OBSERVED: container on_click messages are omitted from retained focus and
AccessKit semantics; existing coordinate-only routing cannot return a typed
message. Original Gallery collects container hits but its painter does not
consume their indices, shifting subsequent Button states. Scope: existing
Row/Column/Stack messages, no new widget variant or disabled-container API.

Acceptance: active containers register focus, expose Button/Focus/Click and a
name from passive descendant labels, keeping child controls independent.
Resolve Click/Activate to current typed messages with a complete retained
binding; reject unsupported, passive, disabled Button, missing/synthetic/stale
targets and partial bindings. Preserve identity/focus across keyed reorder and
remove focus eligibility when the handler disappears. Original Gallery Focus
page demonstrates keyboard activation through this API, passive containers,
nested Buttons, themed focus rings and matching hit/paint indices.

Exact serialized leases: `crates/acme-widgets/src/view_bridge.rs`,
`crates/acme-widgets/src/container_focus_tests.rs`,
`crates/acme-accessibility/src/lib.rs`,
`crates/acme-accessibility/src/typed_actions.rs`,
`crates/acme-accessibility/src/tests_container_actions.rs`,
`apps/gallery/src/main.rs`, `apps/gallery/src/pages/accessibility.rs`,
`apps/gallery/src/render/content.rs`, `apps/gallery/src/container_actions_tests.rs`,
`docs/adr/0006-container-activation.md`, `docs/status/todos.md`,
`docs/agent-team/ROADMAP.md`, `docs/team/roadmap-041/{BRIEF.md,REPORT.md,evidence.json,gallery-container-focus.png,gallery-container-activated.png}`
and generated `logs/*` enumerated in evidence.

Read dependencies: team/spec/plan/todos/roadmap and 027/028/040 reports,
container/view/binding, FocusManager, AccessKit walk/routing and original
Gallery input/hit/focus/paint paths. Checks: regression-first, locked touched
and direct-dependent tests, strict Clippy/fmt, actual CPU profile, workspace
check/tests at slice milestone and automated native CPU keyboard evidence.
No dependency/manifest/Scene/platform changes, global event framework rewrite
or performance claim. Platform AccessKit adapter, human Gallery, Windows
注音/physical DPI/touch/GPU/screen-reader acceptance remain NOT_RUN.
Rollback: revert this PR.

Lease refinement after tracing actual frame paint: include
`apps/gallery/src/render/frame.rs`. Its page painter currently walks the entire
root from hit index 11, repainting chrome and offsetting page states by another
11; the new actual-frame focus-ring test will reproduce this together with
unconsumed container indices. Restrict that pass to the actual page subtree
and apply its scroll clip. This is necessary for trustworthy Gallery evidence.

Continuation refinement: exact lease also includes
`apps/gallery/src/render/hit_test.rs` and
`docs/team/roadmap-041/gallery-container-dark.png`. The expanded actual-frame
regression reproduced eight scrolled focus regions instead of five:
`collect_data_widget_hits` was registering the three active containers a
second time after `collect_hit_regions`. Remove that duplicate registration
while preserving recursive data-widget traversal. This keeps one keyboard
stop per control and prevents duplicate targets from overriding child hits.
The unrelated deletion of CPU `coverage/analytic_tests.rs` was restored with
`git checkout --` for that single file; it remains required by its parent
module and is outside this slice. No reset/stash/drop/force-push was used.
