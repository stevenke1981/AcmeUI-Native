//! Shared types and constants for the Gallery application.

use acme_theme::ThemeMode;

// ── Constants ────────────────────────────────────────────────────────────────

pub const SIDEBAR_WIDTH: f32 = 224.0;
pub const TOOLBAR_HEIGHT: f32 = 48.0;
/// Long Traditional Chinese string for testing widget string handling.
pub const LONG_CHINESE_TEXT: &str =
    "在一個寧靜的午後，古老的書架上擺滿了泛黃的書籍，每一本都承載著時代的記憶與智慧";

/// Unique marker for the TextInput placeholder slot (must not appear in normal labels).
pub const TEXT_INPUT_MARKER: &str = "\0_ti_ph_\0";

/// Expandable tree node keys (bit `i` in `Gallery::tree_expanded`).
pub const TREE_EXPAND_KEYS: &[&str] = &["docs", "docs_zh", "images", "code", "code_src"];
/// Default: all expandable nodes open (matches the original static demo).
pub const TREE_EXPAND_DEFAULT: u32 = 0b1_1111;

pub const VLIST_ITEM_COUNT: usize = 250;
pub const VLIST_ITEM_HEIGHT: f32 = 28.0;
pub const VLIST_VIEWPORT_H: f32 = 360.0;
pub const TABLE_ROW_COUNT: usize = 28;

pub struct CategoryInfo {
    pub name: &'static str,
    pub pages: &'static [&'static str],
}

pub const CATEGORIES: &[CategoryInfo] = &[
    CategoryInfo {
        name: "Foundations",
        pages: &[
            "Typography",
            "Colors",
            "Icons",
            "Spacing",
            "Motion",
            "Style",
        ],
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
pub enum Density {
    Compact,
    Comfortable,
}

impl Density {
    pub fn spacing_scale(self) -> f32 {
        match self {
            Self::Compact => 0.75,
            Self::Comfortable => 1.0,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Compact => "Compact",
            Self::Comfortable => "Comfortable",
        }
    }
    pub fn toggle(self) -> Self {
        match self {
            Self::Compact => Self::Comfortable,
            Self::Comfortable => Self::Compact,
        }
    }
}

/// Configures a screenshot-style render of a single template at a fixed size.
#[allow(dead_code)]
pub struct ScreenshotConfig {
    pub width: f32,
    pub height: f32,
    pub theme_variant: ThemeMode,
    pub density: Density,
    pub show_focus: bool,
    pub show_error: bool,
    pub show_loading: bool,
    pub show_empty: bool,
}

/// All messages handled by the gallery.
#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
pub enum GalleryMessage {
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
pub struct HitRegion {
    pub rect: [f32; 4],
    pub message: GalleryMessage,
    pub scrolled: bool,
}
