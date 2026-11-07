# REPORT P5b — mobile_toggle, mobile_stepper, mobile_segmented (ROADMAP 014)

Branch `sdk/roadmap-014-mobile`. Written in stages: the audit below was recorded **before** any code change.

## 1. Audit (before changes)

| Component | What the builder produces | Messages it emits | Missing against the todo / task |
|---|---|---|---|
| `mobile_toggle` | `Row` of two `Label`s: the text and a glyph (`●` or `○`) | A fixed `on_change(M)` is stored as the `Row`'s container message. The new value is never computed. | No track or thumb geometry (only a text glyph). No `disabled` effect: the field exists but is ignored by `From`. No `on_change(bool)` with the new value. The press target is a `Row`; the Gallery hit collector only collects `Button`s, so a Row message is never delivered. Height is not 44 px. No state in the accessibility name. |
| `mobile_stepper` | `Row` of `[button −, Label value, button +]` | Fixed `on_decrement(M)` and `on_increment(M)`. No computed value. | The value is clamped for display only, and no `on_change(new_value)` is emitted. `i64::clamp` **panics when `min > max`**. `step` is stored but never used. Buttons are the desktop 36 px size, not 44 px. The disabled-at-limit logic exists (`min` for −, `max` for +). No interaction tests. |
| `mobile_segmented` | `Row` of `Label`s, with `"▣ "` prepended to the selected text | `on_select: Option<fn(usize) -> M>` is stored but never used. Segments are not pressable. | Segments are not interactive and have no theme highlight (a text prefix only). No 44 px touch height. An out-of-range `selected_index` is harmless, but it is untested. 0 or 1 segment is untested. |

Reference (`mobile_button`): sized variants, disabled state, a 44 px minimum height, a Gallery page, and layout tests.

## 2. Design, files changed, evidence

P5 completed section 1, added the four mobile test/support files, registered
the tests, added the stepper callback API, and partially implemented the toggle.
P2 took over the existing dirty worktree at base `5eb5283`, without reverting
P5's work. The handoff snapshot and hashes are in
`target/p2-mobile-evidence/handoff/` and `handoff.json`.

P2's first run, before any source change:
`cargo +1.88.0 test -p acme-ui --all-features mobile_ -- --nocapture`:
**RED: 60 passed, 20 failed** (80 executed; shell exit 1).
The log preserves the actual test failures; the native Cargo exit was not separately captured on this first run.
Seven segmented interaction tests, ten stepper interaction tests and three
obsolete toggle Row-structure tests failed. All eight inherited toggle
interaction tests passed. Raw output: `target/p2-mobile-evidence/initial-red.log`.

P2 owns only the three assigned component files, their test/support files,
the Gallery mobile demo and its tests in `apps/acme-gallery/src/main.rs`, the
three assigned todo lines, and this report. No dependencies or shared manifests
will change. Acceptance: laid-out hit dispatch, new-value messages, bounds and
disabled behavior, semantic theme styling, 44 px targets, accessibility walk,
one killed mutation per component, and the four task gates on Rust 1.88.0.

### Authorship and final design

| Area | P5's inherited work | P2's takeover work |
|---|---|---|
| Toggle | Track/thumb Stack composition, semantic colours, 44 px Button host, `on_toggle(fn(bool) -> M)`, checked name and disabled activation. All 8 inherited interaction tests already passed at takeover. | Retained that implementation; updated the 3 obsolete Row assertions to check the real Stack/Button structure and activation, added semantic-token coverage, and validated geometry/new-value/disabled behavior. |
| Stepper | Callback field/method and 12 interaction tests, with the original implementation still ignoring the callback/step. | Implemented computed values, clamp/saturating arithmetic, 44-by-44 buttons, disabled state and instance-prefixed child keys; added disabled and overflow regression tests. |
| Segmented | Registered 8 new interaction tests; production remained labels with an unused callback. | Replaced labels with real 44 px buttons, mapped each index, used Primary/Secondary theme variants, added disabled support and its test, and made the selection name explicit. |
| Test support | Real layout/snapshot helper with simulated point hits and Button activation. | Retained it; formatting only. The helper is not represented as an OS input test or Gallery dispatch itself. |
| Gallery | Existing mobile-button page only. | Added stateful toggle/stepper/segmented demos, actual Gallery hit/activate/rebuild tests, themed track/thumb paint and scroll clipping tests. |
| Accessibility/docs | Audit section 1 and report scaffold. | Added standalone public-API tests, completed sections 2 onward, and annotated the 3 todo lines with the outstanding baseline lint gate. |

