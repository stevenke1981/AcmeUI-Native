# ROADMAP 052 — ConfirmDialog typed Confirm/Cancel reachability

2026-11-13 Asia/Taipei. Base `47b14c926d2d4bbd57d6b8a7dbf1acc524d35384` (ROADMAP 051 branch,
PR #60). Execution: **Codex CLI single writer + author review** — Codex wrote the
tests and the fix; it then hit its ChatGPT usage limit before writing this report,
so the reviewer re-ran RED, ran all gates and wrote this file. Independent review NOT_RUN.
[Scope](BRIEF.md).

## Finding (REPRODUCED, RED before fix)

`WidgetNode::Dialog` was lowered to a layout **leaf** in both layout paths
(`to_layout` and `to_layout_with_context`/binding) in `acme-widgets`, while
`children()` and the retained view bridge expose its `content`. ConfirmDialog's
Confirm/Cancel Buttons therefore had retained, focusable views but **no layout
rectangle**, so they could not be laid out, painted or hit-tested. Typed
messages and retained Confirm focus were already correct (green before the fix).

RED (production changes reverted, new tests kept; log: `v052/red.txt` in the run dir):
- `confirm_dialog_plain_layout_includes_content` and
  `confirm_dialog_gallery_layout_controls_are_reachable`: `Dialog content must participate in layout (left: 1, right: 0)`.
- `acme-widgets/tests/layout_binding::dialog_content_and_following_sibling_keep_layout_id_alignment_and_lengths`: `left: 0, right: 1`.

## Change

- `crates/acme-widgets/src/lib.rs`: Dialog lowers to a `Column` container holding its content in both
  layout paths; width/height `Length` values preserved; public `Dialog` struct/builder/variant unchanged.
- `crates/acme-accessibility/src/lib.rs`: DFS ID numbering no longer treats Dialog content as unlaid-out
  (otherwise IDs after a Dialog would misalign); doc comments updated; new bounds test.
- Four existing assertions that pinned the old `Leaf` kind were updated to `Column`
  (2 in command_palette.rs, 1 in confirm_dialog.rs, 1 in acme-widgets lib tests). Behaviour change visible to
  all Dialog users (Modal, CommandPalette, ConfirmDialog): their content is now laid out.
- New guards: `acme-ui/src/overlay/confirm_dialog_tests.rs` (3), `acme-widgets/tests/layout_binding.rs` (1),
  `acme-accessibility` (1). `title` and `open` are still not honored in layout/paint — investigation only, unchanged.
- Optional Gallery preview skipped.

## Verification (Linux x86_64, Rust/Cargo 1.88.0, reviewer-run on the uncommitted tree that became the commit)

| Check | Result |
|---|---|
| `cargo fmt --all --check` | PASS |
| `cargo test --locked -p acme-ui` | PASS: 609 / 2 integration / 1 doc ignored |
| `cargo test --locked -p acme-widgets` | PASS: 150 / 5 / 13 |
| `cargo test --locked -p acme-accessibility` | PASS: 128 |
| `cargo test --locked -p acme-devtools` | PASS: 18 |
| `cargo test --locked -p acme-ui-gallery` / `-p acme-gallery` | PASS: 90 / 39 (1 ignored) |
| `cargo clippy --locked -p acme-ui -p acme-widgets -p acme-accessibility --all-targets -- -D warnings` | PASS |
| `cargo check --locked --workspace --all-targets` | PASS |
| `cargo test --locked --workspace` | PASS (all result lines 0 failed) |

**NOT_RUN:** human Gallery, Windows 注音 IME, physical DPI/touch, GPU, screen reader, native X11
capture, SDK CPU profile, accessibility-test RED run (its fix shares lib.rs with the reverted file set; the
widgets/ui RED runs above cover the defect). Hosted CI: see PR.
