# ROADMAP 014 — P2 analytic CPU arc coverage

Base: `e0e44892997eda7c9ab42b18c303463c750cc602`, branch `sdk/roadmap-014-arc`, plus the scoped working-tree changes in [source-manifest.json](evidence/source-manifest.json).
Execution: main-agent implementation with one independent code-reviewer and focused rechecks; Windows x86_64, Rust 1.88.0. No git add/commit/stash/checkout/push.

## Design and files

- `crates/acme-render-cpu/src/coverage.rs`: remove production 8x8 sampling. Reflect each clipped pixel/corner-square intersection into a positive disc quadrant, then subtract its outside-disc area from the exact sharp-rectangle coverage. Accumulate all four corners, including multiple corners within one subpixel rectangle.
- The disc integral splits into a fully covered strip and a partial strip. Evaluate the latter as a local trapezoid plus `r²(θ - sin θ)/2`, where `θ = 2 asin(chord / (2r))`. A small-angle series evaluates the thin segment stably. All arc arithmetic uses f64; the result is clamped to the bounding-box coverage and returned as f32. Radius clamping and the sharp-rectangle path retain their existing behavior.
- `crates/acme-render-cpu/src/coverage/analytic_tests.rs`: five deterministic regression/property tests, fixed seeds, no new dependencies.
- `src/lib.rs` and `tests/dpi.rs` within that crate: update stale supersampling comments only.
- `tests/golden/rounded_bordered.ppm` within that crate: the single measured fixture update below. Public API and manifests unchanged.

## TDD and mutation evidence

- Initial RED: [red.txt](evidence/red.txt), cargo test exit **101**, 3 new tests failed / 1 passed. A quarter-disc returned **0.8125** instead of **π/4 = 0.7853981634**; reference accuracy and area conservation also failed.
- Numerical RED: [stability-red.txt](evidence/stability-red.txt), exit **101**. Direct antiderivative subtraction at radius `10,000,000` returned **0.9455280900** against a fine reference **0.9293212891**. The local segment formulation fixes this cancellation.
- GREEN: [green-properties.txt](evidence/green-properties.txt), exit **0**, all **5** new tests pass. Includes zero/clamped radii, quarter/full discs, subpixel bounds, all corners, fractional 1/1.25/1.5/2 DPI positions, and radii through `10,000,000`.
- Independent 512x512 reference: **256** random boundary pixels; max absolute error **0.000066616**, limit `1e-3`. The reference clips its sampling domain exactly to avoid quantising straight rectangle edges.
- Coverage bounds and radius monotonicity: **96** random rectangles, six radii per pixel. Area conservation: **128** rectangles, max absolute error **0.000000271**, limit `5e-5`, against `w*h - (4-π)*r²` before byte packing.
- [Mutation runner](evidence/mutation_check.py) compiles modified source in memory, leaving production files intact. [mutation.json](evidence/mutation.json): control **5 PASS**; all **6** mutants compile successfully and then fail with test exit **101**: old 8x8 grid, sharp-only coverage, incorrect segment coefficient, missing bottom-right corner, missing radius clamp, and unstable antiderivative subtraction.

## Golden evidence

[golden-delta.json](evidence/golden-delta.json): `rounded_bordered`, 64x64, **192/4096 pixels changed**, **68** exceeded the existing per-channel tolerance 2; maximum per-channel delta **5** in each of R/G/B. Updated only this fixture after measuring it. The other five scenes pass their original fixtures. Golden tolerances and hand-derived sample assertions were not weakened; final golden suite **7 PASS**.

## Existing benchmark before/after

Command on each side: `cargo run --release --locked -p acme-benchmark -- --json <evidence-file>`.
Same Windows x86_64 host, 24 logical CPUs, Rust 1.88.0, release, **10 warm-ups / 51 samples**, one full run per side; CPU model probe unavailable.

| CPU workload | Before median / p95 | After median / p95 | Observed median change |
| --- | --- | --- | --- |
| 2000 quads, scale 1.0 | 98.461 / 113.150 ms | 110.719 / 133.894 ms | +12.45% |
| 2000 quads, scale 2.0 | 392.441 / 578.133 ms | 398.276 / 438.615 ms | +1.49% |

Raw machine-readable results: [before](evidence/benchmark-before.json), [after](evidence/benchmark-after.json). These observed medians are slower; no speedup is claimed. This shared desktop was not controlled for quiet-system conditions, and one run per side does not meet the guide's three-run comparison method. Stable performance attribution remains **NOT_RUN**.

## Final checks and boundaries

[gates.json](evidence/gates.json), all exit **0** on final code:

- `cargo fmt -p acme-render-cpu -- --check`
- `cargo clippy --locked -p acme-render-cpu --all-targets -- -D warnings`
- `cargo test --locked -p acme-render-cpu`: **65 PASS** = 18 unit + 47 integration, 0 failed/ignored; 0 doctests.
- `cargo check --locked --workspace --all-targets`
- `python scripts/verify_sdk.py --profile cpu --output target/sdk-evidence/roadmap014-p2-final`: **PASS**; portable redacted logs/hashes in [sdk-cpu.json](evidence/sdk-cpu.json).
- Independent review: **APPROVE**, no concrete findings; [review.json](evidence/review.json). Reviewer inspected source and did not rerun tests.

Clip edges still need separate work: `for_pixels` tests clip containment at the pixel centre (`crates/acme-render-cpu/src/lib.rs`), so fractional clip coverage is **not** already exact. Deferred the optional clip change: it would alter the common image/glyph/quad clipping path and the existing DPI/regression contract. Only arc coverage changed here.

**NOT_RUN:** quiet alternating three-run performance comparison, native Gallery/physical-DPI validation, GPU parity, Linux/macOS execution, hosted CI. The headless CPU tests and text-free goldens do not establish those results.
