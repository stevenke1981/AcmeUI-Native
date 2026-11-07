# Retained identity in the remaining native apps

Task ID: retained-apps-2026-11-07. Base: `2ad399c85d7a2b8acddeb383efa87833ec435605` (`master`).

## Scope and reason

Merged PRs #8 and #9 already give the primary Gallery stable retained layout and
accessibility IDs, keyboard focus fixes and automated DPI/IME geometry coverage.
This follow-up closes the explicitly recorded integration gap in
`apps/acme-gallery` and `apps/playground`; it does not repeat that work or claim
manual platform acceptance.

The V2 Showcase still uses sequential frame IDs and duplicate empty sidebar
button keys. The Playground also uses sequential IDs; its positional extraction
and styling reference an obsolete 18-child content structure, while its actual
builder has 14 children. Correcting those indices is a prerequisite to exercising
the real Playground frame path.

## Contracts and non-goals

- Keep the existing retained tree and binding APIs, public signatures,
  dependencies, lockfile, themes and render/input architecture.
- Retained IDs must drive the actual frame layout and resulting hit geometry.
  Reconciliation or binding failure must use a complete sequential layout,
  rather than mixing retained and fallback IDs.
- The Showcase uses `to_layout_with_binding` with its existing layout context.
  Playground preserves its legacy `to_layout` styles/measurement and only maps
  IDs, because switching it to context-based text layout would change geometry.
- Assign unique stable category keys. Keep identity of app chrome when content
  pages are replaced; do not reuse one page's identity for an unrelated page.
- Do not change renderer/platform APIs, IME behavior, accessibility semantics,
  browser settings, repository settings, audit exceptions or CI gates.
- No data cleanup, branch deletion, new spending, release or deployment.

## Ownership and sequence

Execution mode: independent native author workers, then independent review.
The user requested GPT-6.1 Sol with xhigh reasoning; no alternate model/provider
or paid service is introduced.

| Owner | Write scope |
|---|---|
| Showcase author | `apps/acme-gallery/src/main.rs`, optional `retained_tests.rs` |
| Playground author | `apps/playground/src/main.rs`, optional `retained_tests.rs` |
| Lead | This brief, report, changelog and status todos; integration and Git |
| Independent reviewer | Read-only review of the complete candidate |

Authors do not commit, change branches or edit each other's files. Lead integrates
after both handoffs, runs applicable checks, publishes a draft PR, verifies its
exact head and CI results, and merges only after acceptance and user authorization.
The user explicitly confirmed `master` as the merge destination.

## Acceptance

1. Actual normal app builders reconcile with complete bindings.
2. Keyed controls retain IDs across relevant state and page changes; insertion
   and reorder do not transfer an existing keyed control's ID.
3. Layout rectangles and hit regions match the sequential reference, except for
   the documented Playground positional-index correction.
4. Invalid keys or unsupported shapes safely fall back and later recover.
5. Real frame regressions cover both apps, including the Playground panic path.
6. Final-SHA formatting, compilation, tests and independent review are recorded
   with PASS/FAIL/BLOCKED/NOT_RUN labels; missing native manual coverage stays open.

## Verification plan

- Targeted: `cargo test --locked -p acme-ui-gallery -p acme-playground` and
  matching package Clippy/format checks.
- Milestone: locked workspace check/tests, all-feature UI checks and the existing
  changed-file gates under pinned Rust 1.88 in the unmodified CI matrix.
- Preserve Cargo.lock; verify final remote head before reporting publication.
- Manual native window, physical DPI, screen readers and Traditional Chinese
  IME are separate evidence, never inferred from headless frame tests.

Initial desktop compilation is BLOCKED: the pinned toolchain is unavailable and
the offline Cargo cache lacks `cosmic-text`. Local source/format checks and
remote CI are reported separately; no network policy is weakened to fix setup.
