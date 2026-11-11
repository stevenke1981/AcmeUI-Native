# ROADMAP 039 — Gallery theme-pack swatches

2026-11-11 Asia/Taipei. P2 THEME-SWATCHES-039. Base
`c524c69ec35ffac70703b02f4f61cfc4d8b89890`, fetched master after PR #47
merged with all ten hosted CI checks PASS. Third/final slice of run 032.
Execution single-agent-role-passes (Lead/Architecture/UI/UX/Verification),
sole writer and author self-review; independent review NOT_RUN. Owner
authorizes PR/merge after local checks, all hosted CI PASS and MERGEABLE.

OBSERVED: P12 Gallery theme-pack swatches is still unchecked. Design System
only shows current-theme semantic samples, not all registered packs. Journey:
compare the ten pack palettes in one place; existing toolbar light/dark toggle
updates every preview, while cycle-theme continues selecting the application
palette. Acceptance: registry-driven pack names and primary/surface/foreground/
background samples, no copied/hardcoded palette colors; stable keyed identities
across host theme/mode changes; bounded layout at 360/800 px, three logical
scales, correct live paint colors and scroll clips. Native light/dark captures.

No public API/ADR needed: Gallery-only composition of existing Card/Row/Label,
theme registry and toolbar state; no new control/action, semantic role,
theme palette, manifest, Scene or platform change. No new renderer primitive.

Sole-writer exact leases: apps/acme-gallery/src/main.rs,
apps/acme-gallery/src/theme_pack_examples.rs, docs/CHANGELOG.md,
docs/status/todos.md, docs/agent-team/ROADMAP.md,
docs/team/roadmap-039/{BRIEF.md,REPORT.md,evidence.json,gallery-packs-light.png,gallery-packs-dark.png},
docs/team/roadmap-039/logs/* (generated). Shared Gallery entrypoint serialized.
Read dependencies: theme registry/palettes, Card layout, retained binding and
Gallery paint/clip paths, 038 report and the P12 todo.

Checks: test-first missing-preview API RED, registry/layout/paint/identity
regressions, locked Gallery suite, touched fmt/strict Clippy, actual CPU
verification profile, workspace check/tests at completed-slice milestone,
debug build and native Xvfb CPU captures. Reuse target; no release/performance
run. NOT_RUN: human Gallery, Windows 注音 IME, physical DPI/touch, real GPU and
screen readers. Rollback: revert this batch.
