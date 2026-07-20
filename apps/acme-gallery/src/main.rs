//! AcmeUI Gallery — showcase app for acme-ui components including Charts.
//!
//! Architecture:
//!   Row (root)
//!     ├── Column (sidebar, 200px) — AcmeUI + category buttons
//!     └── Column (content area) — flex
//!         ├── Row (toolbar, 40px) — theme toggle
//!         └── ScrollView (content) — flex, overflow scroll

use acme_core::NodeId;
use acme_layout::{LayoutEngine, LayoutNode, LayoutStyle, Length, Overflow, WidgetLayoutContext};
use acme_platform::{Application, FrameContext, PlatformEvent, WindowConfig, WindowId};
use acme_render_wgpu::{Frame, Quad, TextRun};
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextStyle};
use acme_theme::{Theme, ThemeColor};
use acme_ui::charts::{
    area_chart, bar_chart, gauge, line_chart, pie_chart, sparkline, BarEntry, ChartPoint, PieSlice,
};
use acme_widgets::{
    button, column, label, label_with_size, row, scroll_view, separator, ButtonState, CardVariant,
    WidgetNode,
};

fn main() -> Result<(), acme_platform::PlatformError> {
    acme_platform::run(Gallery::new())
}

// ── Constants ────────────────────────────────────────────────────────────────

const SIDEBAR_WIDTH: f32 = 200.0;
const TOOLBAR_HEIGHT: f32 = 40.0;

struct CategoryInfo {
    name: &'static str,
    #[allow(dead_code)]
    pages: &'static [&'static str],
}

const CATEGORIES: &[CategoryInfo] = &[
    CategoryInfo {
        name: "Foundations",
        pages: &["Typography", "Colors"],
    },
    CategoryInfo {
        name: "Inputs",
        pages: &["Button", "Variants"],
    },
    CategoryInfo {
        name: "Layout",
        pages: &["Row & Column", "Scroll"],
    },
    CategoryInfo {
        name: "Overlay",
        pages: &["Tooltip", "Menu"],
    },
    CategoryInfo {
        name: "Data",
        pages: &["Table", "Tree"],
    },
    CategoryInfo {
        name: "Charts",
        pages: &["All Charts"],
    },
];

// ── Messages ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
enum GalleryMessage {
    SelectCategory(usize),
    ToggleTheme,
    DpiInfo,
}

// ── Hit regions ──────────────────────────────────────────────────────────────

struct HitRegion {
    rect: [f32; 4],
    message: GalleryMessage,
    scrolled: bool,
}

// ── State ────────────────────────────────────────────────────────────────────

struct Gallery {
    selected_category: usize,
    dark: bool,
    cursor: (f32, f32),
    hovered: Option<usize>,
    pressed: Option<usize>,
    scroll: f32,
    max_scroll: f32,
    button_info: Vec<HitRegion>,

    fonts: FontSystem,
    atlas: GlyphAtlas,
    layout: LayoutEngine,
}

impl Gallery {
    fn new() -> Self {
        Self {
            selected_category: 5, // Start on Charts page
            dark: false,
            cursor: (0.0, 0.0),
            hovered: None,
            pressed: None,
            scroll: 0.0,
            max_scroll: 0.0,
            button_info: Vec::new(),
            fonts: FontSystem::new(),
            atlas: GlyphAtlas::new(2048, 2048),
            layout: LayoutEngine::new(),
        }
    }

    // ── Widget Tree ──────────────────────────────────────────────────────────

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
            .child(button("info_btn", "ℹ Info").on_click(GalleryMessage::DpiInfo))
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

    fn render_page(&self) -> WidgetNode<GalleryMessage> {
        match self.selected_category {
            0 => self.foundations_page(),
            1 => self.inputs_page(),
            2 => self.layout_page(),
            3 => self.overlay_page(),
            4 => self.data_page(),
            5 => self.charts_page(),
            _ => label("Unknown category"),
        }
    }

