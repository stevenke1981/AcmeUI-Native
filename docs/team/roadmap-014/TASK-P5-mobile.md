# ROADMAP 014 — TASK P5b (Antigravity / Claude Sonnet 5.5): interactive mobile components acceptance

Worktree: this one (branch `sdk/roadmap-014-mobile`, based on master 5eb5283). Work only in this worktree.
You own: `crates/acme-ui/src/mobile/mobile_toggle.rs`, `mobile_stepper.rs`, `mobile_segmented.rs`, new test files you create for them, and — only if a Gallery mobile page exists — the Gallery code that shows these three. Do not touch other components or crates.

## Background
`docs/status/todos.md`, section "P3 — Mobile component real implementations": the `mobile_*` files contain builders, but these items are still unchecked and need **re-audit of interaction and test coverage before they can be marked complete**:
- `mobile_toggle` — switch track + thumb layout, checked state, `on_change`
- `mobile_stepper` — minus/plus buttons flanking a value label, min/max clamp
- `mobile_segmented` — N-segment row, selected highlight, `on_select(usize)`

`mobile_button` is the reference for "done": sized variants, disabled state, touch heights, a Gallery page, and **hit-index regression tests** (see `crates/acme-ui/src/mobile/mobile_button.rs` and how `apps/acme-gallery/src/main.rs` tests hits).

## Required
1. Audit each of the three: what the builder produces, which messages it emits, and what is missing against the todo text. Write the audit (short table) in your report **before** changing code.
2. Make each one meet its todo line, with real interaction:
   - toggle: track + thumb geometry for on/off, checked state reflected, `on_change(bool)` emitted with the **new** value on press; disabled emits nothing.
   - stepper: minus/plus around the value; `on_change(new_value)` clamped to `[min, max]`; minus disabled at min, plus at max; step size supported.
   - segmented: N segments, selected one highlighted (theme colour, not hard-coded), `on_select(index)`; out-of-range selected index handled without panic.
   - Touch targets ≥ 44 px tall (as mobile_button md).
3. Tests: builder-structure tests **and** headless interaction tests that press the real laid-out hit areas (like mobile_button's hit-index tests) and assert the emitted messages. Edge cases: min==max, step larger than range, 0 or 1 segment, disabled.
4. Accessibility: each exposes a sensible role/name/state through the existing accessibility walk (e.g. toggle checked state, stepper value). Add a test using the accessibility crate's public API if feasible; if not feasible without touching that crate, say so in the report.
5. Update the three todo lines in `docs/status/todos.md` to `[x]` with a one-line note — **only** for items that fully pass; leave the rest open with what is missing.

## Hard rules
- TDD: failing tests first (RED evidence), GREEN, at least one mutation check per component.
- No git add/commit/stash/checkout/push. No new dependencies. Rust 1.88.0. No absolute local paths in docs (public repo).
- Verify: `cargo fmt --all -- --check`, `cargo clippy -p acme-ui --all-targets --all-features -- -D warnings`, `cargo test -p acme-ui --all-features`, `cargo check --workspace --all-targets`.

## Report
Write `docs/team/roadmap-014/REPORT-P5-mobile.md`: audit table, design, files changed, RED/GREEN/mutation evidence, commands and results, NOT_RUN (e.g. real touch device). End your final message with a line containing only `P5 DONE`.
