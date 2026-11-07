# ROADMAP 012 — cross review (read-only)

Lead: p3. Both reviews are read-only: do not edit source files, do not git
add/commit/stash/checkout. Write only your review file. Reproduce before you
claim; give each finding a severity (CRITICAL/HIGH/MEDIUM/LOW), file:line,
and a concrete failing scenario. Verdict: APPROVE / APPROVE_WITH_FIXES / REJECT.
Both of you run cargo at the same time; if a build lock waits, just wait.

## p2 Codex reviews p1's D2 and the lead's F1 fix → `REVIEW-D2.md`

Authors were Sigma (p1) and Claude (p3); you wrote neither.
- `crates/acme-textinput/src/preedit.rs` (new), `crates/acme-textinput/src/lib.rs`
  (diff vs `271e5e5`), `crates/acme-textinput/tests/preedit_render.rs`,
  `apps/gallery/src/events/dispatch.rs`, `apps/gallery/src/main.rs`.
- F1 fix by lead: `apps/gallery/src/events/ime.rs` clamps the candidate anchor
  to the field content (tests in the same file).
- Context: `REPORT-P1.md`, `REVIEW-271e5e5.md`.
- Check: empty-preedit path identical; grapheme/char boundaries; password
  masking; selection split; clip and baseline; scroll vs drawn geometry; theme
  colors not hardcoded; file sizes < 800 lines.
- Gates to reproduce: `cargo test --locked -p acme-textinput`,
  `cargo test --locked -p acme-gallery --bins`, clippy -D warnings for both.

## p1 Sigma reviews p2's A and B → `REVIEW-AB.md`

Author was Codex (p2); you wrote neither.
- A: `crates/acme-render-wgpu/src/**` split (vs `git show HEAD:crates/acme-render-wgpu/src/lib.rs`).
  Note it uses `include!` at the crate root for public definitions — judge
  whether that is acceptable vs `mod` + `pub use`, and whether any item moved
  changed visibility or behavior. Evidence: `evidence/a-*.txt`.
- B: `SPEC-B.md`, `crates/acme-text/src/atlas.rs`, `src/atlas/tests.rs`,
  `tests/atlas_lifecycle.rs` (the formerly ignored starvation test). Check the
  eviction invariants: no glyph emitted this frame is invalidated; one eviction
  per request; background eviction stays stale-only; renderers (CPU/wgpu) still
  receive correct uploads/clears.
- Gates to reproduce: `cargo test --locked -p acme-render-wgpu`,
  `cargo test --locked -p acme-text`, clippy -D warnings for both.

When done reply in your pane: `REVIEW DONE`.
