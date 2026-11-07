# Retained-app integration report

Base: `2ad399c85d7a2b8acddeb383efa87833ec435605` (`master`).
Scope: ROADMAP 003 follow-up in the two applications explicitly left sequential
by PR #8. Primary Gallery, rendering, accessibility and text-input libraries are
unchanged.

## Changes

- V2 Showcase now reconciles its real description and uses complete retained
  bindings for frame layout, so rendering and hit regions consume stable IDs.
  Sidebar categories have unique keys and unrelated pages have distinct roots.
- Playground keeps legacy layout/measurement and maps only supported, fully
  matched widget/layout shapes to retained IDs. Errors use a whole sequential
  frame and recover on a later valid description.
- Fixed Playground's stale style/extraction indices. Its actual content has
  14 children; the old mapping expected 18 and indexed the children of a leaf
  label. The style values and input/rendering logic are preserved.
- Added app-builder, identity, fallback/recovery, layout parity and actual
  frame/hit-geometry regression tests. A seeded retained tree distinguishes
  retained IDs from coincidentally equal first-frame sequential IDs.

## Team and ownership

Two distinct GPT-6.1 Sol xhigh author instances own disjoint app files; the Lead
owns integration and documentation. A separate read-only instance performs
independent review. Role files alone are not treated as evidence of a worker.
The brief records scope, acceptance and exact ownership.

## Initial local evidence

These checks precede remote compilation; they are not a Rust test pass.

| Check | Result |
|---|---|
| Upstream source, blobs/tree and signed baseline commit identity | PASS; SHA verified |
| `cargo +stable metadata --offline --no-deps --format-version 1` | PASS |
| `python -m unittest discover -s scripts/tests -p test_verify_sdk.py -v` | PASS; 7 tests |
| `Cargo.lock` versus base | PASS; unchanged |
| App source formatting and patch whitespace | Recorded in author/review handoff |
| Rust compilation, app tests, workspace tests, Clippy | BLOCKED locally; offline cache lacks `cosmic-text` |
| Pinned Rust 1.88 local execution | NOT_RUN; no complete compiler available |
| Windows native windows, physical DPI, IME and screen readers | NOT_RUN |

The existing, unmodified GitHub Actions matrix is the pinned-toolchain execution
path. Its checks must be evaluated on the published candidate head, not inherited
from the green base. Current CI/review/merge status belongs to the PR and final
delivery; this initial report does not claim those pending results passed.

## Boundaries and remaining work

- Automated frame geometry is not native visual, physical-DPI, Traditional
  Chinese IME or assistive-technology sign-off. Existing manual tasks remain open.
- The apps still use their existing positional hover/pressed/activation routing.
  Broader interaction-state migration is outside this patch.
- Playground's compatibility adapter deliberately rejects future unsupported
  variants rather than guessing their view/layout traversal. Extend it only
  with matching layout and fallback tests, or separately migrate the app to
  context-based text layout with explicit geometry acceptance.
- No dependency, public API, lockfile, platform/rendering implementation, CI,
  branch-protection, audit-exception, release or deployment change.

Rollback: revert the bounded retained-app change. Do not undo earlier merged
retained-tree, focus, accessibility or DPI/IME work.
