# ROADMAP 006 — Report

Branch `sdk/roadmap-006-bench-atlas`, base master `d4dc148` merged as `9c4e629`.
Windows 10 Pro 19045, Rust 1.88.0 MSVC.

## Execution mode (actual)

| Item | Who | Notes |
|---|---|---|
| B1 benchmark runner | implemented in a previous session (uncommitted work tree inherited) | author agent/model not recorded in this session |
| B2 atlas lifecycle tests | same | tests encoded the pre-fix bugs |
| `GlyphAtlas` deferred eviction (`acme-text/src/lib.rs`) | previous session, uncommitted | left the B2 tests failing (3/6) |
| Test realignment, review fixes, docs | Lead (Opus 5.5, this session) | |
| Independent review | Sonnet, read-only (`team-independent-reviewer`) | APPROVE_WITH_FIXES |

## Outcome

- `apps/benchmark` runs headless (no window / GPU device), 13 cases, warm-up +
  sampled min/median/p95/mean, `--quick`, `--json` schema v1 with environment.
  `tests/quick_suite.rs` runs the quick suite and parses the JSON back.
- Atlas: mid-frame eviction no longer overwrites glyphs already handed out in
  the frame (reproduced overlap → now `overflow_mid_frame_defers_eviction_to_next_frame`).
  Lifecycle invariant added: all `PreparedText`s of one frame share one generation.
- 600-frame rotation (256x256, 24 px, 62 Latin + 215 CJK + 4 emoji on this
  host): hot-set hit rate 98.33 %, overall 96.71 %, 8 evictions + 1 explicit
  clear, generation 9. Measured, not asserted.

## Review findings and disposition

| ID | Sev | Disposition |
|---|---|---|
| F1 `RESULTS_PLACEHOLDER`, unverified power-plan claim | MED | fixed: one real full release run recorded with caveats; claim removed |
| F2 no CHANGELOG / stale struct doc | MED | fixed |
| F3 `clear()` kept `needs_space` → needless later wipe | LOW | fixed + unit test (RED without fix, GREEN with) |
| F4 `height * 3` u32 overflow | LOW | fixed (u64 math) + unit test |
| F5 O(entries) scan per dropped glyph | LOW | deferred (correct; perf only) |
| F6 tofu collapse could fail pressure asserts on CI without CJK fonts | MED | fixed: `shapeable_chars` keeps only distinct atlas entries |
| F7 tests need system fonts | LOW | accepted (existing unit tests already require `system_faces > 0`) |
| F8 `--json --quick` wrote file `--quick` | LOW | fixed + test |
| F9 bad JSON path failed after the whole run | LOW | fixed: path opened before the suite |
| F10 git SHA from CWD, no dirty flag; rustc from PATH | LOW | deferred, documented here |
| F11 no debug-build warning | LOW | fixed (stderr warning) |

## Evidence (this session)

- `cargo test --locked --workspace` (before review fixes): 1426 passed, 0 failed.
- After fixes: `cargo test --locked -p acme-text -p acme-benchmark` PASS
  (11 + 5 [+1 ignored] + 33 + 2); clippy `-D warnings` PASS; rustfmt PASS.
- Full workspace re-run after fixes: see commit message / PR checks.
- `cargo run --release -p acme-benchmark` full and `--quick`: PASS (numbers in BENCHMARKS.md).

## NOT_RUN

- Gallery screenshot / text goldens (system fonts not portable).
- GPU (`acme-render-wgpu`) timing; end-to-end frame time on a real window.
- Linux/macOS font behaviour of the lifecycle tests (CI will exercise them).
- Multi-run baseline on a quiet machine.
