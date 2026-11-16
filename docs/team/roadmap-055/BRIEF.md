# ROADMAP 055 — RadioCards typed selection and arrow-key navigation

- Base: `c29e89f` (`chore/land-stacked-052-054`, master `efaba73` + stacked 052a–054a).
- Execution mode: Claude Code agent team. The Lead (Opus 5.5) integrates. The adviser (Fable 5.2) handles audit, contract and independent review. Opus 5.5 implements the component. Sonnet 5.5 owns the Gallery, as single writer of `main.rs`. Haiku 5.5 runs verification.
- Severity: P1. `RadioCards::on_select` is stored (`crates/acme-ui/src/inputs/radio_cards.rs:35,64-67`), but `From` (`:70-88`) never reads it. Options are passive Card/Label trees with no keys, so pointer and keyboard selection are impossible.

## Expected behavior
1. With `on_select`, each option is an instance-keyed Button `"{id}:option:{value}"` bound to `on_select(value)`, inside a Card keyed `"{id}:card:{value}"`. The selected/unselected state uses `ButtonVariant` plus the marker, with theme tokens only.
2. Without `on_select`, the legacy passive tree is unchanged.
3. Additive `RadioCards::navigate(&self, forward: bool) -> Option<M>` wraps. With no or unknown selection it returns the first option (forward) or the last (backward). It returns `None` with no options or no handler.
4. Gallery → Inputs hosts a controlled demo: pointer selects; F6 enters the group, arrows move with wrap, Enter/Space confirm, Esc leaves. The host is scoped to category 2 and respects the IME guard.

## Write set
- Component (Opus): `crates/acme-ui/src/inputs/radio_cards.rs`
- Gallery (Sonnet): `apps/acme-gallery/src/radio_cards.rs`, `apps/acme-gallery/src/radio_cards_tests.rs`, `apps/acme-gallery/src/main.rs` (single-writer lease)
- Lead: `docs/team/roadmap-055/*`, `docs/status/todos.md`, `docs/status/final.md`, `docs/agent-team/ROADMAP.md`

## Non-goals
CheckboxCards, Sidebar and NavigationMenu; disabled options; generic Tab traversal; AccessKit RadioGroup roles; manifests, lockfile, Scene and platform changes.
