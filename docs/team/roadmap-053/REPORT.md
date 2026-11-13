# ROADMAP 053 — Modal typed Close conversion

2026-11-13 Asia/Taipei. Base `13ee4d912c9ac90c489d06e21d5a1125c4d11543` (ROADMAP 052 branch,
PR #61). Execution: **Codex CLI (OpenRouter nvidia/nemotron-3-ultra-550b-a55b:free) single writer + author review**
— Codex wrote the crate tests, the production fix and the Gallery preview/tests.
It then looped on Gallery test discovery after running `-p acme-gallery` (the
desktop Gallery crate is `acme-ui-gallery`) and did not write these docs, so the
reviewer stopped that session, removed an unused footer `AboutShow` coupling and
a duplicate test-module include, wrote this file and ran the gates. Independent
review NOT_RUN. [Scope](BRIEF.md).

## Finding (REPRODUCED, RED before fix)

`ModalBuilder` stored `on_close` and `closable`, but `build()` rendered the close
control as an unkeyed Row wrapping a "✕" Label and never bound the message.
After ROADMAP 052, that Label already had a layout rectangle (two geometry
guards were green before the production change). Typed Close was not converted.

RED (new tests against the unfixed `build()`, log: `v053/red.txt` in the run dir):
- `converted_modal_close_respects_current_message_and_optional_control`,
  `modal_body_focus_survives_optional_close_header_and_current_close_updates`,
  `modal_body_and_footer_focus_stable_across_footer_changes` and
  `modal_retains_title_identity`: `index out of bounds: the len is 0 but the index is 0`
  (tests expected a keyed body/title tree the old header did not build).
- `modal_close_updates_when_on_close_changes`: `Expected Button`.

Already GREEN before the fix: `modal_plain_layout_includes_content` and
`modal_gallery_layout_close_has_finite_positive_non_overlapping_rect`.

## Change

- `crates/acme-ui/src/overlay/modal.rs`: when closable and `on_close` is Some,
  emit an instance-keyed Close Button (`{id}_close`) carrying the current
  message; when closable and `on_close` is None, keep a passive "✕" Label;
  `closable(false)` omits the control. Header title row, body column and
  footer row get stable keys so optional close/footer changes do not remount
  siblings. Public fields/signatures, unused `backdrop` metadata and Dialog
  composition are unchanged.
- New guards: `crates/acme-ui/src/overlay/modal_tests.rs` (7). Existing five
  unit tests in `modal.rs` kept. Reviewer inlined four Clippy format-args in the
  geometry test so `-D warnings` passes.
- Controlled Overlay Gallery close/restore preview with stale-close rejection
  and pointer/theme/scale/layout tests (`apps/acme-gallery/src/modal.rs`,
  `modal_tests.rs`, `main.rs` wiring). Dialog `title`/`open` policy, backdrop
  painting and Escape/focus trap were not changed.

## Verification (Linux x86_64, Rust/Cargo 1.88.0, reviewer-run on the uncommitted tree)

| Check | Result |
|---|---|
| `cargo fmt --all --check` | PASS |
| `cargo test --locked -p acme-ui` | PASS: 616 / 2 integration / 1 doc ignored |
| `cargo test --locked -p acme-ui-gallery` | PASS: 92 |
| `cargo clippy --locked -p acme-ui -p acme-ui-gallery --all-targets -- -D warnings` | PASS |
| `cargo check --locked --workspace --all-targets` | PASS |
| `cargo test --locked --workspace` | PASS (all result lines 0 failed) |
| Mutation: Close Button lowered to Label | FAIL as required (`converted_modal_close_respects_current_message_and_optional_control`: configured close must carry its message; passive close remains Label) |

**NOT_RUN:** human Gallery, Windows 注音 IME, physical DPI/touch, GPU, screen reader, native X11
capture, SDK CPU profile. Hosted CI: see PR.
