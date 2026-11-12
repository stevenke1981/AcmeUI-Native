# ROADMAP 051 — Notification typed action/Close conversion and identity

2026-11-13 Asia/Taipei. Base `d303108ae934499cd5124cd5ec6bb32e547455f5`
(ROADMAP 050 merged), branch `sdk/roadmap-051-notification-actions`.
Execution: **Codex CLI single writer + author review**. Independent review NOT_RUN.

Bounded acceptance: investigate conversion before changing production code.
Current typed action/Close messages, distinct retained identities across keyed
notifications/reorder, and optional description/action/Close continuity must be
covered using RetainedTree, bind_retained and FocusManager. Preserve the four
existing unit tests, public API, Card/Row/Column and icon/title/description layout.
Change runtime keys only if a behavioral regression demonstrates a defect.

Add a minimal host-controlled Overlay Gallery preview with action/Close counts,
Show notification restore, stale-message rejection, pointer and real-layout
finite/nonoverlapping hit, theme/scale/clip checks. Keep mobile_notification separate.
No dependencies, manifests, Scene/platform changes, GPUI or hardcoded colors.

Use only targeted crate tests/Clippy and cargo fmt, reusing target/. No commits,
pushes, workspace gates, CPU profile, evidence.json or PNG artifacts in this slice.
The requested `acme-gallery` package is apps/gallery; the edited apps/acme-gallery
is package `acme-ui-gallery`, so also test/lint that actual target.

Human Gallery, Windows 注音 IME, physical DPI/touch, GPU, screen reader and native
X11 CPU-window capture are NOT_RUN. Reviewer verification table stays pending.