    fn foundations_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(16.0)
            .padding(24.0)
            .child(label_with_size("Foundations", 24.0))
            .child(separator())
            .child(
                self.page_section(
                    "Typography",
                    column()
                        .gap(6.0)
                        .child(label_with_size("Heading 24px", 24.0))
                        .child(label_with_size("Subheading 18px", 18.0))
                        .child(label_with_size("Body 16px (default)", 16.0))
                        .child(label_with_size("Caption 13px", 13.0))
                        .child(label_with_size("Small 12px", 12.0))
                        .build(),
                ),
            )
            .child(
                self.page_section(
                    "Colors",
                    column()
                        .gap(6.0)
                        .child(label("Theme colors are applied at render time."))
                        .child(label("Toggle dark/light via toolbar."))
                        .build(),
                ),
            )
            .build()
    }

    fn inputs_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(16.0)
            .padding(24.0)
            .child(label_with_size("Inputs", 24.0))
            .child(separator())
            .child(
                self.page_section(
                    "Button",
                    column()
                        .gap(8.0)
                        .child(label("Standard button component:"))
                        .child(button("demo_btn", "Click Me").on_click(GalleryMessage::DpiInfo))
                        .child(
                            button("demo_primary", "Primary")
                                .primary()
                                .on_click(GalleryMessage::DpiInfo),
                        )
                        .build(),
                ),
            )
            .child(
                self.page_section(
                    "Variants",
                    column()
                        .gap(8.0)
                        .child(button("v1", "Default").on_click(GalleryMessage::DpiInfo))
                        .child(
                            button("v2", "Primary")
                                .primary()
                                .on_click(GalleryMessage::DpiInfo),
                        )
                        .build(),
                ),
            )
            .build()
    }

    fn layout_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(16.0)
            .padding(24.0)
            .child(label_with_size("Layout", 24.0))
            .child(separator())
            .child(
                self.page_section(
                    "Row & Column",
                    column()
                        .gap(8.0)
                        .child(label("Row (horizontal):"))
                        .child(
                            row()
                                .gap(8.0)
                                .child(label("[A]"))
                                .child(label("[B]"))
                                .child(label("[C]"))
                                .build(),
                        )
                        .child(label("Column (vertical):"))
                        .child(
                            column()
                                .gap(4.0)
                                .child(label("Item 1"))
                                .child(label("Item 2"))
                                .child(label("Item 3"))
                                .build(),
                        )
                        .build(),
                ),
            )
            .child(
                self.page_section(
                    "Scroll",
                    column()
                        .gap(4.0)
                        .child(label("The page you're viewing is inside a ScrollView."))
                        .child(label("Try scrolling to see all content."))
                        .build(),
                ),
            )
            .build()
    }

    fn overlay_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(16.0)
            .padding(24.0)
            .child(label_with_size("Overlay", 24.0))
            .child(separator())
            .child(
                self.page_section(
                    "Tooltip",
                    column()
                        .gap(6.0)
                        .child(label("Tooltip shows on hover (requires runtime)."))
                        .child(label("Widget definition: tooltip(key, anchor, text)"))
                        .build(),
                ),
            )
            .child(
                self.page_section(
                    "Menu",
                    column()
                        .gap(6.0)
                        .child(label("Dropdown menu with items."))
                        .child(label("menu(key).item(menu_item(id, label)).build()"))
                        .build(),
                ),
            )
            .build()
    }

    fn data_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(16.0)
            .padding(24.0)
            .child(label_with_size("Data", 24.0))
            .child(separator())
            .child(
                self.page_section(
                    "Table",
                    column()
                        .gap(6.0)
                        .child(label("Tables with sortable columns and selectable rows."))
                        .child(label(
                            "Define columns with header + width, add rows with cells.",
                        ))
                        .build(),
                ),
            )
            .child(
                self.page_section(
                    "Tree",
                    column()
                        .gap(6.0)
                        .child(label("Hierarchical tree with expand/collapse."))
                        .child(label("Nodes have keys, labels, and optional children."))
                        .build(),
                ),
            )
            .build()
    }

    fn charts_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(20.0)
            .padding(24.0)
            .child(label_with_size("Charts", 24.0))
            .child(separator())
            .child(label(
                "All charts are static representations built from existing primitives.",
            ))
            // Bar Chart
            .child(
                self.page_section(
                    "BarChart",
                    bar_chart::<GalleryMessage>("demo_bar")
                        .bar(BarEntry::new("Q1", 85.0))
                        .bar(BarEntry::new("Q2", 62.0))
                        .bar(BarEntry::new("Q3", 93.0))
                        .bar(BarEntry::new("Q4", 47.0))
                        .max_value(100.0)
                        .build(),
                ),
            )
            // Line Chart
            .child(
                self.page_section(
                    "LineChart",
                    line_chart::<GalleryMessage>("demo_line")
                        .point(ChartPoint::new("Jan", 30.0))
                        .point(ChartPoint::new("Feb", 55.0))
                        .point(ChartPoint::new("Mar", 42.0))
                        .point(ChartPoint::new("Apr", 78.0))
                        .point(ChartPoint::new("May", 90.0))
                        .build(),
                ),
            )
            // Pie Chart
            .child(
                self.page_section(
                    "PieChart",
                    pie_chart::<GalleryMessage>("demo_pie")
                        .slice(PieSlice::new(
                            "Sales",
                            40.0,
                            ThemeColor::rgba(0.3, 0.6, 1.0, 1.0),
                        ))
                        .slice(PieSlice::new(
                            "Marketing",
                            25.0,
                            ThemeColor::rgba(0.1, 0.8, 0.4, 1.0),
                        ))
                        .slice(PieSlice::new(
                            "Engineering",
                            20.0,
                            ThemeColor::rgba(1.0, 0.7, 0.1, 1.0),
                        ))
                        .slice(PieSlice::new(
                            "Support",
                            15.0,
                            ThemeColor::rgba(0.9, 0.3, 0.3, 1.0),
                        ))
                        .build(),
                ),
            )
            // Area Chart
            .child(
                self.page_section(
                    "AreaChart",
                    area_chart::<GalleryMessage>("demo_area")
                        .point(ChartPoint::new("Mon", 20.0))
                        .point(ChartPoint::new("Tue", 45.0))
                        .point(ChartPoint::new("Wed", 38.0))
                        .point(ChartPoint::new("Thu", 72.0))
                        .point(ChartPoint::new("Fri", 60.0))
                        .build(),
                ),
            )
            // Gauge
            .child(
                self.page_section(
                    "Gauge",
                    gauge::<GalleryMessage>("demo_gauge")
                        .value(72.0)
                        .max(100.0)
                        .label("CPU Usage")
                        .build(),
                ),
            )
            // Sparkline
            .child(
                self.page_section(
                    "Sparkline",
                    column()
                        .gap(6.0)
                        .child(label("Compact inline trend visualization:"))
                        .child(
                            sparkline::<GalleryMessage>("demo_spark")
                                .value(10.0)
                                .value(25.0)
                                .value(18.0)
                                .value(45.0)
                                .value(62.0)
                                .value(55.0)
                                .value(78.0)
                                .value(92.0)
                                .value(85.0)
                                .value(70.0)
                                .build(),
                        )
                        .build(),
                ),
            )
            .build()
    }

    fn page_section(
        &self,
        title: &str,
        content: WidgetNode<GalleryMessage>,
    ) -> WidgetNode<GalleryMessage> {
        column()
            .gap(8.0)
            .child(label_with_size(title, 16.0))
            .child(separator())
            .child(content)
            .build()
    }

    fn hit(&self) -> Option<usize> {
        self.button_info
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, hr)| {
                let r = if hr.scrolled {
                    if self.cursor.1 < 40.0 {
                        return None;
                    }
                    [hr.rect[0], hr.rect[1] - self.scroll, hr.rect[2], hr.rect[3]]
                } else {
                    hr.rect
                };
                if self.cursor.0 >= r[0]
                    && self.cursor.0 <= r[0] + r[2]
                    && self.cursor.1 >= r[1]
                    && self.cursor.1 <= r[1] + r[3]
                {
                    Some(i)
                } else {
                    None
                }
            })
    }

    fn activate(&mut self, index: usize) -> bool {
        let Some(hr) = self.button_info.get(index) else {
            return false;
        };
        match hr.message {
            GalleryMessage::ToggleTheme => {
                self.dark = !self.dark;
                true
            }
            GalleryMessage::SelectCategory(i) => {
                let changed = self.selected_category != i;
                self.selected_category = i;
                if changed {
                    self.scroll = 0.0;
                }
                true
            }
            GalleryMessage::DpiInfo => true,
        }
    }
}

