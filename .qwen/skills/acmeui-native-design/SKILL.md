---
name: acmeui-native-design
description: Reference for building UIs with the AcmeUI-Native Rust widget framework (this repository). Use when adding/modifying widgets, components, themes, or layouts in crates/acme-ui, acme-widgets, acme-theme, acme-layout; when asked to create a UI component, screen, or gallery page in this project; or when questions arise about the WidgetNode builder pattern, semantic theme tokens, theme packs, or WCAG contrast. Triggers on "AcmeUI", "acme-ui", "add a component", "new widget", "theme pack", "WidgetNode", "builder pattern" within this repo. NOT for generic Rust help or unrelated GPUI/web projects.
---

# AcmeUI-Native Design & Component Library

AcmeUI-Native is a **from-scratch native Rust UI framework** (wgpu rendering, winit
windowing, Taffy layout, cosmic-text shaping). It is **not** GPUI and must never
depend on GPUI. Everything is built on a small set of crates and a declarative
`WidgetNode<M>` tree with a fluent builder API.

## Crate architecture

| Crate | Role |
|-------|------|
| `acme-core` | `NodeId`, `WidgetKey`, events, hit-testing primitives |
| `acme-layout` | Taffy-backed `LayoutEngine`, `LayoutNode`, `LayoutStyle`, `Length`, `Edges` |
| `acme-text` | `FontSystem`, `GlyphAtlas`, text measurement/shaping |
| `acme-textinput` | Text editing buffer (cursor/selection/IME) — **never use byte offsets for cursors** |
| `acme-style` | `Style` + `Styled` trait (tailwind-style utilities) |
| `acme-theme` | `Theme`, semantic `ColorTokens`, spacing/radius/typography/shadow tokens, `packs::` brand themes |
| `acme-widgets` | Core `WidgetNode<M>` enum + primitives (Row/Column/Stack/Label/Button/Card/…) |
| `acme-ui` | High-level component library (foundations/inputs/layout/overlay/charts/desktop/browser/mobile) |
| `acme-animation` | Tween/easing engine |
| `acme-accessibility` | AccessKit tree builder |
| `acme-render-wgpu` | wgpu renderer, `Frame`, `Quad`, `TextRun` |
| `acme-platform` | winit `Application` trait, window/event loop |
| `acme-devtools` | Frame metrics, widget tree dump, layout inspector |

Apps: `apps/gallery`, `apps/acme-gallery` (showcase), `apps/playground`, `apps/benchmark`.

## The WidgetNode builder pattern

`WidgetNode<M>` is the core enum (`M` = app message type). Primitives live in
`acme-widgets`; re-exported through `acme-ui`. Build trees with fluent builders,
then `.build()` or `.into()`:

```rust
use acme_widgets::{button, card, column, row, label, separator, WidgetNode};

fn profile_card<M: Clone + 'static>() -> WidgetNode<M> {
    card::<M>()
        .variant(acme_widgets::CardVariant::Elevated)
        .padding(16.0)
        .gap(8.0)
        .child(label("Ada Lovelace"))
        .child(separator())
        .child(
            row::<M>()
                .gap(12.0)
                .child(label("Engineer"))
                .child(button("edit", "Edit").primary()),
        )
        .build()
}
```

Container builders (`row`/`column`/`stack`/`card`) support: `.key()`, `.child()`,
`.gap()`, `.padding()`, `.width()`, `.height()`, `.size()`, `.on_click(msg)`,
`.variant()` (card), `.build()`. Buttons: `.primary()`, `.variant()`, `.size()`,
`.disabled()`, `.loading()`, `.full_width()`, `.on_click(msg) -> WidgetNode`.

## Component library inventory (`crates/acme-ui/src/`)

