# ROADMAP 012 — task for p1 (Sigma)

Lead: p3 (Claude). Read `AGENTS.md` and `docs/team/roadmap-012/BRIEF.md`
first; its hard rules apply (no git add/commit/stash/checkout/switch, no
Cargo.toml/Cargo.lock edits, no new deps, never cargo clean). Branch
`sdk/roadmap-012-split-atlas-preedit`. p2 (Codex) concurrently edits
`crates/acme-render-wgpu/**`, `crates/acme-text/**`, `crates/acme-render-cpu/**`;
do not touch those.

## 1. Independent review of commit `271e5e5` (you did not write it)

`git show 271e5e5`: preedit scroll fix in `crates/acme-textinput`. Read-only.
Reproduce: `cargo test --locked -p acme-textinput` and clippy. Check the empty
preedit path is unchanged, password handling, grapheme/char boundaries, scales
1.0–2.0. Write findings to `docs/team/roadmap-012/REVIEW-271e5e5.md` with
verdict APPROVE / APPROVE_WITH_FIXES / REJECT and severity per finding.
Do not fix the reviewed code yourself.

## 2. D2 — draw the IME preedit text (bug found during D)

`render_text_input` (`crates/acme-textinput/src/lib.rs`) never draws preedit
glyphs: the only `DrawCommand::Text` uses the committed display text, and
section 6 draws only the underline quads. Users see an underline with no
composition text, and committed text after the caret is not shifted.
Also `apps/.../dispatch.rs` (~line 260, find with `git grep -n set_preedit apps`)
passes `None` as the preedit cursor although the platform event carries it.

Fix, test-first (show RED, then GREEN):
- While composing, the rendered text is committed-before-caret + preedit +
  committed-after-caret; the underline and composition-cursor segment line up
  with the preedit glyphs; text after the caret is shifted by the preedit width.
- Password fields: preedit shown masked or not shown — pick the safer option and
  state it.
- Empty preedit: rendered output identical to today (existing tests unchanged).
- `ime_caret_area` contract stays as `tests/ime_dpi.rs` pins it.
- Forward the platform preedit cursor in the Gallery dispatch.
Write set: `crates/acme-textinput/src/**`, `crates/acme-textinput/tests/**`
(new files, or added tests), the one dispatch file in `apps/` that calls
`set_preedit`. If `lib.rs` grows, you may move preedit rendering into a new
`src/preedit.rs`.
Gates: `cargo test --locked -p acme-textinput`, clippy -D warnings, fmt, and
`cargo check --locked` for the app crate you touched. Manual 注音 IME is NOT_RUN.

## Report

Write `docs/team/roadmap-012/REPORT-P1.md` (< 60 lines): review verdict, D2
files, RED/GREEN commands with exit codes and counts, deviations, NOT_RUN.
Then reply in the pane: `P1 DONE`.