impl Application for Gallery {
    fn window_config(&self) -> WindowConfig {
        WindowConfig {
            title: "AcmeUI Gallery".into(),
            width: 1100.0,
            height: 700.0,
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
                    self.pressed = self.hit();
                    true
                } else {
                    let activated = self
                        .pressed
                        .take()
                        .filter(|&value| Some(value) == self.hit());
                    activated.is_some_and(|i| self.activate(i))
                }
            }
            PlatformEvent::Scroll { delta_y, .. } => {
                self.scroll = (self.scroll - delta_y).clamp(0.0, self.max_scroll);
                true
            }
            PlatformEvent::Resized { .. } => true,
            _ => false,
        }
    }

    fn on_gpu_recovered(&mut self, _window: WindowId) {
        self.atlas.clear();
    }

    fn frame(&mut self, context: FrameContext) -> Frame {
        let width = context.logical_width;
        let height = context.logical_height;

        // 1. Build widget tree
        let description = self.description();

        // 2. Theme
        let theme = if self.dark {
            Theme::dark()
        } else {
            Theme::light()
        };

        // 3. Layout context
        let layout_context = WidgetLayoutContext {
            body_font_size: theme.typography.body_size,
            body_line_height: theme.typography.body_size * theme.typography.line_height,
            label_font_size: theme.typography.label_size,
            control_height: 36.0,
            scale_factor: context.scale_factor,
        };

        // 4. Widget → Layout
        let mut root = description.to_layout_with_context(NodeId::new(1), &layout_context);

        // 5. Apply sizes
        apply_gallery_styles(&mut root, width, height);

        // 6. Compute layout
        let snapshot = self
            .layout
            .compute_with_text(
                &root,
                (width, height),
                &mut self.fonts,
                context.scale_factor,
            )
            .expect("finite gallery viewport");

        // 7. Extract IDs
        let ids = extract_gallery_ids(&root);

        // 8. Collect hit regions
        let mut button_info = Vec::new();
        collect_hit_regions(&description, &root, &snapshot, false, &mut button_info);
        self.button_info = button_info;

        // 9. Scroll metrics
        self.max_scroll = snapshot
            .scroll_metrics(ids.scroll_view)
            .map(|m| (m.content_height - m.viewport_height).max(0.0))
            .unwrap_or(0.0);
        self.scroll = self.scroll.clamp(0.0, self.max_scroll);

        let colors = theme.colors;

        // 10. Build frame
        let mut frame = Frame {
            clear: rgba(colors.background),
            ..Frame::default()
        };

        // 11. Sidebar background
        if let Some(r) = snapshot.get(ids.sidebar) {
            frame.quads.push(quad_rect(
                [r.x, r.y, r.width, r.height],
                colors.surface,
                0.0,
                1.0,
                colors.border,
            ));
        }

        // 12. Sidebar title
        if let Some(r) = snapshot.get(ids.sidebar_label) {
            add_text(
                &mut self.fonts,
                &mut self.atlas,
                &mut frame,
                "AcmeUI",
                ([r.x + 4.0, r.y + 2.0], 18.0),
                colors.text,
                context.scale_factor,
                None,
                theme.typography.line_height,
            );
        }

        // 13. Sidebar buttons
        for (i, &btn_id) in ids.sidebar_buttons.iter().enumerate() {
            let Some(r) = snapshot.get(btn_id) else {
                continue;
            };
            let is_selected = i == self.selected_category;
            let st = ButtonState {
                hovered: self.hovered == Some(i),
                pressed: self.pressed == Some(i),
                focused: false,
            };
            let bg = if is_selected {
                colors.accent
            } else if st.hovered {
                colors.surface_hover
            } else {
                colors.surface
            };
            let fg = if is_selected {
                colors.on_accent
            } else {
                colors.text
            };
            frame.quads.push(quad_rect(
                [r.x, r.y, r.width, r.height],
                bg,
                theme.radii.md,
                1.0,
                colors.border,
            ));
            add_text(
                &mut self.fonts,
                &mut self.atlas,
                &mut frame,
                CATEGORIES[i].name,
                ([r.x + 12.0, r.y + 9.0], theme.typography.label_size),
                fg,
                context.scale_factor,
                None,
                theme.typography.line_height,
            );
        }

        // 14. Toolbar background
        if let Some(r) = snapshot.get(ids.toolbar) {
            frame.quads.push(quad_rect(
                [r.x, r.y, r.width, r.height],
                colors.surface,
                0.0,
                1.0,
                colors.border,
            ));
        }

        // 15. Toolbar buttons
        let tb_labels = [if self.dark { "☀ Light" } else { "🌙 Dark" }, "ℹ Info"];
        for (ti, (&btn_id, &label_text)) in
            ids.toolbar_buttons.iter().zip(tb_labels.iter()).enumerate()
        {
            let btn_idx = 6 + ti;
            let Some(r) = snapshot.get(btn_id) else {
                continue;
            };
            let st = ButtonState {
                hovered: self.hovered == Some(btn_idx),
                pressed: self.pressed == Some(btn_idx),
                focused: false,
            };
            let btn = button::<GalleryMessage>("", "");
            let resolved = btn.resolve_style(&theme, st);
            frame.quads.push(quad_rect(
                [r.x, r.y, r.width, r.height],
                resolved.background,
                theme.radii.md,
                1.0,
                if st.hovered {
                    colors.accent
                } else {
                    colors.border
                },
            ));
            add_text(
                &mut self.fonts,
                &mut self.atlas,
                &mut frame,
                label_text,
                ([r.x + 10.0, r.y + 6.0], 13.0),
                resolved.foreground,
                context.scale_factor,
                None,
                theme.typography.line_height,
            );
        }

        // 16. Page content
        if let Some(sv_rect) = snapshot.get(ids.scroll_view) {
            let clip = [sv_rect.x, sv_rect.y, sv_rect.width, sv_rect.height];
            let mut btn_idx = 8;
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
                &mut self.fonts,
                &mut self.atlas,
            );
        }

        frame
    }
}

