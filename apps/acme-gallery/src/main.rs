//! AcmeUI Gallery — showcase app for acme-ui components including Charts.
//!
//! Architecture:
//!   Row (root)
//!     ├── Column (sidebar, 200px) — AcmeUI + category buttons
//!     └── Column (content area) — flex
//!         ├── Row (toolbar, 40px) — theme toggle
//!         └── ScrollView (content) — flex, overflow scroll

#![allow(clippy::collapsible_if)]

use acme_core::NodeId;
use acme_layout::{LayoutEngine, LayoutNode, LayoutStyle, Length, Overflow, WidgetLayoutContext};
use acme_platform::{Application, FrameContext, PlatformEvent, WindowConfig, WindowId};
use acme_render_wgpu::{Frame, Quad, TextRun, scene_from_frame};
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextStyle};
use acme_theme::packs::{available_themes, theme_by_name};
use acme_theme::{Theme, ThemeColor, ThemeMode};
use acme_ui::charts::{
    BarEntry, ChartPoint, PieSlice, area_chart, bar_chart, gauge, line_chart, pie_chart, sparkline,
};
use acme_widgets::{
    ButtonSize, ButtonState, ButtonVariant, CardVariant, WidgetNode, button, card, column, label,
    label_with_size, row, scroll_view, separator,
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
        name: "Design System",
        pages: &["Tokens", "Colors", "Typography", "Spacing"],
    },
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
    CycleTheme,
    DpiInfo,
}

// ── Theme preference persistence ────────────────────────────────────────────

const THEME_PREF_FILE: &str = "acme-gallery-theme.conf";

/// Load the persisted theme preference (pack name, dark mode).
///
/// Returns `None` if the file is missing or unparseable; callers fall back to
/// the default theme.
fn load_theme_pref() -> Option<(String, bool)> {
    let content = std::fs::read_to_string(THEME_PREF_FILE).ok()?;
    let mut pack = None;
    let mut dark = None;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("pack=") {
            pack = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("dark=") {
            dark = Some(value.trim() == "true");
        }
    }
    Some((pack?, dark?))
}

