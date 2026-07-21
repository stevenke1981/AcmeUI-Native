//! AcmeUI Native Gallery — Navigation-based demo with 8 categories,
//! component page templates, 4 reference app templates, and screenshot mode.
//!
//! Architecture:
//!
//!   ┌───────────────────────────────────────────────────────────────┐
//!   │  Application::frame()  — pipeline orchestration               │
//!   ├───────────────────────────────────────────────────────────────┤
//!   │  render/frame.rs — pipeline helpers (build_theme, ...)        │  Layer 4
//!   │  render/frame.rs — RenderCtx + standalone render functions    │   Frame Rendering
//!   ├───────────────────────────────────────────────────────────────┤
//!   │  events/ module                                              │
//!   │   ├── hit.rs       — hit_test (pure query)                   │  Layer 1
//!   │   ├── activate.rs  — ActivationCtx + handle_message          │  Layer 2
//!   │   ├── ime.rs       — compute_ime_caret_rect                  │   State Transition
//!   │   ├── dispatch.rs  — per-event-type handlers                 │  Layer 3
//!   │   │                 (each takes explicit refs)               │   Event Routing
//!   ├───────────────────────────────────────────────────────────────┤
//!   │  Gallery (this file)                                          │
//!   │   ├── event()  — match → dispatch (Layer 4)                  │  Layer 4
//!   │   ├── description / content_area  (widget tree builders)     │   Event Match
//!   │   └── page dispatcher + builders  (pages/*.rs)                │
//!   ├───────────────────────────────────────────────────────────────┤
//!   │  render/ module                                               │
//!   │   ├── content.rs   →  render_content (DFS dispatch)           │  Layer 2
//!   │   ├── hit_test.rs  →  collect_hit_regions                     │   Widget Rendering
//!   │   ├── style.rs     →  push_widget_style / shadow              │
//!   │   └── text.rs      →  add_text (shape → prepare → push)      │
//!   ├───────────────────────────────────────────────────────────────┤
//!   │  render/ module (cont.)                                       │
//!   │   ├── geometry.rs  →  quad_rect, rgba, point_in_rect          │  Layer 1
//!   │   └── layout.rs    →  extract/apply gallery layout            │   Primitives
//!   ├───────────────────────────────────────────────────────────────┤
//!   │  helpers.rs — build_sidebar/toolbar, sf, kpi_card, tree_*    │  Standalone
//!   │  types.rs   — GalleryMessage, HitRegion, Density, constants   │   Utilities
//!   └───────────────────────────────────────────────────────────────┘
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
use acme_layout::LayoutEngine;
use acme_platform::{
    Application, Clipboard, FrameContext, PlatformEvent, PlatformKey, WindowConfig, WindowId,
};
use acme_render_wgpu::Frame;
use acme_text::{FontSystem, GlyphAtlas};
use acme_textinput::TextInputState;
use acme_widgets::{
    WidgetNode, column, row, scroll_view,
};

