# AcmeUI-Native Migration Guide

Status: **draft, not released**. Owner decision pending for the next version number.
Date of this draft: 2026-11-07. Branch: `sdk/roadmap-007-packaging`.

This guide covers the move from the workspace version `0.2.0` to the next release.
It covers the changes in the "Unreleased" sections of [`docs/CHANGELOG.md`](../CHANGELOG.md).
It also lists changes that the CHANGELOG does not describe. Those changes were found by
comparing the source code.

Read this guide with [`SDK_GUIDE.md`](SDK_GUIDE.md). The SDK guide describes the current API.
This guide describes what changed.

## 1. Baseline and scope

1. Use the workspace version `0.2.0` as the baseline. It is set in `Cargo.toml` line 12.
2. The baseline commit is `05ac86e`. It is the first commit where `Cargo.toml` shows `version = "0.2.0"`.
3. The repository has no git tags. Do not use a tag name as the baseline.
4. The target is the code at `HEAD` of this branch, with the "Unreleased" CHANGELOG sections.
5. The next version number is not decided. [`RELEASE.md`](RELEASE.md) proposes `0.3.0`.
6. The CHANGELOG has headings `v0.2.1` to `v0.2.3`. These headings do not match the commit history. See section 6.
7. Line numbers in this guide refer to the working tree at `HEAD` `fa223d9`.

Verification status labels:

- **Verified**: the source was read at the line shown, or a command produced the result.
- **Not verified**: the statement was not checked in this review. It must be checked before a release claim.
- **Compiled**: the snippet is built and run by `examples/sdk-consumer/tests/migration_snippets.rs` (CI job `sdk-consumer`).
- **Not compiled**: a code snippet was taken from the source, but it is not built (old API or fragments).

## 2. Summary of changes

| ID | Change | Crate or app | Type | Action |
|----|--------|--------------|------|--------|
| M1 | `PlatformError` has two new variants. `run()` returns `Err` in two new cases. | `acme-platform` | Breaking for exhaustive `match` | Add match arms. Do not expect `Ok(())` when no window opens. |
| M2 | Renderer selection through `ACME_RENDER_BACKEND`. New public type `RenderBackendPreference`. `cpu-renderer` feature (default on). | `acme-platform` | Additive, behaviour | Check the environment value in your launch scripts. |
| M3 | `GlyphAtlas` no longer clears mid-frame. Eviction runs in `begin_frame`. Glyphs can be dropped for one frame. | `acme-text` | Behaviour | Call `begin_frame()` once per frame. Read `atlas_full_glyphs`. |
| M4 | `apps/benchmark` runs headless. `--window` was removed. `--quick` and `--json PATH` were added. | `acme-benchmark` (app) | CLI | Remove `--window` from scripts. |
| M5 | Retained binding API: `acme_widgets::retained_binding`, `to_layout_with_binding`, `update_with_binding`, `build_tree_update_with_binding`. | `acme-widgets`, `acme-accessibility` | Additive | Optional. Adopt when you need stable node IDs. |
| M6 | Accessibility roles, states and actions changed for buttons, text inputs, tabs, navigation items and dialogs. | `acme-accessibility` | Behaviour | Review tests that check roles or actions. |
| M7 | `AccessibilityAdapter::clear_widget_focus` was added. | `acme-accessibility` | Additive | Optional. |
| M8 | Text input: cursor snaps to grapheme boundary. Escape cancels composition first. Read-only fields ignore undo and redo. `scroll_caret_into_view` was added. | `acme-textinput` | Behaviour, additive | Review keyboard and undo expectations. |
| M9 | Core invalidation: parents and ancestors are marked dirty after child changes. | `acme-core` | Behaviour | Review code that uses `DirtyFlags`. |
| M10 | Gallery and Playground retained IDs, focus, layout, and shortcut changes. | apps only | No library API | None for library consumers. |
| M11 | `RenderBatch.clip` type changed from `Option<[u32; 4]>` to `Option<[f32; 4]>`. Not in CHANGELOG. | `acme-render-wgpu` | Breaking for field users | Change the type in your code. |
| M12 | `Application::background_tick` and `background_poll_interval` were added as default methods. Not in CHANGELOG. | `acme-platform` | Additive | None for implementers that keep the defaults. |
| M13 | CHANGELOG `v0.2.0` says `route_action` returns `Option<PlatformEvent>`. The code returns `Vec<PlatformEvent>`. | documentation | Documentation error | Use `Vec`. |
| M14 | Multi-page glyph atlas: new fields `GlyphDraw::atlas_page`, `acme_text::AtlasUpload::page`, `PreparedGlyph::atlas_page`; new `GlyphAtlasConfig`, `GlyphAtlas::with_config`/`Default`; page edge capped at 2048. | `acme-core`, `acme-text`, renderers | Breaking for struct literals, behaviour | Add the page fields; prefer `GlyphAtlas::default()`. |

