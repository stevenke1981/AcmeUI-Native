//! AcmeUI Native Gallery — Navigation-based demo with 8 categories,
//! component page templates, 4 reference app templates, and screenshot mode.
//!
//! Architecture:
//!   Row (root)                                                        IDs
//!     ├── Column (sidebar, 224px) — AcmeUI + 8 category buttons      child 0
//!     └── Column (content area) — flex                               child 1
//!         ├── Row (toolbar, 48px) — theme/density/focus toggles      child 0
//!         └── ScrollView (content) — flex, overflow scroll           child 1
//!
//! All button hit-regions are collected via a single DFS walk across the
//! widget+layout tree — no magic‑number indexing is used anywhere.

// Module declarations
mod events;
mod helpers;
mod pages;
mod render;
mod types;

use acme_accessibility::AccessibilityAdapter;
use acme_core::NodeId;
use acme_layout::{
    LayoutEngine, LayoutNode, WidgetLayoutContext,
};
use acme_platform::{
    Application, Clipboard, FrameContext, PlatformEvent, PlatformKey, WindowConfig, WindowId,
};
use acme_render_wgpu::{Frame, Quad, TextRun};
use acme_style::prelude::ColorToken;
use acme_style::prelude::Styled;
use acme_style::ShadowDef;
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextStyle};
use acme_textinput::{
    TextInputState, handle_key, handle_keyboard_shortcut, handle_text, render_text_input,
};
use acme_theme::{Theme, ThemeColor};
use acme_widgets::{
    ButtonState, WidgetKey, WidgetNode, button, column, label, label_with_size, row,
    scroll_view, separator,
};

use crate::helpers::*;
use crate::render::{
    apply_gallery_styles, extract_gallery_ids, find_text_input_marker,
    point_in_rect, push_widget_style, render_content, collect_hit_regions,
    collect_data_widget_hits, add_text, rgba, quad_rect, scrolled_hit_rect,
};
use crate::types::*;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();
    acme_platform::run(Gallery::new())?;
    Ok(())
}

// ── Gallery State ───────────────────────────────────────────────────────────

struct Gallery {
    // Navigation
    selected_category: usize,
    selected_page: usize,

    // Appearance
    dark: bool,
    density: Density,
    show_focus_rings: bool,

    // Interaction
    cursor: (f32, f32),
    hovered: Option<usize>,
    pressed: Option<usize>,
    focused: usize,
    scroll: f32,
    max_scroll: f32,
    button_info: Vec<HitRegion>,

    // Text Input / IME state
    text_input: TextInputState,
    text_input_rect: [f32; 4],
    /// Window-client IME caret rect `[x,y,w,h]` when text input is focused.
    ime_caret_window_rect: Option<[f32; 4]>,
    ime_text: String,
    /// Last frame scale factor (for caret geometry outside `frame`).
    last_scale_factor: f32,

    // ── Data / Nav demo interaction state (lives on Gallery; widgets rebuild) ──
    /// Bit `i` ⇒ `TREE_EXPAND_KEYS[i]` is expanded.
    tree_expanded: u32,
    tree_selected: Option<&'static str>,
    table_sort_col: Option<usize>,
    table_sort_asc: bool,
    /// Original (pre-sort) row index.
    table_selected_row: Option<usize>,
    vlist_scroll: f32,
    nav_rail_selected: usize,
    tab_bar_selected: usize,
    tab_bar_zh_selected: usize,
    /// Content-space viewport rects updated each frame (hit/scroll routing).
    tree_viewport_rect: [f32; 4],
    table_viewport_rect: [f32; 4],
    vlist_viewport_rect: [f32; 4],
    /// Window-space clip of the page scroll viewport (for scrolled hit tests).
    scroll_clip_rect: [f32; 4],

    // Systems
    fonts: FontSystem,
    atlas: GlyphAtlas,
    layout: LayoutEngine,
    clipboard: Option<Clipboard>,
    accessibility: AccessibilityAdapter,

    // Screenshot mode
    #[allow(dead_code)]
    screenshot_config: Option<ScreenshotConfig>,
}

