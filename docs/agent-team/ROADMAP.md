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
| 015 | Lead + Rendering | Area-based CPU clip edges: exact rect∩clip area for sharp quads, clip-area mask for rounded quads, images and glyphs | Integer clips and goldens byte-identical, RED/GREEN + mutation tests at five scales, coincident-edge seam regression, benchmark before/after (observation); live Gallery and GPU parity NOT_RUN |
| 016 | Lead + Platform + Rendering | Host input/window controls, app renderer preference, sRGB passthrough, exact rounded-quad clip masks, drag cursors and DPI resize recovery | [Track reports](../team/roadmap-016/): targeted tests/Clippy and headless checks; native GPU colour, pointer capture and physical DPI transitions NOT_RUN |
| 017 | Lead + Rendering | Fractional GPU clips and analytic quad coverage in shaders; physical instance clip bounds | [Q1 report](../team/roadmap-017/REPORT-Q1-gpu-fractional-clip.md): 71 headless tests, WGSL validation and CPU formula comparisons; real GPU drawing NOT_RUN |
| 018 | Lead + Rendering | Fully covered CPU pixel spans and opaque blending fast paths | [R1 report](../team/roadmap-018/REPORT-R1-cpu-fast-path.md): tests/goldens and before/after frame timing reported by the original batch; timing was not re-measured in 019 |
| 019 | Lead + UI/UX + Verification (single-agent role passes) | Mobile card retained text identity, optional-slot acceptance coverage and Gallery examples with clipped surfaces | [Brief](../team/roadmap-019/BRIEF.md); verification and delivery recorded in [report](../team/roadmap-019/REPORT.md); manual hardware acceptance remains separate |
| 020 | Lead + UI/UX + Verification (single-agent role passes) | Responsive determinate mobile progress, numeric/retained acceptance, Gallery demos and content-label baseline alignment | [Brief](../team/roadmap-020/BRIEF.md); tests, CPU-window evidence and delivery snapshot in [report](../team/roadmap-020/REPORT.md); manual hardware acceptance remains separate |
| 021 | Lead + UI/UX + Verification (single-agent role passes) | Mobile loader acceptance, wrapping messages, Gallery wrap constraints and prepared multiline baseline offsets | [Brief](../team/roadmap-021/BRIEF.md); tests, native CPU capture and evidence in [report](../team/roadmap-021/REPORT.md); spinner rotation and manual hardware acceptance remain open |

| 022 | Lead + UI/UX + Rust Core + Verification (single-agent role passes) | Real mobile search, Gallery text-input host, responsive TextInput styling, baseline/cache regressions and historical mobile Clippy reconciliation | [Brief](../team/roadmap-022/BRIEF.md); [report](../team/roadmap-022/REPORT.md) with locked checks and automated native CPU interaction captures; manual hardware acceptance remains open |

| 023 | Lead + UI/UX + Verification (single-agent role passes) | Themed mobile chips with separate tap/dismiss actions, retained identity and Gallery filter/removal/restore | [Brief](../team/roadmap-023/BRIEF.md); [report](../team/roadmap-023/REPORT.md) with locked checks and native CPU pointer captures; manual hardware acceptance remains open |

| 024 | Lead + UI/UX + Verification (single-agent role passes) | Circular mobile avatars, centered initials/empty placeholder, Label sizing/alignment and themed Stack painting | [Brief](../team/roadmap-024/BRIEF.md); [report](../team/roadmap-024/REPORT.md) with locked checks and native CPU capture; manual hardware acceptance remains open |

| 025 | Lead + UI/UX + Verification (single-agent role passes) | Full-width themed mobile banners, wrapping text, separate instance actions, retained/layout coverage and Gallery pointer flows | [Brief](../team/roadmap-025/BRIEF.md); [report](../team/roadmap-025/REPORT.md) with locked checks and native CPU captures; manual hardware acceptance remains open |

| 026 | Lead + UI/UX + Verification (single-agent role passes) | Wrapping mobile notification cards, existing Card layout Style, independent actions and host-owned timed Gallery dismissal with stale-press cancellation | [Brief](../team/roadmap-026/BRIEF.md); [report](../team/roadmap-026/REPORT.md) with RED/GREEN, locked checks and native timed CPU captures; manual hardware acceptance remains open |

| 027 | Lead + UI/UX + Rendering + Verification (single-agent role passes) | Responsive mobile list items, scoped Gallery pointer/keyboard host and Frame→Scene text clip preservation exposed by native scrolling | [Brief](../team/roadmap-027/BRIEF.md); [report](../team/roadmap-027/REPORT.md) with RED/GREEN, locked gates, native input captures and toolbar pixel check; generic Row focus/typed semantics and manual hardware acceptance remain open |

| 028 | Lead + UI/UX + Verification (single-agent role passes) | Mobile action destructive/disabled states, bounded wrapping label identity, semantic Label colours and Gallery pointer acceptance | [Brief](../team/roadmap-028/BRIEF.md); [report](../team/roadmap-028/REPORT.md) with locked gates and native CPU captures; generic Row focus/typed semantics and manual acceptance remain open |

| 029 | Lead + UI/UX + Verification (single-agent role passes) | Visible themed mobile sheet handle, finite fixed geometry and centered Gallery preview | [Brief](../team/roadmap-029/BRIEF.md); [report](../team/roadmap-029/REPORT.md) with locked gates and native CPU capture; actual dragging and manual acceptance remain open |

