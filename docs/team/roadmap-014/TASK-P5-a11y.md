# ROADMAP 014 — TASK P5 (Antigravity / Claude Sonnet 5.5): AccessKit request mapping and the TreeItem press point

Worktree: this one (branch `sdk/roadmap-014-a11y`, based on `sdk/roadmap-013`). Work only in this worktree.
You own **only** `crates/acme-accessibility/**`. Codex works in parallel on `crates/acme-render-cpu`.

## Background (from ROADMAP 013 and its review)
- `AccessibilityAdapter::route_action(&AccessibilityAction)` turns crate-level actions into `PlatformEvent`s, but nothing converts an AccessKit `ActionRequest` (what a platform adapter or screen reader delivers) into an `AccessibilityAction`. The new `Expand`/`Collapse` variants are therefore unreachable from AccessKit. Open todo: "App/winit bridge mapping of AccessKit `Action::Expand`/`Collapse` requests to the new variants."
- Review finding (LOW): `Click`/`Expand`/`Collapse` on a TreeItem press the **row centre**. If rich item content (now walked since 013) has a Button at the centre, the press hits the button instead of selecting the row.

## Required
1. **Request mapping.** Add a pure, public, additive conversion from `accesskit::ActionRequest` to `Option<AccessibilityAction>`, e.g. `AccessibilityAction::from_request(&ActionRequest) -> Option<Self>` (or `TryFrom`). Map at least Focus, Click (and Default action if this AccessKit version has one), Expand, Collapse, ScrollIntoView, and SetValue (from the request's value data; a non-string value → `None`). Unsupported actions → `None`, never a panic. Convert the target id with the exact inverse of `to_access_id` (`crates/acme-accessibility/src/lib.rs:26`); add that inverse if missing and test the round trip, including virtual ids (≥ 2^56).
   Optionally add `AccessibilityAdapter::handle_request(&ActionRequest) -> Vec<PlatformEvent>` that composes the two. Document in the module docs how an app wires a platform adapter's requests into it (no new dependency; do not add `accesskit_winit`).
2. **Press point.** For TreeItem targets, press a point that selects the row without hitting its content: the row's leading edge inside the row (e.g. the indent/disclosure area: `x = row.left + small inset`, `y = row centre`), not the centre. Keep `Click` on every other role unchanged (still the centre). Decide the rule from the node's role in the stored tree; test both a TreeItem with a centred Button child and an ordinary Button.
3. Keep all existing tests green; behaviour of non-TreeItem actions must not change.

## Hard rules
- TDD: failing tests first (RED evidence), then GREEN, plus at least one mutation check (break the new code, show a test fails, revert).
- Only `crates/acme-accessibility/**` and `docs/team/roadmap-014/` (your report). No git add/commit/stash/checkout/push. No new dependencies. Rust 1.88.0.
- Verify: `cargo fmt -p acme-accessibility -- --check`, `cargo clippy -p acme-accessibility --all-targets -- -D warnings`, `cargo test -p acme-accessibility`, `cargo check --workspace --all-targets`.
- Do not write absolute local paths into docs (the repo is public).

## Report
Write `docs/team/roadmap-014/REPORT-P5-a11y.md`: design, files changed, AccessKit types used, RED/GREEN/mutation evidence, commands and results, NOT_RUN. End your final message with a line containing only `P5 DONE`.
