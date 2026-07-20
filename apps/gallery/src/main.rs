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

use acme_accessibility::AccessibilityAdapter;
use acme_core::NodeId;
use acme_layout::{
    LayoutEngine, LayoutKind, LayoutNode, LayoutStyle, Length, Overflow, WidgetLayoutContext,
};
use acme_platform::{
    Application, Clipboard, FrameContext, PlatformEvent, PlatformKey, WindowConfig, WindowId,
};
use acme_render_wgpu::{Frame, Quad, TextRun};
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextStyle};
use acme_textinput::{
    TextInputState, handle_key, handle_keyboard_shortcut, handle_text, render_text_input,
};
use acme_theme::{Theme, ThemeColor, ThemeMode};
use acme_widgets::{
    ButtonState, ButtonVariant, DataGridColumn, DataGridRow, SortDirection, TabItem, TableColumn,
    TableRow, TreeNode, WidgetKey, WidgetNode, breadcrumb, button, column, datagrid, label,
    label_with_size, nav_item, nav_rail, row, scroll_view, separator, sidebar, tab_bar, table,
    tree, virtual_list,
};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();
    acme_platform::run(Gallery::new())?;
    Ok(())
}

// ── Constants ────────────────────────────────────────────────────────────────

const SIDEBAR_WIDTH: f32 = 224.0;
const TOOLBAR_HEIGHT: f32 = 48.0;
/// Long Traditional Chinese string for testing widget string handling.
const LONG_CHINESE_TEXT: &str =
    "在一個寧靜的午後，古老的書架上擺滿了泛黃的書籍，每一本都承載著時代的記憶與智慧";

/// Unique marker for the TextInput placeholder slot (must not appear in normal labels).
const TEXT_INPUT_MARKER: &str = "\0_ti_ph_\0";

/// Expandable tree node keys (bit `i` in `Gallery::tree_expanded`).
const TREE_EXPAND_KEYS: &[&str] = &["docs", "docs_zh", "images", "code", "code_src"];
/// Default: all expandable nodes open (matches the original static demo).
const TREE_EXPAND_DEFAULT: u32 = 0b1_1111;

const VLIST_ITEM_COUNT: usize = 250;
const VLIST_ITEM_HEIGHT: f32 = 28.0;
const VLIST_VIEWPORT_H: f32 = 360.0;
const TABLE_ROW_COUNT: usize = 28;

struct CategoryInfo {
    name: &'static str,
    pages: &'static [&'static str],
}

const CATEGORIES: &[CategoryInfo] = &[
    CategoryInfo {
        name: "Foundations",
        pages: &["Typography", "Colors", "Icons", "Spacing", "Motion"],
    },
    CategoryInfo {
        name: "Inputs",
        pages: &[
            "Button",
            "TextInput",
            "Checkbox",
            "Radio",
            "Switch",
            "Slider",
        ],
    },
    CategoryInfo {
        name: "Navigation",
        pages: &["NavRail", "Sidebar", "TabBar", "Breadcrumb"],
    },
    CategoryInfo {
        name: "Overlay",
        pages: &["Tooltip", "Popover", "Menu", "Dialog"],
    },
    CategoryInfo {
        name: "Data",
        pages: &["Tree", "Table", "DataGrid", "VirtualList"],
    },
    CategoryInfo {
        name: "Patterns",
        pages: &["Settings Page", "Dashboard", "IDE Layout", "SpeakType"],
    },
    CategoryInfo {
        name: "Accessibility",
        pages: &["Focus", "Screen Reader", "Keyboard Nav", "Reduced Motion"],
    },
    CategoryInfo {
        name: "Stress Tests",
        pages: &["1000 Labels", "Deep Nesting", "Rapid Updates", "Long Text"],
    },
];

// ── Helper Types ────────────────────────────────────────────────────────────

/// Spacing density for the gallery.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Density {
    Compact,
    Comfortable,
}

impl Density {
    fn spacing_scale(self) -> f32 {
        match self {
            Self::Compact => 0.75,
            Self::Comfortable => 1.0,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Comfortable => "Comfortable",
        }
    }
    fn toggle(self) -> Self {
        match self {
            Self::Compact => Self::Comfortable,
            Self::Comfortable => Self::Compact,
        }
    }
}

/// Configures a screenshot-style render of a single template at a fixed size.
#[allow(dead_code)]
struct ScreenshotConfig {
    width: f32,
    height: f32,
    theme_variant: ThemeMode,
    density: Density,
    show_focus: bool,
    show_error: bool,
    show_loading: bool,
    show_empty: bool,
}

