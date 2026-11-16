# Delivery Status

## 2026-11-16 — ROADMAP 055 RadioCards typed selection and arrow-key navigation

- `RadioCards::on_select` was stored but never wired. Options are now instance-keyed Buttons, and `navigate(forward)` is added; there is a Gallery Inputs demo with pointer and keyboard (F6/arrows/Enter/Space/Esc) host. Workspace tests 2050 passed / 0 failed; clippy/fmt PASS; [report](../team/roadmap-055/REPORT.md).
- Execution: Claude Code agent team (Fable 5.2 adviser/reviewer; Opus 5.5 component; Sonnet 5.5 Gallery; Haiku 5.5 verification). Human Gallery, Windows 注音 IME, DPI/touch, GPU, screen reader NOT_RUN.

## 2026-11-16 — Landed stacked ROADMAP 052a–054a

- PRs #61/#62/#64 had merged into stacked base branches, not `master`. Landed onto `master` via `chore/land-stacked-052-054`; their reports moved to `docs/team/roadmap-05{2,3,4}-*/` because the numbers were reused on `master`.

## 2026-11-15 — ROADMAP 052 Immediate-mode Painter, HitTestLayer & backlog completion

- Implemented immediate-mode `Painter` / `Canvas` / `painter_bridge` and `HitTestLayer` / `ClickDetector` / `hit_bridge`, migrating `apps/playground`, `apps/gallery`, and `apps/acme-gallery` off `Frame`/`scene_from_frame` while keeping `confirm_dialog.rs`, `modal.rs`, `context_menu.rs`, and `Cargo.lock` untouched.
- Added `HighContrastPack`, zero-dependency `Theme::to_json`/`from_json` + token validation, `acme-platform` `KeyModifiers`/`KeyInput`, `LiveRegion`, `FrameDirtyTracker`, UTF-8 clamping in `acme-textinput`, `ProgressRing`, `QRCode`, chart migrations, `DropdownMenu`, `Drawer`, `DatePicker`, `Combobox`, `SliderMarks`, `BottomNav`, `PullToRefresh`, and `MobileTabBar`/`MobileDrawer`/`MobileCarousel`.
- Resolved all P1/P2 findings (`Dialog`/`Popover`/`Table`/`DataGrid` overlay & compound-cell geometry, `Drawer`/`MobileDrawer` scrim stack sizing + gallery hover/press alignment, `SceneClipGuard` + `finish_checked()`, UTF-8 offsets, AA tone contrast); [report](../team/roadmap-052/REPORT.md). Live GPU window, Windows 注音 IME, physical DPI/touch, and screen readers remain NOT_RUN.

## 2026-11-14 — ROADMAP 054a ContextMenu typed item-message conversion

- RED guards prove ContextMenu ignored `MenuItem.message` and rendered unkeyed Labels. Open menus now emit instance-keyed Buttons with the current message; missing callbacks and disabled items stay passive Labels, and separators stay Separator nodes. A stable dropdown Card key preserves anchor identity through item changes.
- Added conversion/focus/geometry guards and a host-controlled Overlay Gallery Cut/Copy/restore preview with stale-message rejection and pointer/theme/scale/layout tests. [Report](../team/roadmap-054-context-menu/REPORT.md); Reviewer-run gates PASS (incl. workspace tests).
- Execution: Codex CLI (OpenRouter nvidia/nemotron-3-ultra-550b-a55b:free) + author review. Human Gallery, Windows 注音 IME, physical DPI/touch, GPU, screen reader, native X11 capture and SDK CPU profile NOT_RUN.


## 2026-11-13 — ROADMAP 053a Modal typed Close conversion