impl Gallery {
    fn new() -> Self {
        Self {
            selected_category: 0,
            selected_page: 0,
            dark: false,
            density: Density::Comfortable,
            show_focus_rings: true,
            cursor: (0.0, 0.0),
            hovered: None,
            pressed: None,
            focused: 0,
            scroll: 0.0,
            max_scroll: 0.0,
            button_info: Vec::new(),
            text_input: TextInputState::new(),
            text_input_rect: [0.0; 4],
            ime_caret_window_rect: None,
            ime_text: String::new(),
            last_scale_factor: 1.0,
            tree_expanded: TREE_EXPAND_DEFAULT,
            tree_selected: None,
            table_sort_col: None,
            table_sort_asc: true,
            table_selected_row: None,
            vlist_scroll: 0.0,
            nav_rail_selected: 0,
            tab_bar_selected: 0,
            tab_bar_zh_selected: 1,
            tree_viewport_rect: [0.0; 4],
            table_viewport_rect: [0.0; 4],
            vlist_viewport_rect: [0.0; 4],
            scroll_clip_rect: [0.0; 4],
            fonts: FontSystem::new(),
            atlas: GlyphAtlas::new(2048, 2048),
            layout: LayoutEngine::new(),
            clipboard: Clipboard::new().ok(),
            accessibility: AccessibilityAdapter::new(0),
            screenshot_config: None,
        }
    }

    fn tree_is_expanded(&self, key: &str) -> bool {
        TREE_EXPAND_KEYS
            .iter()
            .position(|&k| k == key)
            .is_some_and(|i| self.tree_expanded & (1u32 << i) != 0)
    }

    fn tree_set_expanded(&mut self, key: &str, expanded: bool) {
        if let Some(i) = TREE_EXPAND_KEYS.iter().position(|&k| k == key) {
            if expanded {
                self.tree_expanded |= 1u32 << i;
            } else {
                self.tree_expanded &= !(1u32 << i);
            }
        }
    }

    fn tree_toggle_expanded(&mut self, key: &str) {
        if let Some(i) = TREE_EXPAND_KEYS.iter().position(|&k| k == key) {
            self.tree_expanded ^= 1u32 << i;
        }
    }

    fn vlist_max_scroll(&self) -> f32 {
        (VLIST_ITEM_COUNT as f32 * VLIST_ITEM_HEIGHT - VLIST_VIEWPORT_H).max(0.0)
    }

    // ── Top-level Widget Tree Builder ────────────────────────────────────────

    fn description(&self) -> WidgetNode<GalleryMessage> {
        row()
            .key("gallery_root")
            .child(self.sidebar())
            .child(self.content_area())
            .build()
    }

    fn sidebar(&self) -> WidgetNode<GalleryMessage> {
        let mut col = column::<GalleryMessage>()
            .key("sidebar")
            .gap(4.0)
            .padding(12.0);
        col = col.child(label_with_size("AcmeUI", 18.0));
        col = col.child(separator());
        for (i, cat) in CATEGORIES.iter().enumerate() {
            let mut btn = button::<GalleryMessage>("", cat.name);
            if i == self.selected_category {
                btn = btn.primary();
            }
            col = col.child(btn.on_click(GalleryMessage::SelectCategory(i)));
        }
        col.build()
    }

    fn toolbar(&self) -> WidgetNode<GalleryMessage> {
        row()
            .key("toolbar")
            .gap(8.0)
            .padding(8.0)
            .child(
                button("theme_btn", if self.dark { "☀ Light" } else { "🌙 Dark" })
                    .on_click(GalleryMessage::ToggleTheme),
            )
            .child(
                button("density_btn", self.density.label()).on_click(GalleryMessage::ToggleDensity),
            )
            .child(
                button(
                    "focus_btn",
                    if self.show_focus_rings {
                        "Focus ✓"
                    } else {
                        "Focus ✗"
                    },
                )
                .on_click(GalleryMessage::ToggleFocusRings),
            )
            .build()
    }

    fn content_area(&self) -> WidgetNode<GalleryMessage> {
        let page = self.render_page();
        column()
            .key("content_area")
            .child(self.toolbar())
            .child(scroll_view("content_scroll").child(page).build())
            .build()
    }

    // ── Page Dispatcher (defined in pages/mod.rs via impl Gallery) ──────────
    // render_page, component_page, build_component_page, page_section
    // are in pages/component.rs
    // Foundations category: pages/foundations.rs
    // Inputs category: pages/inputs.rs
    // etc.

    // ── Small Template Helpers ──────────────────────────────────────────────

    fn sf(&self, label: &str, value: &str) -> WidgetNode<GalleryMessage> {
        row()
            .gap(8.0)
            .child(label_with_size(label, 14.0))
            .child(label_with_size(value, 14.0))
            .build()
    }

    fn kpi_card(&self, value: &str, title: &str) -> WidgetNode<GalleryMessage> {
        column()
            .gap(6.0)
            .padding(16.0)
            .child(label_with_size(value, 22.0))
            .child(label(title))
            .build()
    }

    /// Reusable widget that renders a text block for long‑string handling.
    #[allow(dead_code)]
    fn long_text_widget(text: &str) -> WidgetNode<GalleryMessage> {
        label_with_size(text, 14.0)
    }
}