/// All messages handled by the gallery.
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
enum GalleryMessage {
    SelectCategory(usize),
    SelectPage(usize),
    ToggleTheme,
    ToggleDensity,
    ToggleFocusRings,
    FocusDemo,
    DpiInfo,
    /// NavRail destination index.
    NavRailSelect(usize),
    /// Primary TabBar tab index.
    TabBarSelect(usize),
    /// Secondary (zh) TabBar tab index.
    TabBarZhSelect(usize),
    /// Select a tree node by stable key.
    TreeSelectKey(&'static str),
    /// Toggle expand/collapse for a tree node key.
    TreeToggleKey(&'static str),
    /// Sort table by column index (toggles direction when repeated).
    TableSort(usize),
    /// Select table row by original (pre-sort) data index.
    TableSelectRow(usize),
}

/// A hit region stored after layout each frame.
///
/// `scrolled` is true for targets inside the page `ScrollView`. Their layout
/// rects are in content space; hit tests subtract `Gallery::scroll` from `y`.
struct HitRegion {
    rect: [f32; 4],
    message: GalleryMessage,
    scrolled: bool,
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

    // ── Page Dispatcher ─────────────────────────────────────────────────────

    fn render_page(&self) -> WidgetNode<GalleryMessage> {
        match self.selected_category {
            0 => self.foundations_page(),
            1 => self.inputs_page(),
            2 => self.navigation_page(),
            3 => self.overlay_page(),
            4 => self.data_page(),
            5 => self.patterns_page(),
            6 => self.accessibility_page(),
            7 => self.stress_tests_page(),
            _ => label("Unknown category"),
        }
    }

    // Per-category page builders — each returns a Column suitable as scroll‑content

    fn foundations_page(&self) -> WidgetNode<GalleryMessage> {
        self.component_page("Typography")
    }

    fn inputs_page(&self) -> WidgetNode<GalleryMessage> {
        if self.selected_page == 1 {
            return self.textinput_page();
        }
        let name = CATEGORIES[1].pages[self.selected_page.min(5)];
        self.component_page(name)
    }

    fn textinput_page(&self) -> WidgetNode<GalleryMessage> {
        // Component page with a special placeholder label for the live text input.
        let mut secs = standard_component_sections();
        secs.push((
            "States",
            column()
                .gap(8.0)
                .child(label(
                    "Click the input below to focus, then type or use IME:",
                ))
                // Marker label — replaced by a real render_text_input in frame()
                .child(label(TEXT_INPUT_MARKER))
                .child(label("")) // committed-text output area
                .build(),
        ));
        self.build_component_page("TextInput", secs)
    }

    fn navigation_page(&self) -> WidgetNode<GalleryMessage> {
        match self.selected_page.min(3) {
            0 => self.nav_rail_page(),
            1 => self.sidebar_widget_page(),
            2 => self.tab_bar_page(),
            _ => self.breadcrumb_page(),
        }
    }

    fn nav_rail_page(&self) -> WidgetNode<GalleryMessage> {
        let g = spacing(self.density, 8.0);
        let sel = self.nav_rail_selected;
        let names = ["Home", "Search", "Library", "Settings"];
        let sel_name = names.get(sel).copied().unwrap_or("?");
        let expanded = nav_rail::<GalleryMessage>("demo_rail")
            .item(
                nav_item("Home")
                    .icon("⌂")
                    .on_click(GalleryMessage::NavRailSelect(0)),
            )
            .item(
                nav_item("Search")
                    .icon("⌕")
                    .on_click(GalleryMessage::NavRailSelect(1)),
            )
            .item(
                nav_item("Library")
                    .icon("☰")
                    .on_click(GalleryMessage::NavRailSelect(2)),
            )
            .item(nav_item("Settings").icon("⚙").disabled(true))
            .selected(sel)
            .collapsed(false)
            .build();
        let collapsed = nav_rail::<GalleryMessage>("demo_rail_c")
            .item(
                nav_item("Home")
                    .icon("⌂")
                    .on_click(GalleryMessage::NavRailSelect(0)),
            )
            .item(
                nav_item("Search")
                    .icon("⌕")
                    .on_click(GalleryMessage::NavRailSelect(1)),
            )
            .item(
                nav_item("Library")
                    .icon("☰")
                    .on_click(GalleryMessage::NavRailSelect(2)),
            )
            .selected(sel.min(2))
            .collapsed(true)
            .build();
        let secs = vec![
            (
                "Anatomy",
                column()
                    .gap(8.0)
                    .child(label("NavRail — vertical destinations"))
                    .child(label("  key, items[], selected, collapsed"))
                    .child(label("  item: label + optional icon / message / disabled"))
                    .build(),
            ),
            (
                "Expanded",
                column()
                    .gap(g)
                    .child(label(format!("Selected: {sel_name} (click items)")))
                    .child(expanded)
                    .build(),
            ),
            (
                "Collapsed",
                column()
                    .gap(g)
                    .child(label("Icons / short labels only — shares selection state"))
                    .child(collapsed)
                    .build(),
            ),
            ("Density", density_demo()),
            ("Long Traditional Chinese Text", long_text_section()),
        ];
        self.build_component_page("NavRail", secs)
    }

    fn sidebar_widget_page(&self) -> WidgetNode<GalleryMessage> {
        let g = spacing(self.density, 8.0);
        let demo = sidebar::<GalleryMessage>("demo_sidebar")
            .width(224.0)
            .header("Explorer")
            .child(button("sb_files", "Files").on_click(GalleryMessage::DpiInfo))
            .child(button("sb_search", "Search").on_click(GalleryMessage::DpiInfo))
            .child(label("— Recent —"))
            .child(label("readme.md"))
            .child(label("main.rs"))
            .build();
        let secs = vec![
            (
                "Anatomy",
                column()
                    .gap(8.0)
                    .child(label("Sidebar — fixed-width panel"))
                    .child(label("  key, width (default 224), header, children[]"))
                    .build(),
            ),
            (
                "Demo",
                column()
                    .gap(g)
                    .child(label("width = 224px, header + body"))
                    .child(demo)
                    .build(),
            ),
            ("Density", density_demo()),
            ("Long Traditional Chinese Text", long_text_section()),
        ];
        self.build_component_page("Sidebar", secs)
    }

    fn tab_bar_page(&self) -> WidgetNode<GalleryMessage> {
        let g = spacing(self.density, 8.0);
        let primary_labels = ["Overview", "Details", "History", "Settings"];
        let zh_labels = ["日", "週", "月"];
        let mut tabs = tab_bar::<GalleryMessage>("demo_tabs").selected(self.tab_bar_selected);
        for (i, label_text) in primary_labels.iter().enumerate() {
            tabs = tabs.item(TabItem::new(*label_text).on_click(GalleryMessage::TabBarSelect(i)));
        }
        let tabs = tabs.build();
        let mut tabs_sel =
            tab_bar::<GalleryMessage>("demo_tabs_2").selected(self.tab_bar_zh_selected);
        for (i, label_text) in zh_labels.iter().enumerate() {
            tabs_sel = tabs_sel
                .item(TabItem::new(*label_text).on_click(GalleryMessage::TabBarZhSelect(i)));
        }
        let tabs_sel = tabs_sel.build();
        let primary_name = primary_labels
            .get(self.tab_bar_selected)
            .copied()
            .unwrap_or("?");
        let zh_name = zh_labels
            .get(self.tab_bar_zh_selected)
            .copied()
            .unwrap_or("?");
        let secs = vec![
            (
                "Anatomy",
                column()
                    .gap(8.0)
                    .child(label("TabBar — horizontal tab strip"))
                    .child(label("  key, tabs[], selected index"))
                    .child(label("  selected tab rendered as [Label]"))
                    .build(),
            ),
            (
                "Demo",
                column()
                    .gap(g)
                    .child(label(format!("selected = {primary_name} (click tabs)")))
                    .child(tabs)
                    .child(label(format!("selected = {zh_name}")))
                    .child(tabs_sel)
                    .build(),
            ),
            ("Density", density_demo()),
            ("Long Traditional Chinese Text", long_text_section()),
        ];
        self.build_component_page("TabBar", secs)
    }

    fn breadcrumb_page(&self) -> WidgetNode<GalleryMessage> {
        let g = spacing(self.density, 8.0);
        let trail = breadcrumb::<GalleryMessage>("demo_bc")
            .segment("Home")
            .segment("Library")
            .segment("Data")
            .segment("表單")
            .build();
        let trail_gt = breadcrumb::<GalleryMessage>("demo_bc_gt")
            .separator(">")
            .segment("Root")
            .segment("src")
            .segment("main.rs")
            .build();
        let secs = vec![
            (
                "Anatomy",
                column()
                    .gap(8.0)
                    .child(label("Breadcrumb — path trail with separators"))
                    .child(label("  key, segments[], separator (default \"/\")"))
                    .build(),
            ),
            (
                "Demo",
                column()
                    .gap(g)
                    .child(label("separator = /"))
                    .child(trail)
                    .child(label("separator = >"))
                    .child(trail_gt)
                    .build(),
            ),
            ("Density", density_demo()),
            ("Long Traditional Chinese Text", long_text_section()),
        ];
        self.build_component_page("Breadcrumb", secs)
    }

    fn overlay_page(&self) -> WidgetNode<GalleryMessage> {
        let name = CATEGORIES[3].pages[self.selected_page.min(3)];
        self.component_page(name)
    }

    fn data_page(&self) -> WidgetNode<GalleryMessage> {
        let page = self.selected_page.min(3);
        let title = CATEGORIES[4].pages[page];
        let body = match page {
            0 => self.tree_demo(),
            1 => self.table_demo(),
            2 => self.datagrid_demo(),
            _ => self.virtual_list_demo(),
        };
        column()
            .gap(spacing(self.density, 16.0))
            .padding(spacing(self.density, 24.0))
            .child(label_with_size(title, 24.0))
            .child(self.data_page_tabs(page))
            .child(separator())
            .child(body)
            .build()
    }

    /// In-category page switcher for Data demos (Tree / Table / DataGrid / VirtualList).
    fn data_page_tabs(&self, active: usize) -> WidgetNode<GalleryMessage> {
        // WidgetKey only accepts &str (not String) — keep static keys.
        const TAB_KEYS: [&str; 4] = ["data_tab_0", "data_tab_1", "data_tab_2", "data_tab_3"];
        let mut tabs = row::<GalleryMessage>().gap(spacing(self.density, 8.0));
        for (i, name) in CATEGORIES[4].pages.iter().enumerate() {
            let mut btn = button::<GalleryMessage>(TAB_KEYS[i], *name);
            if i == active {
                btn = btn.primary();
            }
            tabs = tabs.child(btn.on_click(GalleryMessage::SelectPage(i)));
        }
        tabs.build()
    }

    fn tree_demo(&self) -> WidgetNode<GalleryMessage> {
        let gap = spacing(self.density, 8.0);
        // All nodes start collapsed; Gallery bits drive Tree.expanded HashSet.
        let mut tree_widget = tree::<GalleryMessage>("gallery_tree")
            .indent(20.0)
            .viewport_height(320.0)
            .child(
                TreeNode::new("docs", label("Documents"))
                    .expanded(false)
                    .child(TreeNode::new("docs_readme", label("README.md")))
                    .child(TreeNode::new("docs_guide", label("Getting Started")))
                    .child(
                        TreeNode::new("docs_zh", label("繁體中文說明"))
                            .expanded(false)
                            .child(TreeNode::new("docs_zh_ime", label("IME 輸入注意事項")))
                            .child(TreeNode::new("docs_zh_a11y", label("無障礙指南"))),
                    ),
            )
            .child(
                TreeNode::new("images", label("Images"))
                    .expanded(false)
                    .child(TreeNode::new("img_logo", label("logo.png")))
                    .child(TreeNode::new("img_banner", label("banner.webp"))),
            )
            .child(
                TreeNode::new("code", label("Code"))
                    .expanded(false)
                    .child(
                        TreeNode::new("code_src", label("src/"))
                            .expanded(false)
                            .child(TreeNode::new("code_main", label("main.rs")))
                            .child(TreeNode::new("code_lib", label("lib.rs"))),
                    )
                    .child(TreeNode::new("code_toml", label("Cargo.toml"))),
            );
        for (i, &key) in TREE_EXPAND_KEYS.iter().enumerate() {
            if self.tree_expanded & (1u32 << i) != 0 {
                tree_widget.expanded.insert(WidgetKey::from(key));
            }
        }
        if let Some(sel) = self.tree_selected {
            tree_widget.selected = Some(WidgetKey::from(sel));
        }
        let sel_label = self.tree_selected.unwrap_or("(none)");
        column()
            .gap(gap)
            .child(label(
                "Hierarchical Tree with expand/collapse. Nested categories demo:",
            ))
            .child(label(
                "Click row to select · click again (or chevron zone) to toggle · ←/→ collapse/expand",
            ))
            .child(label(format!("Selected: {sel_label}")))
            .child(tree_widget.build())
            .child(label(
                "State lives on Gallery; Tree rebuilds each frame from expand bits + selection.",
            ))
            .build()
    }

    fn table_demo(&self) -> WidgetNode<GalleryMessage> {
        let gap = spacing(self.density, 8.0);
        let headers = ["Name", "Status", "Owner", "Updated"];
        let widths = [160.0_f32, 100.0, 100.0, 120.0];
        let mut tbl = table::<GalleryMessage>("gallery_table")
            .sticky_header(true)
            .row_height(28.0);

        for (ci, (header, width)) in headers.iter().zip(widths.iter()).enumerate() {
            let mut title = (*header).to_string();
            if self.table_sort_col == Some(ci) {
                title.push_str(if self.table_sort_asc { " ↑" } else { " ↓" });
            }
            let mut col = TableColumn::new(
                ["name", "status", "owner", "updated"][ci],
                label(title),
                *width,
            )
            .sortable(true)
            .resizable(true);
            if self.table_sort_col == Some(ci) {
                col = col.sort_indicator(if self.table_sort_asc {
                    SortDirection::Ascending
                } else {
                    SortDirection::Descending
                });
            }
            tbl = tbl.column(col);
        }

        let order = table_display_order(self.table_sort_col, self.table_sort_asc);
        let display_selected = self
            .table_selected_row
            .and_then(|orig| order.iter().position(|&o| o == orig));

        for &orig in &order {
            let cells = table_row_cells(orig);
            tbl = tbl.add_row(TableRow::new(vec![
                label(cells[0].clone()),
                label(cells[1].clone()),
                label(cells[2].clone()),
                label(cells[3].clone()),
            ]));
        }

        let mut node = tbl.build();
        if let WidgetNode::Table(ref mut t) = node {
            t.sort_column = self.table_sort_col;
            t.sort_ascending = self.table_sort_asc;
            t.selected_row = display_selected;
        }

        let sort_info = match self.table_sort_col {
            Some(c) => format!(
                "sort = {} ({})",
                headers[c],
                if self.table_sort_asc { "asc" } else { "desc" }
            ),
            None => "sort = none".into(),
        };
        let sel_info = match self.table_selected_row {
            Some(i) => format!("selected row = Project {i:02}"),
            None => "selected row = none".into(),
        };

        column()
            .gap(gap)
            .child(label(
                "Table with sticky header, 4 columns, and 28 sample rows.",
            ))
            .child(label(
                "Click header to sort · click row to select. State lives on Gallery.",
            ))
            .child(label(format!("{sort_info} · {sel_info}")))
            .child(node)
            .build()
    }

    fn datagrid_demo(&self) -> WidgetNode<GalleryMessage> {
        let gap = spacing(self.density, 8.0);
        let mut grid = datagrid::<GalleryMessage>("gallery_datagrid")
            .frozen_cols(1)
            .frozen_rows(0)
            .default_row_height(28.0)
            .default_col_width(120.0)
            .viewport_width(640.0)
            .viewport_height(280.0)
            .column(
                DataGridColumn::new("id", label("ID"), 72.0)
                    .frozen(true)
                    .sortable(true),
            )
            .column(DataGridColumn::new("product", label("Product"), 140.0).sortable(true))
            .column(DataGridColumn::new("region", label("Region"), 100.0))
            .column(DataGridColumn::new("qty", label("Qty"), 72.0))
            .column(DataGridColumn::new("total", label("Total"), 100.0));

        let regions = ["APAC", "EMEA", "AMER", "JP"];
        let products = ["Widget A", "Widget B", "Gadget C", "Module D", "Kit E"];
        for i in 0..12 {
            let id = format!("R{i:03}");
            let product = products[i % products.len()];
            let region = regions[i % regions.len()];
            let qty = format!("{}", 10 + i * 3);
            let total = format!("${}.00", 120 + i * 17);
            grid = grid.add_row(
                DataGridRow::new(vec![
                    label(id),
                    label(product),
                    label(region),
                    label(qty),
                    label(total),
                ])
                .row_number(format!("{}", i + 1)),
            );
        }
        // Light merge demo: first data row product+region span (colspan 2)
        grid = grid.merge_cells(0, 1, 2, 1);

        column()
            .gap(gap)
            .child(label(
                "DataGrid with frozen first column, 5 columns × 12 rows, and one cell merge.",
            ))
            .child(label(
                "Frozen cols stay visible during horizontal scroll; merge is declarative (layout still shows cell slots).",
            ))
            .child(grid.build())
            .build()
    }

    fn virtual_list_demo(&self) -> WidgetNode<GalleryMessage> {
        let gap = spacing(self.density, 8.0);

        let mut list = virtual_list::<GalleryMessage>("gallery_vlist")
            .item_height(Some(VLIST_ITEM_HEIGHT))
            .viewport_height(VLIST_VIEWPORT_H)
            .overscan(4)
            .scroll_offset(self.vlist_scroll);

        for i in 0..VLIST_ITEM_COUNT {
            list = list.child(label(format!("Item {i}: 項目內容 demo")));
        }

        column()
            .gap(gap)
            .child(label_with_size("VirtualList", 16.0))
            .child(label(
                "Fixed item height path · only the viewport window (+ overscan) is painted.",
            ))
            .child(label(format!(
                "{} items × {}px · viewport {}px · overscan 4 · scroll {:.0}px",
                VLIST_ITEM_COUNT, VLIST_ITEM_HEIGHT, VLIST_VIEWPORT_H, self.vlist_scroll
            )))
            .child(list.build())
            .child(label(
                "Hover the list and scroll to move VirtualList; scroll outside moves the page.",
            ))
            .build()
    }

    fn patterns_page(&self) -> WidgetNode<GalleryMessage> {
        match self.selected_page {
            0 => self.settings_page(),
            1 => self.dashboard_page(),
            2 => self.ide_layout_page(),
            3 => self.speaktype_page(),
            _ => label("Unknown template"),
        }
    }

    fn accessibility_page(&self) -> WidgetNode<GalleryMessage> {
        let name = CATEGORIES[6].pages[self.selected_page.min(3)];
        self.component_page(name)
    }

    fn stress_tests_page(&self) -> WidgetNode<GalleryMessage> {
        let name = CATEGORIES[7].pages[self.selected_page.min(3)];
        self.component_page(name)
    }

    // ── Component Page Template ─────────────────────────────────────────────

    /// Build a quick component page with standard sections.
    fn component_page(&self, title: &str) -> WidgetNode<GalleryMessage> {
        let secs = standard_component_sections();
        self.build_component_page(title, secs)
    }

    fn build_component_page(
        &self,
        title: &str,
        sections: Vec<(&'static str, WidgetNode<GalleryMessage>)>,
    ) -> WidgetNode<GalleryMessage> {
        let mut page = column::<GalleryMessage>()
            .gap(spacing(self.density, 28.0))
            .padding(spacing(self.density, 24.0));
        page = page.child(label_with_size(title, 24.0));
        page = page.child(separator());
        for (section_title, content) in sections {
            page = page.child(self.page_section(section_title, content));
        }
        page.build()
    }

    fn page_section(
        &self,
        title: &str,
        content: WidgetNode<GalleryMessage>,
    ) -> WidgetNode<GalleryMessage> {
        column()
            .gap(spacing(self.density, 10.0))
            .child(label_with_size(title, 16.0))
            .child(separator())
            .child(content)
            .build()
    }

    // ── 4 Reference Template Pages ──────────────────────────────────────────

    /// Settings: sidebar-style with form sections and a danger zone.
    fn settings_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(spacing(self.density, 16.0))
            .padding(spacing(self.density, 24.0))
            .child(label_with_size("Settings", 24.0))
            .child(label("Configure your application preferences"))
            .child(separator())
            .child(
                column()
                    .gap(8.0)
                    .child(label_with_size("General", 18.0))
                    .child(self.sf("Username", "eda"))
                    .child(self.sf("Language", "繁體中文"))
                    .child(self.sf("Theme", "System"))
                    .build(),
            )
            .child(separator())
            .child(
                column()
                    .gap(8.0)
                    .child(label_with_size("Notifications", 18.0))
                    .child(label("☐  Email notifications"))
                    .child(label("☐  Push notifications"))
                    .child(label("☑  Weekly digest"))
                    .build(),
            )
            .child(separator())
            .child(
                column()
                    .gap(8.0)
                    .padding(16.0)
                    .child(label_with_size("Danger Zone", 18.0))
                    .child(label("These actions cannot be undone."))
                    .child(
                        button("delete_account", "Delete Account")
                            .variant(ButtonVariant::Danger)
                            .on_click(GalleryMessage::DpiInfo),
                    )
                    .build(),
            )
            .build()
    }

    /// Dashboard: KPI row, insight card, and activity list.
    fn dashboard_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(spacing(self.density, 16.0))
            .padding(spacing(self.density, 24.0))
            .child(
                row()
                    .gap(16.0)
                    .child(label_with_size("Dashboard", 24.0))
                    .child(button("refresh_btn", "↻ Refresh").on_click(GalleryMessage::DpiInfo))
                    .build(),
            )
            .child(
                row()
                    .gap(12.0)
                    .child(self.kpi_card("$48,290", "Revenue"))
                    .child(self.kpi_card("2,847", "Users"))
                    .child(self.kpi_card("1,203", "Active"))
                    .child(self.kpi_card("+12.5%", "Growth"))
                    .build(),
            )
            .child(
                column()
                    .gap(8.0)
                    .child(label_with_size("Revenue Overview", 16.0))
                    .child(label(
                        "[ Chart placeholder — area chart would render here ]",
                    ))
                    .build(),
            )
            .child(
                column()
                    .gap(6.0)
                    .child(label_with_size("Recent Activity", 16.0))
                    .child(label("•  New user registered — 2m ago"))
                    .child(label("•  Order #3842 completed — 15m ago"))
                    .child(label("•  Server deployment finished — 1h ago"))
                    .child(label("•  Payment received — 2h ago"))
                    .build(),
            )
            .build()
    }

    /// Desktop IDE: menu bar, nav rail, file tree, editor, terminal, status bar.
    fn ide_layout_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(0.0)
            .child(
                row()
                    .gap(12.0)
                    .padding(8.0)
                    .child(label("File"))
                    .child(label("Edit"))
                    .child(label("View"))
                    .child(label("Help"))
                    .build(),
            )
            .child(separator())
            .child(
                column()
                    .gap(0.0)
                    .child(
                        row()
                            .gap(0.0)
                            .child(self.ide_nav_rail())
                            .child(self.ide_file_tree())
                            .child(self.ide_editor())
                            .build(),
                    )
                    .child(self.ide_terminal())
                    .build(),
            )
            .child(separator())
            .child(
                row()
                    .gap(16.0)
                    .padding(6.0)
                    .child(label("Ln 42, Col 8"))
                    .child(label("Rust"))
                    .child(label("main"))
                    .child(label("UTF-8"))
                    .build(),
            )
            .build()
    }

