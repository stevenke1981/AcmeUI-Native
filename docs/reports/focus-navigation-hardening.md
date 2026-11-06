# Focus navigation hardening

## Scope and baseline

This is one bounded stabilization task, not completion of the framework roadmap.
The inspected base is `a083bd20b979928d86fe60bc7a882d5e9ae6d398` on `master`.
The original `crates/acme-core/src/event.rs` blob is
`2b6f120e753f2aba83576bdd6cc2d6e9bf02d05b`.

Repository rules read: `AGENTS.md`, `docs/specs/spec.md`,
`docs/specs/plan.md`, and the relevant entries in `docs/status/todos.md`.
The existing retained-tree/history fixes are preserved.

## Plan and acceptance

1. Prevent duplicate `NodeId` entries from trapping Tab/Shift+Tab navigation.
2. Cache the unique eligible order when `set_order` runs, rather than building a
   new temporary `Vec` on every `focus_next` call.
3. Preserve unique-input ordering, explicit focus requests, focus reset on
   ineligibility, and pointer-capture lifetime semantics.
4. Add public-API regressions and a reproducible headless example.
5. Keep public function signatures, dependencies, rendering, themes, IME, and
   the existing CI workflow unchanged. Deliver on a topic branch without merging.

## Findings and implementation

With `[(a, true), (a, true), (b, true)]`, the previous implementation filtered
an order containing both copies of `a`. After focusing `a`, `position` always
found its first occurrence, so advancing selected the second `a` forever.
The observable sequence was `a -> a -> a`, never reaching `b`.

`set_order` now adds an eligible ID to the cached order only when inserting it
into the eligibility set succeeds. Duplicate entries use the first eligible
occurrence. A false duplicate does not revoke a true occurrence; callers should
still supply one entry per live node. No duplicate-input error API was added.

`focus_next` reads that cached order directly. It performs no temporary-list
allocation and no per-entry eligibility hash lookups. Finding the current index
remains O(n); this is not an O(1) navigation claim or a measured speedup.

Pointer capture remains independent of keyboard eligibility. Rebuilding the tab
order must not cancel a drag captured by a nonfocusable node. `remove_node`
remains responsible for clearing a removed node's focus and all its captures.

## Work products

- `crates/acme-core/src/event.rs`: focused implementation change and API docs.
- `crates/acme-core/tests/focus_navigation.rs`: 11 public-API regression tests.
- `crates/acme-core/examples/focus_navigation.rs`: headless lifecycle example.

The tests cover duplicate IDs, conflicting eligibility, forward/reverse wrapping,
empty/disabled/singleton orders, reorder/rebuild, rejected focus requests, node
removal, and pointer capture. One test enumerates all 32 eligibility masks for
five nodes, six initial-focus states, both directions, and twelve navigation
steps (4,608 transitions) against a separate reference traversal.

## Execution record and limitations

The `team-orchestrate`, `team-plan`, `team-work`, `team-review`, `team-pr`, and
`team-ci` skill entrypoints were read. Their additional packaged role/reference
files were not available through the skill resource reader. The repository's
own rules remain authoritative.

Planning, implementation, and author self-check were performed sequentially by
one assistant instance. No separate subagent or independent reviewer was
launched. Role separation in this document is not evidence of multiple agents.

| Check | Status at author handoff | Evidence / limitation |
| --- | --- | --- |
| Original source identity | PASS | Reconstructed source matches the original Git blob SHA above byte-for-byte. |
| Public function signatures | PASS | Original and changed `pub fn` signatures compared programmatically. |
| Patch whitespace | PASS | `git diff --check` in a local verification snapshot, exit 0. |
| Rust formatting / compilation / Clippy / tests | BLOCKED locally | Container has no `cargo`, `rustc`, or `rustfmt`; direct GitHub clone failed with DNS resolution error. |
| Regression tests | AUTHORED, NOT_RUN locally | Eleven Rust tests are present; no local Rust test pass is claimed. |
| Headless example | NOT_RUN locally | The example is not a Gallery or GPU validation. |
| Independent review | NOT_RUN | Author self-check is not independent review. |
| Gallery / Windows keyboard / IME / DPI | NOT_RUN | No GUI execution or screenshot evidence was produced. |
| Remote CI | Check the PR for the final SHA | Existing PR-triggered CI is unchanged; baseline results do not validate this patch. |

A supplementary Python model was also executed during authoring to check the
algorithm, including 4,608 unique-input transitions and 9,331 duplicate/eligibility
sequences. This is model-level evidence only, not execution of the Rust source.
The model and its output are retained in the delivery evidence bundle.

## Verification commands

Run in a complete checkout using the repository's pinned toolchain:

```sh
cargo test -p acme-core --test focus_navigation
cargo test -p acme-core
cargo run -p acme-core --example focus_navigation
cargo clippy -p acme-core --all-targets -- -D warnings
```

For changed-file formatting, use the existing
`scripts/check_changed_rustfmt.py` with `BASE_SHA` set to the inspected baseline.
The existing CI also checks workspace targets, feature variants, Clippy, and
workspace tests. Record actual command exits and the exact tested commit.

## Scoped task status / merge gate

- [x] Inspect current source and preserve prior stabilization work.
- [x] Implement the cached unique focus order without a public API change.
- [x] Add eleven regression tests and a headless example.
- [x] Check source identity, public signatures, and patch whitespace.
- [ ] Obtain Rust compile, formatting, Clippy, and test evidence for the final SHA.
- [ ] Obtain independent review.
- [ ] Perform application-level keyboard/Gallery validation where integrated.
- [ ] Merge only after explicit authorization; no automatic merge or release.

Keep the PR draft until its outstanding verification/review gates are resolved.
To abandon the change, leave the topic branch unmerged; `master` is untouched.
If subsequently merged, use a normal revert under the maintainer's approval,
not a force push or history rewrite.