## 3. Details

### M1. `PlatformError` and `run()` (breaking)

**What changed.** `PlatformError` has two new variants. `run()` now returns an error in two more cases.

**Who is affected.** Any code that matches `PlatformError` without a wildcard arm. Any code that expects
`run()` to return `Ok(())` when no window opens.

**Before** (`05ac86e`, `crates/acme-platform/src/lib.rs` lines 212-227). Verified.

```rust,ignore
pub enum PlatformError {
    #[error("event loop error: {0}")]
    EventLoop(String),
}
// run() returned Ok(()) after run_app() returned without an error.
```

**After** (`crates/acme-platform/src/lib.rs` lines 238-248 and 255-263). Verified.

- `PlatformError::NoWindows { reasons: Vec<String> }`. No window could be created. `reasons` has one entry for each failure.
- `PlatformError::Renderer(String)`. The event loop stopped because a renderer failed and could not recover.
- `PlatformError` has no `#[non_exhaustive]` attribute. Verified by grep in `crates/acme-platform/src`.
- If a renderer error and an event loop error both occur, the renderer error is returned (`finish_run`, `crates/acme-platform/src/lib.rs` line 277).
- When the last window closes normally, `run()` still returns `Ok(())`.

**Before (0.2.0 code, fails to compile now).** Not compiled.

```rust,ignore
fn describe(error: acme_platform::PlatformError) -> String {
    match error {
        acme_platform::PlatformError::EventLoop(message) => message,
    }
}
```

**Required action.** Add the new arms or a wildcard arm. Example (adapted from `docs/guides/SDK_GUIDE.md` section 4; compiled by `m01_*` in `migration_snippets.rs`):

```rust
match acme_platform::run(app) {
    Ok(()) => {}
    Err(acme_platform::PlatformError::NoWindows { reasons }) => {
        eprintln!("no window could be opened: {}", reasons.join("; "));
        std::process::exit(1);
    }
    Err(error) => {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
```

The `NoWindows` error also covers a `cpu` request when the `cpu-renderer` feature is off. The error text is
`CPU backend requires the acme-platform/cpu-renderer feature` (`crates/acme-platform/src/backend.rs` line 17;
`crates/acme-platform/src/lib.rs` lines 375 and 416).

### M2. Renderer backend selection (`ACME_RENDER_BACKEND`)

**What changed.** The environment variable `ACME_RENDER_BACKEND` now selects the renderer per window.
The `cpu-renderer` Cargo feature was added and is on by default. Verified.

**Before.** `05ac86e` has no `ACME_RENDER_BACKEND` reference and no `[features]` table in
`crates/acme-platform/Cargo.toml`. Verified by `git grep` and `git show`.

**After.**

