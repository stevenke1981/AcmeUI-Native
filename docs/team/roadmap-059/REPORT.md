# ROADMAP 059 — Report: Layout keyboard hosts polish (NavigationMenu + Sidebar)

## Execution (actual)
| Role | Model | Work |
|---|---|---|
| Lead / integration / real-window GUI / gates | Claude Opus 5.5 | Branch, brief, GUI verification (found the frame-contrast P1), gate runs, driver guard fix, docs, PR |
| Adviser + independent reviewer | Claude Fable 5.2 | Contract (items A–I) and read-only review |
| Component | Claude Opus 5.5 (subagents ×2) | E2 in `checkbox_cards.rs`; `Interactive` ring border in `acme-widgets` `painter_bridge.rs` |
| Gallery (single writer of `main.rs`) | Claude Sonnet 5.5 (subagents ×2) | A–H; then the ring border in the Gallery painter and review P2 fixes |

## Change
- **A** A press on the Snackbar deactivates the NavigationMenu and Sidebar hosts.
- **B** Ctrl/Alt/Meta chords are ignored by both Layout hosts; held Enter/Space (and Esc in the menu) auto-repeat is ignored; arrow repeat still roves.
- **C** A child of a disabled parent can never be selected (demo entry 優先支援 under the disabled 支援).
- **D** Pin tests for ArrowUp with no child, unhandled keys, and a real-click category change.
- **E1** An always-present owner frame around each host; `CardVariant::Interactive` while that host owns the keyboard, `Plain` otherwise; no layout or hit-rect shift.
- **E2** The focused CheckboxCard uses `Interactive` instead of `Elevated`.
- **Painter (from the real-window P1):** `Interactive` is drawn with `colors.ring` at 2px (unless the card sets a width), in both the Gallery painter and the shared `painter_bridge`. Previously the Gallery used `accent` (≈1.1:1, invisible) and the shared painter drew it like `Outlined`.
- **F** Status labels in zh-TW: `鍵盤選單／鍵盤群組：使用中／未使用`.
- **G** F6 reveal margin 80, so the section heading stays visible.
- **H** Child reveal scrolls minimally and keeps the parent row visible; in a short viewport the child wins, with no oscillation.
- **Tooling:** `skills/human-acceptance` driver now aborts if the target loses the foreground or a pointer target is over another window.
- **I** (dropdown-close scroll clamp) stays a known limitation.

## Tests
Component: E2 test replaced (11 CheckboxCards tests); new `painter_bridge` test for the Interactive ring border. Gallery: 15 new tests appended (168 Gallery tests), including the painted ring colour and ≥3:1 contrast in light and dark, hit-rect stability across F6/F7/Esc, and a 200px short-viewport test. Mutation checks for the three pins; the category-change pin only fails when both deactivation layers are removed (each layer is already covered by an older test).

## Verification
Tested source: HEAD `4b9e7ad` plus the uncommitted 059 diff, Windows 11, rustc/cargo 1.88.0.

| Command | Result |
|---|---|
| `cargo fmt -p acme-ui -p acme-ui-gallery -- --check` | PASS |
| `cargo test --locked -p acme-ui --all-features` | PASS (916 passed, 0 failed, 2 ignored) |
| `cargo test --locked -p acme-ui-gallery` | PASS (168 passed, 0 failed) |
| `cargo clippy --locked -p acme-ui --all-features -p acme-ui-gallery --all-targets --no-deps -- -D warnings` | PASS |
| `cargo clippy --locked -p acme-widgets --all-targets --no-deps -- -D warnings` | PASS (`logs/painter-green.txt`) |
| `cargo check --locked --workspace --all-targets` | PASS |
| `cargo test --locked --workspace` | PASS (2136 passed, 0 failed, 11 ignored) |

Real-window GUI: see [GUI.md](GUI.md).

## Independent review (Fable 5.2): no P0/P1
- P2-1 short-viewport jump and P2-2 missing test → fixed. P2-4 shared painter → fixed (ring border). P2-5 geometry → test added.
- P2-3 evidence: the C test's RED log line predates an assertion edit; by inspection it is RED on master (`支援 ▾` is absent there), but the log does not show the final assertion failing.

## Follow-ups
- 062: ring contrast below 3:1 in dracula light and gruvbox dark; extend the all-pack ring test to assert every pack.
- Snackbar deactivation and owner frames for the Inputs hosts.
- `foundations::focus_ring` bakes in a light-theme colour.

## NOT_RUN
Real-window Snackbar press, Alt+Space system menu, short-window resize, Windows 注音 IME, screen reader, touch.

## Rollback
Revert the commit. No manifest or lockfile change.
