# ROADMAP 054 — ContextMenu typed item-message conversion

2026-11-14 Asia/Taipei. Base `29e7cc83d2f2ad0db7003afb8df1c79de99ded17` (ROADMAP 053 branch,
PR #62). Execution: **Codex CLI (OpenRouter nvidia/nemotron-3-ultra-550b-a55b:free) + author review**.
sigma + Groq `qwen/qwen3.8-27b` failed twice (Groq ITPM 7000; requests of 7685 then 10267 tokens)
and wrote no code. A first Nemotron session wrote an initial test file and then spent its time
blocked in the workspace-write sandbox (`index.crates.io` DNS). A second Nemotron session, with
sandbox bypass, got a RED run (existing tests green; new conversion tests failing because items
were still Labels) and a first production conversion, but then over-wrapped items in keyed Rows
inside a Column (which would have changed existing unit tests) and looped on test compile errors.
The reviewer stopped that session, restored the original closed-state and Card-child structure,
applied a smaller Button-binding fix, rewrote the conversion tests, added the Gallery preview and
these docs, and ran the gates. Independent review NOT_RUN. [Scope](BRIEF.md).

## Finding (REPRODUCED, RED before fix)

`ContextMenuBuilder` stored `MenuItem` values that already carry `message`, `disabled`,
`separator` and `key`, and `MenuItem::activate()` already returns a message only for
enabled non-separators, but the open conversion rendered every non-separator as an
unkeyed Label and never bound `item.message`. `children` (submenus) are stored and
still unused; that remains out of scope. Closed menus already rendered just the
anchor (guard was green before the production change).

RED (new tests against the unfixed conversion, log: `v054/red.txt` in the run dir):
- `converted_context_menu_items_respect_current_message_and_optional_control`:
  `enabled item with a message must be a Button, got Label(...)`.
- `context_menu_item_message_updates_when_changed`: `Cut must be a Button`.
- `context_menu_anchor_identity_and_focus_survive_open_item_changes`: `Cut must stay a Button`.
- `context_menu_actionable_items_have_finite_positive_non_overlapping_rects`:
  `Cut, Copy and empty-key items must be laid out` (0 Buttons found).

Already GREEN before the fix: `context_menu_closed_still_renders_only_the_anchor`
and all seven existing unit tests in `context_menu.rs`.

## Change

- `crates/acme-ui/src/overlay/context_menu.rs`: when open, an enabled non-separator
  item with `message` becomes an instance-keyed Button (`{menu_id}_{item.key}`, or
  `{menu_id}_item_{i}` if the key is empty) carrying the current message. A missing
  message stays a passive Label; disabled items stay Labels with no activation;
  separators stay Separator nodes. The dropdown Card gets a stable `{id}_dropdown`
  key so open-item changes do not remount the already-keyed Stack anchor. Closed
  menus still render just the anchor. Public fields/signatures and the From-only
  conversion are unchanged. DropdownMenu, Drawer, HoverCard, CommandPalette and
  submenu `children` were not changed.
- New guards: `crates/acme-ui/src/overlay/context_menu_tests.rs` (5). Existing seven
  unit tests in `context_menu.rs` kept. The geometry guard requires finite,
  positive, non-overlapping item rects; Stack itself can report a zero height in
  this layout path, so parent containment is only asserted when that box is
  positive.
- Controlled Overlay Gallery Cut/Copy/restore preview with stale-click rejection
  and pointer/theme/scale/layout tests (`apps/acme-gallery/src/context_menu.rs`,
  `context_menu_tests.rs`, `main.rs` wiring).

## Verification (Linux x86_64, Rust/Cargo 1.88.0, reviewer-run on the uncommitted tree)

| Check | Result |
|---|---|
| `cargo fmt --all --check` | PASS |
| `cargo test --locked -p acme-ui` | PASS: 621 / 2 integration / 1 doc ignored |
| `cargo test --locked -p acme-ui-gallery` | PASS: 94 |
| `cargo clippy --locked -p acme-ui -p acme-ui-gallery --all-targets -- -D warnings` | PASS |
| `cargo test --locked --workspace` | PASS (all result lines 0 failed) |

**NOT_RUN:** human Gallery, Windows 注音 IME, physical DPI/touch, GPU, screen reader, native X11
capture, SDK CPU profile. Hosted CI: see PR.
