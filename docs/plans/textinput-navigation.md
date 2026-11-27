# Text input navigation improvement plan

The review started at `354aabe9eb3fac326579b86992e452bc201479a4` on Windows with Rust 1.88.0. It followed ordinary arrow keys from the Gallery's focused field through `handle_key` to `TextInputState`. Navigation, selection extension, Backspace and Delete all use the adjacent-grapheme helpers in `crates/acme-textinput/src/boundary.rs`.

| Order | Source observation | Proposed change and acceptance |
| --- | --- | --- |
| 1. Adjacent graphemes | Both helpers start iterating at the beginning of the field, including when the caret is near the end. | Start segmentation near the caret using the locked Unicode library. Preserve results against whole-string segmentation and compare the same release benchmark before and after. |
| 2. Word selection | `TextInputState::select_word_at_offset` collects every word segment into a vector before finding the clicked segment. | Search the iterator directly. Check first, middle, shared and final boundaries in mixed text, and measure early and late double-clicks. |
| 3. No-op deletion | `delete_before_cursor` and `delete_after_cursor` clone the buffer before detecting the respective start or end of the field. | Check whether an edit is possible before taking an undo snapshot. Exercise edge selections and undo/redo, then measure repeated Backspace at the start and Delete at the end of a long field. |

## First batch

GPT-6.2 Sol owns the boundary implementation and its internal oracle tests in `crates/acme-textinput/src/boundary.rs`. GPT-6 Luna owns the public editing tests and measurement in `crates/acme-textinput/tests/grapheme_navigation.rs` and the focused keyboard scenario in `apps/gallery/src/dpi_tests.rs`. The lead integrates the two branches, records measurements and updates `docs/status/todos.md`. A separate reviewer checks the integrated changes. Items 2 and 3 are proposed follow-ups.

| Gate | Target |
| --- | --- |
| Unicode behavior | Previous and next results match whole-string segmentation at every tested offset. Include offsets inside UTF-8 scalars, inside clusters and beyond the end. Exercise combining marks, joined emoji, flag parity, Indic conjuncts and CRLF. |
| Input routes | Preserve public navigation, selection and deletion of whole graphemes. Exercise the focused Gallery field through application keyboard events. |
| Performance | At least halve the median time for a near-end left/right pair on the longest benchmark input. Compare the same harness, machine and release profile and record sample counts. This is a local acceptance target for this workload. |
| Integration | Run locked text-input and Gallery tests, formatting, targeted Clippy and the SDK CPU profile. Fix any changed-code diagnostics. Independent review must have zero unresolved correctness blockers. |

The change stays inside the private boundary helpers and uses the existing dependency. Native rendering, composition event handling and the public interface are outside this batch. Rollback is a revert of the implementation commit. The behavior tests and benchmark also run on the original implementation.