    fn ide_nav_rail(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(8.0)
            .padding(8.0)
            .child(label("📁"))
            .child(label("🔍"))
            .child(label("🔧"))
            .child(label("📦"))
            .child(label("🧪"))
            .build()
    }

    fn ide_file_tree(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(4.0)
            .padding(8.0)
            .child(label("src/"))
            .child(label("  ├─ main.rs"))
            .child(label("  ├─ lib.rs"))
            .child(label("  └─ render/"))
            .child(label("Cargo.toml"))
            .child(label("README.md"))
            .build()
    }

    fn ide_editor(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(4.0)
            .padding(12.0)
            .child(label_with_size("fn main() {", 14.0))
            .child(label_with_size("    let msg = \"Hello, AcmeUI!\";", 14.0))
            .child(label_with_size("    println!(\"{}\", msg);", 14.0))
            .child(label_with_size("}", 14.0))
            .build()
    }

    fn ide_terminal(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(2.0)
            .padding(6.0)
            .child(label("$ cargo build --release"))
            .child(label("   Compiling acme-core v0.1.0"))
            .child(label("   Compiling acme-widgets v0.1.0"))
            .child(label("    Finished release [optimized] target"))
            .build()
    }

    /// SpeakType: recording status, provider dots, transcript, big record button.
    fn speaktype_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(spacing(self.density, 16.0))
            .padding(spacing(self.density, 24.0))
            .child(
                row()
                    .gap(16.0)
                    .child(label("⚙ Settings"))
                    .child(label("📊 History"))
                    .child(label("📝 Notes"))
                    .build(),
            )
            .child(separator())
            .child(
                row()
                    .gap(8.0)
                    .child(label("🔴 Recording"))
                    .child(label("00:42"))
                    .build(),
            )
            .child(
                row()
                    .gap(16.0)
                    .child(label("🟢 OpenAI"))
                    .child(label("🟡 Anthropic"))
                    .child(label("⚪ Local"))
                    .build(),
            )
            .child(label("⌘⇧R  Start/Stop  ·  ⌘⇧S  Save transcript"))
            .child(
                column()
                    .gap(4.0)
                    .child(label_with_size("Recent Transcript", 16.0))
                    .child(label(
                        "Hello, this is a test of the speech recognition system.",
                    ))
                    .child(label("The quick brown fox jumps over the lazy dog."))
                    .child(label("今日的天氣真好，適合出門散步。"))
                    .build(),
            )
            .child(
                button("record_btn", "⏺ Start Recording")
                    .primary()
                    .on_click(GalleryMessage::DpiInfo),
            )
            .build()
    }

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

    // ── Long text widget ────────────────────────────────────────────────────

    /// Reusable widget that renders a moderately‑wide text block to demonstrate
    /// long‑string handling.
    #[allow(dead_code)]
    fn long_text_widget(text: &str) -> WidgetNode<GalleryMessage> {
        label_with_size(text, 14.0)
    }
}

