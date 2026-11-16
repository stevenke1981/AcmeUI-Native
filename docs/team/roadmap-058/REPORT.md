# ROADMAP 058 — Report: CheckboxCards typed toggling and roving keyboard focus

## Execution (actual)
| Role | Model | Work |
|---|---|---|
| Lead / integration / real-window GUI | Claude Opus 5.5 | Branch, brief, GUI verification, gates re-run, docs, commit and PR |
| Adviser + independent reviewer | Claude Fable 5.2 | Decisions D1–D7 (contract); read-only review of the finished diff |
| Component implementer | Claude Opus 5.5 (subagent) | `crates/acme-ui/src/inputs/checkbox_cards.rs` |
| Gallery worker (single writer of `main.rs`) | Claude Sonnet 5.5 (subagent ×2: build, then review fixes in a fresh agent) | `apps/acme-gallery/src/{checkbox_cards.rs,checkbox_cards_tests.rs,radio_cards.rs,main.rs}` |
| Verification | Claude Haiku 5.5 (subagent), then Lead re-run after fixes | Gates and raw logs |

## Problem (OBSERVED)
`CheckboxCardsBuilder::on_toggle` existed but was never bound; every card was a passive Label, so nothing could be toggled by pointer or keyboard. RED: `logs/component-red.txt`.

## Change
- **Component (additive):** handler-mode keys `{id}`, `{id}:card:{v}`, `{id}:option:{v}`; full-width Buttons bound to `on_toggle(v)`; checked = Primary `☑`, unchecked = Ghost `☐`. New `focused(value)` (Elevated card + `▸ `) and `navigate_focus(current, forward) -> Option<&str>` (wraps, never emits). Legacy tree unchanged without a handler. New pub field `focused` (follows the existing all-pub builder pattern).
- **Gallery → Inputs:** "CheckboxCards — typed toggling" after RadioCards (電子郵件 / 簡訊 / 推播通知 / 每週摘要). Pointer toggles and moves focus; F7 enters; arrows move focus; Space toggles (auto-repeat ignored); Enter is unhandled; Esc leaves; Ctrl/Alt/Meta chords are ignored. Reveal only when hidden, including the focused card. Unknown values are rejected.
- **Page owner:** `inputs_keyboard_event` — F6 RadioCards and F7 CheckboxCards are mutually exclusive.
- **055 P2 fold-ins:** RadioCards F6 reveals only when hidden, uses `reset_hover_press()`, ignores modifier chords; tests for pointer-off deactivation and the preedit guard.

## Tests
Component: 7 new (11 total). Gallery: 20 new in `checkbox_cards_tests.rs` (150 Gallery tests). Mutation checks: forcing reveal every time fails the reveal-only-when-hidden tests; removing focus reveal fails the three focus-reveal tests.

## Verification
Tested source: HEAD `3a1814c` plus the uncommitted 058 diff (`logs/env.txt`), Windows 11, rustc/cargo 1.88.0. Final run after review fixes:

| Command | Result |
|---|---|
| `cargo fmt -p acme-ui -p acme-ui-gallery -- --check` | PASS |
| `cargo test --locked -p acme-ui --all-features` | PASS |
| `cargo test --locked -p acme-ui-gallery` | PASS (150 passed, 0 failed) |
| `cargo clippy --locked -p acme-ui --all-features -p acme-ui-gallery --all-targets --no-deps -- -D warnings` | PASS |
| `cargo check --locked --workspace --all-targets` | PASS |
| `cargo test --locked --workspace` | PASS (2117 passed, 0 failed, 11 ignored) |

Real-window GUI: 14/14 steps PASS with real pointer and keyboard input at 150% DPI, including held Space and repeated still-pointer clicks; see [GUI.md](GUI.md).

## Independent review (Fable 5.2): no P0, no functional P1
- Process P1 (workspace log incomplete, no CPU profile): the workspace log was read while still being written; the final run is complete (above). CPU profile evidence is recorded separately after commit.
- P2 fixed: held-Space flicker, modifier chords, Space on an off-screen card, F7 hiding the focused card in a short viewport, stale hover on a no-op confirm; tests added for reset_hover_press on every key, pointer/focus `None` contract, arrow reveal.
- P2 noted: `gallery_radio_cards_preedit_blocks_arrows` pins a pre-existing guard; the new pub field `focused` is an API addition.

## Follow-ups
- Focus visual on checked cards is weak (Elevated barely visible) → 059 focus-visual item.
- ☑ renders via colour-emoji fallback → future icon pass.
- Selected Gallery category contrast → 060.

## NOT_RUN
Windows 注音 IME composition (the preedit guard cannot trigger on Inputs; tested only by setting `preedit`), screen reader, touch, other DPI/monitors, dark theme in the real window.

## Rollback
Revert the three tracked source files and delete `apps/acme-gallery/src/checkbox_cards{,_tests}.rs`. No manifest or lockfile change.
