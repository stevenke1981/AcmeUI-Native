# ROADMAP 012 — P2 report

Lead: p3. Base/tested HEAD: `271e5e552c6f02d0856ca8fdd7b0a502c6eb8a09` + dirty files.
Mode: Codex main implementation; independent Codex gpt-6.1-terra source review.
Environment: Windows 10 MSVC, Rust/Cargo 1.88.0; metadata: `evidence/p2-environment.json`.
Candidate file hashes/line counts: `evidence/p2-source-manifest.json`.

## A — complete and verified

Files: wgpu `src/lib.rs`, `draws.rs`, `images.rs`, `instances.rs`, `pipelines.rs`,
`surface.rs`, `tests.rs`, `types.rs`, `uploads.rs`, `renderer/{mod,state,init,frame,
atlas,buffers,recovery,scene_images}.rs`. Excluded files/shaders unchanged.
Repaired the partial split's missing surface imports and unused frame import.
Public definitions use root `include!` instead of private-module re-exports:
the latter create extra rustdoc redirects; root declarations preserve exact paths.
Detached HEAD worktree `D:/AcmeUI-Native-p2-head-271e5e5` built, then removed.
`cargo doc --locked -p acme-render-wgpu --no-deps`: before/after exit 0.
API: sorted `a-doc-{before,after}.txt` identical (20 HTML paths); `a-doc-diff.txt` PASS.
`a-api-{before,after}.txt` identical (286 declaration lines); `a-api-diff.txt` PASS.
Final token comparison: 64 function names, no body changes (`a-move-check.txt`).
`cargo test --locked -p acme-render-wgpu`: HEAD and final exit 0, 59 passed,
2 ignored; doc tests 0 passed/1 ignored. `lib.rs`: 20 lines; all Rust files <800.

## B — spec and implementation complete

`SPEC-B.md`: DO demand LRU for all-fresh starvation; REJECT per-format allocators
(budget/observable semantics need justification) and mid-frame untouched-page
eviction (would revise the shared per-frame generation contract). No public API additions.
Files: `acme-text/src/atlas.rs`, `src/atlas/tests.rs`, `tests/atlas_lifecycle.rs`.
Recent pages can be reused before current-frame use; pending overflow is resolved
at the next boundary. One eviction settles one request. Background remains stale-only.
RED: `cargo test --locked -p acme-text`: exit 101, 32 passed/2 failed (`b-red.log`).
RED: same command + `--test atlas_lifecycle bug_glyph_dropped_while_atlas_holds_only_recent_entries -- --exact`:
exit 101, 0 passed/1 failed (`b-red-starvation.log`, dropped O, atlas_full=1).
Review RED: same crate test + `satisfied_demand_does_not_evict_a_fresh_page`:
exit 101, 0 passed/1 failed (`b-review-red.log`, unwanted second eviction).
GREEN: `cargo test --locked -p acme-text`: exit 0, 35 unit + 6 lifecycle + 4 page tests;
45 passed, 0 failed/ignored (`b-final-test.log`). No CPU/wgpu B production changes.

## Final gates and handoff

Both touched crates: `cargo clippy --locked -p <crate> --all-targets -- -D warnings`
and `cargo fmt -p <crate> -- --check`: exit 0. wgpu/UI `cargo check --locked -p <crate> --all-targets`: exit 0.
`python scripts/verify_sdk.py --profile cpu --output docs/team/roadmap-012/evidence/cpu-final`:
exit 0, all PASS; 60 CPU tests (`cpu-final/report.json`, `02.log`).
Independent final A/B review: APPROVE; one concrete finding reproduced/fixed;
aborted workspace-gated Rust review disclosed in `evidence/p2-review.md`.
No staging/commit/stash/branch switch, manifest/lockfile/new dependency changes, or cargo clean.
NOT_RUN: real GPU drawing/device loss/memory/timing, manual Gallery/IME/DPI acceptance, hosted CI.

## Fix wave

Resolves REVIEW-AB A1/A2/B1/B2/B3; supersedes the original include!/doc rule and source manifest above.
Files: wgpu `lib.rs`, `types.rs`, `renderer/{mod,state}.rs`; text `atlas.rs`, `atlas/tests.rs`; `SPEC-B.md`.
A1: real modules and explicit root re-exports; Renderer fields stay private to the renderer subtree.
RED coverage assertion over `cargo fmt -p acme-render-wgpu -- --emit stdout`: exit 1,
types/state absent; GREEN exit 0, both present (`fix-a1-{red,green}.log`). Both files formatted.
A2: standalone `rustfmt --edition 2024 --check` was already exit 0 before this wave;
the reported isolated-format failure was NOT_REPRODUCED. Both files now pass the crate and direct checks.
B1: explicit entry occupancy plus demand/stale switch. `cargo test --locked -p acme-text --lib`:
RED exit 101, 38 passed/1 failed (demand returned None, expected occupied page 1); GREEN full suite below.
The empty-page-only control already passed before the fix; no prior empty-victim failure is claimed.
B2/B3: no-victim `evict_stale` keeps the pending request/counters; return counts stale entries,
`page_evictions` counts emptied pages. Source/SPEC corrected; both contract tests passed before/after.
GREEN `cargo test --locked -p acme-text`: exit 0, 39 unit + 6 lifecycle + 4 page = 49 passed, 0 ignored.
GREEN `cargo test --locked -p acme-render-wgpu`: exit 0, 59 passed/2 ignored; doc 0 passed/1 ignored.
Both crates' `clippy --locked --all-targets -- -D warnings` and `fmt -- --check`: exit 0.
`cargo doc --locked -p acme-render-wgpu --no-deps`: exit 0; re-saved `a-doc-after.txt` / `a-api-after.txt`.
New TASK-FIX rule: root public items/indexes and public batch/golden module pages must match HEAD;
private-module redirect stubs are allowed. 20 public paths / 286 declaration lines identical; 9 verified stubs.
`python scripts/verify_sdk.py --profile cpu --output docs/team/roadmap-012/evidence/cpu-fix`:
exit 0, all PASS, 60 CPU tests. Line limits PASS (`lib.rs` 22; maximum Rust file 768).
Scoped independent fix recheck: APPROVE; no reviewer rerun of checks (`evidence/fix-review.md`).
Fresh raw logs/metadata: `evidence/fix-*`; tested dirty-source hashes: `evidence/fix-source-manifest.json`.
NOT_RUN remains real GPU, manual Gallery/IME/DPI acceptance and hosted CI. No Git/Cargo/dependency mutations.
