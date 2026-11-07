# ROADMAP 015 — P2 area-based clip edges and sharp-edge review fix

Base: `b557ce2870fa38df2b335e0097648535e40f483d`, branch
`sdk/roadmap-015-clip`, plus the scoped uncommitted diff. P2 owns only
`crates/acme-render-cpu/` and this report. Initial evidence is under
`target/p2-clip-evidence/`; the current review-fix evidence is under its
`sharp-fix/` subdirectory. No git add/commit was performed.

## 1. Audit and current design

The original path audit preceded renderer changes. P3's review identified a
MEDIUM coincident-edge seam in the original shape-area × clip-area specification:
when both boundaries cover half a pixel, their product gives 1/4 instead of 1/2.
The review fix makes sharp quads use their exact clipped geometry.

`render_scene` converts logical clips to physical `Bounds`, intersects nested
clips geometrically, and passes the effective clip to each draw. `for_pixels`
enumerates candidate pixel ranges and supplies the rectangular clip-area mask.
For a physical pixel `[x,x+1) × [y,y+1)`, that mask is the product of its x/y
axis overlaps with the clip. An absent clip contributes 1.

| Draw path | Final coverage rule |
|---|---|
| Sharp quad (normalized physical radius == 0) | Compute `area(pixel intersect outer_rect intersect clip)` directly. For a border, also compute `area(pixel intersect inner_rect intersect clip)` directly. |
| Rounded quad | Preserve analytic outer/inner shape areas and multiply the fill/stroke coverage by the clip-area mask, in the original blend order. |
| Image | Preserve centre-based destination membership, nearest-neighbour texel sampling and RGBA alpha; multiply opacity by the clip mask. |
| Alpha8 / RGBA8 glyph | Preserve physical glyph/atlas coordinates, texel sampling, tint and source alpha; multiply opacity by the clip mask. |

For sharp quads the outer and inner bounds are each intersected with the clip
**before** `quad_coverage(..., radius=0, ...)`, which evaluates the exact sharp
rectangle area. Let those clipped areas be `outer` and `inner`: fill receives
`inner`, stroke receives `max(outer-inner,0)`, and an unbordered quad receives
`outer`. The clip is not multiplied a second time. With no clip, the original
bounds and arithmetic are retained. Empty sharp intersections have zero area,
including disjoint portions of the same pixel.

The branch is based on the outer quad radius. A rounded quad whose inner radius
clamps to zero still follows the rounded quad's previous multiplication rule.
**Rounded quads, images and glyphs retain an approximation**: the product of
separate area/alpha masks is not the exact joint geometric intersection.
Correlated fractional edges can still attenuate twice on those paths; this
review fix resolves sharp quads only. Their candidate pixel ranges must still
permit the approximate products.

Physically integer-aligned clips contribute 0 or 1 and retain the previous
output bytes. Logical integer edges may become fractional after DPI scaling.
Gallery already supplies the required logical Scene rectangles, so no Gallery
or public Scene API change is needed. No dependency or manifest changed.

## 2. Files and implementation

All Rust changes remain inside `crates/acme-render-cpu/`:

| File | Final role |
|---|---|
| `src/lib.rs` | Shared area clip mask; exact outer/inner intersections for sharp quads; product approximation for rounded/image/glyph paths. |
| `tests/clip_area.rs` | 16 public-API tests, including new coincident-edge, inner-border and orthogonal-axis coverage across five scales. |
| `tests/dpi.rs` | Fractional clip area expectations across the five DPI scales; full-cover sharp draws are compatible with both the original and revised area convention. |
| `tests/regression.rs` | Four hand-derived fractional boundary pixel values, replacing the old binary centre-clipping rule. |
| `tests/golden.rs` | Opt-in raw captures only; all existing golden comparisons, sample assertions and tolerances retained. |
| `examples/clip_bench.rs` | Unchanged font-free benchmark from the initial phase. |

This review wave changes only `src/lib.rs`, `tests/clip_area.rs` and this report
relative to the reviewed handoff hashes in `sharp-fix/handoff.json`. Coverage/
arc code, image/glyph sampling, committed golden fixtures and other crates are
untouched.

## 3. Current TDD and mutation evidence

### RED and GREEN

Before the production fix, the four new `sharp_` tests produced runtime
**RED: 3 failed / 1 passed**, Cargo exit **101**
(`sharp-fix/red-runtime.log`). The three failures are:

- Clip equals a fractional sharp quad boundary: compare every output byte with
  the same quad rendered without a clip, for source alpha 128 and 255.
- The same comparison for a sharp bordered quad, covering the outer edge.
- Clip equals the inner border rectangle, or is strictly inside it: compare
  against a plain fill of that exact rectangle; no green stroke seam is allowed.

The orthogonal control (shape edge on x, clip edge on y) already passed before
the fix. It independently asserts the product of the two axis overlaps.

Every new case executes scales **1 / 1.25 / 1.5 / 1.75 / 2**. Self-clipping
cases explicitly assert that the physical edge is fractional, so the integer
DPI cases cannot make the test vacuous. The final focused run is **16 passed /
0 failed**, exit 0 (`sharp-fix/green.log`).

Existing sharp scrolling expectations now use the hand-derived intersection
`x=[1.25,4.75), y=[1.5,4.75-scroll)`; the former sharp product/disjoint examples
were updated to the reviewed exact-intersection rule. The sharp inner-coincident
border sample is half-red at `(0,1)`, quarter-red at `(0,0)`, with no green
stroke. Rounded-product and image/glyph tests are retained unchanged.

### Mutations on the final sharp fix

All three mutations compiled, executed exactly one named test, and failed an
assertion with exit **101**. `finally` restored the original source bytes after
every run; restoration is recorded in `sharp-fix/mutations.json`.

| Mutation | Test that killed it |
|---|---|
| Force `sharp=false`, restoring shape-area × clip-area for sharp quads | `sharp_coincident_outer_clip_preserves_edges_at_every_dpi` |
| Omit the inner rectangle/clip intersection | `sharp_border_clip_at_or_inside_inner_has_only_exact_fill_at_every_dpi` |
| Omit the outer rectangle/clip intersection | `fractional_quad_clip_at_every_dpi` |

The raw logs are `sharp-fix/mutation-<name>.log`. The full gates ran after all
mutants were restored.

Historical initial-phase evidence remains intact: baseline **65 passed**,
initial area-mask RED **8 failed / 3 passed**, original GREEN **77 passed**,
and six original product-path mutations. Those older `mutations.json` and
logs describe the superseded all-product candidate; they are not presented as
fresh validation of the sharp-intersection fix.

## 4. Fresh byte compatibility and golden delta

The integer-clip test was rerun on the final fix. It compares complete output
buffers against the legacy integer centre-mask of an otherwise identical
unclipped rendering: sharp and rounded/bordered quad, image, Alpha8 and RGBA8;
source alpha 153; layer opacity 0.75; all five scales. The `[4,8)` logical edges
are physically integral in every case.

`ACME_CLIP_CAPTURE` saved fresh final output under `sharp-fix/capture-final/`.
It matches both the original renderer's `capture-before/` output and the
reviewed product candidate's `capture-after/` output byte-for-byte:

| Capture set | Frames / pixels | Changed pixels | Maximum channel/byte delta |
|---|---|---:|---:|
| Integer clips, five paths × five scales | **25 / 6,400** | **0** | **0** |
| Existing 64×64 golden scenes | **6 / 24,576** | **0** | **0** |

All six outputs also equal their committed fixtures exactly: `solid_quads`,
`rounded_bordered`, `nested_clips`, `nested_layers`, `overlapping_alpha`,
`rgba_image`. `sharp-fix/pixel-delta.json` records the comparisons and maxima.
**No re-bless** was performed; `ACME_BLESS_GOLDEN` was unset. Tolerances remain
**2 per channel / 0 pixels beyond tolerance**, and existing sample assertions
were not weakened.

## 5. Historical benchmark before/after (initial area-mask phase)

These measurements predate the sharp-intersection review fix. They describe
the initially reviewed all-product implementation, not the final sharp path.
Benchmarking was not rerun in this focused fix; the fresh acceptance gates and
pixel comparisons are in section 6.

One full run on each side was performed in the initial phase. Same Windows x86_64 host, **24 logical
CPUs**, Rust **1.88.0**, release profile, **10 warm-ups / 51 samples**. No other
test/build job ran concurrently with either timed measurement. Other desktop
applications and power state were not controlled or changed.

Existing benchmark command on both sides:

```sh
cargo +1.88.0 run --release --locked -p acme-benchmark -- --json target/p2-clip-evidence/benchmark-before.json
cargo +1.88.0 run --release --locked -p acme-benchmark -- --json target/p2-clip-evidence/benchmark-after.json
```