- `crates/acme-platform/Cargo.toml` line 13-14: `default = ["cpu-renderer"]`, `cpu-renderer = ["dep:acme-render-cpu", "dep:softbuffer"]`.
- `RenderBackendPreference` (`crates/acme-platform/src/backend.rs` line 27) has variants `Auto` (default), `Gpu` and `Cpu`.
- It is re-exported at `crates/acme-platform/src/lib.rs` line 7.
- `RenderBackendPreference::from_env()` (line 65) is read when a window is created (`lib.rs` line 369).

Parsing rules (from `from_env_value`, `backend.rs` lines 40-63, and its unit tests):

| Value | Result |
|-------|--------|
| unset, empty, or only spaces | `Auto` |
| `auto`, `gpu`, `cpu` | the matching variant. Case does not matter. Surrounding spaces are removed. |
| any other text | `from_env_value` returns `Err(message)`. `from_env` logs it with `tracing::warn!` and uses `Auto`. |

A value such as `" cpu"` with a leading space now forces the CPU renderer. The CHANGELOG batch-002 entry
says that an earlier state ignored such a value. Your baseline `0.2.0` had no such variable, so this is a new rule
for you.

Plan mapping (`plan_renderer`, `backend.rs` lines 106-119):

| Requested | `cpu-renderer` on | `cpu-renderer` off |
|-----------|-------------------|--------------------|
| `auto`, `gpu` | GPU, then CPU if GPU setup fails | GPU only |
| `cpu` | CPU only | window creation fails with the CPU feature error |

**Code.** Compiled by `m02_*` in `migration_snippets.rs`.

```rust
use acme_platform::RenderBackendPreference;

assert_eq!(
    RenderBackendPreference::from_env_value(Some(" GPU ")),
    Ok(RenderBackendPreference::Gpu)
);
```

**Required action.** Check any launch script that sets `ACME_RENDER_BACKEND`. Set one of `auto`, `gpu` or `cpu`.
If you build with `default-features = false`, `cpu` does not work.

### M3. `GlyphAtlas` eviction (behaviour)

**What changed.** The automatic eviction in the middle of a frame was removed. Eviction runs only in
`GlyphAtlas::begin_frame()`. Verified.

**Why.** The old code cleared the atlas during a frame. That overwrote glyphs that a `PreparedText` in the same frame
already used. The old code is at `47f7f46^`, `crates/acme-text/src/lib.rs` lines 479-485:

```rust,ignore
if self.cursor_y + height > self.height {
    // Atlas full — try to evict stale entries (60 frames ≈ 1s at 60fps).
    if self.evict_stale(60) > 0 {
        // ...
```

**After** (`crates/acme-text/src/lib.rs`):

- `insert` (line 487) returns `None` when the atlas is full and a glyph of the current frame is in use
  (`in_use_this_frame`, lines around 512-515). It also sets `needs_space`. The glyph is then dropped for this frame.
- `FontSystem::prepare` (line 204) resets `diagnostics.atlas_full_glyphs` at the start (line 206) and counts dropped glyphs.
- `GlyphAtlas::begin_frame` (line 442) increments the frame counter. It clears the atlas only when `needs_space` is set
  or the used shelf height is at least 75 % of the atlas height, and some entries were idle for more than
  `STALE_AGE_FRAMES` (60) frames (line 392).
- `GlyphAtlas::clear()` (line 426) also resets `needs_space`.
- `PreparedText.atlas_generation` (line 332) is copied from the atlas generation. The generation changes when the atlas is cleared.
  Because clearing can happen at frame start, a generation change can happen earlier than before.
- `FallbackDiagnostics.atlas_full_glyphs` (line 86) is read with `FontSystem::diagnostics()` (line 117).

**Who is affected.** Any code that draws text with `GlyphAtlas` and `FontSystem::prepare`.

**Before and after code.** The Gallery calls `begin_frame()` once per frame before it prepares text
(`apps/gallery/src/main.rs` line 466; atlas created with `GlyphAtlas::new(2048, 2048)` at line 205). Adapted; compiled by `m03_*` in `migration_snippets.rs`.

