# ROADMAP 012 — task for p2 (Codex)

Lead: p3 (Claude). You own A, then B. Read `AGENTS.md` and
`docs/team/roadmap-012/BRIEF.md` first; its hard rules apply (no git
add/commit/stash/checkout/switch, no Cargo.toml/Cargo.lock edits, no new deps,
never cargo clean, files < 800 lines). Branch `sdk/roadmap-012-split-atlas-preedit`
(HEAD `271e5e5`). p1 (Sigma) concurrently edits `crates/acme-textinput/**` and
`crates/acme-ui/**` only; do not touch those.

## A — finish the wgpu split (BRIEF section A)

A stopped helper left an UNVERIFIED partial split in the working tree:
`crates/acme-render-wgpu/src/lib.rs` (modified) plus new `draws.rs images.rs
instances.rs pipelines.rs renderer/ surface.rs tests.rs types.rs uploads.rs`,
and `docs/team/roadmap-012/evidence/`. It was stopped mid-edit (it was
reverting a bad regex change in `images.rs` lines ~73/116 and `draws.rs`
~58-59 that changed function parameters' visibility). Either finish it or redo
it from `git show HEAD:crates/acme-render-wgpu/src/lib.rs`, your choice.

Acceptance (fresh evidence, not the helper's):
- Public API identical to HEAD: compare rustdoc output of HEAD (build it in a
  temporary `git worktree add` of HEAD outside the repo dir, then remove that
  worktree) with the split version; save both sorted lists and the diff result
  to `docs/team/roadmap-012/evidence/a-doc-{before,after}.txt`.
- Same test counts as HEAD for `cargo test --locked -p acme-render-wgpu`.
- `cargo clippy --locked -p acme-render-wgpu --all-targets -- -D warnings`,
  `cargo check --locked -p acme-ui --all-targets`, fmt: all exit 0.
- `lib.rs` < 150 lines, every file < 800 lines.

## B — atlas follow-ups (after A is green)

From `docs/status/todos.md` ROADMAP 011: (1) per-format allocators, (2) mid-frame
eviction of untouched pages, (3) starvation when every page holds only fresh
entries (ignored test in `crates/acme-text/tests/atlas_lifecycle.rs` ~line 644).
Read `docs/team/roadmap-011/SPEC.md` and `REPORT.md`.
1. Write `docs/team/roadmap-012/SPEC-B.md` (< 250 lines): for each item decide
   do / reject with reasons, invariants, public API additions (prefer none),
   failing tests to write first, renderer impacts (CPU and wgpu). Smallest
   correct change; explicitly reject what is not worth doing.
2. Implement what the SPEC accepts, test-first (show RED, then GREEN).
   Write set: `crates/acme-text/**`, `crates/acme-render-cpu/**`,
   `crates/acme-render-wgpu/**`, `docs/team/roadmap-012/SPEC-B.md`.
3. Gates per crate you touched: test, clippy -D warnings, fmt; then
   `python scripts/verify_sdk.py --profile cpu`.

## Report

When done, write `docs/team/roadmap-012/REPORT-P2.md` (< 80 lines): files,
RED/GREEN commands with exit codes and counts, API diff result, deviations,
NOT_RUN (real GPU is NOT_RUN). Then reply in the pane: `P2 DONE`.