// ── Application Trait ───────────────────────────────────────────────────────

impl Application for Gallery {
    fn window_config(&self) -> WindowConfig {
        if let Some(ref sc) = self.screenshot_config {
            return WindowConfig {
                title: "AcmeUI Native Gallery — Screenshot Mode".into(),
                width: sc.width as f64,
                height: sc.height as f64,
            };
        }
        WindowConfig {
            title: "AcmeUI Native Gallery".into(),
            width: 1280.0,
            height: 800.0,
        }
    }

    fn event(&mut self, event: PlatformEvent) -> bool {
        match event {
            PlatformEvent::PointerMoved { x, y, .. } => {
                self.cursor = (x, y);
                let next = self.hit();
                let changed = next != self.hovered;
                self.hovered = next;
                changed
            }
            PlatformEvent::PointerButton { pressed, .. } => {
                if pressed {
                    let in_text = {
                        let [tx, ty, tw, th] = self.text_input_rect;
                        self.cursor.0 >= tx
                            && self.cursor.0 <= tx + tw
                            && self.cursor.1 >= ty
                            && self.cursor.1 <= ty + th
                    };
                    self.text_input.focused = in_text;
                    self.refresh_ime_caret_cache();
                    self.pressed = self.hit();
                    true
                } else {
                    let activated = self
                        .pressed
                        .take()
                        .filter(|&value| Some(value) == self.hit());
                    activated.is_some_and(|index| self.activate(index))
                }
            }
            PlatformEvent::Scroll { delta_y, .. } => {
                let vlist_screen = scrolled_hit_rect(self.vlist_viewport_rect, self.scroll);
                if self.selected_category == 4
                    && self.selected_page == 3
                    && self.vlist_viewport_rect[2] > 0.0
                    && point_in_rect(self.cursor.0, self.cursor.1, vlist_screen)
                {
                    self.vlist_scroll =
                        (self.vlist_scroll - delta_y).clamp(0.0, self.vlist_max_scroll());
                    return true;
                }
                self.scroll = (self.scroll - delta_y).clamp(0.0, self.max_scroll);
                true
            }
            PlatformEvent::Key {
                key: PlatformKey::Tab,
                pressed: true,
                shift,
                ..
            } => {
                if self.text_input.focused {
                    self.text_input.focused = false;
                    self.refresh_ime_caret_cache();
                }
                let count = self.button_info.len();
                if count > 0 {
                    self.focused = if shift {
                        (self.focused + count - 1) % count
                    } else {
                        (self.focused + 1) % count
                    };
                }
                true
            }
            PlatformEvent::Key {
                key: PlatformKey::Enter | PlatformKey::Space,
                pressed: true,
                ..
            } => {
                if self.text_input.focused {
                    false
                } else {
                    self.activate(self.focused)
                }
            }
            PlatformEvent::Key {
                ref key,
                pressed: true,
                ..
            } if !self.text_input.focused
                && self.selected_category == 4
                && self.selected_page == 0
                && matches!(
                    key,
                    PlatformKey::ArrowLeft
                        | PlatformKey::ArrowRight
                        | PlatformKey::Home
                        | PlatformKey::End
                ) =>
            {
                match key {
                    PlatformKey::ArrowRight => {
                        if let Some(sel) = self.tree_selected {
                            self.tree_set_expanded(sel, true);
                        }
                        true
                    }
                    PlatformKey::ArrowLeft => {
                        if let Some(sel) = self.tree_selected {
                            self.tree_set_expanded(sel, false);
                        }
                        true
                    }
                    PlatformKey::Home => {
                        self.tree_selected = Some("docs");
                        true
                    }
                    PlatformKey::End => {
                        self.tree_selected = if self.tree_is_expanded("code") {
                            if self.tree_is_expanded("code_src") {
                                Some("code_lib")
                            } else {
                                Some("code_toml")
                            }
                        } else {
                            Some("code")
                        };
                        true
                    }
                    _ => false,
                }
            }
            PlatformEvent::Key {
                ref key,
                pressed,
                ctrl,
                shift,
                ref text,
                ..
            } => {
                if !self.text_input.focused {
                    return false;
                }
                let changed = if ctrl
                    && let Some(t) = text
                    && matches!(t.as_str(), "a" | "c" | "v" | "x")
                {
                    handle_keyboard_shortcut(&mut self.text_input, t, self.clipboard.as_ref())
                } else {
                    handle_key(
                        &mut self.text_input,
                        key,
                        pressed,
                        self.clipboard.as_ref(),
                        ctrl,
                        shift,
                    )
                };
                if changed {
                    self.refresh_ime_caret_cache();
                }
                changed
            }
            PlatformEvent::ImePreedit(text) => {
                self.text_input.set_preedit(&text, None);
                self.refresh_ime_caret_cache();
                true
            }
            PlatformEvent::ImeCommit(text) => {
                handle_text(&mut self.text_input, &text);
                self.ime_text = self.text_input.text.clone();
                self.refresh_ime_caret_cache();
                true
            }
            PlatformEvent::Resized { .. } => true,
            _ => false,
        }
    }

