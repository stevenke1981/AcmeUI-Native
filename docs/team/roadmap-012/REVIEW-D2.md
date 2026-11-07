# ROADMAP 012 — independent D2 / F1 review

Reviewer: p2 Codex; authors: p1 Sigma (D2), p3 Claude (F1).
Verdict: **APPROVE_WITH_FIXES**. F1's anchor clamp is correct in the reproduced
cases; fix D2's selection offset and composition colour, plus the formatting gate.
Read-only review: no production/test source edits or Git mutations. Only this
report was written; transient Rust probe build products are under ignored `target/`.

## Candidate and execution

HEAD: `271e5e552c6f02d0856ca8fdd7b0a502c6eb8a09`, branch
`sdk/roadmap-012-split-atlas-preedit`, plus the assigned dirty D2/F1 files.
Windows 10 MSVC, Rust/Cargo 1.88.0; checks ran 2026-11-08 00:49–00:54 Asia/Taipei.
Reviewed source hashes were unchanged after the checks and probes:

| File | SHA-256 |
| --- | --- |
| `crates/acme-textinput/src/preedit.rs` | `d99e5d1c9e18097d977630dc6e7fb9b1a0f915c232657049bfa09cf0c456a280` |
| `crates/acme-textinput/src/lib.rs` | `b61c1b781705e05d9fa9e4c892da4e74ddf4cbb8821aa871796b48b370d0ae09` |
| `crates/acme-textinput/tests/preedit_render.rs` | `cff724b62feed1857d2d6a02cef875cdef5f2ce639e321989cde1dcdb04a1446` |
| `apps/gallery/src/events/dispatch.rs` | `a11660d7e4da71ac77f0d2709d4c0e425bfab6053206da55c200afc5b63fbdfc` |
| `apps/gallery/src/main.rs` | `d5f2ad113360fa585f4b1447c5dd543d239db2ad1fe363fa69ba9d80ece06a15` |
| `apps/gallery/src/events/ime.rs` | `e374900a5cebbf4f2ee13e416fa4ffa2d169ad8278a9efe7f65dd475e3a40014` |

## Findings

### F1 — MEDIUM — backwards selection highlight is shifted twice

`crates/acme-textinput/src/preedit.rs:89`, `:126`, `:132`.
`tail_shift` already includes `runs.x` (the committed prefix width), but `span`
adds the full-display prefix width again. A selection extending right of the
caret therefore highlights pixels to the right of its actual trailing text.

Reproduction through the public keyboard path:

```rust
state.text = "ab".into();
state.focused = true;
state.cursor_end();
handle_key(&mut state, &PlatformKey::ArrowLeft, true, None, false, true);
state.set_preedit("ㄓㄨˋ", None);
// render_text_input: [0,0,240,40], Theme::light(), scale 1.0, focused.
// Compare the alpha-0.25 selection quad's x with the final Text run's origin.x.
```

Observed cursor=1, selected `b`, tail origin **46.875977**, highlight origin
**53.999023**: error **7.123047 logical px**, exactly the extra `a` prefix width.
`target/review-d2-probe.exe --nocapture` reproduces an assertion failure (exit 101).
The existing selection test selects forward and leaves the caret at the selection
end, so it never covers this tail-highlight branch. Fix by using only the preedit
width as the full-display tail shift, or measure offsets relative to the tail;
add backwards/cross-caret selection coverage across the scale matrix.

### F2 — LOW — composition in an empty field inherits placeholder colour

`crates/acme-textinput/src/lib.rs:938` (colour chosen), `:993` (passed to D2).
With empty committed text, a nonempty placeholder and active preedit, the
placeholder is correctly suppressed by the composing branch, but its muted
colour is still used to draw the user's actual composition.

Reproduced with `placeholder="type here"`, `preedit="ㄓㄨˋ"`, a focused empty field,
light theme and scale 1.0. The composition Text colour is
`(0.41960785, 0.44705883, 0.5019608, 1)`, the muted token, instead of foreground
`(0.05882353, 0.078431375, 0.09803922, 1)`. The second probe assertion fails.
Choose the normal/readonly text token when composition is active; retain the
placeholder token only when placeholder text is actually drawn. Add this case.

### F3 — LOW — the F1 test fails the crate formatting check

`apps/gallery/src/events/ime.rs:122`.
`cargo fmt -p acme-textinput -p acme-gallery -- --check` exits **1**:
rustfmt requires the new `assert!(state.scroll_offset > 0.0, ...)` on multiple
lines. No unrelated formatting difference was reported. Apply the crate formatter.

## Reproduced gates and controls

- `cargo test --locked -p acme-textinput`: **exit 0**, **160 passed**, no failed
  or ignored tests (105 unit + 20 IME DPI + 2 multipage + 12 preedit render +
  8 preedit scroll + 13 caret scroll; zero doctests).
- `cargo test --locked -p acme-gallery --bins`: **exit 0**, **34 passed, 1 ignored**.
- `cargo clippy --locked -p acme-textinput --all-targets -- -D warnings`: **exit 0**.
- `cargo clippy --locked -p acme-gallery --all-targets -- -D warnings`: **exit 0**.
  The concurrent Cargo cache lock was waited out; no command was restarted.
- Formatting: **FAIL**, as F3 above.
- Additional public-API probe, compiled from stdin with `rustc --test` against
  the just-built rlibs: **exit 101**, **1 passed / 2 failed**. Backwards selection
  and placeholder colour fail; malformed/mid-char/huge/reversed preedit offsets
  do not panic for plain or password fields, including combining text and emoji.
- Differential control: compiled `git show 271e5e5:crates/acme-textinput/src/lib.rs`
  from stdin as a temporary rlib. Compared complete HEAD/current Scene command
  lists for empty preedit: **48 cases identical** (4 scales × plain/password ×
  empty/ASCII/mixed-grapheme text × selected/unselected), with scroll and outer clip.
- F1 independent control at scales **1.0/1.25/1.5/2.0**: wide preedit gives raw
  content-local anchor **−139.25293**, while the clamped client anchor is **48**,
  exactly field x=40 plus padding=8. Existing unscrolled anchor also unchanged.
  `target/review-d2-controls.exe --nocapture`: **exit 0, 4 passed**, including
  the 48-case differential test, independent clamp test and 2 F1 module tests.

## Other scope checks and limits

Password composition is generated from one mask per grapheme; no plaintext run
is submitted. Split runs preserve their common origin baseline, physical glyph
coordinates and shared content/outer clip. Gallery forwards the platform byte
cursor through its event call site. `ime_caret_area` itself is unchanged.

File lengths: preedit 363, new tests 544, dispatch 377, main 651, IME 145 (<800).
`acme-textinput/src/lib.rs` remains **2826** lines, down from HEAD's **2852**:
the global <800 goal is not met there, but this is a pre-existing oversized file.
The hardcoded selection tint at lib.rs:1033 already exists in HEAD; D2 adds no
new hardcoded theme colour. It is recorded as legacy debt, not a new finding.

NOT_RUN: actual Windows 注音 IME/candidate-window interaction, native Gallery
visual/DPI acceptance, screen reader, real GPU drawing, hosted CI. Headless
geometry and CPU raster tests do not establish those manual/runtime results.