- RED guards prove Modal ignored `on_close` and rendered an unkeyed ✕ Label. Closable modals now emit an instance-keyed Button with the current message; a missing callback stays a passive Label and `closable(false)` hides the control. Title/body/footer keys preserve identity through optional close and footer changes.
- Added conversion/focus/geometry guards and a host-controlled Overlay Gallery close/restore preview with stale-message rejection and pointer/theme/scale/layout tests. [Report](../team/roadmap-053-modal-close/REPORT.md); Reviewer-run gates PASS (incl. workspace tests).
- Execution: Codex CLI (OpenRouter nvidia/nemotron-3-ultra-550b-a55b:free) single writer + author review. Human Gallery, Windows 注音 IME, physical DPI/touch, GPU, screen reader, native X11 capture and SDK CPU profile NOT_RUN.

## 2026-11-13 — ROADMAP 052a ConfirmDialog typed Confirm/Cancel reachability

- RED guards prove Dialog omitted content from layout. Both paths now include content, preserving Length/public API and retained binding; accessibility numbering follows the new layout.
- Typed/passive messages and retained Confirm focus were already green. Added geometry, child/ID alignment, Length and accessibility bounds guards. Four old Leaf assertions updated; optional Gallery preview skipped. [Report](../team/roadmap-052-confirm-dialog/REPORT.md); Reviewer-run gates PASS (incl. workspace tests).
- Execution: Codex CLI single writer + author review. Human Gallery, Windows 注音 IME, physical DPI/touch, GPU, screen reader, native X11 capture and SDK CPU profile NOT_RUN.

## 2026-11-13 — ROADMAP 051 Notification typed action/Close identity

- Regression-first investigation found current typed messages, scoped retained identities/reorder and optional-sibling continuity already green. Added guards; production conversion and public API remain unchanged.
- Added host-controlled Overlay Gallery action/Close/restore preview with stale-message rejection and pointer/theme/scale/layout tests. [Report](../team/roadmap-051/REPORT.md); reviewer-run gates PASS.
- Execution: Codex CLI single writer + author review. Human Gallery, Windows 注音 IME, physical DPI/touch, GPU, screen reader and native X11 CPU-window capture NOT_RUN.

## 2026-11-12 — ROADMAP 050 Collapsible typed toggle

- Enabled Collapsible headers now carry the current typed toggle message; disabled and unconfigured headers remain passive. Existing Row/chevron/title/body composition and host-owned visibility are preserved. Header identity/focus survives open/title/message updates and keyed instance reorder; disabled headers lose focus eligibility.
- Added conversion/focus and controlled Gallery pointer/disabled-stale-action/theme/scale/clip/layout evidence. Exact tested source and checks: [report](../team/roadmap-050/REPORT.md).
- Execution: single-agent-role-passes; author self-review. Disclosure roles, animation and generic keyboard hosting remain separate. Human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen readers remain NOT_RUN.

## 2026-11-12 — ROADMAP 049 Anchor typed navigation

- Configured Anchor links now carry the current href message; passive Label text, active marker and host-owned state remain compatible. Instance/path keys preserve focused action identity through active href/callback updates and keyed instance reorder.
- Added conversion/focus and Gallery pointer/theme/scale/clip/layout evidence. Exact tested source and checks: [report](../team/roadmap-049/REPORT.md).
- Execution: single-agent-role-passes; author self-review. Automatic scrolling, deeper nesting and generic navigation hosting remain separate. Human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen readers remain NOT_RUN.

## 2026-11-12 — ROADMAP 048 Fullscreen Close conversion

- Configured Fullscreen Close now carries its typed message; passive Label and hidden-control behavior remain compatible. Stable header/body keys preserve body focus when the close header changes.
- Added conversion/focus and Gallery close/reopen/stale-message/theme/scale/clip evidence. Exact tested source and checks: [report](../team/roadmap-048/REPORT.md).
- Execution: single-agent-role-passes; author self-review. Viewport placement, backdrop painting and full overlay/focus-trap semantics remain separate. Human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen readers remain NOT_RUN.

## 2026-11-12 — ROADMAP 047 toolbar investigation