```rust
// Once per frame, before any text is prepared for that frame.
self.atlas.begin_frame();
let prepared = fonts.prepare(&layout, &mut self.atlas);

// Counts glyphs that were dropped during this prepare call.
let dropped = fonts.diagnostics().atlas_full_glyphs;
```

**Required action.**

1. Call `begin_frame()` exactly once for each frame. Call it before `prepare`.
2. Read `atlas_full_glyphs` after `prepare`. If the value stays above zero, give the atlas more pages with `GlyphAtlas::with_config` (M14). Pages larger than 2048 × 2048 are not accepted.
3. Do not assume that `atlas_generation` stays the same during a frame.

**Known limitation.** The test `bug_glyph_dropped_while_atlas_holds_only_recent_entries` is `#[ignore]`d
(`crates/acme-text/tests/atlas_lifecycle.rs` line 644). A glyph can still be dropped while every entry was used within the last 60 frames.

### M4. Benchmark application (`apps/benchmark`)

**What changed.** The benchmark runs without a window and without a GPU device. The command line has new options.
Verified.

**Before.** Commit `a1c3ebc^` ran the benchmark inside a window through `acme_platform::run(BenchmarkApp::new())`.
It used `window_config()`.

**After.**

- `apps/benchmark/src/main.rs` line 20 calls `options::parse(...)`. Errors stop the program before any window is created.
- `--quick`: small warm-up and sample counts. The numbers are noisy (`apps/benchmark/src/options.rs` lines 8, 31).
- `--json PATH` or `--json=PATH`: write results to a file. The schema is `acmeui-bench`, version 1 (`options.rs` line 9; `report.rs` line 37 writes `package_version` from `CARGO_PKG_VERSION`).
- `--window`: returns the error `--window was removed: the benchmark runs headless only` (`options.rs` lines 37-38).
- `--help`: prints the usage text.

**Required action.** Remove `--window` from scripts. Use `--json PATH` for machine-readable output.
Do not use `--quick` numbers as performance evidence. The CHANGELOG says there are no FPS targets and no CI thresholds.
Methodology is in [`BENCHMARKS.md`](BENCHMARKS.md).

Not verified: the JSON field list in `apps/benchmark/src/json.rs` and the numbers themselves.

Command for the help text (not run in this review, because it builds the wgpu dependency tree):

```text
cargo run --locked -p acme-benchmark -- --help
```

### M5. Retained binding API (additive)

**What changed.** A binding layer gives widgets stable `NodeId` values across frames. Layout and accessibility can use the same IDs.
This is new public API. The old functions still exist. Verified.

**Public identifiers** (all verified):

| Identifier | Location |
|------------|----------|
| `acme_widgets::retained_binding` (public module) | `crates/acme-widgets/src/lib.rs` line 15 |
| `bind_retained<'a, M>(roots: &'a [WidgetNode<M>], tree: &RetainedTree) -> RetainedBinding<'a>` | `crates/acme-widgets/src/retained_binding.rs` line 67 |
| `RetainedBinding<'a>` with `id_for`, `len`, `is_empty`, `mismatches` | `retained_binding.rs` line 30 and following |
| `SYNTHETIC_ID_BASE: u64 = 1 << 48` | `retained_binding.rs` line 26 |
| `SyntheticIds::new()`, `allocate()` | `retained_binding.rs` lines 120-140 |
| `is_synthetic(id: NodeId) -> bool` | `retained_binding.rs` line 147 |
| `WidgetNode::to_layout_with_binding(&self, binding: &RetainedBinding<'_>, context: &WidgetLayoutContext) -> LayoutNode` (needs `M: Clone`) | `crates/acme-widgets/src/lib.rs` line 477 |
| `AccessibilityAdapter::update_with_binding(&mut self, root, snapshot, binding)` | `crates/acme-accessibility/src/lib.rs` line 118 |
| `build_tree_update_with_binding(root, snapshot, binding, focus_id, label_override)` | `crates/acme-accessibility/src/lib.rs` line 312 |

