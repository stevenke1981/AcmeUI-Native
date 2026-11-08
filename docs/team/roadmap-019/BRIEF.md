# ROADMAP 019 — Mobile card acceptance

- Task: MOBILE-CARD-019, priority P2.
- Base: `99e8b160da10d7e69f247c438a3ca0cd4cee0ee5` (updated `origin/master`).
- Branch: `sdk/roadmap-019-mobile-card`.
- Execution: `single-agent-role-passes` (Lead, UI/UX, Verification), one writer;
  author self-review, no independent-agent review claimed.

## Audit and expected behavior

OBSERVED: `mobile_card` already composes a Card with optional media followed by
a title/subtitle Column and maps elevation to existing semantic Card variants.
Its three tests do not establish ordering, geometry, retained identity, omitted
slots, or outlined behavior. The text Column has no explicit key: inserting
media changes its positional key. The existing Gallery Mobile page has buttons,
toggle, stepper and segmented controls, but no mobile cards.

User journey: compare title-only, outlined title/subtitle, and media cards in the
Gallery; switch themes and scroll without losing card content or painting beyond
the viewport. Declarative applications can add/remove media while preserving
the card's text subtree identity.

## Acceptance

1. Reproduce the text identity change before fixing it; keep existing builder
   signatures and semantic variants, and derive the internal text key from the
   card key.
2. Cover defaults, optional-slot combinations, title/subtitle content and order,
   elevated/outlined variants, actual layout geometry, and retained text IDs
   across media insertion/removal.
3. Add three distinct-key Gallery Mobile card examples and headless frame tests
   for themed card surfaces, content geometry and viewport clipping. Content
   quads must share the existing clipped-quad path to preserve parent/child paint
   order (including buttons embedded in media).
4. Run locked tests and strict Clippy for `acme-ui` (all features and mobile-only)
   and `acme-ui-gallery` under Rust 1.88.0. Preserve command logs and tested SHA.
   Run workspace gates at the completed-slice milestone, locally or in hosted CI.
5. Update the roadmap (including historical 016–018 rows), todos and report with
   actual evidence. Push, open a PR, and merge only when MERGEABLE and CI passes,
   as explicitly authorized by the owner.

## Exact file leases (serialized)

- `crates/acme-ui/src/mobile/mobile_card.rs`
- `crates/acme-ui/src/mobile/mobile_card_tests.rs` (new)
- `apps/acme-gallery/src/main.rs` (shared entrypoint, sole writer)
- `apps/acme-gallery/src/mobile_card_tests.rs` (new)
- `docs/status/todos.md`
- `docs/agent-team/ROADMAP.md`
- `docs/team/roadmap-019/BRIEF.md`
- `docs/team/roadmap-019/REPORT.md`
- `docs/team/roadmap-019/evidence.json`
- `docs/team/roadmap-019/logs/` (exact generated log files listed in evidence)
- `docs/team/roadmap-019/gallery-mobile-cards.png` (if native capture succeeds)

Read dependencies: AGENTS/team workflow and role contracts; spec/plan/todos;
016–018 reports; Card, Label, layout/view bridges and retained binding; Gallery
frame/hit paths and current tests; pinned toolchain, CI, verification runner.

Non-goals: other mobile components, shadows/renderer redesign, public API or
dependency expansion, speculative optimization, mobile runtime support.
Manual Windows 注音 IME, GPU drawing/recovery, screen readers, physical DPI and
real touch-device acceptance remain NOT_RUN. An automated X11 capture, if made,
is evidence of a virtual-display CPU window only, not human validation.

Rollback: revert this bounded commit/PR; no manifest or lockfile changes.
