# ROADMAP 032 — Floating action button acceptance

Task FAB-032, P2, resumed 2026-11-11 Asia/Taipei. Base
`931c7b83646da8f3fdf96cc4a2a9f0f4b28fef7a` after fetching and fast-forwarding
the interrupted branch with its uncommitted FAB source preserved. PR #40 is
already merged and uses ROADMAP 031 for render reliability; the interrupted
FAB brief is moved to 032 without overwriting that report. No open PRs. Branch
sdk/roadmap-031-floating-action-button. Execution **single-agent-role-passes**
(Lead, UI/UX, Verification), sole writer, author self-review; independent review
NOT_RUN. Owner authorizes PR/merge after local tests, all hosted CI PASS and
MERGEABLE. No Codex review wait, release or other repository changes.

OBSERVED: overlay/float_button exists but returns an unpainted fixed 56 px Stack;
shape/placement have no effect, icon hides label, no mini/extended variants or
typed Button descendant. P4 FAB acceptance remains unchecked. Reuse this builder, not a second
FAB API. Placement is host-owned metadata; no global overlay manager rewrite.

Acceptance:
1. Mini 40 px / normal 56 px / extended 56 px high with icon and label, themed
   primary Button, circular or square shape, elevated Card surface. Keep default
   constructor, key, icon, label, shape, placement and on_click entrypoints.
   Add only the size selection needed by the documented contract. The previous
   placeholder Stack becomes an elevated Card with a typed Button descendant.
2. Real narrow/wide layout, optional icon/label/no-handler/public fields and
   keyed retained identity checks; no duplicate action targets.
3. Gallery examples dispatch actions through the existing Button host. Paint
   existing Button radius/border style, bounded text and semantic Card elevation;
   theme/scale/clip/paint-index tests plus actual native CPU capture.
4. Locked touched UI/Gallery tests, overlay+foundations feature checks, strict Clippy,
   actual CPU profile; workspace check/tests at this completed-slice milestone.

Exact sole-writer leases: crates/acme-ui/src/overlay/float_button.rs,
crates/acme-ui/src/overlay/float_button_tests.rs, apps/acme-gallery/src/main.rs,
apps/acme-gallery/src/fab.rs, apps/acme-gallery/src/fab_tests.rs,
docs/status/todos.md, docs/agent-team/ROADMAP.md,
docs/team/roadmap-032/{BRIEF.md,REPORT.md,evidence.json,gallery-mobile-fab.png},
docs/team/roadmap-032/logs/* (generated evidence only). Shared Gallery writes
serialized. No manifest/Scene/platform changes. Rollback: revert this batch.
Read dependencies: Button/Card/Style/layout/text, retained binding and Gallery
render/hit paths, theme shadows, 021/029/030 reports, AGENTS/spec/plan/team/roles.
Non-goals: viewport anchor placement, expandable groups, generic keyboard host,
animation, performance work. Manual human Gallery, Windows 注音 IME, physical
DPI/touch, GPU/recovery and screen readers NOT_RUN.

Lease extension after full Gallery reproduction: apps/acme-gallery/src/
mobile_{action,banner,chip,list_item,notification}_tests.rs, only assertions
for button text clips (now button-box intersection) and elevated Card paint
order (four theme shadow layers before the surface). Preserve viewport bounds
and nonempty glyph assertions. No component behavior rewrite in those files.
