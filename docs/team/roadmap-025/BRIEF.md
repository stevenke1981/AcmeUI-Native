# ROADMAP 025 — Mobile banner acceptance

2026-11-10, Asia/Taipei. Task MOBILE-BANNER-025, priority P2. Base
`51022116707860313010e54adcd4d60fd250d93a`, fetched master/PR #33 merge.
Branch `sdk/roadmap-025-mobile-banner`. Execution **single-agent-role-passes**
(Lead, UI/UX, Verification), sole writer, author self-review; independent review
NOT_RUN. Owner explicitly authorizes PR creation and merge after local checks,
all hosted CI PASS and GitHub MERGEABLE; no Codex review wait or release.

OBSERVED: banner has icon/text and optional action/dismiss, but no full-width
surface or wrapping. Buttons use constant keys across instances. Gallery lacks
banner examples. Existing tests only check Row kind/child counts.

Acceptance:
1. Preserve builder/public fields and four tones. Use semantic soft status
   backgrounds, full available width and wrapping Unicode text; no widget colours
   or new public API. Keep icon and optional 44 px actions separate from text.
2. Derive action/dismiss keys from instance key. Verify independent messages,
   optional/public-field combinations, real narrow/wide layout, retained identity
   across action changes and keyed instance reorder.
3. Gallery: four tones, bilingual long text, action/dismiss/restore state, actual
   pointer dispatch tests, themed/clipped paint and native CPU window capture.
4. Run locked UI all-feature/mobile-only and Gallery tests, strict touched/mobile
   Clippy, formatting, actual verify_sdk.py CPU profile. Workspace check/test at
   completed-slice milestone. Record exact SHA, dirty state, argv, timestamps,
   OS/toolchain, exit status and raw log hashes. Review before push/PR/CI/merge.

Exact sole-writer leases (no parallel workers):
- `crates/acme-ui/src/mobile/mobile_banner.rs`
- `crates/acme-ui/src/mobile/mobile_banner_tests.rs` (new)
- `apps/acme-gallery/src/main.rs` (shared, serialized)
- `apps/acme-gallery/src/mobile_banner.rs` (new private Gallery helper)
- `apps/acme-gallery/src/mobile_banner_tests.rs` (new)
- `docs/status/todos.md`
- `docs/agent-team/ROADMAP.md`
- `docs/team/roadmap-025/BRIEF.md`
- `docs/team/roadmap-025/REPORT.md`
- `docs/team/roadmap-025/evidence.json`
- `docs/team/roadmap-025/logs/` (generated files enumerated in evidence)
- `docs/team/roadmap-025/gallery-mobile-banners.png`
- `docs/team/roadmap-025/gallery-mobile-banner-dismissed.png`

Read dependencies: spec/plan/team workflow/roles/ownership, todos/P3/roadmap,
021/024 reports; banner/container/Label/style/retained/layout; Gallery render and
event paths; CI/verification runner. No manifest/Scene/platform contract, runtime
timer, accessibility-role expansion, release or speculative performance work.
Manual human Gallery, Windows 注音 IME, physical DPI/touch, GPU/recovery and
screen readers remain NOT_RUN. Rollback: revert this bounded PR.