**Unchanged functions.** `WidgetNode::to_layout` (`acme-widgets/src/lib.rs` line 154),
`WidgetNode::to_layout_with_context` (line 454), `AccessibilityAdapter::update` (accessibility `lib.rs` line 95)
and `build_tree_update` (line 288).

**Binding rules** (from the module documentation, `retained_binding.rs` lines 1-16 and 60-65):

- The binding uses the address of each widget inside the borrowed description. Do not clone or rebuild the description
  between `bind_retained` and layout or accessibility.
- Call `bind_retained` only after `RetainedTree::reconcile_roots` succeeded for the same roots.
- Check `binding.mismatches() == 0` before you use the binding. Otherwise use the sequential functions for that frame.
- Synthetic IDs (`>= SYNTHETIC_ID_BASE`) are used for rows and cells that have no widget. They are not stable across shape changes.

**Pattern.** The Playground applies the same rules (`apps/playground/src/main.rs` lines 248-262), but it assigns IDs with its own helper `assign_retained_ids`. The snippet below is adapted. `viewport: (f32, f32)` and `scale_factor: f32` come from the window. It needs a function that returns `Result`.

```rust
use acme_widgets::retained_binding::bind_retained;
use acme_widgets::view_bridge::widget_to_views;

let views = widget_to_views(std::slice::from_ref(description));
if tree.reconcile_roots(&views).is_ok() {
    let binding = bind_retained(std::slice::from_ref(description), &tree);
    if binding.mismatches() == 0 {
        let root = description.to_layout_with_binding(&binding, &layout_context);
        let snapshot = layout_engine.compute_with_text(&root, viewport, &mut fonts, scale_factor)?;
        accessibility.update_with_binding(description, &snapshot, &binding);
    }
}
```

The Gallery follows the same rule (`apps/gallery/src/main.rs` lines 474-478 and 506-512). It also falls back to
`to_layout(NodeId::new(1))` and `update(...)` when the binding is not usable. Verify the fallback in your code.

**Required action.** None if you keep the old functions. Adopt the binding API only when you need stable IDs.

### M6. Accessibility roles, states and actions (behaviour)

**What changed.** The AccessKit tree now uses more roles and states. Some actions were removed. Verified by source.

| Widget | Before (`05ac86e`) | After (HEAD) | Source (HEAD) |
|--------|--------------------|--------------|---------------|
| Button, disabled | `Click` action always added (`05ac86e` lines 311-316) | disabled state set. `Click` not added. | `apply_widget_state`, `crates/acme-accessibility/src/lib.rs` lines 575-588 |
| TabBar tab | generic role. No tab role. | role `Tab`. `selected` set (`false` when not selected). | lines 405, 496, 597 |
| NavRail selected item | no `selected` state | `selected` set on the selected item | lines 599-600 |
| TextInput, password | role `TextInput`. No password role. | role `PasswordInput`. No value property. | lines 455, 609-613 |
| TextInput, disabled, read-only, invalid | not mapped | disabled, read-only and invalid states are set; `validation_message` sets invalid and becomes the description (ROADMAP 008) | lines 611-629 |
| TextInput, `SetValue` action | not mapped | added only when the field is not disabled and not read-only | lines 625-628 |
| Dialog, modal | role `Dialog`. No modal state. | `modal` state when the dialog is modal | lines 451, 589-591 |

Other facts:

- `AccessibilityAdapter::clear_widget_focus()` was added (`lib.rs` line 239). It makes the next `TreeUpdate` report the root as focused.
- `AccessibilityAdapter::focus_widget(NodeId)` (line 232) has the same signature as before.
- `AccessibilityAdapter::route_action` returns `Vec<PlatformEvent>` (line 185). See M13.
- Table and DataGrid cells with label or container content are `Cell` (ROADMAP 008); controls in cells keep their own role. Rows and tree items have no retained node, so there is no `Row` or `TreeItem` role.

