# ROADMAP 035 — Fractional full sizing

2026-11-11 Asia/Taipei. P2 FULL-SIZING-035. Base
`7d385ebac8f0285a10b404218143bd802c4a12e7`, fetched clean master;
PRs through #43 merged and master hosted CI PASS. Execution:
**single-agent-role-passes** (Lead/Rust Core/UI/UX/Verification), sole writer,
author self-review; independent review NOT_RUN. Owner authorizes PR and merge
only after local targeted checks, all hosted CI PASS and MERGEABLE.

OBSERVED: Styled::{w_full,h_full}, Tailwind {w,h}::{full,screen} and
Button.full_width emit Percent(100.0); LayoutEngine consumes fractional
percentages without division. Existing mobile-button test codifies the wrong
unit. Reproduce before fixing; preserve the engine convention and all APIs.

Acceptance: full helpers resolve to the containing content box (240 px root
regression; padded child at multiple viewport sizes), in both dimensions;
Tailwind screen remains a parent-relative alias, explicitly documented (there
is no viewport unit). Button.full_width uses the same fraction and explicit
style overrides retain precedence. Gallery Design System displays a bounded
240 x 96 full-size example; real layout, theme/clip tests and automated native
CPU evidence. Document changed geometry without claiming a new public API.

Exact sole-writer leases: crates/acme-style/src/{styled.rs,tailwind.rs},
crates/acme-style/tests/full_sizing.rs, crates/acme-layout/src/lib.rs (Length
variant documentation only), crates/acme-widgets/src/lib.rs,
crates/acme-ui/src/mobile/mobile_button.rs (tests only),
apps/acme-gallery/src/{main.rs,style_sizing_tests.rs}, docs/CHANGELOG.md,
docs/status/todos.md, docs/agent-team/ROADMAP.md,
docs/team/roadmap-035/{BRIEF.md,REPORT.md,evidence.json,gallery-full-sizing.png},
docs/team/roadmap-035/logs/* (generated). No manifest/Scene/platform changes.
Shared Gallery entrypoint and widget enum remain single-writer.

Checks: regression RED/GREEN; locked style/layout/widgets/UI all-feature and
Gallery tests; touched strict all-target Clippy and fmt; actual CPU profile;
workspace check/test at completed-slice milestone. Reuse target; no release
build, cargo clean or performance claim. NOT_RUN: human Gallery, Windows 注音
IME, physical DPI/touch, GPU and screen readers. Rollback: revert this batch.