The toggle keeps the existing fixed-message `.on_change(M)` API. Value-aware
users call `.on_toggle(fn(bool) -> M)`, which emits the opposite of `checked`
and takes precedence over the fixed message. No fixed-message caller changes
are required. As with other declarative builders, callbacks map the next
message while building; the application receives it on activation and rebuilds
the widget from updated state.

The stepper first sorts inverted endpoints and clamps the initial value.
Non-positive steps become one, then saturating add/subtract is clamped to the
normalized range. Explicit increment/decrement messages retain precedence for
their respective buttons. At a limit, or when globally disabled, the relevant
Button's existing activation guard emits nothing. `min == max` disables both.

Segmented selection outside the list highlights nothing; an empty list has no
hit areas. The selected button uses a semantic Primary variant plus
`▣ <name> (selected)` as a non-colour and accessible-name cue. All supplied
segments remain pressable unless disabled, including the already selected one.
No component contains hard-coded theme colours.

Changed source files are the three components; their three interaction test
files and shared test support; new `mobile_accessibility_tests.rs`; and
`apps/acme-gallery/src/main.rs`. Documentation changes are this report and only
the three assigned lines in `docs/status/todos.md`. The task file, manifests,
lockfile, other components and other crates were not modified.

## 3. RED and GREEN evidence

- Initial inherited state: **60 passed / 20 failed** in the 80-test mobile
  filter, before P2 changed any source. See section 2 and `initial-red.log`.
- Added disabled regressions first: compilation failed because the stepper and
  segmented builders lacked `.disabled`; `added-tests-red.log`, exit **101**.
- An intermediate implementation used floats with the Styled trait's `Length`
  arguments and failed to compile (`first-green.log`). Correcting the calls to
  `Length::px` produced **84 passed / 0 failed** (`components-green.log`).
- Gallery dispatch tests first failed to compile against the absent demo
  state/messages (`gallery-red.log`, exit **101**). After implementation,
  the hit tests and the existing retained mobile test passed.
- Independent review found that the new track/thumb paint ignored the scroll
  viewport. A dedicated regression first failed with **0 clipped quads versus
  2 expected** (`gallery-clip-red.log`, exit **101**). The fix uses the renderer's
  existing `ClippedQuad` with the original scroll clip. The focused Gallery run
  then passed **5/5** (`gallery-clip-green.log`), including light/dark frame
  paint and partial/fully-scrolled-out clipping assertions.

All relative logs in this report live under `target/p2-mobile-evidence/`.

## 4. Mutation checks

Each mutation was applied separately to the working source, compiled, and run
against exactly one named test using `--exact --nocapture`. Each run executed
one test, failed an assertion (not compilation), and returned **101**. A
`finally` block restored the original source bytes; hashes verified restoration.

| Component/path | Deliberate defect | Test that killed it | Result |
|---|---|---|---|
| Toggle | `f(!b.checked)` changed to `f(b.checked)` | `mobile::mobile_toggle::interaction_tests::press_emits_the_new_value` | KILLED |
| Stepper | Remove `.clamp(min, max)` from the computed increment | `mobile::mobile_stepper::interaction_tests::a_step_larger_than_the_range_clamps` | KILLED |
| Segmented | Send `f(0)` for every segment instead of `f(i)` | `mobile::mobile_segmented::interaction_tests::pressing_each_segment_emits_its_index` | KILLED |
| Gallery | Suppress styled-row backgrounds, leaving the switch unpainted | `tests::mobile_toggle_frame_contains_themed_track_and_thumb` | KILLED |

Exact command vectors, edits, exit codes and restoration results are recorded
in `mutations.json`; raw logs are `mutation-<name>.log`. Component commands use
`cargo +1.88.0 test --locked -p acme-ui --all-features <test> -- --exact --nocapture`;
the Gallery mutation uses `-p acme-ui-gallery`. The complete final test suites
ran after all mutations had been restored.

## 5. Accessibility public API

**3/3 PASS**, separately compiled from
`crates/acme-ui/src/mobile/mobile_accessibility_tests.rs`. The tests call
`acme_accessibility::build_tree_update` using real widget layout snapshots and
inspect the returned public AccessKit nodes:

- Toggle: Button role, `Wi-Fi · On/Off` name, disabled flag and 44 px bounds.
- Stepper: value label, minus/plus Button roles, disabled-at-min state.
- Segmented: Button names including explicit selection text, disabled flags.

No dependency was added. The test is a standalone Rust test harness because
`acme-ui` does not depend on `acme-accessibility`. Build the already existing
workspace libraries first:

```sh
cargo +1.88.0 build --locked -p acme-ui --all-features -p acme-accessibility -p acme-core -p acme-layout
```

