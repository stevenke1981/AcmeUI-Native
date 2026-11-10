# ROADMAP 031 — render reliability

## Scope and changes

Base: `1f23c910f05d7c975094df1e3b2be65ee2437ec9` (master). Final code candidate: `118a8039ae78d85d9c5219fde01f7510dc239b0a`.
Execution: lead implementation plus independent Scene-contract author in isolated worktrees; separate Rust reviewer. User authorized commit, push, PR, default-branch merge and release build. The repository default branch is `master`, not `main`.

1. **REPRODUCED: glyph upload loss.** A scene containing new glyphs returned before atlas writes when surface acquisition timed out or was occluded/outdated. The next application frame could already be a CPU atlas hit, permanently omitting that one-shot upload. Uploads now precede acquisition; an empty submit flushes staging writes on a skipped frame. The Windows test injects all three acquisition results plus zero-sized suspension, recovers on a real surface and reads back exact atlas texels.
2. **REPRODUCED: missing redraw and unsafe polling deadlines.** The runtime cleared dirty after Skip. It now retains dirty and requests another redraw after 100 ms. Background polling and retries share the earliest wakeup; successful rendering cancels retries. Poll intervals are normalized to 1 ms–24 h, changed intervals reschedule, and None cancels polling. The previous zero interval, Duration::MAX and dynamic change paths fail deterministic regressions.
3. **REPRODUCED: inconsistent scene rejection.** Scene validation checked only clips; GPU batching accepted unmatched layers. Shared validation now covers underflow and unclosed clip/layer stacks. GPU preflight occurs before live-image updates, uploads or surface acquisition. CPU retains its existing allocation/upload checks and rejection behavior.

## Compatibility

Renderer and batch public method signatures remain unchanged. Invalid public `compile_scene` calls now log a warning and return no batches; `Scene::validate` supplies the error. `SceneError` adds `LayerUnderflow` and `LayerUnbalanced`; downstream exhaustive matches must add these cases. Poll durations outside the documented range now clamp. Valid scenes retain painter order. GPU group-opacity composition remains separate unfinished work.

## Evidence

| Check | Result |
|---|---|
| Scene core / GPU baseline regressions | FAIL as intended: 2 core tests + 1 GPU batching test; see scene-HANDOFF.md |
| Windows acquisition regression before fix | FAIL as intended: zero uploaded bytes instead of four; gpu-red-fresh.txt |
| Scheduler baseline regressions | FAIL as intended: missing retry, zero deadline, duration overflow, stale changed interval; scheduler-red.txt |
| Windows GPU acquisition + exact atlas readback | PASS: one test covers Timeout, Occluded, Outdated, zero-sized suspension, real subsequent surface presentation, cache-hit scene, and invalid-scene early rejection; suspended-green.txt |
| Four affected crates | PASS at e0dcd42: 275 tests, six ordinary ignored tests/doctests; targeted-final.txt. The new GPU test was run explicitly, not counted as executed by the ordinary suite. |
| Affected Clippy | PASS: four crates at e0dcd42 (clippy-final.txt); wgpu/platform/Gallery after review fix (review-fix-clippy.txt) |
| Review fix targeted tests | PASS: 133 tests, five ignored tests/doctests; review-fix-tests.txt |
| Real Gallery frame generation | PASS: 36 pages × 2 themes × 2 frames; gallery-frames.txt. Headless frame/command evidence, not visual or human approval. |
| CPU verification profile | PASS at 118a803; gates.json includes exact source, environment, commands and log hashes |
| Workspace fmt / all-target check / GPU-only platform check | PASS at e0dcd42 |
| Workspace tests / UI all features | PASS: 1,790 / 798 passed, 11 / 2 ignored respectively, at e0dcd42 |
| Independent review | PASS: suspended-path finding fixed and rechecked at 118a803; REVIEW.md |
| Pre-review-fix release build | PASS at e0dcd42; gates.json. This is not the final merged artifact. |
| Hosted CI / merge / final release build | Pending at PR preparation; final statuses and merged-SHA artifact evidence supplied in PR and delivery. |
| Native Gallery screenshot | BLOCKED: agent-desktop status returned PERM_DENIED because a private-file DACL grants an untrusted principal. No UI action or host permission change performed. |
| Human Traditional Chinese IME, physical DPI/touch, screen reader | NOT_RUN |

The first main-agent GPU test invocation reused a stale shared-target test binary and discovered **zero tests**. It is excluded from PASS evidence; the source was recompiled and the named test positively discovered for both RED and GREEN. Author cargo use was serialized. Raw failed integration attempts remain under the local target/roadmap-031/diagnostics directory. The main-agent RED additions were uncommitted; source baseline and boundary-refactor provenance are stated here rather than claiming a nonexistent RED commit. Final source hashes and environment are in environment.json. Worker RED source hashes are in scene-source-manifest.json.

## Remaining work

- GPU group opacity and CPU/GPU pixel parity are not addressed by this patch.
- Native AccessKit adapter, real desktop input wrappers and other optimization checklist items remain separate batches.
- Crates.io naming/publication, package signing and a versioned public binary release are not performed by a release build.
- Native desktop capture can be retried after the supported agent-desktop host DACL issue is resolved; this patch does not change host security settings.

Rollback: revert this batch through a new PR. Existing untracked user reports and .sites-runtime content are preserved.
