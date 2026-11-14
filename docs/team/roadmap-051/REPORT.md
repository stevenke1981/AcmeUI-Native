# ROADMAP 051 — Notification typed action/Close conversion and identity

2026-11-13 Asia/Taipei. Execution: **Codex CLI single writer + author review**.
Base `d303108ae934499cd5124cd5ec6bb32e547455f5`. Reviewer gates below ran on the
uncommitted working tree that became the single feature commit (docs-only edits
after the run: this table and ROADMAP row placement). Independent review NOT_RUN.
[Scope](BRIEF.md).

## Findings and change

**Already GREEN; no behavioral RED assertion.** Before any production edit,
all three new conversion regressions passed (7 Notification tests including the
four unchanged originals). An initial test-authoring compile error used an
unsupported ContainerBuilder.children method; corrected to child calls before
the behavioral run. This compile error is not evidence of a component defect.

The current Buttons already carry configured typed messages. RetainedTree matches
keys within each parent's children (`crates/acme-core/src/tree.rs`), so constant
`notif_action`/`notif_close` keys under distinct keyed Cards yield distinct NodeIds
and survive parent reorder. The body is always first; the optional description
is appended inside its content Column. Optional controls therefore do not shift
its identity, and the keyed surviving Button is reused when its sibling changes.
No production conversion change is justified; notification.rs only wires tests.
Public API, constant keys, composition and all four existing tests are preserved.

Four guard tests cover current/updated typed messages, both controls in both
owners retaining focus through keyed reorder, and optional siblings preserving
body Row/icon/content Column/title IDs and surviving action/Close focus, including
all optional combinations with both controls absent. The
body is passive and not focusable; body focus is not claimed. Removing a control
and later adding it may mount a new identity; continuity applies to surviving nodes.

The Overlay page adds a controlled desktop Notification preview before Fullscreen,
with visible state, action/Close counters, Show notification restore and guards
rejecting action/Close messages while hidden. Two Gallery tests cover pointer
action/Close exactly once, restore and cached stale-message rejection, plus real
layout/hit/paint checks for light/dark themes and scales 1/1.25/2, visible/hidden.
The desktop helper is named desktop_notification_examples to avoid the existing
mobile helper name (caught during the first edited-app compile). The existing
mobile_notification component and routing are unchanged.

## Reviewer verification

Run by the reviewer on Linux x86_64, Rust/Cargo 1.88.0, reusing target/. Workspace
tests, SDK CPU profile and native verification were not run.
`-p acme-gallery` targets apps/gallery; `-p acme-ui-gallery` targets the edited preview.

| Check | Result |
|---|---|
| Locked `cargo test --locked -p acme-ui` | PASS: 606 unit / 2 integration / 1 doc ignored |
| Locked `cargo test --locked -p acme-gallery` | PASS: 39 passed, 1 ignored |
| Locked `cargo test --locked -p acme-ui-gallery` (edited app) | PASS: 90 |
| `cargo clippy --locked -p acme-ui -p acme-ui-gallery --all-targets -- -D warnings` | PASS |
| `cargo fmt --all --check` | PASS |
| `cargo check --locked --workspace --all-targets` | PASS |
| Mutation probe (duplicate control key / wrong key in notification.rs, reverted) | 3 of the 4 new crate guards FAIL, as intended |

**NOT_RUN:** human Gallery, Windows 注音 IME, physical DPI/touch, GPU, screen
reader, native X11 CPU-window capture, SDK CPU profile. No evidence.json or PNGs.
Desktop placement/timing and full overlay semantics remain outside this slice.
