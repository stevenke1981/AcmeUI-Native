# ROADMAP 005 — DPI and IME acceptance: automated part (BATCH-005)

ROADMAP row 005 asks for actual 注音 sessions at 100/125/150/200 % Windows
scaling. **Those are manual and stay NOT_RUN / pending human sign-off**
(`AGENTS.md`: never claim Traditional Chinese IME works without manual
validation). This batch builds the automated evidence that can run headless in
CI, so the manual session only has to confirm what automation cannot see.

Base: the ROADMAP 004 working tree (`sdk/roadmap-004-focus-semantics`). Rust
1.88.0 pinned, Windows MSVC host. Read `AGENTS.md`,
`docs/guides/MANUAL_VALIDATION.md`, `docs/guides/TEXT_AND_IME.md`.

Scale matrix for every item: `[1.0, 1.25, 1.5, 2.0]` (also 1.75 where cheap).

## Work items (new files only)

| ID | Role | Write set |
|---|---|---|
| D1 | team-test-engineer | new `apps/gallery/src/dpi_tests.rs` + the single line `#[cfg(test)] mod dpi_tests;` in `apps/gallery/src/main.rs` (add it next to `mod retained_ids_tests;`, touch nothing else there) |
| D2 | team-test-engineer | new `crates/acme-render-cpu/tests/dpi.rs` |
| D3 | team-test-engineer | new `crates/acme-textinput/tests/ime_dpi.rs` |

### D1 — Gallery at every scale

Use the pattern of `apps/gallery/src/retained_ids_tests.rs` (`Gallery::new()`,
`frame(FrameContext { scale_factor, .. })`, `build_layout_context`, layout
compute with `scale_factor`). For every page and every scale:
- `frame()` returns a non-empty scene; all layout rects are finite and
  non-negative in size.
- Logical layout is scale-invariant within a documented tolerance (start at
  1.0 logical px; if text shaping makes it differ more, measure and report the
  real maximum per page instead of loosening silently).
- The text-input IME caret rectangle the Gallery reports (find the function the
  runtime uses — `ime_cursor_area` / `compute_ime_caret_rect`) stays inside the
  text input's logical rect at every scale when the input is focused (put the
  Inputs page with the TextInput focused; read events code to focus it).

### D2 — CPU renderer physical mapping at fractional scales

Read `crates/acme-render-cpu/src/lib.rs` and its existing tests. At each scale:
- framebuffer is `ceil/round(logical * scale)` (assert the actual documented
  rule) and resizing between scales has no stale pixels;
- a 1-logical-px border and a 1-logical-px separator quad remain visible (≥1
  physical pixel with the expected colour) at 1.25 and 1.75;
- a clip rect at fractional logical coordinates clips at the documented
  pixel-centre rule at every scale;
- a solid quad at logical (10,10,20,20) covers the expected physical pixel
  range (compute from the rule).
If a test exposes a real renderer bug, keep the failing test marked
`#[ignore = "BUG: ..."]` with a minimal reproduction in the report; do not edit
`src/lib.rs`.

### D3 — Text input caret/preedit geometry vs scale

Read `crates/acme-textinput/src/lib.rs` (preedit, caret geometry, IME caret
helpers) and existing tests. With a mixed Traditional Chinese + ASCII string
(e.g. "注音輸入 abc") and an active preedit ("ㄓㄨˋ"):
- caret positions are monotonic in grapheme order and use char/grapheme
  indices, never byte offsets (AGENTS rule) — assert with a string whose byte
  and char counts differ;
- logical caret x positions at each scale differ by ≤ the documented tolerance
  from scale 1.0;
- the IME candidate rect stays within the input's logical bounds;
- preedit commit/cancel restores caret consistency.

## Hard rules

Only your write set. No git add/commit/stash/checkout/reset, no cargo clean,
`CARGO_TARGET_DIR=target/<id>`, pinned toolchain, `--locked`, no GUI launch, no
new dependencies. Other workers concurrently edit apps/gallery (ROADMAP 004
review fixes may land) and crates/acme-accessibility — leave them alone; if the
build is temporarily broken by someone else, wait and retry. Report
PASS/FAIL/NOT_RUN honestly; manual IME/DPI remains NOT_RUN.

## Report format

```
ID / role / files changed
Summary (≤10 lines) incl. measured tolerances
Commands → status (+ output tails)
Bugs found (minimal repro) / patches requested
Risks / NOT_RUN / follow-ups
```
