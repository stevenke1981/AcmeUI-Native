# ROADMAP 053 — SpeedDial directional expansion

2026-11-16 Asia/Taipei. Execution: **single Codex agent, sole writer and author
self-review**; independent review NOT_RUN. Base
`b209174cc73deef860f2f1bc3531eb93f8406e60`. [Acceptance](BRIEF.md).

`SpeedDialDirection` and the builder's `direction` field/setter support Up,
Down, Left and Right. Up preserves the existing Column/actions/toggle layout.
Down puts the toggle first; Left/Right use a Row and intrinsically sized action
Buttons instead of full-width Buttons. The action list retains declaration order
from top to bottom or left to right, and messages retain their original indices.
Keys, passive labels and host-controlled visibility are preserved. Direct builder
struct literals need the new direction field; existing constructor calls retain
the Up default. There are no manifest, dependency, lockfile, Scene or platform edits.

Gallery adds a direction cycle button, wraps its instruction text and cancels
cached hover/press when changing direction. The pointer test now scrolls targets
into the content viewport, as required by the current clipped HitTestLayer.
Three new component regressions cover direction/action/passive/closed contracts,
retained IDs/focus through direction and keyed reorders, and text-measured geometry
at 400/600 px and scales 1/1.25/2. Gallery checks cover pointer selection in all
directions, press cancellation and matching hit/paint slots with both themes,
scales 1/1.25/2 and open/closed states. Existing vertical 240 px coverage remains.

Verification and native capture results are pending. This document will record
the actual tested implementation SHA and results before delivery.

Viewport anchoring, animation and automatic dismissal remain separate work.
Human Gallery, Windows 注音 IME/candidate placement, physical DPI/touch, GPU
and screen readers are NOT_RUN. No project completion or release claim.