The existing rasterizer cases have no clips; a separate unchanged, font-free
example measures the affected paths at **256×192 / scale 1.25**. Its command on
each side is:

```sh
cargo +1.88.0 run --release --locked -p acme-render-cpu --example clip_bench -- target/p2-clip-evidence/focused-before.json
cargo +1.88.0 run --release --locked -p acme-render-cpu --example clip_bench -- target/p2-clip-evidence/focused-after.json
```

The focused catalog uses 80 rounded/bordered quads, 8 repeated RGBA images, or
300 synthetic 8×12 atlas glyphs. Scene construction and checksums are outside
timing; atlas uploads remain inside normal scene rendering. Integer clip is
physically `[10,10,210,150)`; fractional clip uses logical
`(8.25,8.375,160.5,112.25)`. Unclipped and integer-clip output checksums are
unchanged. All four fractional-path checksums intentionally change.

| Workload | Before median / p95 ms | After median / p95 ms | Observed median change |
|---|---|---|---|
| Existing 2000 quads, scale 1.0 | 106.194 / 118.269 | 109.425 / 123.373 | +3.04% |
| Existing 2000 quads, scale 2.0 | 399.864 / 442.296 | 415.850 / 451.780 | +4.00% |
| Focused quad, no clip | 8.109 / 9.899 | 7.633 / 8.757 | -5.88% |
| Focused quad, integer clip | 5.962 / 7.053 | 6.342 / 6.941 | +6.38% |
| Focused quad, fractional clip | 5.943 / 6.672 | 6.505 / 7.530 | +9.45% |
| Focused image, fractional clip | 6.486 / 7.216 | 7.339 / 8.517 | +13.17% |
| Focused Alpha8, fractional clip | 1.476 / 1.642 | 1.627 / 2.085 | +10.24% |
| Focused RGBA8, fractional clip | 1.588 / 2.122 | 1.744 / 2.224 | +9.83% |

These are **observations**, not a performance acceptance PASS or a stable causal
regression estimate. No regression budget was assigned. Only one run per side
was requested/performed; this does not meet the guide's quiet alternating
three-run attribution method. More pixel-area work is expected in clipped paths,
but the no-clip spread illustrates noise; no speedup is claimed. Raw JSONs,
focused per-sample durations and `benchmark-comparison.json` preserve the data.


## 6. Fresh final verification, review and limitations

Local Windows x86_64; pinned Rust **1.88.0**. All final checks passed on the
restored sharp-fix source:

- `python -B scripts/verify_sdk.py --profile cpu --output target/p2-clip-evidence/sharp-fix/sdk-final`
  — **PASS**. The runner executed actual
  `cargo fmt -p acme-render-cpu -- --check`,
  `cargo test --locked -p acme-render-cpu`, and
  `cargo clippy --locked -p acme-render-cpu --all-targets --no-deps -- -D warnings`.
- Crate tests: **81 passed**, **0 failed / ignored** = 18 unit + 63 integration;
  0 doctests. Golden suite: **7 passed**.
- `cargo +1.88.0 fmt --all -- --check` — **PASS**, exit 0.
- `cargo +1.88.0 check --locked --workspace --all-targets` — **PASS**, exit 0.
- Sharp-product restoration and both missing-intersection mutations —
  **3/3 KILLED**, source bytes restored.
- Fresh integer/golden captures — **31/31 byte-identical**, maximum delta 0.

`sharp-fix/sdk-final/report.json` and its three logs contain the real CPU gate
execution; `sharp-fix/final-checks.json` records source hashes, the workspace
check and global format check. `sharp-fix/summary.json` is the current source
manifest and completion record; the initial top-level summary is historical.

Two actual independent Codex GPT-6.1 Terra reviewer instances performed one
focused static review each of the changed production and test scopes against
the saved handoff. Both returned **APPROVE**, no findings; neither ran tests.
P2 owns implementation and execution evidence. The user's original MEDIUM
coincident-edge finding is addressed by the RED/GREEN cases above.

**NOT_RUN:** benchmarks for this focused sharp-only fix, live Gallery scrolling/
physical DPI, real fonts or screen reader, CPU/GPU parity, quiet repeated
performance attribution, Linux/macOS, hosted CI and release packaging. The
historical benchmark section does not claim final sharp-path performance.
The requested source/test/static gates are all fresh and green. No git add/
commit/stash/checkout/push, Gallery change, new dependency or application
restart was performed.
