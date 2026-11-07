# P2 — acme-ui Clippy baseline cleanup

Branch: `chore/acme-ui-clippy-baseline`. Base and tested HEAD:
`5eb5283627c94ee475700a0eaf5bef65a6ebb707`, plus the uncommitted scoped diff.
Environment: local Windows x86_64, Rust **1.88.0**. Owner: P2.

## Scope and outcome

The initial strict Clippy run reproduced **35 diagnostics in 22 files**.
Changes are limited to those exact source files and this report:

- **31** format/panic argument expressions now use implicit named captures.
  Displayed text, keys, precision, alignment and debug formatting are retained.
- **2** `unwrap_or_else` calls on owned `String` fallbacks use `unwrap_or`.
  The fallback is an already-owned field, with no computation or custom-drop
  effects; the selected displayed value is unchanged.
- **1** integer OR pattern, `4 | 5 | 6 | 7`, becomes the identical `4..=7` range.
- **1** unknown `clippy::manual_is_multiple_of` allow is removed. The leap-year
  modulo calculation is unchanged; no API newer than Rust 1.88 is introduced.

No tests were added, removed, renamed, ignored or weakened. No public API,
dependency, manifest, lockfile, Gallery or unrelated component changes were made.
No new lint suppression or lint configuration was added. The source diff is
**34 inserted lines / 41 deleted lines** across the 22 files, including the two
line compactions required by rustfmt after shortening the range-label calls.

## Per-file changes

All paths in the following table are relative to `crates/acme-ui/src/`.

| File | Diagnostics fixed | Mechanical change |
|---|---:|---|
| `browser/image_gallery.rs` | 2 | Inline grid and thumbnail key captures; retain key text/index order. |
| `charts/area_chart.rs` | 1 | Inline `max_v` in range label; retain `.1` precision; apply rustfmt's call compaction. |
| `charts/bar_chart.rs` | 1 | Inline row key prefix and index. |
| `charts/gauge.rs` | 1 | Inline filled/empty arc strings in the same concatenation order. |
| `charts/line_chart.rs` | 1 | Inline `max_v` in range label; retain `.1` precision; apply rustfmt's call compaction. |
| `charts/scatter_chart.rs` | 1 | Inline point key prefix and index. |
| `desktop/taskbar.rs` | 1 | Inline taskbar item index. |
| `foundations/kbd_combo.rs` | 1 | Inline key text; preserve surrounding brackets. |
| `foundations/line_numbers.rs` | 2 | Inline line number with `:>4` alignment; inline existing test panic index. |
| `foundations/progress.rs` | 1 | Inline existing test panic size with `:?` debug formatting. |
| `foundations/progress_ring.rs` | 1 | Replace `4 | 5 | 6 | 7` with equivalent inclusive `4..=7`. |
| `inputs/cascader.rs` | 1 | Inline key prefix, level and option index in the same order. |
| `inputs/date_picker.rs` | 3 | Remove the Rust-1.88-unknown lint allow; inline day/filler key captures. |
| `inputs/date_range_picker.rs` | 4 | Use `unwrap_or` for the two owned String fields; inline the two date labels. |
| `inputs/form_field.rs` | 1 | Inline validation text; preserve warning glyph and spacing. |
| `inputs/number_input.rs` | 2 | Inline numeric value with the original `.0` and `.2` precision. |
| `inputs/pin_input.rs` | 1 | Inline index in the existing test panic. |
| `inputs/slider.rs` | 2 | Inline numeric value with the original `.0` and `.1` precision. |
| `inputs/time_picker.rs` | 4 | Inline hour/minute up/down key prefixes; retain all key suffixes. |
| `inputs/transfer.rs` | 1 | Inline header/count while preserving spacing and parentheses. |
| `inputs/tree_select.rs` | 2 | Inline prefix/index in both node-key locations. |
| `mobile/mobile_progress.rs` | 1 | Inline percentage value; retain the `%` suffix. |
| **Total** | **35** | **22 files** |

## Commands and evidence

Evidence is under `target/p2-clippy-baseline/`. Every dependency-resolving
Cargo command used `--locked`; the pinned toolchain was selected explicitly.
`baseline.json` and `final-complete.json` record command vectors, timestamps, exit codes,
environment and raw-log hashes. `mechanical-edits.json` records per-file edit
counts and before/after content hashes.

| Stage | Command | Result | Log |
|---|---|---|---|
| Before changes | `cargo +1.88.0 test --locked -p acme-ui --all-features` | **PASS**, exit 0; 734 unit tests passed, 0 failed; 2 ignored doctests. | `test-before.log` |
| Before changes | `cargo +1.88.0 clippy --locked -p acme-ui --all-targets --all-features -- -D warnings` | **FAIL**, exit 101; reproduced 35 diagnostics in 22 files. | `clippy-before.log` |
| After changes | Same strict Clippy command | **PASS**, exit 0; no diagnostic suppression. | `clippy-final.log` |
| After changes | Same all-feature test command | **PASS**, exit 0; 734 unit tests passed, 0 failed; 2 ignored doctests. | `test-final.log` |
| After changes | `cargo +1.88.0 check --locked --workspace --all-targets` | **PASS**, exit 0. | `workspace-check-final.log` |
| After changes | `cargo +1.88.0 fmt --all -- --check` | **PASS**, exit 0. | `fmt-check-final.log` |

`test-comparison-final.json` verifies that both result tuples and all **736 discovered
test entries** (734 executed unit tests and 2 ignored doctests) have identical
names and statuses before and after. This is stronger than just comparing
counts; no empty suite or test filtering was used.

The first format check requested call compaction in `area_chart.rs` and
`line_chart.rs` (`fmt-check.log`). Only those two files were formatted with
Rust 1.88 rustfmt. All four checks then passed on the final source; the earlier
attempt and logs remain preserved in `final.json`. Final source hashes and
scope verification are recorded in `summary.json`.

## Independent review and boundaries

Two actual native Codex GPT-6.1 Terra reviewer instances checked non-overlapping
scopes, read-only, without running tests or changing files:

- Rust reviewer: `date_picker`, `date_range_picker` and `progress_ring` —
  **APPROVE**, no findings. Confirmed the Rust 1.88 compatibility, inclusive
  range equivalence and owned-String fallback semantics.
- Code reviewer: the other 19 format-only files — **APPROVE**, no findings.
  Confirmed text/key output, precision, alignment and panic debug formatting.

P2 owns all implementation and execution evidence. No git add/commit/stash/
checkout/push, hosted CI, dependency installation or application restart was
performed. Native GUI acceptance and hosted CI are **NOT_RUN**; these local
checks do not claim either.