// ── Free‑standing page‑section helpers ─────────────────────────────────────

fn standard_component_sections() -> Vec<(&'static str, WidgetNode<GalleryMessage>)> {
    vec![
        ("Anatomy", anatomy_diagram()),
        ("Variants", variants_demo()),
        ("Sizes", sizes_demo()),
        ("Light / Dark", light_dark_demo()),
        ("Density", density_demo()),
        ("Keyboard Behavior", keyboard_behavior()),
        ("Accessibility Properties", accessibility_props()),
        ("Long Traditional Chinese Text", long_text_section()),
        ("Screenshot Mode", screenshot_info()),
    ]
}

fn anatomy_diagram() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)
        .child(label("── Component Anatomy ──"))
        .child(label("┌─────────────────────┐"))
        .child(label("│  Container / Root   │"))
        .child(label("│  ├─ Label / Content  │"))
        .child(label("│  └─ (children...)    │"))
        .child(label("└─────────────────────┘"))
        .build()
}

fn variants_demo() -> WidgetNode<GalleryMessage> {
    row()
        .gap(8.0)
        .child(
            button("v_primary", "Primary")
                .primary()
                .on_click(GalleryMessage::DpiInfo),
        )
        .child(button("v_secondary", "Secondary").on_click(GalleryMessage::DpiInfo))
        .child(
            button("v_ghost", "Ghost")
                .variant(ButtonVariant::Ghost)
                .on_click(GalleryMessage::DpiInfo),
        )
        .child(
            button("v_danger", "Danger")
                .variant(ButtonVariant::Danger)
                .on_click(GalleryMessage::DpiInfo),
        )
        .build()
}

