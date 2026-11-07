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
| 008 | Lead + UI/UX + Accessibility | Follow-ups: Gallery hover/press drift, AccessKit validation and cell roles, compiled migration snippets, benchmark CI smoke | Headless RED/GREEN tests, consumer test, CI artifact; screen-reader checks stay manual |
| 009 | Lead + Accessibility + Rendering | Virtual TreeItem/Row/ColumnHeader AccessKit nodes; exact-area CPU quad coverage | RED/GREEN tests incl. real-layout bounds, re-blessed golden with delta, A/B benchmark; screen-reader checks stay manual |
| 010 | Lead + Accessibility | Accessibility walk mirrors layout ID consumption (cells, rows, tree items, trailing widgets); ragged rows | Real-layout bound tests on sequential/bound/unbound paths, mutation RED, Gallery bounds test |
| 011 | Lead + Architecture + Rendering + Text | Multi-page glyph atlas: page contract, page-granular LRU eviction, CPU/wgpu per-page atlases, apps on default atlas | Spec, RED/GREEN + mutation tests, randomized eviction simulation, CPU e2e vs single-page reference, benchmark counters; GPU drawing NOT_RUN |
| 012 | Lead + Rendering + Text + Accessibility | wgpu crate-root split, demand-LRU atlas starvation fix, IME preedit scroll and drawing | API page/declaration diff vs HEAD, RED/GREEN tests, cross-agent reviews (Sigma and Codex); IME/GPU manual checks NOT_RUN |
| 013 | Lead + Accessibility + Text | TreeItem Focus/Click/Expand/Collapse actions and walked item content; acme-textinput lib.rs split | RED/GREEN + mutation tests (p5), test-list and rustdoc item diff vs master, cross reviews (lead and Codex); screen reader NOT_RUN |
| 014 | Lead + Accessibility + Rendering + Mobile | AccessKit request mapping and trailing-edge TreeItem press; exact analytic arc coverage in the CPU renderer; interactive mobile toggle/stepper/segmented | RED/GREEN + mutation tests per part, golden delta and benchmark before/after, cross reviews (lead, Codex); screen reader, touch device, quiet-host benchmark NOT_RUN |

## Existing work to preserve

The current `docs/status/todos.md` already records retained-tree invalidation fixes and glyph-atlas aging/eviction. Re-audit their integration and evidence before opening replacement tasks. Do not assume those features are absent from old summaries.

## Release boundary

A passing library build does not establish production readiness. Keep separate evidence for Windows/macOS/Linux runtime behavior, IME, DPI, screen readers, CPU/GPU transition, packaging, and public API compatibility. Mobile/Web runtime support is not established by the existing `acme-ui --lib` cross-target checks.