- The stored ROADMAP 046 first/finished toolbar pixel regions are identical. Expanded layout/glyph probes and new light/dark X11 CPU captures do not reproduce the reported clipping difference; no runtime fix is claimed.
- Added real-frame toolbar paint-invariance and pointer-slot guards across scroll/Tour changes, themes, scales and viewport heights. Exact tested source, checks and captures: [report](../team/roadmap-047/REPORT.md).
- Execution: single-agent-role-passes; author self-review. Human Gallery, Windows 注音 IME, physical DPI/touch, GPU and screen readers remain NOT_RUN. Tour full overlay/focus/spotlight behavior remains separate.

## 2026-07-23 — AcmeUI design Skill scope

### Completed

- Removed the blanket restriction that excluded general Rust, GPUI, and Web UI projects.
- Added stack-aware routing so cross-project use reuses design and Visual QA workflows without implicitly migrating the target to AcmeUI-Native.
- Scoped WidgetNode, wgpu/winit, Cargo, and no-GPUI implementation rules to AcmeUI-Native targets.
- Updated Codex-facing Skill metadata and user documentation.

### Acceptance truth

- Non-AcmeUI projects may invoke the Skill.
- Skill activation alone does not authorize an architecture migration.
- Runtime code and framework behavior are unchanged.

## 2026-07-23 — README visual identity

### Completed

- Added a project-owned, text-free concept hero that illustrates the
  declarative widget → layout → GPU rendering story without inventing a logo
  or imitating an operating-system screenshot.
- Added the hero to both README languages with accurate localized alt text and
  an explicit concept-art disclaimer.
- Surfaced the existing real Gallery capture in both READMEs and kept its
  remaining manual validation caveat visible.
- Optimized the generated source to a 1600×900 WebP stored in the repository.

### Acceptance truth

- Visual asset review: passed for composition, absence of text/logos, and
  truthful labeling.
- Markdown asset references: validated locally in both READMEs.
- Runtime behavior and public API: unchanged.
- Gallery manual Light/Dark and pointer-interaction sign-off: still pending.

## 2026-07-23 — Mobile button and Gallery hardening

### Completed

- `mobile_button` now maps Sm/Md/Lg to the matching primitive size and preserves 36/44/52px mobile touch heights.
- Full-width, disabled, and `on_press` behavior are covered by focused tests.
- The `mobile` Cargo feature now declares its real `foundations` dependency, so it compiles correctly with default features disabled.
- The V2 Gallery has a Mobile page with all three sizes, a visible disabled state, and last-action feedback.
- Gallery category IDs, layout styling, toolbar offsets, and content hit indices now derive from the category catalog instead of a hardcoded count.
- Disabled or message-less buttons remain drawable while only interactive buttons consume hit-test indices.
- The Gallery starts on the newest showcase category so the delivered component is immediately discoverable.

### Review findings and recommended next work

1. **Complete the remaining mobile components.** The component files now contain partial builders, but the todo list still tracks fourteen unfinished contracts. Implement one component per bounded pass with builder, tests, and Gallery evidence.
2. **Make visual regression deterministic.** Add an in-app or renderer-level screenshot path that can capture wgpu surfaces without relying on desktop focus or occlusion, then compare Light/Dark 1280×720 goldens in CI.
3. **Restore the workspace formatter and clippy gates on the active Rust toolchain.** Rust 1.97 reformats pre-existing files and exposes 15 existing warnings in unrelated modules; clear them in a separate mechanical cleanup so both gates become meaningful again.
4. **Decompose `apps/acme-gallery/src/main.rs`.** Page construction, runtime state, rendering, and hit testing still share one large file. Split only after screenshot coverage exists to control regressions.
5. **Keep manual platform claims open.** Traditional Chinese 注音, GPU device loss, and Windows DPI checklists remain unsigned and must not be promoted to validated status.

### Acceptance truth

- Functional and layout acceptance: passed by targeted tests and native build.
- Theme implementation: semantic tokens only; no widget color literals were added.
- Visual acceptance: blocked by Windows capture permissions/interface support; no screenshot or score is claimed.
- Traditional Chinese IME: unchanged and still pending manual validation.
