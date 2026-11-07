# BATCH-002 Report — SDK production hardening (2026-11-07)

Brief: [BRIEF.md](BRIEF.md). Base: `a083bd2` + PR #5 + PR #6 merged on
`sdk/production-hardening`. Toolchain: Rust 1.88.0 MSVC, Windows 10 host.

## Team

| ID | Role / model | Result |
|---|---|---|
| Lead | Opus 5.5 | Brief, PR #5/#6 integration, root manifest, CI job, review fixes, docs |
| W1 | team-rust-engineer / Haiku | `RenderBackendPreference`, renderer plan, `NoWindows`/`Renderer` errors |
| W2 | team-test-engineer / Haiku | CPU golden tests, 6 fixtures, mutation-proven detection |
| W3 | team-build-engineer / Haiku | Crate metadata, `examples/sdk-consumer`, `SDK_GUIDE.md` |
| W4 | team-rust-engineer / Haiku | acme-ui test dead-code warnings (10 files); Lead finished the 4 `mobile_*` files |
| Review | team-independent-reviewer / Sonnet | APPROVE_WITH_FIXES, no CRITICAL/HIGH |

A first W4 attempt with the external `sigma` agent was stopped: its prompt file
was never created (the shell command was denied), it received an empty prompt and
started exploring unrelated directories. It changed no repository file.

## Review fixes applied by the Lead

- M1/M2/L6: guide links `docs/CHANGELOG.md`, golden tests documented as in CI,
  sketch marked `ignore`, time-bound "verified" column renamed to local evidence.
- M3: `PlatformError::Renderer` from a renderer exit now carries the root cause.
- L1: successful device-loss recovery keeps the window dirty (original behaviour).
- L2: a recorded fatal error is no longer lost when `run_app` also errors.
- Pre-existing rustdoc broken link in `Application::windows` fixed.
- Not changed: L4 (golden output path ignores `CARGO_TARGET_DIR`), L5, L7 — low value.

## Evidence

See the final Lead gate run in the PR description (exact commands and results).

## NOT_RUN

Native window runs (normal, forced CPU, no-window failure), injected GPU device
loss, 注音 IME, DPI 125/150/200%, screen readers, macOS/Linux locally (CI only),
the `Runtime` fatal-path wiring end to end (pure helpers are unit-tested).

## Next batch

ROADMAP 003: prove the retained tree is used by the real app path, then 004
(most-used controls and keyboard/focus workflows with Gallery evidence).
