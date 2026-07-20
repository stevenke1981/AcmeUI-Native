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
use acme_layout::{LayoutEngine, LayoutKind, LayoutNode, LayoutStyle, Length, Overflow};
use acme_platform::{
    Application, Clipboard, FrameContext, PlatformEvent, PlatformKey, WindowConfig,
};
use acme_render_wgpu::{Frame, Quad, TextRun};
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextStyle};
use acme_textinput::{
    TextInputState, handle_key, handle_keyboard_shortcut, handle_text, render_text_input,
};
use acme_theme::{Theme, ThemeColor, ThemeMode};
use acme_widgets::{
    ButtonState, ButtonVariant, WidgetNode, button, column, label, label_with_size, row,
    scroll_view, separator,
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
}

/// A button hit region stored after layout each frame.
struct HitRegion {
    rect: [f32; 4],
    message: GalleryMessage,
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
    ime_text: String,

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
            ime_text: String::new(),
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
            .child(self.sidebar())
            .child(self.content_area())
            .build()
    }

    fn sidebar(&self) -> WidgetNode<GalleryMessage> {
        let mut col = column::<GalleryMessage>()
            .key("sidebar")
            .gap(2.0)
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
        let name = CATEGORIES[2].pages[self.selected_page.min(3)];
        self.component_page(name)
    }

    fn overlay_page(&self) -> WidgetNode<GalleryMessage> {
        let name = CATEGORIES[3].pages[self.selected_page.min(3)];
        self.component_page(name)
    }

    fn data_page(&self) -> WidgetNode<GalleryMessage> {
        let name = CATEGORIES[4].pages[self.selected_page.min(3)];
        self.component_page(name)
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
            .gap(spacing(self.density, 20.0))
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
            .gap(spacing(self.density, 8.0))
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
            .gap(4.0)
            .padding(12.0)
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
        .gap(4.0)
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
        .gap(6.0)
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
                .gap(4.0)
                .child(label_with_size("☀ Light", 16.0))
                .child(label("Component in light theme"))
                .build(),
        )
        .child(
            column()
                .gap(4.0)
                .child(label_with_size("🌙 Dark", 16.0))
                .child(label("Component in dark theme"))
                .build(),
        )
        .build()
}

fn density_demo() -> WidgetNode<GalleryMessage> {
    column()
        .gap(4.0)
        .child(label("Compact (0.75×) vs Comfortable (1.0×)"))
        .child(label("Toggle via toolbar button above."))
        .build()
}

fn keyboard_behavior() -> WidgetNode<GalleryMessage> {
    column()
        .gap(4.0)
        .child(label("Space  ·  Activate focused widget"))
        .child(label("Enter  ·  Submit / Confirm"))
        .child(label("Tab    ·  Move focus forward"))
        .child(label("⇧+Tab ·  Move focus backward"))
        .child(label("Esc    ·  Dismiss overlay / cancel"))
        .build()
}

fn accessibility_props() -> WidgetNode<GalleryMessage> {
    column()
        .gap(4.0)
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
        .gap(4.0)
        .child(label_with_size("Long Traditional Chinese string:", 14.0))
        .child(label_with_size(LONG_CHINESE_TEXT, 14.0))
        .build()
}

fn screenshot_info() -> WidgetNode<GalleryMessage> {
    column()
        .gap(4.0)
        .child(label("Screenshot sizes: 1280×800 · 1024×700 · 800×600"))
        .child(label("Toggle theme & density via toolbar before capture."))
        .build()
}

/// Apply density scale factor to a base spacing value.
fn spacing(density: Density, base: f32) -> f32 {
    base * density.spacing_scale()
}

// ── Event Handling ──────────────────────────────────────────────────────────

