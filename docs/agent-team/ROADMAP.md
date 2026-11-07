# SDK Stabilization Roadmap

This is a prioritized plan, not a claim that all areas have been audited or implemented.

| Batch | Owner | Scope | Acceptance evidence |
|---|---|---|---|
| 001 | Rendering + Verification | CPU overflow/bounds regressions, synchronized dependencies, team contracts | Failing reproduction, passing locked headless tests, current CI |
| 002 | Lead + Rendering + UI/UX | Explicit auto/GPU/CPU selection, actionable startup errors, fallback integration | Policy unit tests, GPU-to-CPU recovery test, native Gallery evidence |
| 003 | Rust Core + Architecture | Verify retained-tree integration and lifecycle; preserve existing invalidation fixes | State/reorder tests and proof that actual app path uses retained results |
| 004 | UI/UX + Accessibility | Stabilize the most-used controls and keyboard/focus workflows | Gallery scenarios, accessible semantics, applicable screen-reader checks |
| 005 | Rust Core + Accessibility | Windows Traditional Chinese IME and DPI acceptance | Actual 注音 sessions at 100/125/150/200 percent scaling |
| 006 | Rendering + Verification | Pixel baselines, atlas lifecycle, measured CPU/GPU performance | Reproducible images and benchmark methodology; no invented FPS targets |
| 007 | Architecture + Lead | SDK examples, versioning, packaging, migration guidance | External consumer build, release build, compatibility and license review |

## Existing work to preserve

The current `docs/status/todos.md` already records retained-tree invalidation fixes and glyph-atlas aging/eviction. Re-audit their integration and evidence before opening replacement tasks. Do not assume those features are absent from old summaries.

## Release boundary

A passing library build does not establish production readiness. Keep separate evidence for Windows/macOS/Linux runtime behavior, IME, DPI, screen readers, CPU/GPU transition, packaging, and public API compatibility. Mobile/Web runtime support is not established by the existing `acme-ui --lib` cross-target checks.