// ── Layout IDs ───────────────────────────────────────────────────────────────

struct GalleryNodeIds {
    sidebar: NodeId,
    sidebar_label: NodeId,
    sidebar_buttons: [NodeId; 6],
    toolbar: NodeId,
    toolbar_buttons: [NodeId; 2],
    scroll_view: NodeId,
}

fn extract_gallery_ids(root: &LayoutNode) -> GalleryNodeIds {
    let sb = &root.children[0];
    let ca = &root.children[1];
    let tb = &ca.children[0];
    GalleryNodeIds {
        sidebar: sb.id,
        sidebar_label: sb.children[0].id,
        sidebar_buttons: [
            sb.children[2].id,
            sb.children[3].id,
            sb.children[4].id,
            sb.children[5].id,
            sb.children[6].id,
            sb.children[7].id,
        ],
        toolbar: tb.id,
        toolbar_buttons: [tb.children[0].id, tb.children[1].id],
        scroll_view: ca.children[1].id,
    }
}

// ── Styles ───────────────────────────────────────────────────────────────────

fn apply_gallery_styles(root: &mut LayoutNode, width: f32, height: f32) {
    root.style = LayoutStyle {
        kind: acme_layout::LayoutKind::Row,
        width: Length::px(width),
        height: Length::px(height),
        gap: 0.0,
        ..Default::default()
    };

    let sb = &mut root.children[0];
    sb.style = LayoutStyle {
        kind: acme_layout::LayoutKind::Column,
        width: Length::px(SIDEBAR_WIDTH),
        height: Length::px(height),
        gap: 4.0,
        padding: acme_layout::Edges {
            left: 12.0,
            right: 12.0,
            top: 16.0,
            bottom: 16.0,
        },
        ..Default::default()
    };
    for i in 2..8 {
        sb.children[i].style.width = Length::px(SIDEBAR_WIDTH - 24.0);
        sb.children[i].style.height = Length::px(38.0);
    }

    let cw = (width - SIDEBAR_WIDTH).max(400.0);
    let ca = &mut root.children[1];
    ca.style = LayoutStyle {
        kind: acme_layout::LayoutKind::Column,
        width: Length::px(cw),
        height: Length::px(height),
        gap: 0.0,
        ..Default::default()
    };

    let tb = &mut ca.children[0];
    tb.style = LayoutStyle {
        kind: acme_layout::LayoutKind::Row,
        width: Length::px(cw),
        height: Length::px(TOOLBAR_HEIGHT),
        gap: 8.0,
        padding: acme_layout::Edges {
            left: 12.0,
            right: 12.0,
            top: 6.0,
            bottom: 6.0,
        },
        ..Default::default()
    };
    for btn in &mut tb.children {
        btn.style.width = Length::px(110.0);
        btn.style.height = Length::px(28.0);
    }

    let sh = (height - TOOLBAR_HEIGHT).max(100.0);
    let sv = &mut ca.children[1];
    sv.style = LayoutStyle {
        kind: acme_layout::LayoutKind::Column,
        width: Length::px(cw),
        height: Length::px(sh),
        overflow: Overflow::Scroll,
        flex_grow: 1.0,
        ..Default::default()
    };
    if let Some(content) = sv.children.first_mut() {
        content.style.width = Length::px(cw);
    }
}

