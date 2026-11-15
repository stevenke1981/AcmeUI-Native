# ROADMAP 052 — Immediate-mode Painter, HitTestLayer, Theme JSON & backlog completion

2026-11-15 Asia/Taipei. Execution: **Agent Team workflow (`team_chief_orchestrator` / parallel domain workers + independent reviewer `team_independent_reviewer`)**.
Base `a3e758852b77c7e94c344a267c55d7cb7441c965`. Branch `feat/immediate-painter-hit-layer`.

## Summary of changes

- **Immediate-mode painter & Canvas widget**: Added `acme_core::Painter` (`PaintClipGuard`, `finish_checked`, clip/offset stack), `acme_widgets::Canvas` / `canvas()`, and `acme_widgets::painter_bridge` (`paint_widget_tree_canvases`, `paint_widget_tree_overlays_or_primitives`, `paint_widget_tree_with_scroll` preserving unconstrained `<M>` signatures). Migrated `apps/playground`, `apps/gallery`, and `apps/acme-gallery` off `Frame`/`scene_from_frame` and adopted `SceneClipGuard` + `finish_checked()` in `apps/acme-gallery`.
- **Hit detection & ClickDetector**: Added `acme_core::{HitTestLayer, HitTestEntry, HitResult, HitZLayer, ClickDetector}` and `acme_widgets::hit_bridge` (`build_hit_layer`, `HitRegistry`) with z-ordered overlay/scrim blocking, scroll viewport clipping, and click/double-click/drag-cancel detection; integrated into `apps/playground`, `apps/gallery`, and `apps/acme-gallery`.
- **Theme packs, JSON serialization & validation**: Added `HighContrastPack`, zero-dependency `Theme::to_json` / `Theme::from_json`, UTF-8-safe `\uXXXX` hex escape parsing, and `Theme::validate()` checks for finite non-negative `SpacingTokens`/`RadiusTokens` and positive finite `TypographyTokens`.
- **Platform, accessibility & text safety**: Added `KeyModifiers` and `KeyInput` (`acme-platform`), `LiveRegion` (`acme-accessibility`), and `floor_char_boundary` clamping in `acme-textinput` (`set_selection_range`, `select_word_at_offset`, `preedit::drawn_offset`).
- **Widgets & controls**: Added `ProgressRing`, `QRCode`, chart painter migrations, `DropdownMenu`, `Drawer`, `DatePicker`, `Combobox`, `SliderMarks`, `BottomNav`, `PullToRefresh`, P4 mobile builders (`MobileTabBar`, `MobileDrawer`, `MobileCarousel`), `FrameDirtyTracker`, and AA `contrasting_fg` fallback in `resolve_tone`.
- **Preservation boundaries**: Left `confirm_dialog.rs`, `modal.rs`, `context_menu.rs`, and root `Cargo.lock` untouched (`git diff --exit-code -- Cargo.lock` = 0).

## Review findings resolved (`P1` #1–#8, `P2` #1)

1. **`P1 #1` Dialog detached overlay geometry**: `painter_bridge` and `hit_bridge` lay out detached dialog subtrees via shared `resolve_dialog_geometry` (`OVERLAY_SYNTHETIC_BASE`) instead of zero-sized unlaid children.
2. **`P1 #2` Popover overlay placement & `acme-gallery` clips**: Added shared `resolve_popover_overlay` placement, balanced `SceneClipGuard` RAII clip scopes with `finish_checked()`, and removed invalid second-child fallback in `apps/acme-gallery`.
3. **`P1 #3` Table/DataGrid headerless rows & compound cells**: Both `painter_bridge` and `hit_bridge` compute `skip_headers` accurately and walk compound cell snapshots via `resolved_cell_snapshot`.
4. **`P1 #4` Drawer/MobileDrawer backdrop scrim sizing & gallery index alignment**: Applied `.w_full().min_h(Length::Px(160.0))` on open `Stack` roots so `Percent(1.0)` scrims get non-zero hit/paint rects, plus `mobile_drawer_open_hover_press_and_scrim_close_stay_aligned` regression in `apps/acme-gallery`.
5. **`P1 #5` `acme-gallery` Canvas hit index parity**: Gated `WidgetNode::Canvas` hit slot progression on `rect.width > 0.0 && rect.height > 0.0`, matching `collect_hit_regions_with_nodes`.
6. **`P1 #6` UTF-8 safe `\uXXXX` parsing**: Fixed `parse_hex4` in `crates/acme-theme/src/json.rs` to slice `self.bytes` + `std::str::from_utf8`.
7. **`P1 #7` Multi-byte selection/preedit offsets**: Clamped byte offsets in `acme-textinput` using `floor_char_boundary`.
8. **`P1 #8` High-contrast tone AA ratio**: Added `contrasting_fg` fallback in `acme-ui::resolve_tone`.
9. **`P2 #1` Theme token validation**: Extended `Theme::validate()` to reject non-finite/negative spacing/radii and non-positive typography values.

## Verification

Run on Windows x86_64 with `--locked` after the final code edit:

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `git diff --exit-code -- Cargo.lock` | PASS (0 diff) |
| `cargo check --locked --workspace --all-targets` | PASS |
| `cargo check --locked -p acme-platform --no-default-features` | PASS |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | PASS |
| `cargo clippy --manifest-path examples/sdk-consumer/Cargo.toml --locked --all-targets -- -D warnings` | PASS |
| `cargo test --locked --workspace` | PASS (all workspace crates pass: `acme-accessibility` 129, `acme-animation` 22, `acme-benchmark` 34+2, `acme-core` 83+11, `acme-devtools` 19, `acme-gallery` 40/1 ignored, `acme-layout` 8, `acme-platform` 65, `acme-playground` 10, `acme-render-cpu` 20+8+20+9+8+7+15 / 1 ignored `frame_timing`, `acme-render-wgpu` 73 / 3 ignored, `acme-style` 11+2, `acme-text` 44+6+4, `acme-textinput` 108+20+2+18+8+13, `acme-theme` 31, `acme-ui` 865+2, `acme-ui-gallery` 94, `acme-widgets` 174+5+12; doc-tests 0 failed, 6 ignored) |
| `cargo test --locked -p acme-ui --all-features` | PASS (865 unit + 2 integration; 2 doc-tests ignored) |
| `cargo test --locked -p acme-ui --no-default-features --features inputs` | PASS (419 unit + 1 integration) |
| `cargo test --locked -p acme-ui --no-default-features --features layout` | PASS (523 unit + 2 integration; 1 doc-test ignored) |
| `cargo test --manifest-path examples/sdk-consumer/Cargo.toml --locked` | PASS (2 unit + 12 `migration_snippets` + 4 `painter_hit_canvas_theme`) |
| `cargo deny check bans licenses sources` | PASS |
| `cargo audit --ignore RUSTSEC-2026-0194 --ignore RUSTSEC-2026-0195` | PASS (exit 0; 11 allowed transitive warnings) |
| `python scripts/check_package.py --allow-dirty` | PASS (14 libraries, apps excluded) |
| `python scripts/verify_sdk.py --profile cpu --output target/cpu-evidence-20261115-final` | PASS |

**NOT_RUN:** live GPU window inspection, Windows 注音 IME manual session, physical HiDPI/touch hardware verification, screen readers, macOS/Linux local execution (pending GitHub Actions CI).
