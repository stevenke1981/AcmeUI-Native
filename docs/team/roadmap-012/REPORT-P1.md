# REPORT P1 — review of `271e5e5` + D2 (draw the IME preedit)

## 1. Review verdict: APPROVE_WITH_FIXES

Details: `docs/team/roadmap-012/REVIEW-271e5e5.md`. Reviewed code not changed by me.

- Reproduced on the exact reviewed tree (`git diff 271e5e5 -- crates/acme-textinput`
  empty): `cargo test --locked -p acme-textinput` → exit 0, 148 tests
  (105+20+2+8+13+0); `clippy --all-targets -- -D warnings` → exit 0; `fmt --check` → exit 0.
- Empty preedit bit-identical (`to_bits()` test + my probe: 344.0 both ways),
  password underline path correct, char-boundary/out-of-range preedit cursors safe,
  logical widths scale-invariant 1.0–2.0.
- **F1 Major:** the IME anchor still follows the committed caret while the scroll
  now grows, so `ime_caret_area()[0]` reached −93.79 px (content 120, wide preedit)
  and Gallery's `compute_ime_caret_rect` does not clamp → candidate window left of
  the field. Owner-level fix; not mine (contract pinned by `ime_dpi.rs`).
- F2 minor: no malformed-`preedit_cursor` test. F3 minor: preedit shaped twice per refresh.
- First `cargo test` hit p2's mid-save wgpu tree; re-run green. Not this commit's code.

## 2. D2 — preedit glyphs are drawn

Files: `crates/acme-textinput/src/preedit.rs` (new, 363), `src/lib.rs` (`mod preedit`,
sections 3/6, `preedit_extent` measures the drawn string),
`tests/preedit_render.rs` (new, 12 tests), `apps/gallery/src/events/dispatch.rs` (+
`main.rs` call site). No `Cargo.toml`, no deps, no other crate.

- Composing draws three runs — committed-before-caret, the composition, and the
  after-caret text shifted by the composition width — one baseline, one clip.
  Underline and composition-cursor segment take the composition run's x/width.
- Password fields: **masked** (safer: never reveal what is about to be committed),
  one mask char per grapheme; the cursor is re-counted in mask offsets and
  `scroll_caret_into_view` measures the same masked string, so scroll = drawn frame.
- Empty preedit: untouched single-run path, pinned by comparing whole command lists.
- `ime_caret_area` unchanged (`ime_dpi.rs` 20, `preedit_scroll.rs` 8 pass).
- Gallery forwards the platform preedit cursor, pinned by
  `gallery_forwards_the_platform_preedit_cursor`.
- Consequence fixed: the selection highlight is split at the caret so it stays on
  the shifted tail.

RED (fix forced off, `preedit_runs` → `None`): `cargo test --locked -p acme-textinput
--test preedit_render` → **exit 101, 3 passed / 9 failed** (the 3 are no-preedit controls).
GREEN: same → **exit 0, 12 passed**.

Gates: `test -p acme-textinput` → exit 0, **160 passed** (105+20+2+12+8+13);
`clippy -p acme-textinput --all-targets -- -D warnings` → 0; `fmt --check` for
acme-textinput and acme-gallery → 0; `check -p acme-gallery --all-targets` → 0;
`test -p acme-gallery --bins` → 0 (32 passed, 1 ignored).

## 3. Deviations

- Preedit code lives in `preedit.rs` (allowed), including `selection_spans`.
- `preedit_extent` now measures the masked string for password fields (needed for
  "scroll matches the drawn frame"; no change for plain fields).
- `handle_ime_preedit` gained a `cursor` parameter (additive); its only caller updated.

## 4. NOT_RUN

- Manual 注音 composition in a real Windows IME — NOT_RUN. Candidate-window
  placement (the lead's F1 fix) is NOT_RUN here; p2 reviewed it in `REVIEW-D2.md`.
- No wgpu frame rendered; raster evidence is `acme-render-cpu`.
- Workspace-wide gates belong to the lead; I ran my crate plus the app channel above.

## 5. Fix wave — `REVIEW-D2.md` F1 (MEDIUM) and F2 (LOW)

`REVIEW-D2.md` F3 (fmt of `apps/gallery/src/events/ime.rs`) was the lead's and is
already fixed; my files were fmt-clean in that run.

- **F1 backwards selection shifted twice.** `preedit.rs` `selection_spans` used
  `tail_shift = runs.x + preedit_width(..)`; a tail byte's own offset already
  contains the committed prefix, so the highlight landed one prefix too far right.
  Now the tail shift is the composition width only, and the `selection_spans`
  doc comment states why `runs.x` must not be added there.
- **F2 composition inherited the placeholder colour.** `lib.rs` `render_color`
  chose the muted token from `is_empty && has_placeholder` before the composing
  branch was known. Now `composing = !state.preedit.is_empty()` suppresses the
  placeholder token, so the composition takes the normal / readonly text token.
- Deviation (documented): `cross_caret_selection_hits_both_runs_once` uses
  `select_all`, not Shift+Left. Every selection API leaves the caret at one end
  of the selection (`extend_selection_left`, and `select_all` keeps the caret),
  so a backwards Shift+Left selection can never put the caret inside it.
  `select_all` with the caret mid-text is the reachable cross-caret state; it
  reproduces the two-span split.

Tests added (`tests/preedit_render.rs`, +6, now 18 in the file): backward-selection
precondition, backwards highlight vs tail glyph, cross-caret two spans,
placeholder hidden + text token, placeholder muted guard, readonly disabled token.
F1 and F2 cases run at scales 1.0/1.25/1.5/2.0.

RED (both fixes reverted in place, then restored):
`cargo test --locked -p acme-textinput --test preedit_render` → **exit 101,
3 failed / 15 passed**; the failures are exactly the F1 backwards test, the F1
cross-caret test and the F2 colour test (highlight 70.45996 vs tail 55.106445;
colour `0.4196,0.4471,0.5020` = muted instead of `0.0588,0.0784,0.0980`).

GREEN: same command → **exit 0, 18 passed**. Independent re-check of p2's exact
reproduction (`cursor_end`, Shift+Left, `ㄓㄨˋ`): tail origin **46.875977**,
highlight **46.875977**, error **0.000000** (was 53.999023 / 7.123047).

Gates: `cargo test --locked -p acme-textinput` → **0**, **166 passed**
(105 + 20 + 2 + 18 + 8 + 13); `cargo test --locked -p acme-gallery --bins` → **0**
(34 passed, 1 ignored); `cargo clippy --locked -p acme-textinput --all-targets --
-D warnings` and `-p acme-gallery --all-targets -- -D warnings` → **0**;
`cargo fmt -p acme-textinput -p acme-gallery -- --check` → **0**.
File lengths: `preedit.rs` 367, `tests/preedit_render.rs` 783, `lib.rs` 2829
(pre-existing oversized, unchanged by this wave).

NOT_RUN (unchanged): real Windows 注音 IME, native Gallery visual/DPI acceptance,
real GPU drawing, hosted CI. No theme colour is hardcoded by these fixes.