// ── Content Render ───────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn render_content(
    frame: &mut Frame,
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &acme_layout::LayoutSnapshot,
    theme: &Theme,
    scale: f32,
    scroll_y: f32,
    clip: [f32; 4],
    btn_idx: &mut usize,
    hovered: Option<usize>,
    pressed: Option<usize>,
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
) {
    let colors = theme.colors;
    match widget {
        WidgetNode::Label(l) => {
            if let Some(rect) = snapshot.get(layout.id) {
                let fs = l.font_size.unwrap_or(theme.typography.body_size);
                let y_text = rect.y - scroll_y;
                add_text(
                    fonts,
                    atlas,
                    frame,
                    &l.text,
                    ([rect.x + 4.0, y_text], fs),
                    colors.text,
                    scale,
                    Some(clip),
                    theme.typography.line_height,
                );
            }
        }
        WidgetNode::Button(btn) => {
            if btn.activate().is_none() {
                return;
            }
            if let Some(rect) = snapshot.get(layout.id) {
                let y = rect.y - scroll_y;
                let idx = *btn_idx;
                *btn_idx += 1;
                let st = ButtonState {
                    hovered: hovered == Some(idx),
                    pressed: pressed == Some(idx),
                    focused: false,
                };
                let resolved = btn.resolve_style(theme, st);
                frame.quads.push(quad_rect(
                    [rect.x, y, rect.width, rect.height],
                    resolved.background,
                    theme.radii.md,
                    1.0,
                    resolved.border,
                ));
                add_text(
                    fonts,
                    atlas,
                    frame,
                    &btn.label,
                    ([rect.x + 10.0, y + 8.0], theme.typography.label_size),
                    resolved.foreground,
                    scale,
                    Some(clip),
                    theme.typography.line_height,
                );
            }
        }
        WidgetNode::Separator(_) => {
            if let Some(rect) = snapshot.get(layout.id) {
                let y = rect.y - scroll_y;
                frame.quads.push(quad_rect(
                    [rect.x, y, rect.width, 1.0],
                    colors.border,
                    0.0,
                    0.0,
                    colors.border,
                ));
            }
        }
        // Card: paint as colored container
        WidgetNode::Card(c) => {
            if let Some(rect) = snapshot.get(layout.id) {
                let y = rect.y - scroll_y;
                let (bg, border) = match c.variant {
                    CardVariant::Elevated => (colors.surface, colors.border),
                    CardVariant::Outlined => (colors.background, colors.border),
                    CardVariant::Interactive => (colors.surface, colors.accent),
                    _ => (colors.surface, colors.border),
                };
                frame.quads.push(quad_rect(
                    [rect.x, y, rect.width, rect.height],
                    bg,
                    theme.radii.sm,
                    1.0,
                    border,
                ));
            }
            // Recursively render children
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                render_content(
                    frame, w, l, snapshot, theme, scale, scroll_y, clip, btn_idx, hovered, pressed,
                    fonts, atlas,
                );
            }
        }
        WidgetNode::Tooltip(t) => {
            render_content(
                frame, &t.child, layout, snapshot, theme, scale, scroll_y, clip, btn_idx, hovered,
                pressed, fonts, atlas,
            );
        }
        WidgetNode::Popover(p) => {
            render_content(
                frame,
                &p.children[0],
                layout,
                snapshot,
                theme,
                scale,
                scroll_y,
                clip,
                btn_idx,
                hovered,
                pressed,
                fonts,
                atlas,
            );
        }
        _ => {
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                render_content(
                    frame, w, l, snapshot, theme, scale, scroll_y, clip, btn_idx, hovered, pressed,
                    fonts, atlas,
                );
            }
        }
    }
}

