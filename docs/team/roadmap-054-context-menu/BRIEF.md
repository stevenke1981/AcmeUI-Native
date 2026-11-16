# ROADMAP 054 — ContextMenu typed item-message conversion

2026-11-14 Asia/Taipei. Base `29e7cc83d2f2ad0db7003afb8df1c79de99ded17`,
branch `sdk/roadmap-054-context-menu`, stacked on ROADMAP 053 (PR #62). Checks
apply to uncommitted changes on that base, not a new commit.
Execution: **Codex CLI (OpenRouter nvidia/nemotron-3-ultra-550b-a55b:free) + author review**.
sigma + Groq `qwen/qwen3.8-27b` was tried first and did not produce code.
Independent review NOT_RUN.

Investigate regression-first: ContextMenu current typed item messages when open,
passive Labels when `message` is None, no activation for disabled items,
Separator nodes for separators, closed menus still rendering just the anchor,
retained anchor identity and focus through open-item changes, and finite
positive non-overlapping actionable item rectangles via real LayoutEngine text
layout.

If proven broken, bind the current item message to an instance-keyed Button
(`{menu_id}_{item.key}`, or `{menu_id}_item_{i}` when the item key is empty)
while preserving public fields/signatures, closed-state rendering, existing
unit tests and DropdownMenu/Drawer/HoverCard/CommandPalette behavior. Do not
add public `build()`, on_open/on_close, submenu rendering or right-click
gesture hosting. Stop if more than approximately five production files or API
changes are required. Add a small host-controlled Overlay Gallery
select/restore preview with stale-click rejection. Submenu `children`,
DropdownMenu dispatch and hardware acceptance remain separate.

Run targeted locked crate tests, strict touched-crate Clippy and cargo fmt only;
reuse target/. No commits, pushes, workspace gates, evidence.json or PNGs.
Human Gallery, Windows 注音 IME, physical DPI/touch, GPU, screen reader,
native X11 capture and SDK CPU profile: NOT_RUN.
