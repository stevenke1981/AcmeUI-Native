# ROADMAP 056 — NavigationMenu typed activation and roving keyboard navigation

- Base: master `a84af74`. Severity: P1. NavigationMenu Buttons are built without `on_click` (`crates/acme-ui/src/layout/navigation_menu.rs:84,94-95`), so the bar cannot navigate. The Gallery collects hits only for `activate()`=Some (`apps/acme-gallery/src/main.rs:3596`).
- Execution mode: Claude Code agent team. Fable 5.2 is adviser (contract) and independent reviewer. Opus 5.5 implements the component. Sonnet 5.5 builds the Gallery as the single writer of `main.rs`. Haiku 5.5 runs verification. Opus 5.5 Lead integrates.

## Decisions (Lead-confirmed)
- Without a handler, the tree and the legacy `{id}_{value}` / `{id}_{value}_{child}` keys are byte-identical.
- With `on_select`, keys are: Row `{id}`, Column `{id}:group:{value}`, Button `{id}:item:{value}`, Card `{id}:menu:{value}`, child `{id}:sub:{parent}:{child}`.
- Activation is F6, scoped to Layout (category 3). Submenu keyboard support is one level only.

## Expected behavior
1. Additive `on_select(fn(&str) -> M)`. Enabled items and children emit `on_select(value)`. Disabled ones never emit and get no hit region. Values must be unique and must not contain `:` (documented).
2. With a handler, the selected item is `ButtonVariant::Primary` and others are `Ghost`. Items with children carry a text marker. Additive `active_child(value)` renders that child Primary plus a marker. Theme tokens only.
3. Additive `navigate(&self, forward) -> Option<M>` roves enabled top-level items: it wraps and skips disabled ones. With no or unknown selection it returns the first or last enabled item. It returns None with no handler or no enabled item.
4. Additive `navigate_child(&self, current: Option<&str>, forward) -> Option<&str>` moves to the next enabled child of the selected item, wrapping. It returns None if the selected item is missing, disabled or has no enabled child.
5. Gallery → Layout: a "NavigationMenu — typed activation" section after PageHeader, with one disabled item and one disabled child. Behavior:
   - F6 enters the menu and scrolls it into view.
   - Left/Right call `navigate`; Down/Up call `navigate_child`.
   - Enter/Space activate the focused child, or else confirm the item.
   - Esc clears child focus first, then leaves.
   - Blur, a category change, or a pointer press off the menu deactivate it.
   - The preedit guard applies to all keys. Hover/press are cleared with `reset_hover_press()`.

## Write set
- Component (Opus): `crates/acme-ui/src/layout/navigation_menu.rs`
- Gallery (Sonnet): `apps/acme-gallery/src/navigation_menu.rs` (new), `apps/acme-gallery/src/navigation_menu_tests.rs` (new), `apps/acme-gallery/src/main.rs`
- Lead: `docs/team/roadmap-056/*`, `docs/status/{todos,final}.md`, `docs/agent-team/ROADMAP.md`

## Non-goals
Hover-open, timers and animation; overlay positioning; nesting deeper than one level; Home/End and typeahead; Tab traversal; AccessKit menubar roles; enforcing value uniqueness; Sidebar, Anchor and BottomNavigation; the 055 P2s; manifests, lockfile, Scene and platform.

## Risks
- The Layout page grows, and `anchor_tests`, `bottom_navigation_tests` and `page_header_tests` use fixed 760/900 viewports. The section must go after PageHeader, and those tests must pass unmodified.
- F6 is now used on a third page. The 055 and Mobile F6/F7 tests must pass unmodified.
- Glyph coverage for ▾/▸ is unverified; fall back to ASCII if needed.
- The in-flow dropdown shifts content below it, so the slot test must cover both open and closed.