**Required action.** Update tests that check for `Button` roles with `Click` on disabled buttons, or for `TextInput` roles on password fields.
Do not read the value of a password field from the tree. It is not present.

```rust
// Clear focus when keyboard focus moves to a row that has no accessibility node.
adapter.clear_widget_focus();
```

### M7. `clear_widget_focus` (additive)

Covered in M6. The method exists at `crates/acme-accessibility/src/lib.rs` line 239.

### M8. Text input (`acme-textinput`)

Source file: `crates/acme-textinput/src/lib.rs`.

| Topic | Before (`05ac86e`) | After (HEAD) | Lines (HEAD) |
|-------|--------------------|--------------|--------------|
| `move_cursor_to_offset(byte_offset)` | clamps the offset to the text length, then sets the cursor | also moves the cursor back to the start of the grapheme cluster that contains the offset. Signature unchanged. | 207-220 |
| Escape key | blurs the field when it has focus | when a preedit (composition) is active, Escape cancels it and keeps focus. The second Escape blurs. | `handle_key`, `PlatformKey::Escape` arm, lines 1278-1290 |
| `undo()` and `redo()` on a read-only field | no check. Undo could change a read-only field. | return `false`. State and history do not change. | lines 131-133, 163-165 |
| `scroll_caret_into_view(...)` | not present | new free function. The caller decides when to call it. | line 1185 |
| `commit_preedit()` | takes the preedit text and clears it | same. No change. | line 592 |

Signature of the new function (`lib.rs` lines 1185-1191). This is a signature listing, not a complete item, so it uses the `text` fence:

```text
pub fn scroll_caret_into_view(
    state: &mut TextInputState,
    fonts: &mut FontSystem,
    style: &TextStyle,
    scale: f32,
    content_width: f32,
)
```

Compiled by `m08_*` in `migration_snippets.rs`. Example:

```rust
acme_textinput::scroll_caret_into_view(&mut state, &mut fonts, &style, scale, content_width);
```

**Required action.**

1. Check any Escape handling in your code. Escape now has two steps while a composition is active.
2. Check any undo feature for read-only fields. Undo now does nothing on those fields.
3. Call `scroll_caret_into_view` after a caret change if the caret can leave the visible area.
4. Do not claim Traditional Chinese IME support from this change. See section 5.

### M9. Core invalidation (`acme-core`)

**What changed.** `RetainedTree::reconcile_roots` now marks nodes dirty when their child list changes. It also marks ancestors
dirty when layout can change. Verified.

**Before.** The comparison of old and new child lists (`old_children != children`) is not present in
`05ac86e:crates/acme-core/src/tree.rs`. Verified by grep.

**After** (`crates/acme-core/src/tree.rs`).

- A node whose child list changed (insert, remove or reorder) gets `CHILDREN | LAYOUT | PAINT | SEMANTICS` (lines 230-238).
- A node whose kind, focusable state or disabled state changed gets `STYLE | LAYOUT | PAINT | SEMANTICS` (lines 218-229).
- When `LAYOUT` is set on a changed node, `mark_dirty(id, LAYOUT)` walks up the parents (lines 244-246 call `mark_dirty`; `mark_dirty` itself is at lines 139-156).
- `clear_dirty(id)` resets the flags of one node (lines 157-160).
- Unchanged frames stay clean. The example checks this (`crates/acme-core/examples/reconcile_dirty.rs`).

**Required action.** If you skip work by reading `Node.dirty`, read the flags after each reconcile. More nodes are dirty after a child change.
Call `clear_dirty(id)` after you process each node.

Command for the example (not run in this review):

```text
cargo run --locked -p acme-core --example reconcile_dirty
```

### M10. Application changes (not library API)

These changes are in the apps. They are not SDK API:

