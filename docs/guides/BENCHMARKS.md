# Benchmarks

`apps/benchmark` (crate `acme-benchmark`) measures the CPU-side hot paths of
AcmeUI: layout, retained-tree reconciliation and binding, frame and widget-tree
construction, text shaping, and the software rasterizer. It is a headless
runner: it never opens a window and never creates a GPU device, so it runs on
a plain Windows shell or a CI image.

## Quick start

```bash
# Full run (release is the only profile whose numbers mean anything)
cargo run --release --locked -p acme-benchmark -- --json target/bench/run1.json

# Smoke run: few samples, numbers are noisy, for "does it still run" checks
cargo run --release --locked -p acme-benchmark -- --quick --json target/bench/smoke.json
```

Options:

| Flag | Meaning |
|------|---------|
| `--quick` | 2 warm-up runs and 9 samples per case instead of 10 and 51. |
| `--json PATH` | Also write the results as JSON (schema v1). Parent directories are created. |
| `--help` | Print usage. |

There is no `--window` option. The old runner opened a window only to get a
frame callback; every measurement it made runs without one, so the window path
was removed.

## What each case measures

Every case runs its closure `warmup` times untimed, then `samples` times
timed. One sample is one call. Input data is built outside the timed region
unless the case says otherwise.

| Case name | What is timed | Notes |
|-----------|---------------|-------|
| `layout/100_nodes`, `layout/500_nodes`, `layout/1000_nodes` | `LayoutEngine::compute` on a balanced tree (fanout 4) at 800x600. | Taffy layout only. Tree built outside the timer. |
| `reconcile/100_keys_5_rounds_fresh_tree` | Five keyed orderings applied to an empty `RetainedTree`: mount, reverse (reuse), remove 10, re-add and rotate, remove 20 and add 10 new keys. | Includes creating and dropping the tree on each sample. `mounted`, `reused`, `removed` counts are in `meta`. |
| `retained/widget_to_views_1011_widgets` | `widget_to_views` on a keyed tree: root column, 10 keyed rows, 100 keyed buttons per row. | Widget-to-shadow-view conversion. |
| `retained/reconcile_steady_1011_widgets` | `reconcile_roots` with the same views every sample (steady state, all nodes reused). | Pre-converted views, so only reconciliation is timed. |
| `retained/bind_1011_widgets` | `bind_retained` on the reconciled tree. Fails the run if any mismatch is reported. | |
| `retained/to_layout_with_binding_1011_widgets` | `to_layout_with_binding` using a binding built once outside the timer. | Layout-tree construction only, no Taffy pass. |
| `frame_build/1200_quads_100_clipped_50_text` | Building a `Frame` with 1000 solid quads, 200 rounded quads with borders, 100 clipped quads, and 50 empty text runs. | Includes the `Vec` allocations for the frame. |
| `widget_tree/build_50_rows` | Building a 50-row column of rows, each with a label and a button. | Widget-builder cost. |
| `text/shape_prepare_4_strings` | `FontSystem::shape` + `prepare` for four mixed Latin, CJK and symbol strings into one persistent 2048x2048 `GlyphAtlas`. | Depends on the system fonts. See limitations. |
| `text/atlas_pressure_multipage` | 144-frame text rotation into a 256x256 atlas with a 4-page budget (timed); the same workload is replayed untimed with a 1-page budget for counters. | `meta`: pages, page evictions, generation, drawn and refused glyphs for both budgets. Depends on the system fonts. |
| `render_cpu/2000_quads_scale_1.0` | `CpuRenderer::render_scene` on a 1100x760 framebuffer with 2000 quads (a third rounded with a border). | Software rasterizer, no GPU. |
| `render_cpu/2000_quads_scale_2.0` | Same scene at scale factor 2.0. The logical extent is halved, so the framebuffer is covered at the same physical size. | |

Each case also records a `meta` object: element counts such as the number of
widgets or quads, and for the render cases an FNV-1a checksum of the output
pixels. The checksum is computed once, outside the timer. If it changes
between runs of the same code, the rasterized output changed.

## JSON output (schema v1)

```json
{
  "schema": "acmeui-bench",
  "schema_version": 1,
  "package": "acme-benchmark",
  "package_version": "0.2.0",
  "mode": "quick",
  "config": {"warmup": 2, "samples": 9},
  "environment": {"git_sha": "...", "rustc": "rustc 1.88.0 (...)", "os": "windows",
                  "arch": "x86_64", "profile": "release", "cpu_count": 24,
                  "cpu_model": "...", "timestamp_utc": "2026-11-07T00:00:00Z"},
  "benchmarks": [
    {"name": "layout/100_nodes",
     "stats": {"samples": 9, "min_ns": 0, "median_ns": 0.0, "p95_ns": 0,
               "mean_ns": 0.0, "max_ns": 0},
     "meta": {"layout_results": 100}}
  ]
}
```

- All durations are nanoseconds. `median_ns` and `mean_ns` may be fractional.
- `git_sha`, `rustc` and `cpu_model` are `null` when the probe is unavailable.
  The probes never fail the run.
- `profile` is `debug` or `release` from `cfg!(debug_assertions)`. Debug
  numbers are not comparable with release numbers.
- Statistics: `min` is the fastest sample; `median` is the middle sample (the
  mean of the two middle samples for an even count); `p95` is the nearest-rank
  95th percentile, always an observed sample; `mean` is the arithmetic mean.

## How to compare two runs

Numbers from one run on one machine are only a snapshot. To decide whether a
change made something faster or slower:

1. Use the **same machine**, a **release** build, and a quiet system: close
   browsers and other heavy programs, keep the machine on AC power, and keep
   the power plan fixed.