Compile the test with `rustc +1.88.0 --edition 2024 --test`,
`-L dependency=target/debug/deps`, and four `--extern` mappings to
`target/debug/lib{acme_ui,acme_accessibility,acme_core,acme_layout}.rlib`.
On Windows, also pass the native `-L` search paths emitted in
`target/debug/build/*/output` (`cargo:rustc-link-search=`). Then execute the
resulting test binary with `--nocapture`. The exact successful argument vector
is preserved in `a11y-command.json`, with output in
`a11y-public-api-linked.log`. The first two manual compile attempts exposed
missing root rlibs and native link paths; both were corrected in the validation
command, without source, dependency or manifest workarounds.

These three standalone tests are **not** auto-discovered by `cargo test -p
acme-ui`; their execution and counts are reported separately. The existing
accessibility walk exposes these compositions as Buttons and text. Toggle and
selection state are conveyed by names, not newly introduced Switch/Radio roles
or typed checked/selected attributes. No accessibility crate changes or native
screen-reader validation are claimed.

## 6. Final gates and outstanding baseline lint failure

Local Windows x86_64; Rust **1.88.0**. Base/tested HEAD remains
`5eb5283627c94ee475700a0eaf5bef65a6ebb707`, with the uncommitted scoped diff.
All final dependency-resolving Cargo commands used `--locked`; the initial
inherited-state reproduction used the task's original unlocked test command.

| Command | Result |
|---|---|
| `cargo +1.88.0 fmt --all -- --check` | **PASS**, exit 0 |
| `cargo +1.88.0 clippy --locked -p acme-ui --all-targets --all-features -- -D warnings` | **FAIL**, exit 101: 35 existing diagnostics in 22 untouched files |
| `cargo +1.88.0 test --locked -p acme-ui --all-features` | **PASS**, exit 0: **766 passed, 0 failed**; 2 pre-existing ignored doctests |
| `cargo +1.88.0 check --locked --workspace --all-targets` | **PASS**, exit 0 |
| `cargo +1.88.0 test --locked -p acme-ui-gallery` | **PASS**, exit 0: **16 passed, 0 failed** |
| Standalone accessibility public API harness | **PASS**, exit 0: **3 passed** |
| Component + Gallery mutation checks | **4/4 KILLED**, source restored |

The strict Clippy command was run unchanged and was not suppressed or weakened.
Its first diagnostic is the pre-existing `clippy::manual_is_multiple_of` allow
in `inputs/date_picker.rs:102`, unknown to Rust 1.88. Other diagnostics include
uninlined format arguments, a manual range pattern and unnecessary lazy
evaluations in unrelated components. Every diagnostic's entire source file was
compared against `HEAD` and found unchanged (normalizing CRLF only):
`final/clippy-baseline.json` records all **35 locations / 22 files**.

No lint diagnostic targets any assigned component, helper, or new test.
Fixing those 22 unrelated files would violate this task's write scope. Therefore
the overall requested gate set is **not fully green**. The three todo entries
remain **unchecked**, with explicit notes that behavior/hit/accessibility/
mutation validation passes and final acceptance awaits the baseline Clippy
repair. P3 should assign or integrate that separate cleanup before closing them.

`final/results.json` records exact commands, timestamps, toolchain, statuses,
exit codes and raw-log hashes (`final/01.log` through `05.log`).

## 7. Review, control and remaining validation

P2 performed implementation and execution. Two actual independent native
reviewer instances used Codex GPT-6.1 Terra with non-overlapping scopes:

- Component Rust reviewer: no confirmed component correctness defect; noted
  that shared hit helpers simulate dispatch. This report labels that evidence
  accordingly; Gallery integration tests separately exercise the real
  `collect_hit_regions`, `hit`, `activate` and rebuild path.
- Gallery/accessibility reviewer: one **MEDIUM** clipping finding, reproduced
  RED, fixed and rechecked **APPROVE**. Reviewers ran no tests; execution
  evidence belongs to P2. No unresolved scoped review finding remains.

Human-facing flow: Gallery exposes current toggle, quantity and segment state
and rebuilds after actual hit activation. Agent-facing flow: headless tests
can operate the same laid-out controls and inspect emitted messages. These are
automated acceptance checks, not a claim of manual human/device approval.

**NOT_RUN:** real mobile/touch device, native Gallery interaction/screenshot,
screen reader, manual keyboard/DPI/IME acceptance, hosted CI, release packaging.
No git add/commit/stash/checkout/push or hosted CI action was performed. All
writes stayed in this worktree; no dependency or installed application changed.

**Handoff:** scoped implementation and tests are complete; required strict
Clippy acceptance remains blocked by verified pre-existing out-of-scope code.