use crate::events::{
    compute_ime_caret_rect, handle_enter_space_key, handle_general_key, handle_ime_commit,
    handle_ime_preedit, handle_pointer_moved, handle_pointer_pressed, handle_pointer_released,
    handle_scroll_event, handle_tab_key, handle_tree_arrow_key, ActivationCtx,
};
use crate::render::{
    apply_gallery_styles, build_layout_context, build_theme, collect_data_widget_hits,
    collect_hit_regions, compute_scroll_state, compute_toolbar_labels,
    extract_gallery_ids, rgba,
    RenderCtx,
    render_sidebar, render_toolbar, render_page_content, render_text_input_overlay,
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

    // ── Top-level Widget Tree Builder ────────────────────────────────────────

    fn description(&self) -> WidgetNode<GalleryMessage> {
        row()
            .key("gallery_root")
            .child(crate::helpers::build_sidebar(self.selected_category))
            .child(self.content_area())
            .build()
    }

    fn content_area(&self) -> WidgetNode<GalleryMessage> {
        let page = self.render_page();
        column()
            .key("content_area")
            .child(crate::helpers::build_toolbar(
                self.dark,
                self.density.label(),
                self.show_focus_rings,
            ))
            .child(scroll_view("content_scroll").child(page).build())
            .build()
    }

    // ── Page Dispatcher (defined in pages/mod.rs via impl Gallery) ──────────
    // render_page, component_page, build_component_page, page_section
    // are in pages/component.rs
    // Foundations category: pages/foundations.rs
    // Inputs category: pages/inputs.rs
    // etc.

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
            PlatformEvent::PointerMoved { x, y, .. } => handle_pointer_moved(
                &mut self.cursor,
                &mut self.hovered,
                x,
                y,
                &self.button_info,
                self.scroll,
                self.scroll_clip_rect,
            ),
            PlatformEvent::PointerButton { pressed, .. } => {
                if pressed {
                    handle_pointer_pressed(
                        &mut self.pressed,
                        &mut self.text_input,
                        self.text_input_rect,
                        self.cursor,
                        &self.button_info,
                        self.scroll,
                        self.scroll_clip_rect,
                        self.dark,
                        &mut self.fonts,
                        self.last_scale_factor,
                        &mut self.ime_caret_window_rect,
                    )
                } else {
                    let scroll = self.scroll;
                    let act_ctx = ActivationCtx {
                        dark: &mut self.dark,
                        density: &mut self.density,
                        show_focus_rings: &mut self.show_focus_rings,
                        selected_category: &mut self.selected_category,
                        selected_page: &mut self.selected_page,
                        scroll: &mut self.scroll,
                        nav_rail_selected: &mut self.nav_rail_selected,
                        tab_bar_selected: &mut self.tab_bar_selected,
                        tab_bar_zh_selected: &mut self.tab_bar_zh_selected,
                        tree_expanded: &mut self.tree_expanded,
                        tree_selected: &mut self.tree_selected,
                        table_sort_col: &mut self.table_sort_col,
                        table_sort_asc: &mut self.table_sort_asc,
                        table_selected_row: &mut self.table_selected_row,
                    };
                    handle_pointer_released(
                        &mut self.pressed,
                        self.cursor,
                        &self.button_info,
                        scroll,
                        self.scroll_clip_rect,
                        act_ctx,
                    )
                }
            }
            PlatformEvent::Scroll { delta_y, .. } => handle_scroll_event(
                self.selected_category,
                self.selected_page,
                self.cursor,
                self.vlist_viewport_rect,
                self.scroll,
                self.max_scroll,
                &mut self.vlist_scroll,
                &mut self.scroll,
                delta_y,
            ),
            PlatformEvent::Key {
                key: PlatformKey::Tab,
                pressed: true,
                shift,
                ..
            } => handle_tab_key(
                &mut self.focused,
                &mut self.text_input,
                self.text_input_rect,
                self.dark,
                &mut self.fonts,
                self.last_scale_factor,
                &mut self.ime_caret_window_rect,
                &self.button_info,
                shift,
            ),
            PlatformEvent::Key {
                key: PlatformKey::Enter | PlatformKey::Space,
                pressed: true,
                ..
            } => {
                if !self.text_input.focused {
                    let act_ctx = ActivationCtx {
                        dark: &mut self.dark,
                        density: &mut self.density,
                        show_focus_rings: &mut self.show_focus_rings,
                        selected_category: &mut self.selected_category,
                        selected_page: &mut self.selected_page,
                        scroll: &mut self.scroll,
                        nav_rail_selected: &mut self.nav_rail_selected,
                        tab_bar_selected: &mut self.tab_bar_selected,
                        tab_bar_zh_selected: &mut self.tab_bar_zh_selected,
                        tree_expanded: &mut self.tree_expanded,
                        tree_selected: &mut self.tree_selected,
                        table_sort_col: &mut self.table_sort_col,
                        table_sort_asc: &mut self.table_sort_asc,
                        table_selected_row: &mut self.table_selected_row,
                    };
                    handle_enter_space_key(&self.text_input, &self.button_info, self.focused, act_ctx)
                } else {
                    false
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
                handle_tree_arrow_key(key, &mut self.tree_selected, &mut self.tree_expanded)
            }
            PlatformEvent::Key {
                ref key,
                pressed,
                ctrl,
                shift,
                ref text,
                ..
            } => handle_general_key(
                &mut self.text_input,
                key,
                pressed,
                ctrl,
                shift,
                text.as_deref(),
                &self.clipboard,
                self.text_input_rect,
                self.dark,
                &mut self.fonts,
                self.last_scale_factor,
                &mut self.ime_caret_window_rect,
            ),
            PlatformEvent::ImePreedit { text, .. } => handle_ime_preedit(
                &text,
                &mut self.text_input,
                self.text_input_rect,
                self.dark,
                &mut self.fonts,
                self.last_scale_factor,
                &mut self.ime_caret_window_rect,
            ),
            PlatformEvent::ImeCommit { text, .. } => handle_ime_commit(
                &text,
                &mut self.text_input,
                &mut self.ime_text,
                self.text_input_rect,
                self.dark,
                &mut self.fonts,
                self.last_scale_factor,
                &mut self.ime_caret_window_rect,
            ),
            PlatformEvent::AccessibilityScrollIntoView { node_id, .. } => {
                // Scroll to the top of the page as a best-effort response.
                // A full implementation would resolve `node_id` against the
                // layout snapshot and scroll the nearest scroll container so
                // that the target rect becomes visible.
                self.scroll = 0.0;
                tracing::debug!(
                    "AccessibilityScrollIntoView(id={node_id}): scroll reset to 0"
                );
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

    /// Frame pipeline: build tree → theme → layout → hit-test → render layers.
    fn frame(&mut self, context: FrameContext) -> Frame {
        let width = context.logical_width;
        let height = context.logical_height;
        self.last_scale_factor = context.scale_factor;

        // ── Layer 1: Build widget tree ──
        let description = self.description();

        // ── Layer 1: Build Theme ──
        let theme = build_theme(self.dark);

        // ── Layer 2: Build layout context from theme ──
        let layout_context = build_layout_context(&theme, context.scale_factor);

        // ── Layer 2: Convert to layout tree with context ──
        let mut root = description.to_layout_with_context(NodeId::new(1), &layout_context);

        // ── Layer 2: Apply sizes, gaps, scroll flags ──
        apply_gallery_styles(&mut root, width, height);

        // ── Layer 2: Compute layout snapshot ──
        let snapshot = self
            .layout
            .compute_with_text(
                &root,
                (width, height),
                &mut self.fonts,
                context.scale_factor,
            )
            .expect("finite Gallery viewport");

        // ── Layer 3: Accessibility ──
        self.accessibility.update(&description, &snapshot);

        // ── Layer 3: Extract structural IDs ──
        let ids = extract_gallery_ids(&root);

        // ── Layer 3: Collect hit regions ──
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

        // ── Layer 3: Scroll metrics ──
        let (max_scroll, clamped_scroll, scroll_clip) =
            compute_scroll_state(&snapshot, ids.scroll_view, self.scroll);
        self.max_scroll = max_scroll;
        self.scroll = clamped_scroll;
        self.scroll_clip_rect = scroll_clip;

        // ── Layer 4: Build frame ──
        let mut frame = Frame {
            clear: rgba(theme.colors.background),
            ..Frame::default()
        };

        // ── Layer 4: Create render context (bundles all per-frame state) ──
        let toolbar_labels = compute_toolbar_labels(self.dark, self.density, self.show_focus_rings);
        let mut ctx = RenderCtx {
            frame: &mut frame,
            fonts: &mut self.fonts,
            atlas: &mut self.atlas,
            snapshot: &snapshot,
            ids: &ids,
            description: &description,
            root: &root,
            theme: &theme,
            scale: context.scale_factor,
            hovered: self.hovered,
            pressed: self.pressed,
            focused: self.focused,
            show_focus_rings: self.show_focus_rings,
            selected_category: self.selected_category,
            selected_page: self.selected_page,
            scroll: self.scroll,
            toolbar_labels: &toolbar_labels,
            text_input_rect: &mut self.text_input_rect,
            text_input: &mut self.text_input,
            ime_caret_window_rect: &mut self.ime_caret_window_rect,
            ime_text: &mut self.ime_text,
        };

        // ── Layer 4: Render sidebar ──
        render_sidebar(&mut ctx);

        // ── Layer 4: Render toolbar ──
        render_toolbar(&mut ctx);

        // ── Layer 4: Render page content ──
        render_page_content(&mut ctx);

        // ── Layer 4: Render text input overlay ──
        render_text_input_overlay(&mut ctx);

        // IME caret cache must be refreshed after text-input overlay updates
        // the text_input_rect and font system.
        self.ime_caret_window_rect = compute_ime_caret_rect(
            &self.text_input,
            self.text_input_rect,
            self.dark,
            &mut self.fonts,
            self.last_scale_factor,
        );

        frame
    }
}
