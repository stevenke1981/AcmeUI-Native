# ROADMAP 016 — Track P: platform input and host control for terminal apps

Owner: agy (Antigravity pane). Lead: Claude (integrates, reviews, commits).
Consumer: the CrewDesk desktop shell, which is being rewritten on AcmeUI-Native.
A terminal host needs every key, every modifier, a cross-thread wake-up and a way to
raise its window.

## Files you own (edit only these)

- `crates/acme-platform/src/lib.rs`
- new files under `crates/acme-platform/src/` (for example `waker.rs`, `keys.rs`)
- `crates/acme-platform/Cargo.toml` (no new dependencies)
- callers that stop compiling because of your API change, only the minimal fix:
  `apps/*/src/**` and other crates' `PlatformEvent::Key` matches.

Do not touch `crates/acme-text`, `crates/acme-core`, or any docs except your report.

## Hard rules

- Do NOT `git add`, `commit`, `stash`, `checkout`, `push`. The lead commits.
- No new dependencies. No GPUI. Keep winit types out of the public API.
- Do not run `cargo clean`.
- Toolchain is pinned: Rust 1.88.0 (`rust-toolchain.toml`).
- Do not write local machine paths into docs.

## Contract (implement exactly; names are the contract)

1. `PlatformKey` gains variants:
   `ArrowUp, ArrowDown, PageUp, PageDown, Insert, Function(u8)` (F1..=F24 → 1..=24).
   Map them from `winit::keyboard::NamedKey`. Keep existing variants.
2. `PlatformEvent::Key` gains fields `alt: bool`, `meta: bool`, `repeat: bool`.
   `text` must carry `KeyEvent::text` when winit provides it (covers Space and shifted
   characters), falling back to `Key::Character` as today.
   Use the modifier state that `handle_modifiers` already tracks.
3. `PlatformEvent::Scroll` gains `delta_x: f32` and `lines: bool`
   (`true` for `MouseScrollDelta::LineDelta`, `false` for pixel deltas; keep `delta_y`
   semantics and sign as today). Also add `x: f32, y: f32` = last pointer position.
4. Cross-thread wake-up:
   - `pub struct Waker` — `Clone + Send + Sync`, method `pub fn wake(&self)`.
     Backed by `winit::event_loop::EventLoopProxy<()>` (build the loop with
     `EventLoop::<()>::with_user_event()`); wake after the loop exited is a no-op.
   - `Application::on_waker(&mut self, waker: Waker) {}` — default no-op, called once
     before the first window is created.
   - `Application::wake(&mut self) -> bool { false }` — called on the UI thread for each
     delivered wake; `true` requests a redraw of every window.
5. Window requests from the app:
   - `pub enum WindowRequest { SetTitle(String), Focus, Minimize, Close }` (`Focus` =
     unminimize + show + focus).
   - `Application::take_window_requests(&mut self, window: WindowId) -> Vec<WindowRequest>`
     default `Vec::new()`. The runtime drains it after every event dispatch, after
     `wake`, and after `frame`, and applies the requests to that window.
6. `WindowConfig` gains `min_width: Option<f64>, min_height: Option<f64>` defaulting to
   `None` (keep `minimum_window_size` working; the per-window value wins).
   Update every `WindowConfig { .. }` literal in the repo.

## Tests (required, headless)

- Key mapping: a pure fn (e.g. `map_named_key(NamedKey) -> PlatformKey`) unit-tested for
  every new variant including F1, F12, F24.
- Waker: `Waker` is `Send + Sync + Clone` (static assert); waking after drop is a no-op.
- `WindowRequest` draining: a pure helper that the runtime uses is unit-tested.
- Existing tests keep passing.

## Acceptance

```
cargo test -p acme-platform
cargo check --workspace --all-targets
PYTHONUTF8=1 python scripts/check_changed_rustfmt.py --base master   # after lead commits
cargo clippy -p acme-platform --all-targets -- -D warnings
```

## Report

Write `docs/team/roadmap-016/REPORT-P-platform.md`: changed files, public API diff,
test commands with real pass/fail counts, anything NOT_RUN. Then reply in your pane with
one line: `TRACK-P DONE` (or `TRACK-P BLOCKED: <reason>`).

## Addendum A (lead, after dispatch)

7. `WindowConfig` also gains `position: Option<(f64, f64)>` (logical, outer top-left,
   default `None`) applied with `with_position` at creation. The consumer restores its
   saved bounds on startup.
8. `PlatformEvent::PointerButton` must carry modifier state: add `shift: bool,
   ctrl: bool, alt: bool` (Shift+drag selection in terminals).
9. IME: the consumer calls `Window::set_ime_allowed(true)` today implicitly? Confirm what
   the runtime does and document it in the report; add
   `WindowRequest::SetImeAllowed(bool)` so a terminal can keep IME on.

## Addendum B (lead)

10. `PlatformEvent::Key` also gains `character: Option<String>` = the `Key::Character`
    payload of the logical key (unaffected by Ctrl, e.g. Ctrl+C gives `Some("c")` while
    `text` is the control character U+0003). Terminals need the letter for Ctrl chords.
