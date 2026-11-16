# ROADMAP 053 — SpeedDial directional expansion

2026-11-16 Asia/Taipei. Base `b209174cc73deef860f2f1bc3531eb93f8406e60`.
Branch `sdk/roadmap-053-speed-dial-direction`. Execution: **single Codex agent,
sole writer and author self-review**; independent review NOT_RUN.

Selected from the incomplete P8 SpeedDial directional expansion item in
`docs/status/todos.md`. One bounded slice, one PR.

Acceptance:

- Provide Up (existing default), Down, Left and Right. Keep declaration order
  within the action list and the existing indexed typed messages, keys, passive
  fallback, toggle and host-controlled open/close behavior.
- Verify actual text-measured layouts put nonoverlapping actions on the requested
  side at 400/600 logical px and scales 1/1.25/2. Existing vertical 240 px coverage
  remains. Retained controls and focused actions survive direction/keyed reorders.
- Add an interactive Gallery direction selector and pointer, theme, scale, clip
  and hit/paint regressions. Capture automated native CPU Gallery evidence.
- Run locked targeted UI/Gallery tests, changed-file formatting, strict touched
  Clippy and actual `scripts/verify_sdk.py --profile cpu`, reusing `target/`.
  Run workspace check/tests at this slice's milestone. Merge with a merge commit
  only after all hosted CI succeeds and GitHub reports MERGEABLE.

The only component API addition is the direction enum, builder field and setter.
Direct builder struct literals need the new field; constructor calls retain the
old default. No dependency/manifest/lockfile/Scene/platform changes. Viewport
anchoring, animation and automatic dismissal are separate work.

Human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen-reader
acceptance remain NOT_RUN. No project completion or release-readiness claim.
