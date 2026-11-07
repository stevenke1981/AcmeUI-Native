# ROADMAP 006 — Benchmarks methodology and atlas lifecycle (BATCH-006)

ROADMAP row 006: pixel baselines, atlas lifecycle, measured CPU/GPU
performance; "reproducible images and benchmark methodology; no invented FPS
targets". Base: `sdk/roadmap-006-bench-atlas` (on ROADMAP 004/005). Rust 1.88.0
pinned, Windows MSVC host. Read `AGENTS.md`.

Already done: text-free CPU pixel goldens (`acme-render-cpu/tests/golden.rs`)
and DPI pixel mapping (`tests/dpi.rs`). **Out of scope:** Gallery screenshot
goldens — the Gallery uses system fonts, which differ per OS and CI image, so a
portable text golden is not reproducible yet (record as NOT_RUN with reason).

## Problems (OBSERVED)

- `apps/benchmark` claims to be headless but runs through
  `acme_platform::run`, i.e. opens a window, so it cannot run in CI. Its output
  is `tracing` log lines only; no machine-readable results, no environment
  record, single runs without statistics.
- Glyph atlas has unit tests per operation but no long-running lifecycle
  evidence (many frames, rotating glyph sets incl. CJK, capacity pressure).

## Work items

| ID | Role | Write set |
|---|---|---|
| B1 | team-performance-engineer | `apps/benchmark/src/**`, `apps/benchmark/Cargo.toml` (path deps on workspace crates only, e.g. add `acme-render-cpu`; no new external crates) , new `docs/guides/BENCHMARKS.md` |
| B2 | team-test-engineer | new `crates/acme-text/tests/atlas_lifecycle.rs` |

### B1 — headless, reproducible benchmark runner

- Default mode runs **without a window**: `cargo run --release -p acme-benchmark -- [--quick] [--json PATH]`.
  Keep a `--window` flag (old behaviour) only if the window measures something
  the headless path cannot; otherwise remove the window path and say so.
- Each benchmark: warm-up iterations, then N timed samples; report
  min / median / p95 / mean (ns) and samples count. `--quick` uses small N for
  CI smoke.
- Benchmarks: existing layout / reconciliation / frame-build cases, plus
  retained reconcile + `bind_retained` + `to_layout_with_binding` on a 1000-node
  keyed tree, and `acme_render_cpu::CpuRenderer::render_scene` of a
  1100x760 scene with ~2000 quads at scale 1.0 and 2.0.
- JSON output (hand-written serialisation, no serde): schema version, git SHA if
  available via `git rev-parse` (optional, never fail), rustc version
  (`rustc -V`, optional), OS/arch (`std::env::consts`), profile (debug/release),
  CPU count, timestamp, and per-benchmark stats.
- Unit tests for the statistics helper (median/p95 on known inputs) and that
  `--quick` run completes and emits valid JSON shape (parse back minimally).
- `docs/guides/BENCHMARKS.md`: how to run, what each case measures, how to
  compare two runs (same machine, release, quiet system, ≥3 runs, compare
  medians), and an explicit statement that there are **no FPS targets or CI
  thresholds** yet and why (shared CI runners are noisy).

### B2 — atlas lifecycle under pressure

Read `crates/acme-text/src/lib.rs` (`GlyphAtlas`, `begin_frame`, eviction,
`clear`, generation, `PreparedText`). Integration tests using only the public
API:
- simulate ≥500 frames rotating through glyph sets (Latin, Traditional Chinese
  common characters, emoji if fonts allow) larger than a small atlas
  (e.g. 256x256): no panic, every glyph needed by the current frame is
  resolvable after preparation, atlas usage never exceeds capacity;
- `clear()` (GPU recovery) mid-run forces re-upload and bumps the generation;
- a hot set reused every frame is not evicted while stale glyphs are;
- report measured hit rate in the test output (`--nocapture`) without asserting
  a made-up target; assert only invariants.
Fonts differ per OS: skip characters the system cannot shape (detect via the
public API) instead of failing, but assert that at least the Latin set works.

## Hard rules

Only your write set. No git add/commit/stash/checkout/reset, no cargo clean,
`CARGO_TARGET_DIR=target/<id>`, pinned toolchain, `--locked` (B1: adding a
path dependency changes Cargo.lock — that is allowed only for path deps; report
the exact lock diff; the Lead commits it). No GUI launch. Honest
PASS/FAIL/NOT_RUN; report real measured numbers, never invent targets.

## Report format

```
ID / role / files changed
Summary (≤10 lines) incl. measured numbers (machine, profile)
Commands → status (+ output tails)
Patches requested for shared files
Risks / NOT_RUN / follow-ups
```