- **foundations/** — accordion, alert, aspect_ratio, avatar, badge, banner, calendar,
  callout, chip, code, collapsible, copy_button, data_list, descriptions, divider,
  drop_zone, empty_state, flex, hero, icon, indicator, kbd, kbd_combo, link, list,
  live_region, media_card, metric_card, progress, progress_ring, qr_code, quote,
  result, skeleton, skeleton_shape, spinner, statistic, status_dot, tag, timeline,
  typography, visually_hidden, watermark
- **inputs/** — autocomplete, button_group, cascader, checkbox, checkbox_cards,
  color_picker, combobox, date_picker, date_range_picker, file_upload, form_field,
  icon_button, input_group, input_otp, mentions, multi_select, number_input,
  password_input, pin_input, radio, radio_cards, range_slider, rating, search_input,
  segmented_control, select, slider, slider_marks, switch, tag_input, text_field,
  textarea, time_picker, toggle_button, toggle_group, transfer, tree_select
- **layout/** — affix, anchor, app_bar, bottom_navigation, breadcrumb, form, grid,
  image_list, inset, masonry, navigation_menu, page_header, pagination, paper,
  resizable, scroll_area, section, settings_page, sidebar, split_panel, status_bar,
  stepper, tabs, toolbar
- **overlay/** — about_dialog, backdrop, command_palette, confirm_dialog, context_menu,
  drawer, dropdown_menu, float_button, fullscreen, hover_card, modal, notification,
  speed_dial, toast, tour
- **charts/** — area, bar, box_plot, bubble, candlestick, donut, funnel, gauge, heatmap,
  histogram, line, parallel_coordinates, pie, radar, radial_bar, scatter, sparkline,
  timeline, treemap, waterfall
- **desktop/** — command_bar, dock, image_view, markdown_view, menubar, navigation_view,
  property_grid, resize_handle, sidenav, status_tray, taskbar, title_bar, window_controls
- **browser/** — audio_player, carousel, code_viewer, document_viewer, embed,
  image_gallery, lightbox, map_view, media_grid, pdf_viewer, rich_text, url_preview,
  video_player, web_frame, web_preview, zoom_view
- **mobile/** — action_sheet, bottom_nav, bottom_sheet, pull_to_refresh, search_bar,
  mobile_{action,avatar,banner,button,card,chip,list_item,loader,notification,
  progress,search,segmented,sheet_handle,stepper,toggle}

The library aligns with shadcn/ui, Ant Design, MUI, Radix UI, and absorbs
gpui-component strengths (see `docs/status/todos.md` P6–P10).

## Theme system

`acme_theme::Theme` holds semantic tokens — **never hardcode colors in widgets**;
consume `theme.colors.<semantic_field>`. Key fields: `background`/`foreground`,
`surface`/`surface_foreground`, `primary`/`primary_foreground`, `secondary`, `accent`,
`muted`, `border`, `ring`, `success`/`warning`/`danger`/`info` (+ `_soft` variants),
`surface_elevated`/`overlay`/`tooltip`, hover/pressed states, `disabled_bg`/`disabled_text`.

```rust
use acme_theme::{Theme, ThemeMode};
let theme = Theme::light();           // or Theme::dark()
let bg = theme.colors.background;     // semantic, renderer-resolved
```

### Brand theme packs (`acme_theme::packs`)

10 curated packs, each `ThemePack` with light + dark: **apple, windows10, windows11,
ubuntu, material, nord, dracula, solarized, gruvbox, one-dark**.

```rust
use acme_theme::packs::{theme_by_name, available_themes};
let theme = theme_by_name("apple", ThemeMode::Dark).unwrap();
for name in available_themes() { /* "apple", "windows10", … */ }
```

Build a pack from a palette via `Theme::from_colors(mode, colors, shadows)` (shares
structural tokens), then tweak `theme.radii`/`spacing` for brand geometry.

### WCAG contrast

```rust
use acme_theme::{contrast_ratio, ContrastLevel};
let ratio = theme.contrast(theme.colors.foreground, theme.colors.background);
let level = ContrastLevel::from_ratio(ratio); // Aaa ≥7, Aa ≥4.5, AaLarge ≥3, Fail
assert!(theme.meets_wcag_aa());               // all key text pairs ≥ 4.5:1
let report = theme.wcag_report();             // Vec<(pair_name, ratio, level)>
```

## Adding a new component (the established pattern)

1. Create `crates/acme-ui/src/<module>/<name>.rs`.
2. Define a builder struct `XxxBuilder<M>` with public config fields. If `M` is unused
   in fields, add `_phantom: std::marker::PhantomData<M>` (and init it).
3. Provide a constructor `pub fn xxx<M: Clone + 'static>(...) -> XxxBuilder<M>`.
4. Add fluent `impl<M: Clone + 'static> XxxBuilder<M>` setters (`mut self -> Self`).
5. Implement `impl<M: Clone + 'static> From<XxxBuilder<M>> for WidgetNode<M>` composing
   existing primitives (`crate::row`, `crate::column`, `crate::card`, `crate::label`,
   `crate::button`, `crate::icon`, …).
6. Register in the module's `mod.rs`: `pub mod xxx;` **and** `pub use xxx::*;`.
7. Add `#[cfg(test)] mod tests` with a local `enum Msg`, asserting the produced
   `WidgetNode` variant, child counts, and builder defaults.
8. Run `cargo test -p acme-ui --all-features -- <name>`.

Reference implementations: `mobile/mobile_button.rs` (button wrap),
`mobile/bottom_nav.rs` (multi-item builder), `foundations/progress.rs` (custom layout),
`packs/apple.rs` (theme pack).

## Hard rules (from AGENTS.md)

- **Never add GPUI.** This is an independent framework.
- **Never hardcode theme colors inside widgets** — use semantic `theme.colors.*`.
- **Never use byte offsets for text cursors** — use char/grapheme indices via acme-textinput.
- **Never expose platform-specific types publicly** — keep winit/wgpu types internal.
- **Never claim Traditional Chinese IME works without manual validation.**
- **Never use `cargo clean` as a routine fix.**

## Verification commands

```bash
cargo check --workspace                       # fast full compile
cargo test -p acme-ui --all-features          # component tests
cargo test -p acme-theme                      # theme + WCAG + packs tests
cargo check -p acme-ui-gallery                # gallery app
```

## Gallery app notes (`apps/acme-gallery`)

Single-file `main.rs` with a manual wgpu render path. The toolbar has a light/dark
toggle, a **theme-pack cycle button** (🎨, shows current pack + WCAG ✓AA/⚠AA), and an
info button. Selection persists to `acme-gallery-theme.conf` (gitignored) via
`load_theme_pref()`/`save_theme_pref()`. **Caution:** the manual renderer uses hardcoded
button-index ranges (sidebar 0–6, toolbar 7–9, content 10+); adding/removing toolbar
buttons requires updating `toolbar_buttons: [NodeId; N]`, `extract_gallery_ids`,
`tb_labels`, and the content `btn_idx` base in lockstep.