fn sizes_demo() -> WidgetNode<GalleryMessage> {
    column()
        .gap(10.0)
        .child(label("XS  ·  S  ·  M  ·  L  ·  XL"))
        .child(
            row()
                .gap(8.0)
                .child(label("[XS btn]"))
                .child(label("[S btn]"))
                .child(label("[M btn — default]"))
                .child(label("[L btn]"))
                .child(label("[XL btn]"))
                .build(),
        )
        .build()
}

fn light_dark_demo() -> WidgetNode<GalleryMessage> {
    row()
        .gap(16.0)
        .child(
            column()
                .gap(8.0)
                .child(label_with_size("☀ Light", 16.0))
                .child(label("Component in light theme"))
                .build(),
        )
        .child(
            column()
                .gap(8.0)
                .child(label_with_size("🌙 Dark", 16.0))
                .child(label("Component in dark theme"))
                .build(),
        )
        .build()
}

fn density_demo() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)
        .child(label("Compact (0.75×) vs Comfortable (1.0×)"))
        .child(label("Toggle via toolbar button above."))
        .build()
}

fn keyboard_behavior() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)
        .child(label("Space  ·  Activate focused widget"))
        .child(label("Enter  ·  Submit / Confirm"))
        .child(label("Tab    ·  Move focus forward"))
        .child(label("⇧+Tab ·  Move focus backward"))
        .child(label("Esc    ·  Dismiss overlay / cancel"))
        .build()
}

fn accessibility_props() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)
        .child(label(
            "role=\"button\"  ·  aria-label=\"...\"  ·  tabindex=\"0\"",
        ))
        .child(label("aria-disabled  ·  aria-expanded  ·  aria-controls"))
        .child(label(
            "Focus ring visible  ·  Screen‑reader labels via AccessKit",
        ))
        .build()
}

fn long_text_section() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)
        .child(label_with_size("Long Traditional Chinese string:", 14.0))
        .child(label_with_size(LONG_CHINESE_TEXT, 14.0))
        .build()
}

fn screenshot_info() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)
        .child(label("Screenshot sizes: 1280×800 · 1024×700 · 800×600"))
        .child(label("Toggle theme & density via toolbar before capture."))
        .build()
}

/// Apply density scale factor to a base spacing value.
fn spacing(density: Density, base: f32) -> f32 {
    base * density.spacing_scale()
}

/// Static-ish table cell text for original row `i`, column `col`.
fn table_cell_text(row: usize, col: usize) -> String {
    const OWNERS: &[&str] = &["Ada", "Lin", "Sam", "Mei", "Kai", "Zoe"];
    const STATUSES: &[&str] = &["Active", "Draft", "Review", "Done", "Blocked"];
    match col {
        0 => format!("Project {row:02}"),
        1 => STATUSES[row % STATUSES.len()].to_string(),
        2 => OWNERS[row % OWNERS.len()].to_string(),
        _ => format!("2026-0{}-{:02}", (row % 9) + 1, (row % 28) + 1),
    }
}

fn table_row_cells(row: usize) -> [String; 4] {
    [
        table_cell_text(row, 0),
        table_cell_text(row, 1),
        table_cell_text(row, 2),
        table_cell_text(row, 3),
    ]
}

/// Display order of original row indices under the current sort.
fn table_display_order(sort_col: Option<usize>, sort_asc: bool) -> Vec<usize> {
    let mut order: Vec<usize> = (0..TABLE_ROW_COUNT).collect();
    if let Some(col) = sort_col {
        order.sort_by(|&a, &b| {
            let cmp = table_cell_text(a, col).cmp(&table_cell_text(b, col));
            if sort_asc { cmp } else { cmp.reverse() }
        });
    }
    order
}

fn point_in_rect(x: f32, y: f32, rect: [f32; 4]) -> bool {
    x >= rect[0] && x <= rect[0] + rect[2] && y >= rect[1] && y <= rect[1] + rect[3]
}

/// Map a content-space layout rect into window space using page scroll.
fn scrolled_hit_rect(rect: [f32; 4], scroll_y: f32) -> [f32; 4] {
    [rect[0], rect[1] - scroll_y, rect[2], rect[3]]
}

// ── Event Handling ──────────────────────────────────────────────────────────

