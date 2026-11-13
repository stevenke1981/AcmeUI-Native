# ROADMAP 053 — Modal typed Close conversion

2026-11-13 Asia/Taipei. Base `13ee4d912c9ac90c489d06e21d5a1125c4d11543`,
branch `sdk/roadmap-053-modal-close`, stacked on ROADMAP 052. Checks apply to
uncommitted changes on that base, not a new commit.
Execution: **Codex CLI (OpenRouter nvidia/nemotron-3-ultra-550b-a55b:free) single writer + author review**. Independent review NOT_RUN.

Investigate regression-first: Modal current typed Close messages when closable,
passive ✕ Label when `on_close` is None, hidden control when `closable(false)`,
retained title/body identity and focus through close/title/footer changes, and a
finite positive Close rectangle inside the dialog via real LayoutEngine text
layout and the Gallery context/sequential/retained paths.

If proven broken, bind the current Close message to an instance-keyed Button
(`{id}_close`) while preserving public fields/signatures, Dialog composition,
unused `backdrop` metadata and existing unit tests. Keep ConfirmDialog,
CommandPalette, AboutDialog and Fullscreen behavior intact. No public API
expansion, dependencies, manifests, Scene/platform edits, GPUI or hardcoded
colors. Stop if more than approximately five production files or API changes
are required. Add a small host-controlled Overlay Gallery close/restore preview
with stale-close rejection. Dialog primitive `title`/`open` policy, backdrop
painting, Escape/focus trap and hardware acceptance remain separate.

Run targeted locked crate tests, strict touched-crate Clippy and cargo fmt only;
reuse target/. No commits, pushes, workspace gates, evidence.json or PNGs.
Human Gallery, Windows 注音 IME, physical DPI/touch, GPU, screen reader,
native X11 capture and SDK CPU profile: NOT_RUN.