| 030 | Lead + UI/UX + Verification (single-agent role passes) | Default/layout/child-count contracts and inventory guard for all 20 public mobile modules; retained Gallery presence for 15 P3 builders; historical count reconciliation | [Brief](../team/roadmap-030/BRIEF.md); [report](../team/roadmap-030/REPORT.md) with coverage matrix, locked gates and native CPU smoke; legacy interactions and manual acceptance remain open |

| 031 | See original render-reliability report | Surface-acquisition glyph uploads, shared Scene validation and bounded frame retries | [Report](../team/roadmap-031/REPORT.md); merged separately in PR #40 before the resumed FAB batch |

| 032 | Lead + UI/UX + Verification (single-agent role passes) | Resume FAB mini/normal/extended acceptance, themed elevation and typed actions | [Brief](../team/roadmap-032/BRIEF.md); [report](../team/roadmap-032/REPORT.md); viewport anchoring and manual hardware acceptance remain separate |

| 033 | Lead + UI/UX + Verification (single-agent role passes) | Existing toast Snackbar actions, wrapping and keyed identity; Gallery bottom placement, restartable expiry and click-through prevention | [Brief](../team/roadmap-033/BRIEF.md); [report](../team/roadmap-033/REPORT.md); manual hardware acceptance remains separate |

| 034 | Lead + UI/UX + Verification (single-agent role passes) | Controlled SwipeActions, backend-neutral threshold state and Gallery pointer/keyboard host; public module inventory 200 | [Brief](../team/roadmap-034/BRIEF.md); [report](../team/roadmap-034/REPORT.md); continuous translation/animation and manual hardware acceptance remain open |

| 035 | Lead + Rust Core + UI/UX + Verification (single-agent-role-passes) | Correct fractional full sizing in Styled/Tailwind/Button, document parent-relative screen aliases, real layout and Gallery evidence | [Brief](../team/roadmap-035/BRIEF.md); [report](../team/roadmap-035/REPORT.md); manual hardware acceptance remains open |

| 036 | Lead + Architecture + UI/UX + Verification (single-agent-role-passes) | Independent Paper square corners with preserved elevation/defaults, compatibility note, retained/layout/Gallery evidence | [Brief](../team/roadmap-036/BRIEF.md); [report](../team/roadmap-036/REPORT.md); manual acceptance remains open |

| 037 | Lead + Architecture + Rust Core + Verification (single-agent-role-passes) | Declare UI inputs/layout feature dependency closure, isolated consumer regressions and minimal-feature CI | [Brief](../team/roadmap-037/BRIEF.md); [report](../team/roadmap-037/REPORT.md); other feature families and manual acceptance remain separate |

| 038 | Lead + Architecture + UI/UX + Verification (single-agent-role-passes) | DataList compact/default/relaxed density with preserved defaults, real layout/retained regressions and six themed Gallery examples | [Brief](../team/roadmap-038/BRIEF.md); [report](../team/roadmap-038/REPORT.md); manual hardware acceptance remains separate |

| 039 | Lead + Architecture + UI/UX + Verification (single-agent-role-passes) | Registry-driven Gallery theme-pack swatches, live light/dark mode, stable identities and bounded/clipped painting | [Brief](../team/roadmap-039/BRIEF.md); [report](../team/roadmap-039/REPORT.md); manual hardware acceptance remains separate |

| 040 | Lead + Architecture + UI/UX + Accessibility + Verification (single-agent-role-passes) | Host-driven spinner rotation/reduced-motion policy, retained fixed geometry and Gallery viewport-gated scheduling | [Brief](../team/roadmap-040/BRIEF.md); [report](../team/roadmap-040/REPORT.md); OS preference detection and manual acceptance remain separate |

| 041 | Lead + Architecture + Rust Core + Accessibility + UI/UX + Verification (single-agent-role-passes) | Retained active-container focus and exact typed activation; original Gallery demo, unique hit/paint slots and clipped themed focus rings | [Brief](../team/roadmap-041/BRIEF.md); [report](../team/roadmap-041/REPORT.md); real platform adapter and manual hardware acceptance remain separate |

| 042 | Lead + Architecture + UI/UX + Verification (single-agent-role-passes) | Cover and repair SpeedDial typed action conversion, preserving passive labels and host-controlled state | [Brief](../team/roadmap-042/BRIEF.md); [report](../team/roadmap-042/REPORT.md); directional expansion and manual hardware acceptance remain separate |

| 043 | Lead + Architecture + UI/UX + Verification (single-agent-role-passes) | Preserve AboutDialog Close messages after conversion; host-controlled Gallery close/restore and optional-content identity | [Brief](../team/roadmap-043/BRIEF.md); [report](../team/roadmap-043/REPORT.md); legacy visibility metadata, full modal behavior and manual hardware acceptance remain separate |

## Existing work to preserve

The current `docs/status/todos.md` already records retained-tree invalidation fixes and glyph-atlas aging/eviction. Re-audit their integration and evidence before opening replacement tasks. Do not assume those features are absent from old summaries.

## Release boundary

A passing library build does not establish production readiness. Keep separate evidence for Windows/macOS/Linux runtime behavior, IME, DPI, screen readers, CPU/GPU transition, packaging, and public API compatibility. Mobile/Web runtime support is not established by the existing `acme-ui --lib` cross-target checks.