- Gallery and Playground: retained IDs for layout and accessibility (`apps/gallery/src/main.rs` lines 474-512; `apps/playground/src/main.rs` lines 248-262).
- Gallery keyboard focus is keyed by stable identity (`apps/gallery/src/focus.rs`).
- Gallery TextInput uses the full control height and keeps the caret in view.
- Gallery supports Ctrl+Z and Ctrl+Y (`docs/CHANGELOG.md`). Ctrl+Shift+Z is also mentioned there.
- Showcase app (`apps/acme-gallery`, package `acme-ui-gallery`): sidebar keys are unique and category pages have separate IDs.

Use these apps as examples. Do not depend on their internal modules.

### M11. `RenderBatch.clip` type (breaking, not in CHANGELOG)

**What changed.** The field type changed from `Option<[u32; 4]>` to `Option<[f32; 4]>`. The doc comment changed from
"Physical-pixel scissor rect" to "Logical-pixel clip rect". Verified.

- Commit `f34a5a6` (2026-07-23, "feat(renderer): add RGBA image primitives") made the change.
- Location: `crates/acme-render-wgpu/src/batch.rs` lines 32-35. `RenderBatch` is `pub` and its fields are `pub`.
- `pub mod batch` is declared in `crates/acme-render-wgpu/src/lib.rs` line 4.

**Required action.** Change any code that reads or writes `RenderBatch.clip`. Use `f32` values. The meaning of the value
(logical or physical pixels) is not verified. Check how the renderer uses it before you rely on the doc comment.

### M12. `Application` default methods (additive, not in CHANGELOG)

These methods were added after the baseline. They have default implementations.

- `fn background_tick(&mut self) -> bool` (default `false`), `crates/acme-platform/src/lib.rs` around line 230.
- `fn background_poll_interval(&self) -> Option<Duration>` (default `None`), line 224.

The required `fn frame(&mut self, context: FrameContext) -> Scene` has the same signature at `05ac86e` (line 208) and at HEAD (line 234).
`ime_cursor_area`, `set_ime_cursor_area` and `on_gpu_recovered` existed at the baseline.

**Required action.** None for implementers that keep the defaults.

### M13. CHANGELOG `v0.2.0` `route_action` entry (documentation error)

The `v0.2.0` section says `route_action(&AccessibilityAction) -> Option<PlatformEvent>`.
At baseline `05ac86e`, the code already returns `Vec<PlatformEvent>` (`crates/acme-accessibility/src/lib.rs` line 140).
Commit `229ca22` made the change, and it is an ancestor of `05ac86e`. Verified with `git merge-base --is-ancestor`.

**Required action.** Use `Vec<PlatformEvent>`. Do not treat the change as a migration step from `0.2.0`. The CHANGELOG entry needs a correction (Lead).

### M14. Multi-page glyph atlas (breaking for struct literals, behaviour)

ROADMAP 011. Details: `docs/team/roadmap-011/SPEC.md`.

**Breaking changes.** Struct-literal constructors must add the new field:

- `acme_core::GlyphDraw { .., atlas_page: 0 }`
- `acme_text::AtlasUpload { page: 0, .. }`
- `acme_text::PreparedGlyph { .., atlas_page: 0 }`

Code that converts `PreparedText` into a `TextPrimitive` must copy the page:
`AtlasUpload.page = region.page` and `GlyphDraw.atlas_page = glyph.atlas_page`. Without this,
glyphs on page 1 and above are drawn from page 0.

**Additive.** `acme_core::{ATLAS_PAGE_EDGE, ATLAS_MAX_PAGES}` (2048, 8); `GlyphFormat` derives `Hash`;
`acme_text::GlyphAtlasConfig { page_width, page_height, max_pages }` (default 2048 × 2048 × 4);
`GlyphAtlas::with_config`, `GlyphAtlas::default()`, `page_count`, `max_pages`, `page_evictions`.

**Behaviour.**

- `GlyphAtlas::new(w, h)` is still a single page. Eviction still clears that page. Placement is
  identical on success. After a failed placement the cursor no longer moves, so a later smaller
  glyph can fit in the current row.
