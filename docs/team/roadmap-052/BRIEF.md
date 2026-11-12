# ROADMAP 052 — ConfirmDialog typed Confirm/Cancel reachability

2026-11-13 Asia/Taipei. Base `47b14c926d2d4bbd57d6b8a7dbf1acc524d35384`,
branch `sdk/roadmap-052-confirm-dialog`, stacked on ROADMAP 051. Checks apply to
uncommitted changes on that base, not a new commit.
Execution: **Codex CLI single writer + author review**. Independent review NOT_RUN.

Investigate regression-first: ConfirmDialog current typed Confirm/Cancel messages,
passive Buttons with None, retained binding without mismatches and Confirm focus
through label/message updates. Require content/layout child alignment, finite,
positive, non-overlapping button rectangles within dialog width via real
LayoutEngine text layout and the Gallery context/sequential/retained paths.

If proven broken, include existing content in both Dialog layout paths while
preserving width/height Length semantics, public variant/struct/builders and
retained IDs. Keep accessibility numbering aligned and Modal/CommandPalette/
AboutDialog/devtools compatible. No public API expansion, dependencies, manifests,
Scene/platform edits, GPUI or hardcoded colors. Stop if more than approximately
five production files or API changes are required. Optional Gallery preview may
be skipped; title/open policy is investigation-only.

Run targeted locked crate tests, strict touched-crate Clippy and cargo fmt only;
reuse target/. No commits, pushes, workspace gates, evidence.json or PNGs.
Human Gallery, Windows 注音 IME, physical DPI/touch, GPU, screen reader,
native X11 capture and SDK CPU profile: NOT_RUN.