/// Persist the theme preference to disk (best-effort; ignore write errors).
fn save_theme_pref(pack: &str, dark: bool) {
    let content = format!("pack={pack}\ndark={dark}\n");
    let _ = std::fs::write(THEME_PREF_FILE, content);
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
    theme_pack: String,
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
        // Restore persisted theme preference, falling back to defaults.
        let (theme_pack, dark) = load_theme_pref()
            .filter(|(pack, _)| pack == "default" || available_themes().contains(&pack.as_str()))
            .unwrap_or_else(|| ("default".to_string(), false));
        Self {
            selected_category: 6, // Start on Charts page
            dark,
            theme_pack,
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
        let theme = self.current_theme();
        let wcag_label = if theme.meets_wcag_aa() { "✓AA" } else { "⚠AA" };
        row()
            .key("toolbar")
            .gap(8.0)
            .padding(8.0)
            .child(
                button("theme_btn", if self.dark { "☀ Light" } else { "🌙 Dark" })
                    .on_click(GalleryMessage::ToggleTheme),
            )
            .child(
                button(
                    "pack_btn",
                    format!("🎨 {} {}", self.theme_pack, wcag_label),
                )
                .on_click(GalleryMessage::CycleTheme),
            )
            .child(button("info_btn", "ℹ Info").on_click(GalleryMessage::DpiInfo))
            .build()
    }

    /// Resolve the active theme from the selected pack and light/dark mode.
    fn current_theme(&self) -> Theme {
        let mode = if self.dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        theme_by_name(&self.theme_pack, mode).unwrap_or_else(|| match mode {
            ThemeMode::Dark => Theme::dark(),
            ThemeMode::Light => Theme::light(),
        })
    }

    /// Advance to the next theme pack ("default" → apple → windows10 → …).
    fn cycle_theme(&mut self) {
        // Build the ordered list: default first, then registered packs.
        let mut order: Vec<String> = vec!["default".to_string()];
        order.extend(available_themes().iter().map(|s| s.to_string()));
        let pos = order
            .iter()
            .position(|p| p == &self.theme_pack)
            .unwrap_or(0);
        self.theme_pack = order[(pos + 1) % order.len()].clone();
        save_theme_pref(&self.theme_pack, self.dark);
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
            0 => self.design_system_page(),
            1 => self.foundations_page(),
            2 => self.inputs_page(),
            3 => self.layout_page(),
            4 => self.overlay_page(),
            5 => self.data_page(),
            6 => self.charts_page(),
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
                        .gap(8.0)
                        .child(label("The complete type scale from the V2 design system:"))
                        .child(label_with_size(
                            "h1 28px — Page titles and hero headings",
                            28.0,
                        ))
                        .child(label_with_size("h2 22px — Major section headers", 22.0))
                        .child(label_with_size("h3 18px — Card and panel titles", 18.0))
                        .child(label_with_size("h4 16px — Subsection headings", 16.0))
                        .child(label_with_size("Body 14px — Default reading text", 14.0))
                        .child(label_with_size("Caption 12px — Helper, timestamps", 12.0))
                        .child(label_with_size("Small 11px — Legal, meta", 11.0))
                        .build(),
                ),
            )
            .child(
                self.page_section(
                    "Color Palette",
                    column()
                        .gap(12.0)
                        .child(label("Semantic color tokens respond to light/dark mode."))
                        .child(label_with_size("Surface Colors", 16.0))
                        .child(
                            row()
                                .gap(12.0)
                                .child(self.color_card("Background", CardVariant::Outlined))
                                .child(self.color_card("Surface", CardVariant::Elevated))
                                .build(),
                        )
                        .child(label_with_size("Interactive Colors", 16.0))
                        .child(
                            row()
                                .gap(8.0)
                                .child(
                                    button("fc_primary", "Primary")
                                        .primary()
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .child(
                                    button("fc_secondary", "Secondary")
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .child(
                                    button("fc_danger", "Danger")
                                        .variant(ButtonVariant::Danger)
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .build(),
                        )
                        .build(),
                ),
            )
            .build()
    }

    fn inputs_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(24.0)
            .padding(24.0)
            .child(label_with_size("Inputs", 24.0))
            .child(separator())
            .child(
                self.page_section(
                    "Button Variants",
                    column()
                        .gap(12.0)
                        .child(label(
                            "Four tone variants cover common interaction patterns:",
                        ))
                        .child(
                            row()
                                .gap(8.0)
                                .child(
                                    button("iv_primary", "Primary")
                                        .primary()
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .child(
                                    button("iv_secondary", "Secondary")
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .child(
                                    button("iv_ghost", "Ghost")
                                        .variant(ButtonVariant::Ghost)
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .child(
                                    button("iv_danger", "Danger")
                                        .variant(ButtonVariant::Danger)
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .build(),
                        )
                        .build(),
                ),
            )
            .child(
                self.page_section(
                    "Button Sizes",
                    column()
                        .gap(12.0)
                        .child(label("Four size tiers from compact to prominent:"))
                        .child(
                            row()
                                .gap(8.0)
                                .child(
                                    button("is_xs", "XS")
                                        .size(ButtonSize::XS)
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .child(
                                    button("is_sm", "Small")
                                        .size(ButtonSize::Small)
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .child(
                                    button("is_md", "Medium")
                                        .size(ButtonSize::Medium)
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .child(
                                    button("is_lg", "Large")
                                        .size(ButtonSize::Large)
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .build(),
                        )
                        .build(),
                ),
            )
            .child(
                self.page_section(
                    "Button States",
                    column()
                        .gap(12.0)
                        .child(label("Default idle state and loading indicator:"))
                        .child(
                            row()
                                .gap(8.0)
                                .child(
                                    button("ist_default", "Default")
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .child(
                                    button("ist_loading", "Loading …")
                                        .loading(true)
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .build(),
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

    fn design_system_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(24.0)
            .padding(24.0)
            .child(label_with_size("Design System", 24.0))
            .child(separator())
            .child(label("All semantic design tokens defined by the V2 theme."))
            // ── Color Palette ────────────────────────────────────────────
            .child(
                self.page_section(
                    "Color Palette",
                    column()
                        .gap(16.0)
                        .child(label_with_size("Base Surface Colors", 16.0))
                        .child(
                            row()
                                .gap(12.0)
                                .child(self.color_card("Background", CardVariant::Outlined))
                                .child(self.color_card("Surface", CardVariant::Elevated))
                                .child(self.color_card("Elevated", CardVariant::Interactive))
                                .build(),
                        )
                        .child(label_with_size("Semantic Colors", 16.0))
                        .child(
                            row()
                                .gap(8.0)
                                .child(
                                    button("ds_primary", "Primary")
                                        .primary()
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .child(
                                    button("ds_secondary", "Secondary")
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .child(
                                    button("ds_ghost", "Ghost")
                                        .variant(ButtonVariant::Ghost)
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .child(
                                    button("ds_danger", "Danger")
                                        .variant(ButtonVariant::Danger)
                                        .on_click(GalleryMessage::DpiInfo),
                                )
                                .build(),
                        )
                        .child(label_with_size("Status Colors", 16.0))
                        .child(
                            row()
                                .gap(12.0)
                                .child(self.swatch("success", "Success"))
                                .child(self.swatch("warning", "Warning"))
                                .child(self.swatch("danger", "Danger"))
                                .child(self.swatch("info", "Info"))
                                .build(),
                        )
                        .child(label_with_size("Utility Colors", 16.0))
                        .child(
                            row()
                                .gap(12.0)
                                .child(self.swatch("muted_foreground", "Muted Fg"))
                                .child(self.swatch("border", "Border"))
                                .child(self.swatch("ring", "Ring"))
                                .child(self.swatch("input", "Input"))
                                .build(),
                        )
                        .build(),
                ),
            )
            // ── Typography ──────────────────────────────────────────────
            .child(
                self.page_section(
                    "Typography Scale",
                    column()
                        .gap(8.0)
                        .child(label_with_size("Heading 1 (h1) — 28px  Page title", 28.0))
                        .child(label_with_size(
                            "Heading 2 (h2) — 22px  Section title",
                            22.0,
                        ))
                        .child(label_with_size("Heading 3 (h3) — 18px  Card title", 18.0))
                        .child(label_with_size("Heading 4 (h4) — 16px  Subsection", 16.0))
                        .child(label_with_size("Body — 14px  Default body text", 14.0))
                        .child(label_with_size("Body Small — 13px  Compact body", 13.0))
                        .child(label_with_size("Label — 13px  Form label", 13.0))
                        .child(label_with_size("Caption — 12px  Helper text, badges", 12.0))
                        .child(label_with_size("Small — 11px  Legal, timestamps", 11.0))
                        .build(),
                ),
            )
            // ── Spacing ─────────────────────────────────────────────────
            .child(
                self.page_section(
                    "Spacing Grid",
                    column()
                        .gap(8.0)
                        .child(label(
                            "The design system uses a 4px base grid. Common spacing tokens:",
                        ))
                        .child(self.spacing_demo("half ", 2))
                        .child(self.spacing_demo("px   ", 4))
                        .child(self.spacing_demo("px1  ", 6))
                        .child(self.spacing_demo("px2  ", 8))
                        .child(self.spacing_demo("px3  ", 12))
                        .child(self.spacing_demo("px4  ", 16))
                        .child(self.spacing_demo("px5  ", 20))
                        .child(self.spacing_demo("px6  ", 24))
                        .child(self.spacing_demo("px8  ", 32))
                        .child(self.spacing_demo("px10 ", 40))
                        .build(),
                ),
            )
            // ── Border Radius ───────────────────────────────────────────
            .child(
                self.page_section(
                    "Border Radius",
                    column()
                        .gap(8.0)
                        .child(label("Corner radii for surfaces and components:"))
                        .child(label("none — 0px    Sharp elements"))
                        .child(label("sm   — 4px    Inputs, data-view cards"))
                        .child(label("md   — 6px    Buttons, default components"))
                        .child(label("lg   — 8px    Cards, dialogs"))
                        .child(label("xl   — 12px   Modals, large surfaces"))
                        .child(label("full — 999px  Badges, pills, avatars"))
                        .build(),
                ),
            )
            // ── Shadow / Elevation ──────────────────────────────────────
            .child(
                self.page_section(
                    "Shadow / Elevation",
                    column()
                        .gap(8.0)
                        .child(label("sm   0 1px  2px  rgba(0,0,0,0.04)  Subtle cards"))
                        .child(label("md   0 4px  12px rgba(0,0,0,0.06)  Popovers, menus"))
                        .child(label("lg   0 8px  24px rgba(0,0,0,0.08)  Dialogs, modals"))
                        .child(label(
                            "xl   0 16px 48px rgba(0,0,0,0.10)  Notifications, tooltips",
                        ))
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

    // ── Helpers for Design System page ──────────────────────────────────

    /// Create a color swatch label: a small colored square + descriptive text.
    /// Uses a special encoding prefix (`\x01`) that `render_content` detects.
    fn swatch(&self, color_name: &str, display: &str) -> WidgetNode<GalleryMessage> {
        label(format!("\x01{}|{}", color_name, display))
    }

    /// Create a small card showing a base surface color.
    fn color_card(&self, label_text: &str, variant: CardVariant) -> WidgetNode<GalleryMessage> {
        card()
            .variant(variant)
            .padding(16.0)
            .child(label(label_text))
            .build()
    }

    /// Create a visual spacing demo: row with separators showing gap width.
    fn spacing_demo(&self, label_text: &str, px: u32) -> WidgetNode<GalleryMessage> {
        let text = format!("{} {:>2}px — {:━>2$}", label_text, px, px as usize);
        row().gap(8.0).child(label(text)).build()
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
                save_theme_pref(&self.theme_pack, self.dark);
                true
            }
            GalleryMessage::CycleTheme => {
                self.cycle_theme();
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

    fn frame(&mut self, context: FrameContext) -> acme_core::Scene {
        let width = context.logical_width;
        let height = context.logical_height;

        // 1. Build widget tree
        let description = self.description();

        // 2. Theme (resolved from selected pack + light/dark mode)
        let theme = self.current_theme();

        // 3. Layout context
        let layout_context = WidgetLayoutContext {
            body_font_size: theme.typography.body,
            body_line_height: theme.typography.body * theme.typography.line_height,
            label_font_size: theme.typography.label,
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

        // 12. Sidebar title — centered
        if let Some(r) = snapshot.get(ids.sidebar_label) {
            let measured = self.fonts.measure(
                "AcmeUI",
                &TextStyle {
                    font_size: 18.0,
                    line_height: 18.0 * theme.typography.line_height,
                    ..TextStyle::default()
                },
                TextConstraints::default(),
            );
            let ox = r.x + (r.width - measured.width).max(0.0) / 2.0;
            let oy = r.y + (r.height - measured.height) / 2.0 + measured.baseline;
            add_text(
                &mut self.fonts,
                &mut self.atlas,
                &mut frame,
                "AcmeUI",
                ([ox, oy], 18.0),
                colors.foreground,
                context.scale_factor,
                None,
                theme.typography.line_height,
            );
        }

        // 13. Sidebar buttons — text centered both horizontally and vertically
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
                1.0,
                colors.border,
            ));
            let label = CATEGORIES[i].name;
            let measured = self.fonts.measure(
                label,
                &TextStyle {
                    font_size: theme.typography.label,
                    line_height: theme.typography.label * theme.typography.line_height,
                    ..TextStyle::default()
                },
                TextConstraints::default(),
            );
            let ox = r.x + (r.width - measured.width).max(0.0) / 2.0;
            // Vertical centering: place the layout's top at
            //   r.y + (r.height - measured.height) / 2
            // then add baseline so origin_y (the baseline position)
            // correctly positions all glyphs.
            let oy = r.y + (r.height - measured.height) / 2.0 + measured.baseline;
            add_text(
                &mut self.fonts,
                &mut self.atlas,
                &mut frame,
                label,
                ([ox, oy], theme.typography.label),
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
        let tb_theme = self.current_theme();
        let tb_wcag = if tb_theme.meets_wcag_aa() { "✓AA" } else { "⚠AA" };
        let tb_labels = [
            (if self.dark { "☀ Light" } else { "🌙 Dark" }).to_string(),
            format!("🎨 {} {}", self.theme_pack, tb_wcag),
            "ℹ Info".to_string(),
        ];
        for (ti, (&btn_id, label_text)) in
            ids.toolbar_buttons.iter().zip(tb_labels.iter()).enumerate()
        {
            let btn_idx = 7 + ti;
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
            let measured_tb = self.fonts.measure(
                label_text,
                &TextStyle {
                    font_size: 13.0,
                    line_height: 13.0 * theme.typography.line_height,
                    ..TextStyle::default()
                },
                TextConstraints::default(),
            );
            let tb_ox = r.x + (r.width - measured_tb.width).max(0.0) / 2.0;
            let tb_oy = r.y + (r.height - measured_tb.height) / 2.0 + measured_tb.baseline;
            add_text(
                &mut self.fonts,
                &mut self.atlas,
                &mut frame,
                label_text,
                ([tb_ox, tb_oy], 13.0),
                resolved.foreground,
                context.scale_factor,
                None,
                theme.typography.line_height,
            );
        }

        // 16. Page content
        if let Some(sv_rect) = snapshot.get(ids.scroll_view) {
            let clip = [sv_rect.x, sv_rect.y, sv_rect.width, sv_rect.height];
            // sidebar(0-6) + toolbar(7-9) = 10, content buttons start here.
            let mut btn_idx = 10;
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

        scene_from_frame(&frame)
    }
}

// ── Layout IDs ───────────────────────────────────────────────────────────────

struct GalleryNodeIds {
    sidebar: NodeId,
    sidebar_label: NodeId,
    sidebar_buttons: [NodeId; 7],
    toolbar: NodeId,
    toolbar_buttons: [NodeId; 3],
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
            sb.children[8].id,
        ],
        toolbar: tb.id,
        toolbar_buttons: [tb.children[0].id, tb.children[1].id, tb.children[2].id],
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
    for i in 2..9 {
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
                let fs = l.font_size.unwrap_or(theme.typography.body);
                let y_text = rect.y - scroll_y;
                let text_color = l.color.unwrap_or(colors.foreground);

                // Color swatch label: "\x01{color_name}|{display_text}"
                if l.text.starts_with('\x01') {
                    if let Some(rest) = l.text.get(1..) {
                        if let Some((color_name, display)) = rest.split_once('|') {
                            if let Some(swatch_color) = resolve_swatch_color(theme, color_name) {
                                let swatch_size = fs * 0.85;
                                let swatch_x = rect.x + 4.0;
                                let swatch_y = y_text + (fs * 1.5 - swatch_size).max(0.0) / 2.0;
                                let swatch_radius = theme.radii.sm.min(swatch_size / 2.0);
                                frame.quads.push(quad_rect(
                                    [swatch_x, swatch_y, swatch_size, swatch_size],
                                    swatch_color,
                                    swatch_radius,
                                    0.0,
                                    swatch_color,
                                ));
                                let text_x = swatch_x + swatch_size + 8.0;
                                add_text(
                                    fonts,
                                    atlas,
                                    frame,
                                    display,
                                    ([text_x, y_text], fs),
                                    text_color,
                                    scale,
                                    Some(clip),
                                    theme.typography.line_height,
                                );
                                return;
                            }
                        }
                    }
                }

                add_text(
                    fonts,
                    atlas,
                    frame,
                    &l.text,
                    ([rect.x + 4.0, y_text], fs),
                    text_color,
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
                    ([rect.x + 10.0, y + 8.0], theme.typography.label),
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
                // Use explicit background_color when set, otherwise derive from variant
                let (bg, border) = if let Some(custom_bg) = c.background_color {
                    (custom_bg, colors.border)
                } else {
                    match c.variant {
                        CardVariant::Elevated => (colors.surface, colors.border),
                        CardVariant::Outlined => (colors.background, colors.border),
                        CardVariant::Interactive => (colors.surface, colors.accent),
                        _ => (colors.surface, colors.border),
                    }
                };
                // Use explicit border_radius when set, otherwise default to sm
                let radius = c.border_radius.unwrap_or(theme.radii.sm);
                frame.quads.push(quad_rect(
                    [rect.x, y, rect.width, rect.height],
                    bg,
                    radius,
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

/// Map a color name to its `ThemeColor` value from the current theme.
/// Used by the Design System page color swatches.
fn resolve_swatch_color(theme: &Theme, name: &str) -> Option<ThemeColor> {
    let c = theme.colors;
    match name {
        "primary" => Some(c.primary),
        "secondary" => Some(c.secondary),
        "accent" => Some(c.accent),
        "muted" => Some(c.muted),
        "muted_foreground" => Some(c.muted_foreground),
        "background" => Some(c.background),
        "foreground" => Some(c.foreground),
        "surface" => Some(c.surface),
        "border" => Some(c.border),
        "ring" => Some(c.ring),
        "input" => Some(c.input),
        "success" => Some(c.success),
        "warning" => Some(c.warning),
        "danger" => Some(c.danger),
        "info" => Some(c.info),
        "success_soft" => Some(c.success_soft),
        "warning_soft" => Some(c.warning_soft),
        "danger_soft" => Some(c.danger_soft),
        "info_soft" => Some(c.info_soft),
        "primary_foreground" => Some(c.primary_foreground),
        "surface_elevated" => Some(c.surface_elevated),
        _ => None,
    }
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