- With more than one page, a full page opens the next one. `begin_frame` evicts at most one
  least-recently-used page that holds stale entries; other pages keep their glyphs. The
  generation increases once per eviction.
- Page edges are clamped to 2048. Both built-in renderers reject uploads outside a 2048 × 2048
  page and pages ≥ 8. The wgpu renderer previously accepted a 4096 × 4096 atlas; use more pages instead.
- Renderers allocate pages lazily. The worst case is 8 pages × (4 MiB alpha + 16 MiB RGBA) = 160 MiB
  per renderer; the default 4 pages bound it at 80 MiB.
- The CPU error text for `InvalidGlyphUpload` changed to
  "atlas page out of range, region outside page, or payload size mismatch".

**Required action.** Add the page fields to struct literals and conversions. Use
`GlyphAtlas::default()` unless you need a single page.

## 4. Consumer checklist

Work through these steps in order.

1. Update the dependency. Use a path or a git `rev`. No crate is published to crates.io yet. See [`SDK_GUIDE.md`](SDK_GUIDE.md) section 1.
2. Build the project. Fix each `PlatformError` `match` (M1).
3. Check `ACME_RENDER_BACKEND` in launch scripts (M2).
4. If you draw text with `GlyphAtlas`, add one `begin_frame()` call per frame and read `atlas_full_glyphs` (M3).
5. Remove `--window` from benchmark scripts (M4).
6. If you use accessibility, update tests for disabled buttons, password fields, tabs and modal dialogs (M6).
7. If you use `RenderBatch.clip`, change the type to `f32` (M11).
8. If you use text input, check Escape and read-only undo behaviour (M8).
9. If you read `Node.dirty`, check invalidation after child changes (M9).
10. If you build `GlyphDraw`, `AtlasUpload` or `PreparedGlyph` values, add the page fields (M14).
11. Run your tests with `--locked`.

## 5. Not verified and limits

- After-form snippets are compiled and run by `examples/sdk-consumer/tests/migration_snippets.rs` (ROADMAP 008). No benchmark or GUI was run for this guide.
- Native windows, GPU device loss, Traditional Chinese IME (注音), DPI scaling at 125/150/200 %, and screen readers are **not validated**.
  See [`MANUAL_VALIDATION.md`](MANUAL_VALIDATION.md). Do not claim these work after the upgrade.
- The meaning of `RenderBatch.clip` (logical or physical pixels) is not verified (M11).
- The benchmark JSON field list is not verified (M4).
- The git dependency path (`acme-ui = { git = ..., rev = ... }`) is not verified by this guide. See `SDK_GUIDE.md` section 1.2.
- A name-level comparison of public items between `05ac86e` and `HEAD` found no removed item name in `crates/*/src`
  (1627 declarations at the baseline, 2235 at `HEAD`). This check did not compare struct fields, enum variants, trait signatures
  or generic bounds. Those are **not verified** except where this guide names them.
- A comparison of `pub fn` declaration lines found no signature change for functions that exist in both revisions.
  Functions with the same name in several `impl` blocks were not separated by this check.
- Policy note: `AGENTS.md` says "Never use byte offsets for text cursors". The public method
  `TextInputState::move_cursor_to_offset(byte_offset: usize)` still takes a byte offset (M8). This is a naming and API decision
  for the owner. It is not changed by this guide.

## 變更歷史

| 版本 | 日期 | 內容 | 影響範圍 |
|------|------|------|----------|
| draft 1 | 2026-11-07 | First draft from ROADMAP 007 P3. Covers all Unreleased CHANGELOG sections and three items found by source comparison that the CHANGELOG does not cover (M11 to M13). | `docs/guides/MIGRATION.md` |
| draft 2 | 2026-11-07 | M14 multi-page glyph atlas (ROADMAP 011); M3 advice now points to `with_config`. | `docs/guides/MIGRATION.md` |