impl Gallery {
    fn hit(&self) -> Option<usize> {
        // Prefer the last (top-most / deepest) match so nested content wins over chrome
        // when rects overlap; content hits are appended after chrome buttons.
        self.button_info
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, hr)| {
                let r = if hr.scrolled {
                    // Must be visible inside the page scroll viewport.
                    if self.scroll_clip_rect[2] > 0.0
                        && !point_in_rect(self.cursor.0, self.cursor.1, self.scroll_clip_rect)
                    {
                        return None;
                    }
                    scrolled_hit_rect(hr.rect, self.scroll)
                } else {
                    hr.rect
                };
                if point_in_rect(self.cursor.0, self.cursor.1, r) {
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
        let message = hr.message;
        match message {
            GalleryMessage::ToggleTheme => {
                self.dark = !self.dark;
                true
            }
            GalleryMessage::ToggleDensity => {
                self.density = self.density.toggle();
                true
            }
            GalleryMessage::ToggleFocusRings => {
                self.show_focus_rings = !self.show_focus_rings;
                true
            }
            GalleryMessage::SelectCategory(i) => {
                let changed = self.selected_category != i;
                self.selected_category = i;
                self.selected_page = 0;
                if changed {
                    self.scroll = 0.0;
                }
                true
            }
            GalleryMessage::SelectPage(i) => {
                self.selected_page = i;
                self.scroll = 0.0;
                true
            }
            GalleryMessage::NavRailSelect(i) => {
                self.nav_rail_selected = i;
                true
            }
            GalleryMessage::TabBarSelect(i) => {
                self.tab_bar_selected = i;
                true
            }
            GalleryMessage::TabBarZhSelect(i) => {
                self.tab_bar_zh_selected = i;
                true
            }
            GalleryMessage::TreeSelectKey(key) => {
                // Second click on the same expandable node toggles expand.
                if self.tree_selected == Some(key) {
                    self.tree_toggle_expanded(key);
                }
                self.tree_selected = Some(key);
                true
            }
            GalleryMessage::TreeToggleKey(key) => {
                self.tree_toggle_expanded(key);
                self.tree_selected = Some(key);
                true
            }
            GalleryMessage::TableSort(col) => {
                if self.table_sort_col == Some(col) {
                    self.table_sort_asc = !self.table_sort_asc;
                } else {
                    self.table_sort_col = Some(col);
                    self.table_sort_asc = true;
                }
                true
            }
            GalleryMessage::TableSelectRow(orig) => {
                self.table_selected_row = Some(orig);
                true
            }
            GalleryMessage::FocusDemo | GalleryMessage::DpiInfo => true,
        }
    }

    /// Recompute window-client IME caret rect from field origin + content-local
    /// caret geometry. Called after focus/text changes and each text-input frame.
    fn refresh_ime_caret_cache(&mut self) {
        if !self.text_input.focused {
            self.ime_caret_window_rect = None;
            return;
        }
        let [fx, fy, fw, fh] = self.text_input_rect;
        if fw <= 0.0 || fh <= 0.0 {
            self.ime_caret_window_rect = None;
            return;
        }
        let theme = if self.dark {
            Theme::dark()
        } else {
            Theme::light()
        };
        let font_size = theme.typography.body;
        let style = TextStyle {
            font_size,
            line_height: font_size * theme.typography.line_height,
            ..TextStyle::default()
        };
        // Content origin matches render_text_input (field + padding).
        let padding = theme.spacing.px2;
        let [cx, cy, cw, ch] =
            self.text_input
                .ime_caret_area(&mut self.fonts, &style, self.last_scale_factor);
        self.ime_caret_window_rect = Some([fx + padding + cx, fy + padding + cy, cw, ch.max(1.0)]);
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
                // VirtualList captures wheel when the cursor is over its viewport.
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
                // Tree keyboard: ←/→ collapse/expand selected; Home/End first/last visible.
                // (PlatformKey has no ArrowUp/Down yet — pointer moves selection.)
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
                        // Last key among currently visible leaves — best-effort.
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
        // The renderer rebuilt empty GPU atlases after device loss; drop the CPU
        // atlas so the next frame re-uploads every glyph instead of referencing
        // blank texture regions.
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

        // ── 9. Collect hit regions (buttons first, then Tree/Table custom hits) ──
        // Button-only DFS order must stay stable so paint btn_idx (chrome=0..10,
        // content from 11) stays aligned with button_info indices.
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

        // ── 8. Scroll metrics ──
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

        // ── 10. Build frame ──
        let mut frame = Frame {
            clear: rgba(colors.background),
            ..Frame::default()
        };

        // ── 11. Sidebar background ──
        if let Some(r) = snapshot.get(ids.sidebar) {
            frame.quads.push(quad_rect(
                [r.x, r.y, r.width, r.height],
                colors.surface,
                0.0,
                1.0,
                colors.border,
            ));
        }

        // ── 12. Sidebar title ──
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

        // ── 13. Sidebar category buttons ──
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

        // ── 14. Toolbar background ──
        if let Some(r) = snapshot.get(ids.toolbar) {
            frame.quads.push(quad_rect(
                [r.x, r.y, r.width, r.height],
                colors.surface,
                0.0,
                1.0,
                colors.border,
            ));
        }

        // ── 15. Toolbar buttons ──
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

        // ── 16. Page content inside scroll view ──
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

        // ── 17. Text Input / IME override on the TextInput page ──
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
                    // Keep IME candidate rect in sync with the rendered caret.
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
            // Not on the TextInput page — clear field geometry / IME cache.
            self.text_input_rect = [0.0; 4];
            if self.ime_caret_window_rect.is_some() {
                self.ime_caret_window_rect = None;
            }
        }

        frame
    }
}

// ── Layout ID Extractors ────────────────────────────────────────────────────
//
// These walk the *same* tree structure produced by to_layout(NodeId::new(1)).
// Child indices are structural positions, not magic numbers.

struct GalleryNodeIds {
    #[allow(dead_code)]
    _root: NodeId,
    sidebar: NodeId,
    sidebar_label: NodeId,
    _sidebar_separator: NodeId,
    sidebar_buttons: [NodeId; 8],
    #[allow(dead_code)]
    content_area: NodeId,
    toolbar: NodeId,
    toolbar_buttons: [NodeId; 3],
    scroll_view: NodeId,
}

fn extract_gallery_ids(root: &LayoutNode) -> GalleryNodeIds {
    let sb = &root.children[0];
    let ca = &root.children[1];
    let tb = &ca.children[0];
    GalleryNodeIds {
        _root: root.id,
        sidebar: sb.id,
        sidebar_label: sb.children[0].id,
        _sidebar_separator: sb.children[1].id,
        sidebar_buttons: [
            sb.children[2].id,
            sb.children[3].id,
            sb.children[4].id,
            sb.children[5].id,
            sb.children[6].id,
            sb.children[7].id,
            sb.children[8].id,
            sb.children[9].id,
        ],
        content_area: ca.id,
        toolbar: tb.id,
        toolbar_buttons: [tb.children[0].id, tb.children[1].id, tb.children[2].id],
        scroll_view: ca.children[1].id,
    }
}

/// Walk the widget+layout tree to find the text‑input marker label.
fn find_text_input_marker(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
) -> Option<NodeId> {
    match widget {
        WidgetNode::Label(l) if l.text == TEXT_INPUT_MARKER => Some(layout.id),
        _ => {
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                if let Some(id) = find_text_input_marker(w, l) {
                    return Some(id);
                }
            }
            None
        }
    }
}

// ── Style Application ───────────────────────────────────────────────────────

