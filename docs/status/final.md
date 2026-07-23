# Delivery Status

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