// ── Hit Region Collection ────────────────────────────────────────────────────

fn collect_hit_regions(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &acme_layout::LayoutSnapshot,
    scrolled: bool,
    result: &mut Vec<HitRegion>,
) {
    match widget {
        WidgetNode::Button(btn) => {
            if let Some(msg) = btn.activate() {
                if let Some(rect) = snapshot.get(layout.id) {
                    result.push(HitRegion {
                        rect: [rect.x, rect.y, rect.width, rect.height],
                        message: *msg,
                        scrolled,
                    });
                }
            }
        }
        WidgetNode::ScrollView(_) => {
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                collect_hit_regions(w, l, snapshot, true, result);
            }
        }
        WidgetNode::Tooltip(t) => {
            collect_hit_regions(&t.child, layout, snapshot, scrolled, result);
        }
        WidgetNode::Popover(p) => {
            collect_hit_regions(&p.children[0], layout, snapshot, scrolled, result);
        }
        _ => {
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                collect_hit_regions(w, l, snapshot, scrolled, result);
            }
        }
    }
}

// ── Render Helpers ───────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn add_text(
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
    frame: &mut Frame,
    text: &str,
    geometry: ([f32; 2], f32),
    color: ThemeColor,
    scale: f32,
    clip: Option<[f32; 4]>,
    line_height_ratio: f32,
) {
    let (origin, size) = geometry;
    let style = TextStyle {
        font_size: size,
        line_height: size * line_height_ratio,
        ..TextStyle::default()
    };
    let layout = fonts.shape(text, &style, TextConstraints::default(), scale);
    let prepared = fonts.prepare(&layout, atlas);
    frame.text.push(TextRun {
        prepared,
        origin,
        color: rgba(color),
        clip,
    });
}

fn rgba(color: ThemeColor) -> [f32; 4] {
    [color.red, color.green, color.blue, color.alpha]
}

fn quad_rect(
    rect: [f32; 4],
    fill: ThemeColor,
    radius: f32,
    border_width: f32,
    border_color: ThemeColor,
) -> Quad {
    Quad {
        rect,
        color: rgba(fill),
        radius,
        border_width,
        border_color: rgba(border_color),
    }
}
