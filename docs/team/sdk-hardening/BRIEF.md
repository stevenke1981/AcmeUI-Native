# SDK Production Hardening — Team Brief (BATCH-002)

Base: branch `sdk/production-hardening` (master `a083bd2` + merged PR #5 focus
navigation + PR #6 CPU renderer). Toolchain pinned to Rust 1.88.0
(`rust-toolchain.toml`). Platform of the authoring host: Windows 10, MSVC.

Read first: `AGENTS.md`, `docs/agent-team/ROADMAP.md`, `docs/agent-team/WORKFLOW.md`.

## Goal

Move AcmeUI-Native toward a production-usable SDK in bounded, verifiable
steps while preserving the existing architecture and public API
compatibility (no removed/renamed public items, no changed public function
signatures, no new required public struct fields on structs users build with
literals).

## Work items (one owner each; write only your listed files)

| ID | Prio | Owner role | Write set |
|---|---|---|---|
| W1 | P1 | team-rust-engineer | `crates/acme-platform/src/lib.rs`, new `crates/acme-platform/src/backend.rs` |
| W2 | P1 | team-test-engineer | new `crates/acme-render-cpu/tests/golden.rs`, new `crates/acme-render-cpu/tests/golden/**` |
| W3 | P2 | team-build-engineer | `crates/*/Cargo.toml` `[package]` metadata only, new `examples/sdk-consumer/**`, new `docs/guides/SDK_GUIDE.md` |

### W1 — Renderer backend policy and fatal-error propagation (acme-platform)

Observed problems:
1. `initialize_renderer` reads `ACME_RENDER_BACKEND` inline; only `cpu` is
   recognised, any other value (typo) silently means auto. No unit test.
2. When every window fails to create or initialise a renderer, `run()` logs
   and returns `Ok(())` — the process exits with success and no window.
3. When a renderer fails fatally at redraw time (`SurfaceAction::Exit`,
   failed GPU recovery and failed CPU fallback), the loop exits and `run()`
   still returns `Ok(())`.

Required:
- Add a pure, unit-tested policy in `backend.rs`:
  `pub enum RenderBackendPreference { Auto, Gpu, Cpu }` (derive Debug, Clone,
  Copy, PartialEq, Eq, Default=Auto) with
  `pub fn from_env_value(value: Option<&str>) -> Result<Self, String>`
  (case-insensitive, trims whitespace; `None`/empty → Auto; `auto`, `gpu`,
  `cpu` accepted; anything else → Err with a message listing valid values)
  and `pub fn from_env() -> Self` (reads `ACME_RENDER_BACKEND`; on Err logs
  `tracing::warn!` and returns Auto). Keep the documented semantics: `gpu`
  prefers GPU but still falls back to CPU when the `cpu-renderer` feature is
  compiled in (same as Auto); `cpu` forces CPU. Re-export from lib.rs.
- Add new `PlatformError` variants (adding variants is acceptable for 0.x
  and will be recorded in CHANGELOG; do NOT add `#[non_exhaustive]`):
  `NoWindows { reasons: Vec<String> }` and `Renderer(String)`.
  `run()` must return `Err(NoWindows{..})` when no window could be created,
  and `Err(Renderer(..))` when the loop exited because of an unrecoverable
  renderer failure. Normal close of the last window still returns `Ok(())`.
  Implement by recording a `fatal: Option<PlatformError>` in `Runtime`.
- Split selection logic so it is testable without a GPU: e.g. a pure
  `fn plan_renderer(pref, cpu_compiled: bool) -> RendererPlan` with tests.
- Do not change `Application`, `WindowConfig`, `PlatformEvent` or existing
  public signatures. No new dependencies.
- Tests: unit tests in `backend.rs` and lib.rs test module.
- Checks: `cargo test --locked -p acme-platform`,
  `cargo test --locked -p acme-platform --no-default-features`,
  `cargo clippy --locked -p acme-platform --all-targets --no-deps -- -D warnings`,
  `cargo fmt -p acme-platform -- --check`.

### W2 — Deterministic golden pixel regression (acme-render-cpu)

Observed: todos list "Screenshot golden / pixel regression" as open. The
CPU renderer (`acme_render_cpu::CpuRenderer`) is headless and deterministic.

Required:
- `tests/golden.rs` renders ≥5 canonical `acme_core::Scene`s that avoid text
  (font availability differs per OS): solid quads, rounded+bordered quads,
  nested clips, nested opacity layers, overlapping alpha, an RGBA image if
  the public API allows it. Read `src/lib.rs` and `tests/regression.rs` for the
  actual public API — do not invent APIs.
- Compare against committed fixtures in `tests/golden/*.ppm` (binary P6) with
  a documented per-channel tolerance (≤2) and a maximum differing-pixel
  count (0 by default). On mismatch, write `target/golden-actual/<name>.ppm`
  and a diff summary in the failure message.
- Regeneration: when env `ACME_BLESS_GOLDEN=1`, write fixtures instead of
  comparing (and still pass). Generate the fixtures once with that and commit
  them is the Lead's job — you produce them in the worktree; keep each fixture
  small (≤ 128×128).
- Do not modify `src/lib.rs`. If you find a real renderer bug, report it with a
  minimal reproduction instead of patching.
- Checks: `cargo test --locked -p acme-render-cpu`,
  `cargo clippy --locked -p acme-render-cpu --all-targets --no-deps -- -D warnings`,
  `cargo fmt -p acme-render-cpu -- --check`.

### W3 — SDK packaging and consumer onboarding

Observed: only `acme-style` has a `description`; no `repository`, `readme`,
`keywords`, `categories`, `rust-version` on most crates; no documented
external-consumer path or compatibility policy.

Required:
- For every `crates/*/Cargo.toml`: add `description` (accurate, one line,
  derived from that crate's lib.rs docs), `repository.workspace = true`,
  `rust-version.workspace = true` where missing, `keywords`/`categories` only
  if valid crates.io categories. Add `repository`/`homepage` to
  `[workspace.package]` is the Lead's job — just use `.workspace = true` and
  tell the Lead the exact lines to add to the root Cargo.toml:
  `repository = "https://github.com/stevenke1981/AcmeUI-Native"`.
  Do NOT change versions, dependencies or features.
- `examples/sdk-consumer/`: a standalone crate (own `[workspace]` table so it
  is NOT a member of the root workspace) that depends on the SDK crates via
  relative `path` deps, builds a minimal window app with the public API
  (`acme_platform::run`, `Application`, a Scene built through the same public
  path the Gallery uses — read `apps/acme-gallery` / `apps/playground` to find
  the real API), plus a headless `#[test]` that builds one frame Scene without
  opening a window. Include its own `Cargo.lock`? No — leave lock out and
  note it; the Lead decides.
- `docs/guides/SDK_GUIDE.md`: getting started (git/path dependency), crate
  map, renderer backend selection (`ACME_RENDER_BACKEND=auto|gpu|cpu`, the
  `cpu-renderer` feature), error handling of `run()` (W1 adds
  `PlatformError::NoWindows`/`Renderer` — describe them), feature flags of
  `acme-ui`, supported targets with honest verification status (desktop
  runtime vs cross-target library checks only), versioning/compatibility
  policy for 0.x (breaking changes bump minor, recorded in CHANGELOG), and
  known limitations (manual IME/DPI/screen-reader validation pending).
- Checks: `cargo check --locked --workspace`, and from
  `examples/sdk-consumer`: `cargo test` and `cargo build` (pinned toolchain).

## Hard rules (all agents)

- Only edit files in your write set. Shared files (root `Cargo.toml`,
  `Cargo.lock`, `.github/**`, `CHANGELOG`, `docs/status/**`, README) are the
  Lead's. If you need a change there, put the exact patch in your report.
- No `git add/commit/stash/checkout/reset`. No `cargo clean`. No new required
  dependencies. Never add GPUI. Never hardcode theme colours inside widgets.
- Run only your own crate's checks with `--locked` (except W3 consumer crate,
  which has no lock in the root graph). Use the pinned toolchain.
- Report honestly: PASS/FAIL/NOT_RUN with the exact command and tail of
  output. Unverified behaviour (native window, DPI, IME) must be stated as
  NOT_RUN.

## Report format

```
ID / role / files changed (list)
Summary of change (≤10 lines)
Commands run → status (+ last lines of output)
Patches requested for shared files (exact diff)
Risks / NOT_RUN / follow-ups
```