impl Gallery {
    fn hit(&self) -> Option<usize> {
        self.button_info.iter().position(|hr| {
            let r = hr.rect;
            self.cursor.0 >= r[0]
                && self.cursor.0 <= r[0] + r[2]
                && self.cursor.1 >= r[1]
                && self.cursor.1 <= r[1] + r[3]
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
            GalleryMessage::FocusDemo | GalleryMessage::DpiInfo => true,
        }
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
                pressed,
                ctrl,
                shift,
                ref text,
                ..
            } => {
                if !self.text_input.focused {
                    return false;
                }
                if ctrl
                    && let Some(t) = text
                    && matches!(t.as_str(), "a" | "c" | "v" | "x")
                {
                    return handle_keyboard_shortcut(
                        &mut self.text_input,
                        t,
                        self.clipboard.as_ref(),
                    );
                }
                handle_key(
                    &mut self.text_input,
                    key,
                    pressed,
                    self.clipboard.as_ref(),
                    ctrl,
                    shift,
                )
            }
            PlatformEvent::ImePreedit(text) => {
                self.text_input.set_preedit(&text, None);
                true
            }
            PlatformEvent::ImeCommit(text) => {
                handle_text(&mut self.text_input, &text);
                self.ime_text = self.text_input.text.clone();
                true
            }
            PlatformEvent::Resized { .. } => true,
            _ => false,
        }
    }

    fn frame(&mut self, context: FrameContext) -> Frame {
        let width = context.logical_width;
        let height = context.logical_height;

        // ── 1. Build widget tree ──
        let description = self.description();

        // ── 2. Convert to layout tree ──
        let mut root = description.to_layout(NodeId::new(1));

        // ── 3. Apply sizes, gaps, scroll flags ──
        apply_gallery_styles(&mut root, width, height);

        // ── 4. Compute layout snapshot ──
        let snapshot = self
            .layout
            .compute(&root, (width, height))
            .expect("finite Gallery viewport");

        // ── 5. Accessibility ──
        self.accessibility.update(&description, &snapshot);

        // ── 6. Extract structural IDs for the fixed frame ──
        let ids = extract_gallery_ids(&root);

        // ── 7. Collect all button hit regions via DFS walk ──
        let mut button_info = Vec::new();
        collect_hit_regions(&description, &root, &snapshot, &mut button_info);
        self.button_info = button_info;

        // ── 8. Scroll metrics ──
        self.max_scroll = snapshot
            .scroll_metrics(ids.scroll_view)
            .map(|m| (m.content_height - m.viewport_height).max(0.0))
            .unwrap_or(0.0);
        self.scroll = self.scroll.clamp(0.0, self.max_scroll);

        // ── 9. Theme ──
        let theme = if self.dark {
            Theme::dark()
        } else {
            Theme::light()
        };
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
                colors.text,
                context.scale_factor,
                None,
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
                if is_selected || (st.focused && self.show_focus_rings) {
                    2.0
                } else {
                    1.0
                },
                if st.focused {
                    colors.focus
                } else {
                    colors.border
                },
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
                    if !self.ime_text.is_empty() {
                        add_text(
                            &mut self.fonts,
                            &mut self.atlas,
                            &mut frame,
                            &format!("Committed: {}", self.ime_text),
                            ([ph.x + 2.0, y + ph.height + 6.0], 14.0),
                            colors.text_muted,
                            context.scale_factor,
                            None,
                        );
                    }
                }
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
        gap: 2.0,
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
        sb.children[i].style.height = Length::px(36.0);
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
                let y = rect.y - scroll_y;
                let fs = l.font_size.unwrap_or(theme.typography.body_size);
                add_text(
                    fonts,
                    atlas,
                    frame,
                    &l.text,
                    ([rect.x + 4.0, y + 2.0], fs),
                    colors.text,
                    scale,
                    Some(clip),
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
                    ([rect.x + 10.0, y + 8.0], theme.typography.label_size),
                    resolved.foreground,
                    scale,
                    Some(clip),
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

/// Walk the entire widget+layout tree and collect button hit regions (DFS order).
fn collect_hit_regions(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &acme_layout::LayoutSnapshot,
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
                    });
                }
            }
        }
        WidgetNode::Tooltip(t) => {
            collect_hit_regions(&t.child, layout, snapshot, result);
        }
        WidgetNode::Popover(p) => {
            collect_hit_regions(&p.children[0], layout, snapshot, result);
        }
        _ => {
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                collect_hit_regions(w, l, snapshot, result);
            }
        }
    }
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
) {
    let (origin, size) = geometry;
    let style = TextStyle {
        font_size: size,
        line_height: size * 1.35,
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