2. Build the baseline and the candidate from the same toolchain (`rust-toolchain.toml`,
   1.88.0) and record both `git_sha` values.
3. Run the **full** suite at least **three times per side**, alternating sides
   if possible, and write each run with `--json`.
4. Compare **medians**, per case, across runs. Report the spread (minimum to
   maximum of the three medians) next to each median.
5. Treat a difference as real only if it is larger than the spread observed
   within each side. Use p95 to check for tail regressions, not to rank runs.
6. Never compare percentages across different machines, power states, or
   toolchains. Re-measure the baseline on the new machine instead.

Do **not** use `--quick` output for comparisons. Nine samples cannot separate a
10% change from noise.

## Reference results

One full run (10 warm-up, 51 samples per case), recorded 2026-11-07 with
`cargo run --release --locked -p acme-benchmark -- --json ...` on the working
tree of ROADMAP 006 (base commit `9c4e629`, benchmark and atlas changes not yet
committed). Machine: Windows 10 Pro 19045, x86_64, AMD64 Family 23 Model 113
(24 logical CPUs), rustc 1.88.0, release profile, other desktop programs
running (not a quiet system).

```text
benchmark                                                 samples          min       median          p95         mean
layout/100_nodes                                               51    205.60 us    225.20 us    310.80 us    240.61 us
layout/500_nodes                                               51      1.58 ms      1.81 ms      2.23 ms      1.87 ms
layout/1000_nodes                                              51      2.97 ms      5.81 ms      7.09 ms      5.65 ms
reconcile/100_keys_5_rounds_fresh_tree                         51    188.00 us    194.00 us    252.00 us    202.93 us
retained/widget_to_views_1011_widgets                          51     38.80 us     39.60 us     52.90 us     41.36 us
retained/reconcile_steady_1011_widgets                         51    396.50 us    403.50 us    554.50 us    431.20 us
retained/bind_1011_widgets                                     51     66.90 us     69.50 us    101.30 us     80.51 us
retained/to_layout_with_binding_1011_widgets                   51    131.90 us    142.20 us    191.10 us    150.54 us
frame_build/1200_quads_100_clipped_50_text                     51     25.70 us     25.90 us     45.30 us     31.65 us
widget_tree/build_50_rows                                      51     31.10 us     33.30 us     51.30 us     37.26 us
text/shape_prepare_4_strings                                   51    439.00 us    476.20 us    661.40 us    503.42 us
render_cpu/2000_quads_scale_1.0                                51    103.01 ms    114.85 ms    126.55 ms    115.32 ms
render_cpu/2000_quads_scale_2.0                                51    385.45 ms    397.90 ms    429.95 ms    402.01 ms
```

These are a single run, so they show orders of magnitude, not a baseline. On
the same machine a `--quick` run minutes earlier measured `layout/1000_nodes`
at a median of 3.82 ms against 5.81 ms here, which is why comparisons need the
three-run procedure above.

## No FPS targets and no CI thresholds

This guide does not define FPS targets, frame-time budgets, or pass/fail
thresholds for any case, and CI does not run the benchmark as a gate.

Why:

- Hosted CI runners share hardware with other jobs. Run-to-run variation on
  such runners is commonly larger than the effects a regression check would
  need to detect, so a threshold would fail or pass at random.
- The reference numbers above come from one run on one desktop. No target taken from them
  would transfer to other hardware, and an invented target would be a claim
  without evidence.
- Frame-rate targets belong to end-to-end measurements on the real display
  path (window, present, vsync), which this headless runner does not measure.

Thresholds may be added later, once a dedicated, quiet benchmark machine exists
and several runs show a stable spread. Until then the only valid use of this
suite is to compare medians on the same machine, as described above.

## Limitations

- **CPU only.** The rasterizer is the software `acme-render-cpu` backend. The
  GPU path (`acme-render-wgpu`) is not measured. `frame_build` constructs
  `Frame` values but does not submit them.
- **Fonts are system fonts.** `text/shape_prepare_4_strings` uses the fonts the
  OS provides, so its numbers depend on the machine. Only compare it on the
  same machine.
- **No Gallery or screenshot measurements.** The Gallery uses system fonts,
  which differ between Windows, macOS, and CI images, so screenshot goldens are
  not portable yet (recorded as NOT_RUN in the roadmap).
- **Synthetic workloads.** Trees, orderings, and scenes are deterministic
  generators of a fixed shape. They measure the engine's cost for that shape,
  not for a real application screen.
- **Single process, single thread.** Each case runs on the main thread with
  no concurrency.
- **Allocator and cache state.** Warm-up gives the steady state, but the
  process allocator still shapes the numbers. Compare full runs, not single
  samples.

## Adding a case

1. Put it in the module of its area under `apps/benchmark/src/cases/`.
2. Build the input outside `measure`, return a value from the closure so the
   optimizer cannot remove the work, and record counts in `meta`.
3. Add its exact name to `EXPECTED_NAMES` in `apps/benchmark/tests/quick_suite.rs`.
4. Run `cargo test --locked -p acme-benchmark`.

## Changelog

| Version | Date | Change |
|---------|------|--------|
| 3 | 2026-11-07 | `text/atlas_pressure_multipage` case (ROADMAP 011). Windows host, `--quick`: median 14.20 ms; 1 page: 2 evictions, 986 drawn; 4 pages: 3 pages open, 0 evictions, 1584 drawn (all non-space glyphs). |
| 2 | 2026-11-07 | Headless runner. Window path removed. Warm-up and sampled statistics, JSON schema v1, environment record, retained-tree and CPU-render cases added. |
| 1 | — | Single-run `tracing` output inside a window. |