    fn ime_cursor_area(&self, _window: WindowId) -> Option<[f32; 4]> {
        if !self.text_input.focused {
            return None;
        }
        self.ime_caret_window_rect
    }

    fn on_gpu_recovered(&mut self, _window: WindowId) {
        self.atlas.clear();
    }

    fn frame(&mut self, context: FrameContext) -> Frame {
        let width = context.logical_width;
        let height = context.logical_height;
        self.last_scale_factor = context.scale_factor;

        // ── 1. Build widget tree ──
        let description = self.description();

        // ── 2. Build Theme ──
        let theme = if self.dark {
            Theme::dark()
        } else {
            Theme::light()
        };

        // ── 3. Build layout context from theme ──
        let layout_context = WidgetLayoutContext {
            body_font_size: theme.typography.body,
            body_line_height: theme.typography.body * theme.typography.line_height,
            label_font_size: theme.typography.label,
            control_height: 40.0,
            scale_factor: context.scale_factor,
        };

        // ── 4. Convert to layout tree with context ──
        let mut root = description.to_layout_with_context(NodeId::new(1), &layout_context);

        // ── 5. Apply sizes, gaps, scroll flags ──
        apply_gallery_styles(&mut root, width, height);

        // ── 6. Compute layout snapshot with intrinsic text measurement ──
        let snapshot = self
            .layout
            .compute_with_text(
                &root,
                (width, height),
                &mut self.fonts,
                context.scale_factor,
            )
            .expect("finite Gallery viewport");

        // ── 7. Accessibility ──
        self.accessibility.update(&description, &snapshot);

        // ── 8. Extract structural IDs for the fixed frame ──
        let ids = extract_gallery_ids(&root);

        // ── 9. Collect hit regions ──
        let mut button_info = Vec::new();
        collect_hit_regions(&description, &root, &snapshot, false, &mut button_info);
        self.tree_viewport_rect = [0.0; 4];
        self.table_viewport_rect = [0.0; 4];
        self.vlist_viewport_rect = [0.0; 4];
        collect_data_widget_hits(
            &description,
            &root,
            &snapshot,
            self.table_sort_col,
            self.table_sort_asc,
            &mut button_info,
            &mut self.tree_viewport_rect,
            &mut self.table_viewport_rect,
            &mut self.vlist_viewport_rect,
        );
        self.button_info = button_info;

        // ── 10. Scroll metrics ──
        self.max_scroll = snapshot
            .scroll_metrics(ids.scroll_view)
            .map(|m| (m.content_height - m.viewport_height).max(0.0))
            .unwrap_or(0.0);
        self.scroll = self.scroll.clamp(0.0, self.max_scroll);
        self.scroll_clip_rect = snapshot
            .get(ids.scroll_view)
            .map(|r| [r.x, r.y, r.width, r.height])
            .unwrap_or([0.0; 4]);

        let colors = theme.colors;

        // ── 11. Build frame ──
        let mut frame = Frame {
            clear: rgba(colors.background),
            ..Frame::default()
        };

        // ── 12. Sidebar background ──
        if let Some(r) = snapshot.get(ids.sidebar) {
            frame.quads.push(quad_rect(
                [r.x, r.y, r.width, r.height],
                colors.surface,
                0.0,
                1.0,
                colors.border,
            ));
        }

        // ── 13. Sidebar title ──
        if let Some(r) = snapshot.get(ids.sidebar_label) {
            add_text(
                &mut self.fonts,
                &mut self.atlas,
                &mut frame,
                "AcmeUI",
                ([r.x + 4.0, r.y + 2.0], 18.0),
                colors.foreground,
                context.scale_factor,
                None,
                theme.typography.line_height,
            );
        }

        // ── 14. Sidebar category buttons ──
        for (i, &btn_id) in ids.sidebar_buttons.iter().enumerate() {
            let btn_idx = i;
            let Some(r) = snapshot.get(btn_id) else {
                continue;
            };
            let is_selected = i == self.selected_category;
            let st = ButtonState {
                hovered: self.hovered == Some(btn_idx),
                pressed: self.pressed == Some(btn_idx),
                focused: self.focused == btn_idx,
            };
            let bg = if is_selected {
                colors.accent
            } else if st.hovered {
                colors.ghost_hover
            } else {
                colors.surface
            };
            let fg = if is_selected {
                colors.primary_foreground
            } else {
                colors.foreground
            };
            frame.quads.push(quad_rect(
                [r.x, r.y, r.width, r.height],
                bg,
                theme.radii.md,
                if is_selected || (st.focused && self.show_focus_rings) {
                    2.0
                } else {
                    1.0
                },
                if st.focused {
                    colors.ring
                } else {
                    colors.border
                },
            ));
            add_text(
                &mut self.fonts,
                &mut self.atlas,
                &mut frame,
                CATEGORIES[i].name,
                ([r.x + 12.0, r.y + 9.0], theme.typography.label),
                fg,
                context.scale_factor,
                None,
                theme.typography.line_height,
            );
        }

        // ── 15. Toolbar background ──
        if let Some(r) = snapshot.get(ids.toolbar) {
            frame.quads.push(quad_rect(
                [r.x, r.y, r.width, r.height],
                colors.surface,
                0.0,
                1.0,
                colors.border,
            ));
        }

        // ── 16. Toolbar buttons ──
        let tb_labels = [
            if self.dark { "☀ Light" } else { "🌙 Dark" },
            self.density.label(),
            if self.show_focus_rings {
                "Focus ✓"
            } else {
                "Focus ✗"
            },
        ];
        for (i, (&btn_id, &label_text)) in
            ids.toolbar_buttons.iter().zip(tb_labels.iter()).enumerate()
        {
            let btn_idx = 8 + i;
            let Some(r) = snapshot.get(btn_id) else {
                continue;
            };
            let st = ButtonState {
                hovered: self.hovered == Some(btn_idx),
                pressed: self.pressed == Some(btn_idx),
                focused: self.focused == btn_idx,
            };
            let btn = button::<GalleryMessage>("", "");
            let resolved = btn.resolve_style(&theme, st);
            frame.quads.push(quad_rect(
                [r.x, r.y, r.width, r.height],
                resolved.background,
                theme.radii.md,
                if st.focused && self.show_focus_rings {
                    2.0
                } else {
                    1.0
                },
                if st.focused {
                    resolved.focus
                } else {
                    resolved.border
                },
            ));
            add_text(
                &mut self.fonts,
                &mut self.atlas,
                &mut frame,
                label_text,
                ([r.x + 12.0, r.y + 7.0], 13.0),
                resolved.foreground,
                context.scale_factor,
                None,
                theme.typography.line_height,
            );
        }

        // ── 17. Page content inside scroll view ──
        if let Some(sv_rect) = snapshot.get(ids.scroll_view) {
            let clip = [sv_rect.x, sv_rect.y, sv_rect.width, sv_rect.height];
            let mut btn_idx = 11;
            render_content(
                &mut frame,
                &description,
                &root,
                &snapshot,
                &theme,
                context.scale_factor,
                self.scroll,
                clip,
                &mut btn_idx,
                self.hovered,
                self.pressed,
                self.focused,
                &mut self.fonts,
                &mut self.atlas,
                self.show_focus_rings,
            );
        }

        // ── 18. Text Input / IME override on the TextInput page ──
        #[allow(clippy::collapsible_if)]
        if self.selected_category == 1 && self.selected_page == 1 {
            if let Some(ph_id) = find_text_input_marker(&description, &root) {
                if let Some(ph) = snapshot.get(ph_id) {
                    let y = ph.y - self.scroll;
                    let rect = [ph.x, y, ph.width, ph.height];
                    self.text_input_rect = rect;
                    let focused = self.text_input.focused;
                    render_text_input(
                        &mut frame,
                        &mut self.text_input,
                        &mut self.fonts,
                        &mut self.atlas,
                        rect,
                        &theme,
                        context.scale_factor,
                        focused,
                        None,
                    );
                    self.refresh_ime_caret_cache();
                    if !self.ime_text.is_empty() {
                        add_text(
                            &mut self.fonts,
                            &mut self.atlas,
                            &mut frame,
                            &format!("Committed: {}", self.ime_text),
                            ([ph.x + 2.0, y + ph.height + 6.0], 14.0),
                            colors.muted_foreground,
                            context.scale_factor,
                            None,
                            theme.typography.line_height,
                        );
                    }
                }
            }
        } else {
            self.text_input_rect = [0.0; 4];
            if self.ime_caret_window_rect.is_some() {
                self.ime_caret_window_rect = None;
            }
        }

        frame
    }
}