fn apply_gallery_styles(root: &mut LayoutNode, width: f32, height: f32) {
    // Root: Row fills window
    root.style = LayoutStyle {
        kind: LayoutKind::Row,
        width: Length::px(width),
        height: Length::px(height),
        gap: 0.0,
        ..Default::default()
    };

    // Sidebar: fixed width, full height
    let sb = &mut root.children[0];
    sb.style = LayoutStyle {
        kind: LayoutKind::Column,
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
    // Category buttons
    for i in 2..=9 {
        sb.children[i].style.width = Length::px(SIDEBAR_WIDTH - 24.0);
        sb.children[i].style.height = Length::px(40.0);

    }

    // Content area: fills remaining width
    let cw = (width - SIDEBAR_WIDTH).max(400.0);
    let ca = &mut root.children[1];
    ca.style = LayoutStyle {
        kind: LayoutKind::Column,
        width: Length::px(cw),
        height: Length::px(height),
        gap: 0.0,
        ..Default::default()
    };

    // Toolbar
    let tb = &mut ca.children[0];
    tb.style = LayoutStyle {
        kind: LayoutKind::Row,
        width: Length::px(cw),
        height: Length::px(TOOLBAR_HEIGHT),
        gap: 8.0,
        padding: acme_layout::Edges {
            left: 16.0,
            right: 16.0,
            top: 8.0,
            bottom: 8.0,
        },
        ..Default::default()
    };
    for btn in &mut tb.children {
        btn.style.width = Length::px(130.0);
        btn.style.height = Length::px(32.0);
    }

    // Scroll view
    let sh = (height - TOOLBAR_HEIGHT).max(100.0);
    let sv = &mut ca.children[1];
    sv.style = LayoutStyle {
        kind: LayoutKind::Column,
        width: Length::px(cw),
        height: Length::px(sh),
        overflow: Overflow::Scroll,
        flex_grow: 1.0,
        ..Default::default()
    };

    // Page-content first child — full width
    if let Some(content) = sv.children.first_mut() {
        content.style.width = Length::px(cw);
    }
}

// ── Render Helpers ──────────────────────────────────────────────────────────

/// DFS walk of page content inside the scroll view.
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
    focused: usize,
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
    show_focus_rings: bool,
) {
    let colors = theme.colors;
    match widget {
        WidgetNode::Label(l) => {
            if l.text == TEXT_INPUT_MARKER {
                return;
            }
            if let Some(rect) = snapshot.get(layout.id) {
                let fs = l.font_size.unwrap_or(theme.typography.body);
                // Vertical center within the label rect using theme line height.
                let line_h = fs * theme.typography.line_height;
                let y_text = rect.y - scroll_y + (rect.height - line_h).max(0.0) * 0.5;
                let text_color = l.color.unwrap_or(colors.foreground);
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
                    focused: focused == idx,
                };
                let resolved = btn.resolve_style(theme, st);
                frame.quads.push(quad_rect(
                    [rect.x, y, rect.width, rect.height],
                    resolved.background,
                    theme.radii.md,
                    if st.focused && show_focus_rings {
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
        WidgetNode::Tooltip(t) => {
            render_content(
                frame,
                &t.child,
                layout,
                snapshot,
                theme,
                scale,
                scroll_y,
                clip,
                btn_idx,
                hovered,
                pressed,
                focused,
                fonts,
                atlas,
                show_focus_rings,
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
                focused,
                fonts,
                atlas,
                show_focus_rings,
            );
        }
        // Tree: children() is empty; layout leaves pair with visible_nodes().
        WidgetNode::Tree(t) => {
            let visible = t.visible_nodes();
            for (node, child_layout) in visible.iter().zip(layout.children.iter()) {
                if let Some(rect) = snapshot.get(child_layout.id) {
                    let y = rect.y - scroll_y;
                    let selected = t.selected.as_ref() == Some(&node.key);
                    if selected {
                        frame.quads.push(quad_rect(
                            [rect.x, y, rect.width.max(1.0), rect.height.max(1.0)],
                            colors.ghost_hover,
                            0.0,
                            0.0,
                            colors.ghost_hover,
                        ));
                    }
                    // Expand/collapse chevron affordance for parents.
                    if node.has_children {
                        let mark = if node.expanded { "▾" } else { "▸" };
                        add_text(
                            fonts,
                            atlas,
                            frame,
                            mark,
                            ([rect.x + 2.0, y + 2.0], theme.typography.body),
                            colors.muted_foreground,
                            scale,
                            Some(clip),
                            theme.typography.line_height,
                        );
                    }
                }
                paint_label_like(
                    frame,
                    &node.content,
                    child_layout,
                    snapshot,
                    theme,
                    scale,
                    scroll_y,
                    clip,
                    fonts,
                    atlas,
                );
            }
        }
        // Table: layout is header-row + data-row containers (not flat all_cells).
        WidgetNode::Table(t) => {
            let mut row_i = 0usize;
            if !t.columns.is_empty() {
                if let Some(header_row) = layout.children.get(row_i) {
                    // Header background
                    if let Some(hr) = snapshot.get(header_row.id) {
                        let y = hr.y - scroll_y;
                        frame.quads.push(quad_rect(
                            [hr.x, y, hr.width.max(1.0), hr.height.max(1.0)],
                            colors.surface,
                            0.0,
                            1.0,
                            colors.border,
                        ));
                    }
                    for (col, cell_layout) in t.columns.iter().zip(header_row.children.iter()) {
                        paint_label_like(
                            frame,
                            &col.header,
                            cell_layout,
                            snapshot,
                            theme,
                            scale,
                            scroll_y,
                            clip,
                            fonts,
                            atlas,
                        );
                    }
                }
                row_i += 1;
            }
            for (display_i, data_row) in t.rows.iter().enumerate() {
                let Some(row_layout) = layout.children.get(row_i) else {
                    break;
                };
                if t.selected_row == Some(display_i)
                    && let Some(rr) = snapshot.get(row_layout.id)
                {
                    let y = rr.y - scroll_y;
                    frame.quads.push(quad_rect(
                        [rr.x, y, rr.width.max(1.0), rr.height.max(1.0)],
                        colors.ghost_hover,
                        0.0,
                        0.0,
                        colors.ghost_hover,
                    ));
                }
                for (cell, cell_layout) in data_row.cells.iter().zip(row_layout.children.iter()) {
                    paint_label_like(
                        frame,
                        cell,
                        cell_layout,
                        snapshot,
                        theme,
                        scale,
                        scroll_y,
                        clip,
                        fonts,
                        atlas,
                    );
                }
                row_i += 1;
            }
        }
        // DataGrid: same row/column container layout as Table.
        WidgetNode::DataGrid(g) => {
            let mut row_i = 0usize;
            if !g.columns.is_empty() {
                if let Some(header_row) = layout.children.get(row_i) {
                    for (col, cell_layout) in g.columns.iter().zip(header_row.children.iter()) {
                        paint_label_like(
                            frame,
                            &col.header,
                            cell_layout,
                            snapshot,
                            theme,
                            scale,
                            scroll_y,
                            clip,
                            fonts,
                            atlas,
                        );
                    }
                }
                row_i += 1;
            }
            for data_row in &g.rows {
                let Some(row_layout) = layout.children.get(row_i) else {
                    break;
                };
                for (cell, cell_layout) in data_row.cells.iter().zip(row_layout.children.iter()) {
                    paint_label_like(
                        frame,
                        cell,
                        cell_layout,
                        snapshot,
                        theme,
                        scale,
                        scroll_y,
                        clip,
                        fonts,
                        atlas,
                    );
                }
                row_i += 1;
            }
        }
        // VirtualList: to_layout emits an empty container; paint the visible window.
        WidgetNode::VirtualList(v) => {
            if let Some(rect) = snapshot.get(layout.id) {
                let (first, last) = v.visible_range();
                let item_h = v.item_height.unwrap_or(32.0).max(1.0);
                // Clip drawing to the list viewport (also honor outer scroll clip).
                let list_clip = [
                    rect.x.max(clip[0]),
                    (rect.y - scroll_y).max(clip[1]),
                    rect.width.min(clip[2]),
                    rect.height.min(clip[3]),
                ];
                for i in first..last {
                    let Some(child) = v.children.get(i) else {
                        break;
                    };
                    let y = rect.y + (i as f32 * item_h) - v.scroll_offset - scroll_y;
                    if let WidgetNode::Label(l) = child {
                        add_text(
                            fonts,
                            atlas,
                            frame,
                            &l.text,
                            (
                                [rect.x + 4.0, y + 2.0],
                                l.font_size.unwrap_or(theme.typography.body),
                            ),
                            colors.foreground,
                            scale,
                            Some(list_clip),
                            theme.typography.line_height,
                        );
                    }
                }
            }
        }
        _ => {
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                render_content(
                    frame,
                    w,
                    l,
                    snapshot,
                    theme,
                    scale,
                    scroll_y,
                    clip,
                    btn_idx,
                    hovered,
                    pressed,
                    focused,
                    fonts,
                    atlas,
                    show_focus_rings,
                );
            }
        }
    }
}

/// Paint a Label (or nested Label-in-container) at a layout leaf position.
#[allow(clippy::too_many_arguments)]
fn paint_label_like(
    frame: &mut Frame,
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &acme_layout::LayoutSnapshot,
    theme: &Theme,
    scale: f32,
    scroll_y: f32,
    clip: [f32; 4],
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
) {
    match widget {
        WidgetNode::Label(l) => {
            if l.text == TEXT_INPUT_MARKER {
                return;
            }
            if let Some(rect) = snapshot.get(layout.id) {
                let fs = l.font_size.unwrap_or(theme.typography.body);
                // Vertical center within the label rect using theme line height.
                let line_h = fs * theme.typography.line_height;
                let y_text = rect.y - scroll_y + (rect.height - line_h).max(0.0) * 0.5;
                add_text(
                    fonts,
                    atlas,
                    frame,
                    &l.text,
                    ([rect.x + 4.0, y_text], fs),
                    theme.colors.foreground,
                    scale,
                    Some(clip),
                    theme.typography.line_height,
                );
            }
        }
        other => {
            // Containers nested inside cells: walk widget children against layout children.
            let wc = other.children();
            if wc.is_empty() {
                return;
            }
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                paint_label_like(
                    frame, w, l, snapshot, theme, scale, scroll_y, clip, fonts, atlas,
                );
            }
        }
    }
}

/// Walk the entire widget+layout tree and collect button hit regions (DFS order).
///
/// `scrolled` becomes true under the page `ScrollView` so hit tests can subtract
/// page scroll. Does **not** emit Tree/Table hits (those are appended separately
/// so button paint indices stay aligned with chrome 0..10 + content from 11).
fn collect_hit_regions(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &acme_layout::LayoutSnapshot,
    scrolled: bool,
    result: &mut Vec<HitRegion>,
) {
    #[allow(clippy::collapsible_if)]
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
        // Tree/Table/VirtualList handled in collect_data_widget_hits.
        WidgetNode::Tree(_) | WidgetNode::Table(_) | WidgetNode::VirtualList(_) => {}
        _ => {
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                collect_hit_regions(w, l, snapshot, scrolled, result);
            }
        }
    }
}

