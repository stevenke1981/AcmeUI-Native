# CPU atlas rejection report

Task: ROADMAP 006, one bounded atlas-lifecycle slice.
Base: 773b5247b05872414c413acd69940186fe97eba8 (master).
Delivery and final exact-head evidence: [PR #11](https://github.com/stevenke1981/AcmeUI-Native/pull/11).

## Observed failure and correction

The old renderer applied uploads to persistent Alpha8/Rgba8 storage before a later upload or scene-structure error could reject the frame. Published pixels stayed unchanged, but a later cache-only text frame used the rejected glyph data.

The fix introduces an allocation-free preflight covering every existing recoverable upload, clip/layer-balance and layer-budget error before changing framebuffer or atlas state. Upload validation has one shared private boundary. Accepted commands still upload and draw in their original painter order, including same-slot replacement; uploads are not hoisted ahead of draws. No full-atlas clone or command-list allocation is added.

Public signatures, error variants/messages, dependencies, Cargo.lock and CI are unchanged. This is not a new geometry-validation policy or a panic/OOM recovery guarantee.

## Regression coverage

Five new public integration tests extend regression.rs from 10 to 15 tests:

- rejected_scene_structure_preserves_both_glyph_atlases: unmatched PopClip/EndLayer, unclosed clip/layer and nesting budget.
- later_bad_upload_in_same_text_preserves_both_glyph_atlases.
- later_bad_text_primitive_preserves_both_glyph_atlases.
- rejected_scene_does_not_seed_either_glyph_atlas.
- accepted_uploads_keep_text_command_order_and_cached_replacements.

Both formats share atlas coordinates in separate storage. Invalid upload cases cover unsupported page, x/y bounds, incorrect payload length and maximal dimensions. Rejection checks assert unchanged published pixels, a cache-only next frame, later successful replacement and cached replacement. The accepted-scene test combines balanced layers/clips and repeated same-slot uploads to detect upload hoisting.

Existing text-free golden fixtures, atlas aging/eviction, cache reuse and clear/reupload tests are preserved. This patch does not claim Gallery/GPU screenshot baselines or measured rendering performance.

## Evidence and execution

Execution mode: separate GPT-6.1 Sol xhigh author and read-only auditor/reviewer, with Lead integration. Role names are not counted as independent GitHub approvals.

PASS: pinned Rust 1.88 offline Windows baseline regression suite (10 tests), before disconnection.
FAIL, expected reproduction: a separately compiled public-API scenario on the base source tree seeded white, rejected an overwrite followed by EndLayer, then rendered black on the next cache-only frame (exit 101). Local first-failure log SHA256: AF36258827C9EC5FD76F24F5FAE3979A96BD77AD13BE3A81686697F601882627.
PASS: unchanged verification-runner self-tests, 7 tests, on the desktop before source changes.

The desktop disconnected before either source patch executed. Both failures occurred while reading the target file; a brief read-only reconnect confirmed no lib/tests changes and no Cargo/rustc processes. The cloud filesystem is read-only and has no Rust toolchain. Source candidates were built in memory from immutable GitHub base blobs and re-fetched byte-for-byte; no filesystem/network restriction was bypassed. Local candidate compilation/tests/Clippy are BLOCKED, not passed.

Initial tests-only commit 5503dec2ca31b8a3483ddf1dc506847f317a3080 failed pinned rustfmt before behavioral tests. The three reported helper-formatting changes were applied without changing gates. Formatted tests-only red commit: 1e85cdadaf64e3a50e99bcf653b83a8881aa7707. [CPU evidence run 42341229081](https://github.com/stevenke1981/AcmeUI-Native/actions/runs/42341229081) passed pinned formatting and failed the four new rejection/cache regressions as expected (11 passed, 4 failed; exit 101). Its clean PR test-merge c528953761947f4e4d87c1ceac29f0216b48b1aa has the identical source tree 11fabd88e9b11892da4370e9d7884a9816ba7cf8. Artifact 13561955655 preserves raw logs/report/source; failing test log SHA256: 1cd7b69c00726e5e75390683b8812237df274d35ca4b2b1297f75e41bf47b841. The accepted-order test already passes on the old renderer, isolating the regression behavior.

Independent static review: PASS, no blocking findings. Reviewed lib blob 2843529237e57bc3ca8e2dfb05d51257d04d7e49 and tests blob 08ae5034e7c0f98778c7acad321dd404e257b1ab. First-error ordering and the equivalent layer-budget expression were checked. Dedicated multiple-invalid-error-precedence and pixel-budget-threshold tests were not added; equivalence there is inspection evidence, not extra runtime coverage.

Final CI execution is recorded in PR #11. Before merge, the exact final head (or identical tree in a PR test-merge) must pass the unmodified Windows/macOS/Linux workspace/all-feature checks/tests, changed-file formatting/Clippy, SDK consumers, cross-target checks, dependency audit and CPU evidence workflow. The latter runs the locked CPU tests/Clippy, verification-runner self-tests, GPU-only platform check and CPU snapshot. An ancestor pass or formatting-only result is not a candidate pass.

## Remaining limits

NOT_RUN: native GUI, physical DPI, manual Traditional Chinese IME, screen readers, real GPU/device-loss paths, release-build/packaging acceptance and measured CPU/GPU performance. Automated synthetic-glyph pixels do not replace those checks. No widget/platform interaction changes require or claim fresh manual acceptance in this patch.

Next roadmap work remains separate: Gallery/GPU baselines, performance methodology, and the previously observed GPU upload-deduplication key (format/page). No GPU fix is included here.

No branch deletion, history rewrite, release, deployment, settings/security exceptions, paid action or user-data cleanup occurred. Rollback is a revert of this bounded patch.
