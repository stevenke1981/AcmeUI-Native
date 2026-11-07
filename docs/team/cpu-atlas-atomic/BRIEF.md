# CPU atlas transaction brief

Task: ROADMAP 006 bounded atlas-lifecycle follow-up, 2026-11-07.
Base: 773b5247b05872414c413acd69940186fe97eba8 (master).

## Audit and scope

PRs #8, #9 and #10 are merged. Their retained identity, focus/semantics and automated DPI/IME work is preserved. Existing CPU text-free golden tests, atlas aging/eviction, and atlas reuse/clear tests already exist; this task does not replace them.

REPRODUCED: CpuRenderer::render_scene mutates persistent glyph atlases before validating the remaining scene. A rejected scene preserves published pixels but can alter cached glyphs used by the next valid frame. Seed an Alpha8 mask [255], then submit [0] with an unmatched EndLayer: Err leaves the displayed pixel white, but the next cache-only frame becomes black.

Goal: validate a complete scene before changing glyph-cache or framebuffer state. Preserve accepted-scene upload order, both supported formats, and existing errors. Avoid cloning full atlas buffers per frame.

Non-goals: GPU upload deduplication, new renderer/API/dependency/CI contracts, Gallery changes, benchmarks, device-loss, release or deployment. This is one atlas-lifecycle slice of ROADMAP 006, not completion of its pixel-baseline/performance scope.

## Ownership and sequence

Execution mode: independent native agents, GPT-6.1 Sol xhigh as requested.

- Lead: this brief, REPORT.md and one todos entry; integration, publication and exact-head CI.
- Rendering author: crates/acme-render-cpu/src/lib.rs and crates/acme-render-cpu/tests/regression.rs.
- Separate read-only auditor/reviewer: source audit, reproducer, final diff and evidence. Does not edit the reviewed artifact.

The author began in a dedicated worktree based on master. The desktop disconnected before either attempted source patch could read its target; both attempts are confirmed not executed. Work continues through authorized GitHub connector source snapshots and immutable blobs in memory. The cloud filesystem is read-only and was not written. No second author or duplicate desktop operation is introduced. Existing desktop reports, caches and worktrees are preserved.

Review follows implementation. A draft PR captures red-before-green CI where feasible; merge follows applicable verification and independent review. No branch deletion, force push, paid action, account/settings change, release or deployment.

## Acceptance

- AC1: rejected scenes cannot change Alpha8/Rgba8 cache contents or published pixels. Cover later invalid uploads and existing scene/command validation errors; inspect subsequent cache-only text pixels.
- AC2: accepted upload/painter order, overlapping uploads, cache reuse and recovery remain correct. Tests are independent of fonts, GPUs and windows.
- AC3: run targeted CPU fmt/tests/Clippy, runner self-tests and milestone checks with locked dependencies. Preserve first failures and distinguish PASS/FAIL/BLOCKED/NOT_RUN. Do not weaken gates for environment limits.
- AC4: independent review has no unresolved blocking finding. The exact final head, or same source tree in a PR test-merge, passes unmodified CI and CPU evidence before merge to master. Verify the resulting master commit/tree and checks.

## Verification boundary

The pinned Rust 1.88 offline baseline CPU regressions passed (10 tests) on Windows. An independently compiled public-API reproduction failed as expected (exit 101), before disconnection. Subsequent local checks are blocked by that disconnection and the read-only cloud's missing Rust toolchain; GitHub CI provides candidate execution evidence.

Native GUI, physical DPI, manual Traditional Chinese IME, screen readers, GPU runtime and measured performance remain NOT_RUN. Synthetic-glyph CPU tests do not prove those capabilities. No widget behavior changed, so existing manual platform sign-off is not claimed or replaced.

Rollback: revert the bounded patch; no persistent user data or migrations.
