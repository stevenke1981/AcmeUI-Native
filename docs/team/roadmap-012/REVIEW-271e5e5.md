# REVIEW — commit `271e5e5` (preedit scroll fix)

Reviewer: p1 (Sigma), independent — author was p2/lead. Read-only review: no code
changed by me. SHA reviewed: `271e5e5` on `sdk/roadmap-012-split-atlas-preedit`.

## Verdict: APPROVE_WITH_FIXES

The core change is correct and well covered: during composition the scroll focus
point is the composition cursor and the scroll range grows by the preedit width,
while an empty preedit is bit-identical to the committed-caret rule. One **Major**
regression in the reported IME anchor is introduced (F1); two minor gaps (F2, F3).
Nothing blocks landing D2 on top of this commit, because D2 keeps the
`ime_caret_area` contract and F1 is a call-site/owner decision.

## Evidence (reproduced)

- `cargo test --locked -p acme-textinput` → exit 0, **148 tests**:
  lib 105, ime_dpi 20, multipage_e2e 2, preedit_scroll 8, scroll_caret 13, doc 0.
- `cargo clippy --locked -p acme-textinput --all-targets -- -D warnings` → exit 0.
- `cargo fmt -p acme-textinput -- --check` → exit 0.
- `git diff 271e5e5 -- crates/acme-textinput` → empty (suite run on exactly the
  reviewed tree; the concurrent wgpu work was transiently unbuildable at first,
  so the first `cargo test` attempt failed in `acme-render-wgpu` — not this
  commit's code. Re-run after p2's tree compiled: all green.)
- Empty preedit unchanged: `empty_preedit_scroll_is_unchanged_and_matches_no_preedit`
  compares `to_bits()`, and my own probe gave `scroll=344.0` for both the plain
  and the `set_preedit("", None)` state at scales 1.0/1.25/1.5/2.0.
- Scale 1.0–2.0: all widths are logical and scale-invariant here (probe:
  `scroll=93.78906`, `preedit_w=212.28906` at every scale). Reviewed tests cover
  1.0/1.25/1.5/2.0; 1.75 (used by `ime_dpi.rs`) is not in the new file — cheap to add.
- Grapheme/char boundaries: `preedit_cursor` end is clamped by
  `byte_offset_to_x_inner` (`floor_char_boundary` + `min(len)`), so an
  out-of-range or mid-char end cannot panic or slice inside a char (probe:
  `Some((0, usize::MAX/2))` → `scroll=93.78906`, no panic).
- Password path: section 6 now anchors the underline with `display_cursor`, which
  is what `ime_caret_area` uses; `password_preedit_underline_starts_at_the_ime_caret_anchor`
  pins it, and `password_caret_follows_masked_text_without_panicking` still passes.

## Findings

**F1 — Major — the IME candidate anchor can be reported outside (left of) the field.**
`ime_caret_area` stays anchored at the *committed* caret, and the new scroll can
push that point far to the left. Probe (content width 120, preedit `ㄓㄨˋㄧㄣ注音輸入法測試字串`,
width 212.29): `scroll_offset = 93.78906`, `ime_caret_area(..)[0] = -93.78906` at
every scale. Gallery's `compute_ime_caret_rect` adds this to the field origin
without clamping (`apps/gallery/src/events/ime.rs:39`), so the candidate window is
placed 93.79 px left of the text area. Before `271e5e5` the anchor could not go
negative (scroll never exceeded the committed caret), so this is a regression
introduced by the commit, and its own test
(`rendered_composition_segment_lies_inside_the_scrolled_field`) accepts it.
Suggested fix (owner-level, since D's contract pins `ime_caret_area`): clamp the
anchor to the field in `compute_ime_caret_rect`, or report the composition cursor
as the anchor when composing. Not fixed here — out of my write set and would
break the pinned contract.

**F2 — Minor — no test for a malformed `preedit_cursor`.**
`Some((0, len + 7))`, mid-char ends and `start > end` rely on the flooring inside
`byte_offset_to_x_inner`; correct today (verified by probe) but untested.

**F3 — Minor — `preedit_extent` shapes the preedit twice** (cursor prefix + full
width) on every scroll refresh, i.e. twice per IME event. Correctness fine;
measurable only under profiling.

**Observation (not a defect) — the composition *start* may scroll out of view**
when the preedit is wider than the field; the reviewed tests document this
deliberately. It is the cause of F1, not a separate bug.

**Observation — test brittleness:** `rendered_composition_segment_lies_inside_the_scrolled_field`
identifies the composition segment as "the only quad 3 px tall". Any other 3 px
quad added to the field's scene later would break it.

**Observation — D2 gap confirmed, not this commit's job:** no `DrawCommand::Text`
carries the preedit; section 6 draws the underline only. Handled by D2 (REPORT-P1).