/// Append Tree / Table hit regions and cache viewport rects for scroll routing.
///
/// Always treated as scrolled content (inside the page scroll view).
#[allow(clippy::too_many_arguments)]
fn collect_data_widget_hits(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &acme_layout::LayoutSnapshot,
    table_sort_col: Option<usize>,
    table_sort_asc: bool,
    result: &mut Vec<HitRegion>,
    tree_viewport: &mut [f32; 4],
    table_viewport: &mut [f32; 4],
    vlist_viewport: &mut [f32; 4],
) {
    match widget {
        WidgetNode::Tree(t) => {
            if let Some(rect) = snapshot.get(layout.id) {
                *tree_viewport = [rect.x, rect.y, rect.width, rect.height];
            }
            let visible = t.visible_nodes();
            for (node, child_layout) in visible.iter().zip(layout.children.iter()) {
                let Some(rect) = snapshot.get(child_layout.id) else {
                    continue;
                };
                let full = [rect.x, rect.y, rect.width, rect.height];
                // Resolve stable &'static str for Copy messages.
                let Some(key) = tree_key_static(node.key.as_str()) else {
                    continue;
                };
                if node.has_children {
                    // Left chevron strip → toggle expand.
                    let chevron_w = 20.0_f32.min(rect.width);
                    result.push(HitRegion {
                        rect: [rect.x, rect.y, chevron_w, rect.height],
                        message: GalleryMessage::TreeToggleKey(key),
                        scrolled: true,
                    });
                    result.push(HitRegion {
                        rect: [
                            rect.x + chevron_w,
                            rect.y,
                            (rect.width - chevron_w).max(0.0),
                            rect.height,
                        ],
                        message: GalleryMessage::TreeSelectKey(key),
                        scrolled: true,
                    });
                } else {
                    result.push(HitRegion {
                        rect: full,
                        message: GalleryMessage::TreeSelectKey(key),
                        scrolled: true,
                    });
                }
            }
        }
        WidgetNode::Table(t) => {
            if let Some(rect) = snapshot.get(layout.id) {
                *table_viewport = [rect.x, rect.y, rect.width, rect.height];
            }
            let mut row_i = 0usize;
            if !t.columns.is_empty() {
                if let Some(header_row) = layout.children.get(row_i) {
                    for (ci, cell_layout) in header_row.children.iter().enumerate() {
                        if !t.columns.get(ci).is_some_and(|c| c.sortable) {
                            continue;
                        }
                        if let Some(rect) = snapshot.get(cell_layout.id) {
                            result.push(HitRegion {
                                rect: [rect.x, rect.y, rect.width, rect.height],
                                message: GalleryMessage::TableSort(ci),
                                scrolled: true,
                            });
                        }
                    }
                }
                row_i += 1;
            }
            let order = table_display_order(table_sort_col, table_sort_asc);
            for (display_i, row_layout) in layout.children.iter().skip(row_i).enumerate() {
                let Some(&orig) = order.get(display_i) else {
                    break;
                };
                if let Some(rect) = snapshot.get(row_layout.id) {
                    result.push(HitRegion {
                        rect: [rect.x, rect.y, rect.width, rect.height],
                        message: GalleryMessage::TableSelectRow(orig),
                        scrolled: true,
                    });
                }
            }
        }
        WidgetNode::VirtualList(_) => {
            if let Some(rect) = snapshot.get(layout.id) {
                *vlist_viewport = [rect.x, rect.y, rect.width, rect.height];
            }
        }
        WidgetNode::Tooltip(t) => {
            collect_data_widget_hits(
                &t.child,
                layout,
                snapshot,
                table_sort_col,
                table_sort_asc,
                result,
                tree_viewport,
                table_viewport,
                vlist_viewport,
            );
        }
        WidgetNode::Popover(p) => {
            collect_data_widget_hits(
                &p.children[0],
                layout,
                snapshot,
                table_sort_col,
                table_sort_asc,
                result,
                tree_viewport,
                table_viewport,
                vlist_viewport,
            );
        }
        _ => {
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                collect_data_widget_hits(
                    w,
                    l,
                    snapshot,
                    table_sort_col,
                    table_sort_asc,
                    result,
                    tree_viewport,
                    table_viewport,
                    vlist_viewport,
                );
            }
        }
    }
}

/// Map a tree node key string to a `'static` constant used in messages.
fn tree_key_static(key: &str) -> Option<&'static str> {
    const ALL: &[&str] = &[
        "docs",
        "docs_readme",
        "docs_guide",
        "docs_zh",
        "docs_zh_ime",
        "docs_zh_a11y",
        "images",
        "img_logo",
        "img_banner",
        "code",
        "code_src",
        "code_main",
        "code_lib",
        "code_toml",
    ];
    ALL.iter().copied().find(|&k| k == key)
}

// ── Text Helper ─────────────────────────────────────────────────────────────

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

// ── Color / Geometry Helpers ────────────────────────────────────────────────

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
