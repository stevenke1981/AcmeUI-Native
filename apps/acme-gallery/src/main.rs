//! AcmeUI Gallery — showcase app for acme-ui components including Charts.
//!
//! Architecture:
//!   Row (root)
//!     ├── Column (sidebar, 200px) — AcmeUI + category buttons
//!     └── Column (content area) — flex
//!         ├── Row (toolbar, 40px) — theme toggle
//!         └── ScrollView (content) — flex, overflow scroll

#![allow(clippy::collapsible_if)]

use acme_core::{
    AtlasUpload, ClickDetector, Color, CursorHint, DrawCommand, GlyphDraw, GlyphFormat,
    HitRegionSpec, HitTestLayer, HitZLayer, NodeId, Painter, Point, PointerOutcome, Rect,
    RectStyle, RetainedTree, TextPrimitive,
};
use acme_layout::{LayoutEngine, LayoutNode, LayoutStyle, Length, Overflow, WidgetLayoutContext};
use acme_platform::{Application, FrameContext, PlatformEvent, WindowConfig, WindowId};
#[cfg(test)]
use acme_render_wgpu::{ClippedQuad, Frame, Quad, TextRun, scene_from_frame};
use acme_text::{FontSystem, GlyphAtlas, PreparedText, TextConstraints, TextStyle};
use acme_theme::packs::{available_themes, theme_by_name};
use acme_theme::{Theme, ThemeColor, ThemeMode};
use acme_ui::charts::{
    BarEntry, ChartPoint, PieSlice, area_chart, bar_chart, gauge, line_chart, pie_chart, sparkline,
};
use acme_ui::foundations::{LiveRegionPoliteness, live_region, progress_ring, qr_code};
use acme_ui::inputs::{SelectOption, SliderMark, combobox, date_picker, slider_marks};
use acme_ui::mobile::{
    MobileAvatarSize, MobileBannerTone, MobileButtonSize, MobileDrawerSide, mobile_action,
    mobile_avatar, mobile_banner, mobile_button, mobile_card, mobile_carousel, mobile_chip,
    mobile_drawer, mobile_list_item, mobile_loader, mobile_notification, mobile_progress,
    mobile_search, mobile_segmented, mobile_sheet_handle, mobile_stepper, mobile_tab_bar,
    mobile_toggle,
};
use acme_ui::overlay::{DrawerPlacement, drawer, dropdown_menu};
use acme_widgets::painter_bridge::{paint_canvas_with_theme, theme_color};
use acme_widgets::retained_binding::{RetainedBinding, bind_retained};
use acme_widgets::style::{Styled, TextAlign};
use acme_widgets::view_bridge::widget_to_views;
use acme_widgets::{
    ButtonSize, ButtonState, ButtonVariant, CardVariant, WidgetNode, button, canvas, card, column,
    label, label_with_size, menu_item, row, scroll_view, separator,
};

#[path = "mobile_search.rs"]
mod mobile_search_host;
use mobile_search_host::SearchDemo;
#[path = "mobile_banner.rs"]
mod banner_examples;
#[path = "mobile_notification.rs"]
mod notification_examples;
use notification_examples::NotificationDemo;
mod about_dialog;
#[path = "mobile_action.rs"]
mod action_examples;
mod anchor;
mod bottom_navigation;
mod calendar_range;
mod collapsible;
mod fab;
mod fullscreen;
mod notification;
mod page_header;
mod tour;
use tour::TourDemo;
mod speed_dial;
use speed_dial::SpeedDialDemo;
mod snackbar;
use snackbar::SnackbarDemo;
mod data_list_examples;
mod paper_examples;
mod spinner_motion;
mod swipe_actions;
mod theme_pack_examples;
use spinner_motion::SpinnerDemo;
use swipe_actions::SwipeDemo;
#[path = "mobile_list_item.rs"]
mod list_examples;

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
    CategoryInfo {
        name: "Mobile",
        pages: &[
            "List items",
            "Banners",
            "Notifications",
            "Avatars",
            "Chips",
            "Search",
            "Loader",
            "Progress",
            "Cards",
            "Controls",
            "Button sizes",
            "States",
        ],
    },
];

// ── Messages ─────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
enum GalleryMessage {
    SelectCategory(usize),
    ToggleTheme,
    CycleTheme,
    DpiInfo,
    ToggleReducedMotion,
    MobilePressed(MobileButtonSize),
    MobileToggle(bool),
    MobileValue(i64),
    MobileSelected(usize),
    MobileChipToggle,
    MobileChipTap,
    MobileChipDismiss,
    MobileChipRestore,
    MobileBannerAction,
    MobileBannerDismiss,
    MobileBannerRestore,
    MobileNotificationAction,
    MobileNotificationDismiss,
    MobileNotificationShow,
    MobileActionPress(bool),
    MobileListItemTap(usize),
    MobileListItemReverse,
    MobileListItemDetails,
    MobileSearchEdited,
    MobileSearchSubmit,
    MobileSearchClear,
    FabPress(usize),
    AboutClose,
    AboutShow,
    NotificationAction,
    NotificationClose,
    NotificationShow,
    FullscreenClose,
    FullscreenShow,
    AnchorNavigate(usize),
    CollapsibleToggle,
    CollapsibleDisable,
    CalendarRangeToggle,
    SpeedDialToggle,
    SpeedDialDirection,
    SpeedDialAction(usize),
    BottomNavigationSelect(usize),
    HeaderSave,
    HeaderReset,
    HeaderToggleDescription,
    TourPrevious,
    TourNext,
    TourFinish,
    TourRestart,
    SnackbarShow(bool),
    SnackbarAction,
    SnackbarDismiss,
    SwipeToggle,
    SwipeRestore,
    SwipeArchive,
    SwipeDelete,
    SelectHighContrast,
    LiveRegionAnnounce,
    DatePickerToggle,
    DatePickerSelectDay(u32),
    DatePickerPrevMonth,
    DatePickerNextMonth,
    ComboboxToggle,
    ComboboxSelect(u8),
    SliderMarkSelect(u8),
    DropdownToggle,
    DropdownSelect(u8),
    DrawerToggle,
    DrawerClose,
    MobileTabSelect(usize),
    MobileDrawerToggle,
    MobileDrawerClose,
    MobileCarouselPrev,
    MobileCarouselNext,
    MobileCarouselSelect(usize),
    CanvasPing,
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
    last_mobile_press: Option<MobileButtonSize>,
    mobile_checked: bool,
    mobile_value: i64,
    mobile_selected: usize,
    search: SearchDemo,
    chip_selected: bool,
    chip_visible: bool,
    chip_action: &'static str,
    banner_visible: bool,
    banner_action: &'static str,
    notification: NotificationDemo,
    action_status: &'static str,
    list_focus: Option<usize>,
    last_list_tap: Option<usize>,
    list_reverse: bool,
    list_details: bool,
    fab_status: &'static str,
    speed_dial: SpeedDialDemo,
    bottom_selected: usize,
    header_description: bool,
    header_saves: usize,
    tour: TourDemo,
    about_visible: bool,
    about_closes: usize,
    notification_visible: bool,
    notification_actions: usize,
    notification_closes: usize,
    fullscreen_visible: bool,
    fullscreen_closes: usize,
    anchor_selected: usize,
    anchor_navigations: usize,
    collapsible_open: bool,
    collapsible_disabled: bool,
    collapsible_toggles: usize,
    calendar_range: bool,
    snackbar: SnackbarDemo,
    swipe: SwipeDemo,
    spinner: SpinnerDemo,
    live_region_status: &'static str,
    date_picker_open: bool,
    date_year: u32,
    date_month: u32,
    date_selected_day: Option<u32>,
    combobox_open: bool,
    combobox_value: &'static str,
    slider_value: f32,
    dropdown_open: bool,
    dropdown_status: &'static str,
    drawer_open: bool,
    mobile_tab_index: usize,
    mobile_drawer_open: bool,
    mobile_carousel_index: usize,
    canvas_pings: usize,

    hit_nodes: Vec<NodeId>,
    hit_layer: HitTestLayer,
    click_detector: ClickDetector,
    viewport_clip: [f32; 4],
    hit_layer_fingerprint: u64,
    event_seq: u64,

    fonts: FontSystem,
    atlas: GlyphAtlas,
    layout: LayoutEngine,
    retained: RetainedLayoutState,
    #[cfg(test)]
    last_frame_layout: Option<(LayoutNode, acme_layout::LayoutSnapshot)>,
}

impl Gallery {
    fn new() -> Self {
        // Restore persisted theme preference, falling back to defaults.
        let (theme_pack, dark) = load_theme_pref()
            .filter(|(pack, _)| pack == "default" || available_themes().contains(&pack.as_str()))
            .unwrap_or_else(|| ("default".to_string(), false));
        Self {
            selected_category: CATEGORIES.len() - 1, // Start on the newest showcase page.
            dark,
            theme_pack,
            cursor: (0.0, 0.0),
            hovered: None,
            pressed: None,
            scroll: 0.0,
            max_scroll: 0.0,
            button_info: Vec::new(),
            last_mobile_press: None,
            mobile_checked: false,
            mobile_value: 2,
            mobile_selected: 0,
            search: SearchDemo::new(),
            chip_selected: true,
            chip_visible: true,
            chip_action: "none",
            banner_visible: true,
            banner_action: "none",
            notification: NotificationDemo::new(),
            action_status: "none",
            list_focus: None,
            last_list_tap: None,
            list_reverse: false,
            list_details: true,
            fab_status: "none",
            speed_dial: SpeedDialDemo::default(),
            bottom_selected: 0,
            header_description: true,
            header_saves: 0,
            tour: TourDemo::default(),
            about_visible: true,
            about_closes: 0,
            notification_visible: true,
            notification_actions: 0,
            notification_closes: 0,
            fullscreen_visible: true,
            fullscreen_closes: 0,
            anchor_selected: 0,
            anchor_navigations: 0,
            collapsible_open: false,
            collapsible_disabled: false,
            collapsible_toggles: 0,
            calendar_range: true,
            snackbar: SnackbarDemo::new(),
            swipe: SwipeDemo::new(),
            spinner: SpinnerDemo::new(),
            live_region_status: "Ready",
            date_picker_open: false,
            date_year: 2026,
            date_month: 11,
            date_selected_day: Some(15),
            combobox_open: false,
            combobox_value: "AcmeUI Native",
            slider_value: 50.0,
            dropdown_open: false,
            dropdown_status: "none",
            drawer_open: false,
            mobile_tab_index: 0,
            mobile_drawer_open: false,
            mobile_carousel_index: 0,
            canvas_pings: 0,
            hit_nodes: Vec::new(),
            hit_layer: HitTestLayer::new(),
            click_detector: ClickDetector::new(),
            viewport_clip: [SIDEBAR_WIDTH, TOOLBAR_HEIGHT, 900.0, 660.0],
            hit_layer_fingerprint: 0,
            event_seq: 0,
            fonts: FontSystem::new(),
            atlas: GlyphAtlas::default(),
            layout: LayoutEngine::new(),
            retained: RetainedLayoutState::new(),
            #[cfg(test)]
            last_frame_layout: None,
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
            let mut btn = button::<GalleryMessage>(format!("category_{i}"), cat.name);
            if i == self.selected_category {
                btn = btn.primary();
            }
            col = col.child(btn.on_click(GalleryMessage::SelectCategory(i)));
        }
        col.build()
    }

    fn toolbar(&self) -> WidgetNode<GalleryMessage> {
        let theme = self.current_theme();
        let wcag_label = if theme.meets_wcag_aa() {
            "✓AA"
        } else {
            "⚠AA"
        };
        row()
            .key("toolbar")
            .gap(8.0)
            .padding(8.0)
            .child(
                button("theme_btn", if self.dark { "☀ Light" } else { "🌙 Dark" })
                    .on_click(GalleryMessage::ToggleTheme),
            )
            .child(
                button("pack_btn", format!("🎨 {} {}", self.theme_pack, wcag_label))
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
            7 => self.mobile_page(),
            _ => label("Unknown category"),
        }
    }

    fn foundations_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .key("foundations_page")
            .gap(16.0)
            .padding(24.0)
            .child(label_with_size("Foundations", 24.0))
            .child(separator())
            .child(self.page_section("Collapsible — typed toggle", self.collapsible_examples()))
            .child(self.page_section("Calendar — date range", self.calendar_range_examples()))
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
            .child(
                self.page_section(
                    "ProgressRing, QRCode & LiveRegion",
                    column()
                        .key("foundations_p4_section")
                        .gap(12.0)
                        .child(
                            row()
                                .key("foundations_p4_canvas_row")
                                .gap(16.0)
                                .child(
                                    progress_ring::<GalleryMessage>(68.0)
                                        .key("foundations_progress_ring")
                                        .size(56.0)
                                        .show_label(true),
                                )
                                .child(
                                    qr_code::<GalleryMessage>("https://acme-ui.native")
                                        .key("foundations_qr_code")
                                        .size(96.0),
                                )
                                .build(),
                        )
                        .child(
                            row()
                                .key("foundations_live_region_row")
                                .gap(8.0)
                                .child(
                                    button("foundations_live_btn", "Announce Status")
                                        .on_click(GalleryMessage::LiveRegionAnnounce),
                                )
                                .child(label(format!(
                                    "LiveRegion status: {}",
                                    self.live_region_status
                                )))
                                .child(
                                    live_region::<GalleryMessage>(self.live_region_status)
                                        .key("foundations_live_region")
                                        .politeness(LiveRegionPoliteness::Polite),
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
            .key("inputs_page")
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
            .child(
                self.page_section(
                    "DatePicker — calendar selector",
                    column()
                        .key("inputs_date_picker_section")
                        .gap(8.0)
                        .child(
                            row()
                                .gap(8.0)
                                .child(
                                    button(
                                        "inputs_date_toggle",
                                        if self.date_picker_open {
                                            "Hide Calendar"
                                        } else {
                                            "Show Calendar"
                                        },
                                    )
                                    .on_click(GalleryMessage::DatePickerToggle),
                                )
                                .child(label(match self.date_selected_day {
                                    Some(day) => format!(
                                        "Selected: {:04}-{:02}-{:02}",
                                        self.date_year, self.date_month, day
                                    ),
                                    None => format!(
                                        "Browsing: {:04}-{:02}",
                                        self.date_year, self.date_month
                                    ),
                                }))
                                .build(),
                        )
                        .child(
                            date_picker::<GalleryMessage>("inputs_date_picker")
                                .year(self.date_year)
                                .month(self.date_month)
                                .selected_day(self.date_selected_day)
                                .open(self.date_picker_open)
                                .placeholder("Pick a date")
                                .on_select_day(GalleryMessage::DatePickerSelectDay)
                                .on_prev_month(GalleryMessage::DatePickerPrevMonth)
                                .on_next_month(GalleryMessage::DatePickerNextMonth),
                        )
                        .build(),
                ),
            )
            .child(
                self.page_section(
                    "Combobox — option filter",
                    column()
                        .key("inputs_combobox_section")
                        .gap(8.0)
                        .child(
                            combobox::<GalleryMessage>("inputs_combobox")
                                .value(self.combobox_value)
                                .placeholder("Choose target...")
                                .option(SelectOption::new("AcmeUI Native", "acme"))
                                .option(SelectOption::new("Rust WGPU", "wgpu"))
                                .option(SelectOption::new("Taffy Layout", "taffy"))
                                .open(self.combobox_open)
                                .on_toggle(GalleryMessage::ComboboxToggle)
                                .on_select_option(combobox_option_message),
                        )
                        .child(label(format!(
                            "Combobox: {} ({})",
                            self.combobox_value,
                            if self.combobox_open { "open" } else { "closed" }
                        )))
                        .build(),
                ),
            )
            .child(
                self.page_section(
                    "SliderMarks — stepped marks",
                    column()
                        .key("inputs_slider_marks_section")
                        .gap(8.0)
                        .child(
                            slider_marks::<GalleryMessage>(self.slider_value)
                                .key("inputs_slider_marks")
                                .min(0.0)
                                .max(100.0)
                                .mark(SliderMark::new(0.0, "0%"))
                                .mark(SliderMark::new(25.0, "25%"))
                                .mark(SliderMark::new(50.0, "50%"))
                                .mark(SliderMark::new(75.0, "75%"))
                                .mark(SliderMark::new(100.0, "100%"))
                                .on_change(slider_mark_message),
                        )
                        .build(),
                ),
            )
            .build()
    }

    fn layout_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .key("layout_page")
            .gap(16.0)
            .padding(24.0)
            .child(label_with_size("Layout", 24.0))
            .child(separator())
            .child(self.page_section("Anchor — typed navigation", self.anchor_examples()))
            .child(self.page_section(
                "BottomNavigation — typed selection",
                self.bottom_navigation_examples(),
            ))
            .child(self.page_section("PageHeader — retained actions", self.page_header_examples()))
            .child(label("Paper: rounded, square elevated, square outlined"))
            .child(self.paper_examples())
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
            .key("overlay_page")
            .gap(16.0)
            .padding(24.0)
            .child(label_with_size("Overlay", 24.0))
            .child(separator())
            .child(self.page_section(
                "Notification — controlled actions",
                self.desktop_notification_examples(),
            ))
            .child(self.page_section("Fullscreen — Close message", self.fullscreen_examples()))
            .child(self.page_section("SpeedDial — typed actions", self.speed_dial_examples()))
            .child(self.page_section("About card — Close message", self.about_dialog_examples()))
            .child(self.page_section("Tour — controlled steps", self.tour_examples()))
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
            .child(
                self.page_section(
                    "DropdownMenu — popup menu",
                    column()
                        .key("overlay_dropdown_section")
                        .gap(8.0)
                        .child(
                            row()
                                .gap(12.0)
                                .child(
                                    dropdown_menu::<GalleryMessage>("overlay_dropdown")
                                        .trigger_label(if self.dropdown_open {
                                            "Actions ▴"
                                        } else {
                                            "Actions ▾"
                                        })
                                        .open(self.dropdown_open)
                                        .on_toggle(GalleryMessage::DropdownToggle)
                                        .item(
                                            menu_item("new", "New Item")
                                                .on_click(GalleryMessage::DropdownSelect(0)),
                                        )
                                        .item(
                                            menu_item("export", "Export Scene")
                                                .on_click(GalleryMessage::DropdownSelect(1)),
                                        )
                                        .separator()
                                        .item(
                                            menu_item("archive", "Archive")
                                                .on_click(GalleryMessage::DropdownSelect(2)),
                                        ),
                                )
                                .child(label(format!(
                                    "Dropdown: {} ({})",
                                    self.dropdown_status,
                                    if self.dropdown_open { "open" } else { "closed" }
                                )))
                                .build(),
                        )
                        .build(),
                ),
            )
            .child(
                self.page_section(
                    "Drawer — slide-over panel",
                    column()
                        .key("overlay_drawer_section")
                        .gap(8.0)
                        .child(
                            row()
                                .gap(8.0)
                                .child(
                                    button(
                                        "overlay_drawer_toggle",
                                        if self.drawer_open {
                                            "Close Drawer"
                                        } else {
                                            "Open Drawer"
                                        },
                                    )
                                    .on_click(GalleryMessage::DrawerToggle),
                                )
                                .child(label(format!(
                                    "Drawer: {}",
                                    if self.drawer_open { "open" } else { "closed" }
                                )))
                                .build(),
                        )
                        .child(
                            drawer::<GalleryMessage>("overlay_drawer")
                                .title("Inspector Drawer")
                                .placement(DrawerPlacement::Right)
                                .backdrop(true)
                                .open(self.drawer_open)
                                .on_close(GalleryMessage::DrawerClose)
                                .content(label("Drawer canvas backdrop & controls.")),
                        )
                        .build(),
                ),
            )
            .build()
    }

    fn data_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .key("data_page")
            .gap(16.0)
            .padding(24.0)
            .child(label_with_size("Data", 24.0))
            .child(separator())
            .child(self.page_section("DataList — density", self.data_list_examples()))
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
            .key("charts_page")
            .gap(20.0)
            .padding(24.0)
            .child(label_with_size("Charts", 24.0))
            .child(separator())
            .child(label(
                "All charts are static representations built from existing primitives.",
            ))
            // Interactive Canvas
            .child(
                self.page_section(
                    "Interactive Canvas — HitTestLayer",
                    row()
                        .gap(12.0)
                        .child(
                            canvas::<GalleryMessage>()
                                .key("charts_interactive_canvas")
                                .size(220.0, 36.0)
                                .cursor(CursorHint::Pointer)
                                .accessible_label("Trigger Canvas Ping")
                                .on_click(GalleryMessage::CanvasPing)
                                .paint(|painter, rect, theme| {
                                    painter.fill_rect(rect, theme_color(theme.colors.accent));
                                    let dot = Rect::new(
                                        rect.origin.x.get() + 10.0,
                                        rect.origin.y.get() + 10.0,
                                        16.0,
                                        16.0,
                                    );
                                    painter.fill_rect(dot, theme_color(theme.colors.primary));
                                }),
                        )
                        .child(label(format!("Canvas clicks: {}", self.canvas_pings)))
                        .build(),
                ),
            )
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

    fn chip_examples(&self) -> WidgetNode<GalleryMessage> {
        let removable = if self.chip_visible {
            row()
                .child(
                    mobile_chip("Rust · 原生")
                        .key("gallery_chip_removable")
                        .on_tap(GalleryMessage::MobileChipTap)
                        .on_dismiss(GalleryMessage::MobileChipDismiss),
                )
                .build()
        } else {
            label("Tag removed — restore it below")
        };
        column()
            .gap(8.0)
            .child(row().child(mobile_chip("Static tag").key("gallery_chip_static")))
            .child(
                row().child(
                    mobile_chip("Filter · 篩選")
                        .key("gallery_chip_filter")
                        .selected(self.chip_selected)
                        .on_tap(GalleryMessage::MobileChipToggle),
                ),
            )
            .child(removable)
            .child(
                row().child(
                    button("gallery_chip_restore", "Restore tag")
                        .on_click(GalleryMessage::MobileChipRestore),
                ),
            )
            .child(label(format!("Chip action: {}", self.chip_action)))
            .build()
    }

    fn sheet_handle_examples(&self) -> WidgetNode<GalleryMessage> {
        column()
            .key("gallery_sheet_handles")
            .gap(12.0)
            .child(
                card()
                    .key("gallery_sheet_preview")
                    .w(Length::px(320.0))
                    .padding(16.0)
                    .gap(12.0)
                    .child(
                        row()
                            .child(row().flex_1())
                            .child(mobile_sheet_handle().key("gallery_handle_default"))
                            .child(row().flex_1()),
                    )
                    .child(label("Options · 選項"))
                    .child(label("Drag indicator · 拖曳提示")),
            )
            .child(
                row().gap(12.0).child(label("Custom 48 × 4")).child(
                    mobile_sheet_handle()
                        .key("gallery_handle_custom")
                        .width(48.0),
                ),
            )
            .build()
    }

    fn mobile_page(&self) -> WidgetNode<GalleryMessage> {
        let status = match self.last_mobile_press {
            Some(MobileButtonSize::Sm) => "Last action: Small button",
            Some(MobileButtonSize::Md) => "Last action: Medium button",
            Some(MobileButtonSize::Lg) => "Last action: Large button",
            None => "Last action: none — activate a button to verify message dispatch",
        };

        column()
            .key("mobile_page")
            .gap(20.0)
            .padding(24.0)
            .child(label_with_size("Mobile", 24.0))
            .child(separator())
            .child(label(
                "Full-width mobile controls reuse semantic theme states and accessible touch heights.",
            ))
            .child(self.page_section("Swipe Actions — reveal then choose", self.swipe_examples()))
            .child(self.page_section("Snackbar — timed bottom message", self.snackbar_examples()))
            .child(self.page_section("Floating Action Buttons — mini, normal, extended", self.fab_examples()))
            .child(self.page_section("Mobile Sheet Handles", self.sheet_handle_examples()))
            .child(self.page_section("Mobile Actions — normal, destructive and disabled", self.action_examples()))
            .child(self.page_section(
                "Mobile List Items — content and navigation", self.list_item_examples(),
            ))
            .child(self.page_section(
                "Mobile Notifications — content and lifetime", self.notification_examples(),
            ))
            .child(self.page_section(
                "Mobile Banners — status and actions", self.banner_examples(),
            ))
            .child(self.page_section(
                "Mobile Avatars — initials and placeholder",
                column().gap(8.0)
                    .child(row().gap(16.0)
                        .child(mobile_avatar("AB").key("gallery_avatar_sm").size(MobileAvatarSize::Sm))
                        .child(mobile_avatar("陳").key("gallery_avatar_md"))
                        .child(mobile_avatar("UI").key("gallery_avatar_lg").size(MobileAvatarSize::Lg))
                        .child(mobile_avatar("").key("gallery_avatar_placeholder")))
                    .child(label("Small · 32 px / Medium · 40 px / Large · 56 px / Empty placeholder"))
                    .build(),
            ))
            .child(self.page_section("Mobile Chips — tags and actions", self.chip_examples()))
            .child(self.page_section(
                "Mobile Search — edit, submit and clear",
                column()
                    .gap(8.0)
                    .child(mobile_search("Search · 搜尋")
                        .key("gallery_search")
                        .value(&self.search.input.text)
                        .on_input(GalleryMessage::MobileSearchEdited)
                        .on_submit(GalleryMessage::MobileSearchSubmit)
                        .on_clear(GalleryMessage::MobileSearchClear))
                    .child(label("Tab to focus; type a query, Enter to search; Tab to Search / Clear."))
                    .child(acme_widgets::label_builder(format!("Last search: {}", self.search.submitted))
                        .wrap(acme_widgets::LabelWrap::Character)
                        .build())
                    .build(),
            ))
            .child(self.page_section(
                "Mobile Loader — rotation and reduced motion",
                column()
                    .gap(12.0)
                    .child(button("spinner_motion", if self.spinner.reduced_motion { "Reduced motion: on" } else { "Reduced motion: off" })
                        .on_click(GalleryMessage::ToggleReducedMotion))
                    .child(mobile_loader()
                        .key("gallery_loader_indicator")
                        .animation(self.spinner.elapsed, self.spinner.reduced_motion))
                    .child(mobile_loader()
                        .key("gallery_loader_message")
                        .animation(self.spinner.elapsed, self.spinner.reduced_motion)
                        .message("Loading · 載入中…"))
                    .child(column()
                        .width(240.0)
                        .child(mobile_loader()
                            .key("gallery_loader_long")
                            .animation(self.spinner.elapsed, self.spinner.reduced_motion)
                            .message("Loading the next page. Please wait while the request finishes. 正在準備資料，請稍候。")))
                    .build(),
            ))
            .child(self.page_section(
                "Mobile Progress — empty, partial, complete",
                column()
                    .gap(12.0)
                    .child(mobile_progress(0.0, 10.0)
                        .key("gallery_progress_empty"))
                    .child(mobile_progress(self.mobile_value as f32, 10.0)
                        .key("gallery_progress_quantity")
                        .label("Quantity · 數量 — adjust with the stepper below"))
                    .child(mobile_progress(10.0, 10.0)
                        .key("gallery_progress_complete")
                        .label("Complete"))
                    .build(),
            ))
            .child(self.page_section(
                "Mobile Card — content and variants",
                column()
                    .gap(12.0)
                    .child(mobile_card("Title only")
                        .key("gallery_card_title"))
                    .child(mobile_card("Outlined card")
                        .key("gallery_card_outlined")
                        .subtitle("Title and subtitle, without media")
                        .elevated(false))
                    .child(mobile_card("Card with media")
                        .key("gallery_card_media")
                        .subtitle("Optional content appears above the title")
                        .media(row()
                            .key("gallery_card_media_slot")
                            .height(80.0)
                            .bg_muted()
                            .child(label("Media slot · 媒體內容"))
                            .build()))
                    .build(),
            ))
            .child(self.page_section(
                "Interactive mobile controls",
                column()
                    .gap(12.0)
                    .child(mobile_toggle("Wi-Fi")
                        .key("gallery_mobile_toggle")
                        .checked(self.mobile_checked)
                        .on_toggle(GalleryMessage::MobileToggle))
                    .child(label("Quantity · 0–10, step 2"))
                    .child(mobile_stepper(self.mobile_value)
                        .key("gallery_mobile_stepper")
                        .min(0).max(10).step(2)
                        .on_change(GalleryMessage::MobileValue))
                    .child(mobile_segmented(vec!["Day", "Week", "Month"])
                        .key("gallery_mobile_segmented")
                        .selected_index(self.mobile_selected)
                        .on_select(GalleryMessage::MobileSelected))
                    .build(),
            ))
            .child(self.page_section(
                "Mobile Button — sizes",
                column()
                    .gap(12.0)
                    .child(WidgetNode::from(
                        mobile_button("mobile_sm", "Small · 36 px")
                            .size(MobileButtonSize::Sm)
                            .on_press(GalleryMessage::MobilePressed(MobileButtonSize::Sm)),
                    ))
                    .child(WidgetNode::from(
                        mobile_button("mobile_md", "Medium · 44 px")
                            .size(MobileButtonSize::Md)
                            .on_press(GalleryMessage::MobilePressed(MobileButtonSize::Md)),
                    ))
                    .child(WidgetNode::from(
                        mobile_button("mobile_lg", "Large · 52 px")
                            .size(MobileButtonSize::Lg)
                            .on_press(GalleryMessage::MobilePressed(MobileButtonSize::Lg)),
                    ))
                    .build(),
            ))
            .child(self.page_section(
                "States",
                column()
                    .gap(12.0)
                    .child(label(status))
                    .child(WidgetNode::from(
                        mobile_button("mobile_disabled", "Disabled")
                            .size(MobileButtonSize::Md)
                            .disabled(true)
                            .on_press(GalleryMessage::MobilePressed(MobileButtonSize::Md)),
                    ))
                    .build(),
            ))
            .child(self.page_section(
                "Mobile Navigation & Carousel (P4)",
                column()
                    .key("gallery_mobile_p4_section")
                    .gap(12.0)
                    .child(
                        mobile_tab_bar::<GalleryMessage>(vec!["Feed", "Explore", "Saved"])
                            .key("gallery_mobile_tab_bar")
                            .selected_index(self.mobile_tab_index)
                            .on_select(GalleryMessage::MobileTabSelect),
                    )
                    .child(
                        row()
                            .key("gallery_mobile_drawer_controls")
                            .gap(8.0)
                            .child(
                                button(
                                    "gallery_mobile_drawer_toggle",
                                    if self.mobile_drawer_open {
                                        "Close Mobile Drawer"
                                    } else {
                                        "Open Mobile Drawer"
                                    },
                                )
                                .on_click(GalleryMessage::MobileDrawerToggle),
                            )
                            .child(label(format!(
                                "Mobile drawer: {}",
                                if self.mobile_drawer_open {
                                    "open"
                                } else {
                                    "closed"
                                }
                            )))
                            .build(),
                    )
                    .child(
                        mobile_drawer::<GalleryMessage>("gallery_mobile_drawer")
                            .title("Navigation")
                            .open(self.mobile_drawer_open)
                            .side(MobileDrawerSide::Left)
                            .on_close(GalleryMessage::MobileDrawerClose)
                            .child(label("Quick mobile links")),
                    )
                    .child(
                        mobile_carousel::<GalleryMessage>("gallery_mobile_carousel")
                            .active_index(self.mobile_carousel_index)
                            .show_indicators(true)
                            .on_prev(GalleryMessage::MobileCarouselPrev)
                            .on_next(GalleryMessage::MobileCarouselNext)
                            .on_select(GalleryMessage::MobileCarouselSelect)
                            .item(
                                card()
                                    .key("gallery_mobile_slide_0")
                                    .padding(8.0)
                                    .child(label("Slide 1 · Overview"))
                                    .build(),
                            )
                            .item(
                                card()
                                    .key("gallery_mobile_slide_1")
                                    .padding(8.0)
                                    .child(label("Slide 2 · Metrics"))
                                    .build(),
                            )
                            .item(
                                card()
                                    .key("gallery_mobile_slide_2")
                                    .padding(8.0)
                                    .child(label("Slide 3 · Summary"))
                                    .build(),
                            ),
                    )
                    .build(),
            ))
            .build()
    }

    fn design_system_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .key("design_system_page")
            .gap(24.0)
            .padding(24.0)
            .child(label_with_size("Design System", 24.0))
            .child(separator())
            .child(
                self.page_section(
                    "Theme packs",
                    column()
                        .gap(8.0)
                        .child(self.theme_pack_swatches())
                        .child(
                            row()
                                .gap(8.0)
                                .child(
                                    button("apply_high_contrast", "Apply High Contrast Theme")
                                        .on_click(GalleryMessage::SelectHighContrast),
                                )
                                .child(
                                    button("cycle_theme_pack", "Next Theme Pack")
                                        .on_click(GalleryMessage::CycleTheme),
                                )
                                .build(),
                        )
                        .build(),
                ),
            )
            .child(label("All semantic design tokens defined by the V2 theme."))
            .child(label("Full sizing: 240 x 96 box, 12 px padding"))
            .child(self.full_sizing_example())
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

    fn full_sizing_example(&self) -> WidgetNode<GalleryMessage> {
        let mut text = label("w_full + h_full");
        if let WidgetNode::Label(label) = &mut text {
            label.style.text_color = Some(acme_widgets::style::ColorToken::PrimaryForeground);
        }
        row()
            .key("full_sizing_parent")
            .w(Length::px(240.0))
            .h(Length::px(96.0))
            .p(12.0)
            .bg_muted()
            .flex_shrink(0.0)
            .child(
                row()
                    .key("full_sizing_child")
                    .w_full()
                    .h_full()
                    .flex_shrink(0.0)
                    .bg_primary()
                    .rounded_lg()
                    .p(8.0)
                    .child(text)
                    .build(),
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
        label(format!("\x01{color_name}|{display}"))
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

    fn reset_hover_press(&mut self) {
        self.pressed = None;
        self.hovered = None;
        self.click_detector.cancel();
    }

    fn sync_hit_layer(&mut self) {
        let fp = hit_layer_fingerprint(
            &self.button_info,
            self.scroll,
            self.viewport_clip,
            self.snackbar.bounds,
        );
        if fp != self.hit_layer_fingerprint {
            self.hit_layer = build_hit_layer(
                &self.button_info,
                &self.hit_nodes,
                self.scroll,
                self.viewport_clip,
                self.snackbar.bounds,
            );
            self.hit_layer_fingerprint = fp;
        }
    }

    fn hit(&self) -> Option<usize> {
        let point = Point::new(self.cursor.0, self.cursor.1);
        let fp = hit_layer_fingerprint(
            &self.button_info,
            self.scroll,
            self.viewport_clip,
            self.snackbar.bounds,
        );
        if fp == self.hit_layer_fingerprint {
            self.hit_layer.hit_test(point).map(|hit| hit.tag as usize)
        } else {
            build_hit_layer(
                &self.button_info,
                &self.hit_nodes,
                self.scroll,
                self.viewport_clip,
                self.snackbar.bounds,
            )
            .hit_test(point)
            .map(|hit| hit.tag as usize)
        }
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
            GalleryMessage::SelectHighContrast => {
                self.theme_pack = "high-contrast".to_string();
                save_theme_pref(&self.theme_pack, self.dark);
                true
            }
            GalleryMessage::SelectCategory(i) => {
                let changed = self.selected_category != i;
                self.selected_category = i;
                if changed {
                    self.scroll = 0.0;
                    self.spinner.visible = false;
                    self.list_focus = None;
                    self.search.blur();
                    self.search.rect = None;
                    self.swipe.cancel_pointer();
                    self.swipe.keyboard = false;
                }
                true
            }
            message @ (GalleryMessage::SwipeToggle
            | GalleryMessage::SwipeRestore
            | GalleryMessage::SwipeArchive
            | GalleryMessage::SwipeDelete) => self.dispatch_swipe(message),
            GalleryMessage::SnackbarShow(with_action) => {
                self.snackbar.show(std::time::Instant::now(), with_action);
                self.pressed = None;
                self.hovered = None;
                true
            }
            GalleryMessage::SnackbarAction | GalleryMessage::SnackbarDismiss => {
                if !self.snackbar.visible {
                    return false;
                }
                if hr.message == GalleryMessage::SnackbarAction && !self.snackbar.with_action {
                    return false;
                }
                self.snackbar
                    .close(if hr.message == GalleryMessage::SnackbarAction {
                        "undone"
                    } else {
                        "dismissed"
                    });
                self.pressed = None;
                self.hovered = None;
                true
            }
            GalleryMessage::CollapsibleToggle => {
                if self.collapsible_disabled {
                    return false;
                }
                self.collapsible_open = !self.collapsible_open;
                self.collapsible_toggles = self.collapsible_toggles.saturating_add(1);
                true
            }
            GalleryMessage::CollapsibleDisable => {
                self.collapsible_disabled = !self.collapsible_disabled;
                self.pressed = None;
                self.hovered = None;
                true
            }
            GalleryMessage::AnchorNavigate(index) => {
                if index >= anchor::HREFS.len() {
                    return false;
                }
                self.anchor_selected = index;
                self.anchor_navigations = self.anchor_navigations.saturating_add(1);
                true
            }
            GalleryMessage::NotificationAction => {
                if !self.notification_visible {
                    return false;
                }
                self.notification_actions = self.notification_actions.saturating_add(1);
                true
            }
            GalleryMessage::NotificationClose => {
                if !self.notification_visible {
                    return false;
                }
                self.notification_visible = false;
                self.notification_closes = self.notification_closes.saturating_add(1);
                self.pressed = None;
                self.hovered = None;
                true
            }
            GalleryMessage::NotificationShow => {
                if self.notification_visible {
                    return false;
                }
                self.notification_visible = true;
                self.pressed = None;
                self.hovered = None;
                true
            }
            GalleryMessage::FullscreenClose => {
                if !self.fullscreen_visible {
                    return false;
                }
                self.fullscreen_visible = false;
                self.fullscreen_closes = self.fullscreen_closes.saturating_add(1);
                self.pressed = None;
                self.hovered = None;
                true
            }
            GalleryMessage::FullscreenShow => {
                if self.fullscreen_visible {
                    return false;
                }
                self.fullscreen_visible = true;
                self.pressed = None;
                self.hovered = None;
                true
            }
            GalleryMessage::AboutClose => {
                if !self.about_visible {
                    return false;
                }
                self.about_visible = false;
                self.about_closes = self.about_closes.saturating_add(1);
                self.pressed = None;
                self.hovered = None;
                true
            }
            GalleryMessage::AboutShow => {
                if self.about_visible {
                    return false;
                }
                self.about_visible = true;
                self.pressed = None;
                self.hovered = None;
                true
            }
            GalleryMessage::SpeedDialToggle => {
                self.speed_dial.open = !self.speed_dial.open;
                self.pressed = None;
                self.hovered = None;
                true
            }
            GalleryMessage::SpeedDialDirection => {
                use acme_ui::overlay::SpeedDialDirection;
                self.speed_dial.direction = match self.speed_dial.direction {
                    SpeedDialDirection::Up => SpeedDialDirection::Down,
                    SpeedDialDirection::Down => SpeedDialDirection::Left,
                    SpeedDialDirection::Left => SpeedDialDirection::Right,
                    SpeedDialDirection::Right => SpeedDialDirection::Up,
                };
                self.pressed = None;
                self.hovered = None;
                true
            }
            GalleryMessage::SpeedDialAction(index) => {
                if !self.speed_dial.open || index > 1 {
                    return false;
                }
                self.speed_dial.last_action = Some(index);
                true
            }
            GalleryMessage::BottomNavigationSelect(index) => {
                if index >= 3 {
                    return false;
                }
                self.bottom_selected = index;
                true
            }
            GalleryMessage::HeaderSave => {
                self.header_saves = self.header_saves.saturating_add(1);
                true
            }
            GalleryMessage::HeaderReset => {
                self.header_saves = 0;
                true
            }
            GalleryMessage::HeaderToggleDescription => {
                self.header_description = !self.header_description;
                self.pressed = None;
                self.hovered = None;
                true
            }
            message @ (GalleryMessage::TourPrevious
            | GalleryMessage::TourNext
            | GalleryMessage::TourFinish
            | GalleryMessage::TourRestart) => {
                if !self.tour.apply(message) {
                    return false;
                }
                self.pressed = None;
                self.hovered = None;
                true
            }
            GalleryMessage::FabPress(index) => {
                self.fab_status = match index {
                    0 => "mini",
                    1 => "normal",
                    2 => "create",
                    _ => "star",
                };
                true
            }
            GalleryMessage::MobileActionPress(destructive) => {
                self.action_status = if destructive {
                    "delete requested"
                } else {
                    "share requested"
                };
                true
            }
            GalleryMessage::MobileListItemTap(index) => {
                self.last_list_tap = Some(index);
                self.list_focus = Some(index);
                true
            }
            GalleryMessage::MobileListItemReverse => {
                self.list_reverse = !self.list_reverse;
                self.pressed = None;
                self.hovered = None;
                true
            }
            GalleryMessage::MobileListItemDetails => {
                self.list_details = !self.list_details;
                self.pressed = None;
                self.hovered = None;
                true
            }
            GalleryMessage::MobileNotificationAction => {
                self.notification.status = "open requested";
                true
            }
            GalleryMessage::MobileNotificationDismiss => {
                self.notification.dismiss();
                true
            }
            GalleryMessage::MobileNotificationShow => {
                self.notification.show_timed(std::time::Instant::now());
                true
            }
            GalleryMessage::MobileBannerAction => {
                self.banner_action = "retry requested";
                true
            }
            GalleryMessage::MobileBannerDismiss => {
                self.banner_visible = false;
                self.banner_action = "dismissed";
                true
            }
            GalleryMessage::MobileBannerRestore => {
                self.banner_visible = true;
                self.banner_action = "restored";
                true
            }
            GalleryMessage::MobileChipToggle => {
                self.chip_selected = !self.chip_selected;
                self.chip_action = "filter toggled";
                true
            }
            GalleryMessage::MobileChipTap => {
                self.chip_action = "tag tapped";
                true
            }
            GalleryMessage::MobileChipDismiss => {
                self.chip_visible = false;
                self.chip_action = "tag dismissed";
                true
            }
            GalleryMessage::MobileChipRestore => {
                self.chip_visible = true;
                self.chip_action = "tag restored";
                true
            }
            GalleryMessage::MobileSearchSubmit => self.search.submit(),
            GalleryMessage::MobileSearchClear => self.search.clear(),
            GalleryMessage::MobileSearchEdited => true,
            GalleryMessage::DpiInfo => true,
            GalleryMessage::ToggleReducedMotion => {
                self.spinner.reduced_motion = !self.spinner.reduced_motion;
                true
            }
            GalleryMessage::MobilePressed(size) => {
                self.last_mobile_press = Some(size);
                true
            }
            GalleryMessage::MobileToggle(checked) => {
                self.mobile_checked = checked;
                true
            }
            GalleryMessage::MobileValue(value) => {
                self.mobile_value = value;
                true
            }
            GalleryMessage::MobileSelected(index) => {
                self.mobile_selected = index;
                true
            }
            GalleryMessage::LiveRegionAnnounce => {
                self.live_region_status = if self.live_region_status == "Ready" {
                    "Updated via LiveRegion"
                } else {
                    "Ready"
                };
                true
            }
            GalleryMessage::CalendarRangeToggle => {
                self.calendar_range = !self.calendar_range;
                self.reset_hover_press();
                true
            }
            GalleryMessage::DatePickerToggle => {
                self.date_picker_open = !self.date_picker_open;
                self.reset_hover_press();
                true
            }
            GalleryMessage::DatePickerSelectDay(day) => {
                self.date_selected_day = Some(day);
                true
            }
            GalleryMessage::DatePickerPrevMonth => {
                if self.date_month <= 1 {
                    self.date_year = self.date_year.saturating_sub(1).max(1);
                    self.date_month = 12;
                } else {
                    self.date_month -= 1;
                }
                true
            }
            GalleryMessage::DatePickerNextMonth => {
                if self.date_month >= 12 {
                    self.date_year = self.date_year.saturating_add(1);
                    self.date_month = 1;
                } else {
                    self.date_month += 1;
                }
                true
            }
            GalleryMessage::ComboboxToggle => {
                self.combobox_open = !self.combobox_open;
                self.reset_hover_press();
                true
            }
            GalleryMessage::ComboboxSelect(index) => {
                self.combobox_value = match index {
                    0 => "AcmeUI Native",
                    1 => "Rust WGPU",
                    _ => "Taffy Layout",
                };
                self.combobox_open = false;
                self.reset_hover_press();
                true
            }
            GalleryMessage::SliderMarkSelect(value) => {
                self.slider_value = (value as f32).clamp(0.0, 100.0);
                true
            }
            GalleryMessage::DropdownToggle => {
                self.dropdown_open = !self.dropdown_open;
                self.reset_hover_press();
                true
            }
            GalleryMessage::DropdownSelect(index) => {
                self.dropdown_status = match index {
                    0 => "New Item",
                    1 => "Export Scene",
                    _ => "Archive",
                };
                self.dropdown_open = false;
                self.reset_hover_press();
                true
            }
            GalleryMessage::DrawerToggle => {
                self.drawer_open = !self.drawer_open;
                self.reset_hover_press();
                true
            }
            GalleryMessage::DrawerClose => {
                if !self.drawer_open {
                    return false;
                }
                self.drawer_open = false;
                self.reset_hover_press();
                true
            }
            GalleryMessage::MobileTabSelect(index) => {
                self.mobile_tab_index = index.min(2);
                true
            }
            GalleryMessage::MobileDrawerToggle => {
                self.mobile_drawer_open = !self.mobile_drawer_open;
                self.reset_hover_press();
                true
            }
            GalleryMessage::MobileDrawerClose => {
                if !self.mobile_drawer_open {
                    return false;
                }
                self.mobile_drawer_open = false;
                self.reset_hover_press();
                true
            }
            GalleryMessage::MobileCarouselPrev => {
                self.mobile_carousel_index = self.mobile_carousel_index.saturating_sub(1);
                self.reset_hover_press();
                true
            }
            GalleryMessage::MobileCarouselNext => {
                self.mobile_carousel_index = (self.mobile_carousel_index + 1).min(2);
                self.reset_hover_press();
                true
            }
            GalleryMessage::MobileCarouselSelect(index) => {
                self.mobile_carousel_index = index.min(2);
                self.reset_hover_press();
                true
            }
            GalleryMessage::CanvasPing => {
                self.canvas_pings += 1;
                true
            }
        }
    }
}

fn slider_mark_message(value: f32) -> GalleryMessage {
    GalleryMessage::SliderMarkSelect(value.round().clamp(0.0, 100.0) as u8)
}

fn combobox_option_message(value: String) -> GalleryMessage {
    match value.as_str() {
        "acme" => GalleryMessage::ComboboxSelect(0),
        "wgpu" => GalleryMessage::ComboboxSelect(1),
        _ => GalleryMessage::ComboboxSelect(2),
    }
}

impl Application for Gallery {
    fn window_config(&self) -> WindowConfig {
        WindowConfig {
            title: "AcmeUI Gallery".into(),
            width: 1100.0,
            height: 700.0,
            ..WindowConfig::default()
        }
    }

    fn event(&mut self, event: PlatformEvent) -> bool {
        let overlay_pointer = match &event {
            PlatformEvent::PointerMoved { x, y, .. }
            | PlatformEvent::PointerButton { x, y, .. } => self.snackbar.covers(*x, *y),
            _ => false,
        };
        if overlay_pointer && matches!(event, PlatformEvent::PointerButton { pressed: true, .. }) {
            self.search.blur();
        }
        if self.selected_category == 7 && (!overlay_pointer || self.swipe.pointer.is_some()) {
            if let Some(changed) = self.swipe_event(&event) {
                return changed;
            }
        }
        if self.selected_category == 7 && !overlay_pointer {
            if let Some(changed) = self.list_item_event(&event) {
                return changed;
            }
            if let Some(changed) = self.search.event(&event, self.scroll) {
                if matches!(
                    event,
                    PlatformEvent::Key {
                        key: acme_platform::PlatformKey::Tab,
                        pressed: true,
                        ..
                    }
                ) {
                    self.scroll = self
                        .search
                        .reveal_scroll(self.scroll)
                        .clamp(0.0, self.max_scroll);
                }
                return changed;
            }
        }
        match event {
            PlatformEvent::PointerMoved { x, y, .. } => {
                self.cursor = (x, y);
                self.sync_hit_layer();
                let _ = self
                    .click_detector
                    .on_pointer_move(&self.hit_layer, Point::new(x, y));
                let next = self.hit();
                let changed = next != self.hovered;
                self.hovered = next;
                changed
            }
            PlatformEvent::PointerButton {
                pressed, button, ..
            } => {
                self.sync_hit_layer();
                self.event_seq = self.event_seq.saturating_add(1);
                let pt = Point::new(self.cursor.0, self.cursor.1);
                let outcomes = self.click_detector.on_pointer_button(
                    &self.hit_layer,
                    button,
                    pressed,
                    pt,
                    self.event_seq as f64 * 0.25,
                );
                if pressed {
                    self.pressed = self.hit();
                    true
                } else {
                    let prev_pressed = self.pressed.take();
                    let mut activated_idx = None;
                    for outcome in outcomes {
                        if let PointerOutcome::Clicked { id, .. } = outcome {
                            if let Some(hit_idx) = self
                                .hit_layer
                                .hit_test(pt)
                                .filter(|hit| hit.id == id)
                                .map(|hit| hit.tag as usize)
                            {
                                if prev_pressed == Some(hit_idx) {
                                    activated_idx = Some(hit_idx);
                                    break;
                                }
                            }
                        }
                    }
                    activated_idx.is_some_and(|i| self.activate(i))
                }
            }
            PlatformEvent::Scroll { delta_y, .. } => {
                self.scroll = (self.scroll - delta_y).clamp(0.0, self.max_scroll);
                self.sync_hit_layer();
                true
            }
            PlatformEvent::Resized { .. } => true,
            _ => false,
        }
    }

    fn ime_cursor_area(&self, _window: WindowId) -> Option<[f32; 4]> {
        self.search.caret
    }

    fn on_gpu_recovered(&mut self, _window: WindowId) {
        self.atlas.clear();
    }

    fn background_poll_interval(&self) -> Option<std::time::Duration> {
        self.notification
            .poll_interval()
            .into_iter()
            .chain(self.snackbar.poll_interval())
            .chain(self.spinner.poll_interval())
            .min()
    }

    fn background_tick(&mut self) -> bool {
        let now = std::time::Instant::now();
        let notification_changed = self.notification.expire(now);
        let snackbar_changed = self.snackbar.expire(now);
        let spinner_changed = self.spinner.tick(now);
        if notification_changed || snackbar_changed {
            // Removing a card changes hit indices while a pointer may be held.
            self.reset_hover_press();
            true
        } else {
            spinner_changed
        }
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

        // 4. Reconcile and use stable IDs for layout, rendering and hit regions.
        let mut root = retained_layout_root(&description, &mut self.retained, &layout_context);

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

        self.spinner.visible = self.selected_category == 7
            && spinner_motion::visible(&description, &root, &snapshot, self.scroll, height);

        #[cfg(test)]
        {
            self.last_frame_layout = Some((root.clone(), snapshot.clone()));
        }

        // 7. Extract IDs
        let ids = extract_gallery_ids(&root);

        // 8. Collect hit regions
        let mut button_info = Vec::new();
        let mut hit_nodes = Vec::new();
        collect_hit_regions_with_nodes(
            &description,
            &root,
            &snapshot,
            false,
            &mut button_info,
            Some(&mut hit_nodes),
        );
        self.button_info = button_info;
        self.hit_nodes = hit_nodes;

        // 9. Scroll metrics
        self.max_scroll = snapshot
            .scroll_metrics(ids.scroll_view)
            .map(|m| (m.content_height - m.viewport_height).max(0.0))
            .unwrap_or(0.0);
        self.scroll = self.scroll.clamp(0.0, self.max_scroll);

        self.sync_swipe_bounds(
            &description,
            &root,
            &snapshot,
            snapshot
                .get(ids.scroll_view)
                .map(|r| [r.x, r.y, r.width, r.height]),
        );
        let colors = theme.colors;

        // 10. Build scene via Painter
        let mut painter = Painter::with_clear(theme_color(colors.background));

        // 11. Sidebar background
        if let Some(r) = snapshot.get(ids.sidebar) {
            painter.rect(
                Rect::new(r.x, r.y, r.width, r.height),
                RectStyle::solid(theme_color(colors.surface))
                    .border(1.0, theme_color(colors.border)),
            );
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
            painter.text_primitive(build_text_primitive(
                &mut self.fonts,
                &mut self.atlas,
                "AcmeUI",
                ([ox, oy], 18.0),
                colors.foreground,
                context.scale_factor,
                theme.typography.line_height,
            ));
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
            painter.rect(
                Rect::new(r.x, r.y, r.width, r.height),
                RectStyle::solid(theme_color(bg))
                    .radius(theme.radii.md)
                    .border(1.0, theme_color(colors.border)),
            );
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
            painter.text_primitive(build_text_primitive(
                &mut self.fonts,
                &mut self.atlas,
                label,
                ([ox, oy], theme.typography.label),
                fg,
                context.scale_factor,
                theme.typography.line_height,
            ));
        }

        // 14. Toolbar background
        if let Some(r) = snapshot.get(ids.toolbar) {
            painter.rect(
                Rect::new(r.x, r.y, r.width, r.height),
                RectStyle::solid(theme_color(colors.surface))
                    .border(1.0, theme_color(colors.border)),
            );
        }

        // 15. Toolbar buttons
        let tb_theme = self.current_theme();
        let tb_wcag = if tb_theme.meets_wcag_aa() {
            "✓AA"
        } else {
            "⚠AA"
        };
        let tb_labels = [
            (if self.dark { "☀ Light" } else { "🌙 Dark" }).to_string(),
            format!("🎨 {} {}", self.theme_pack, tb_wcag),
            "ℹ Info".to_string(),
        ];
        for (ti, (&btn_id, label_text)) in
            ids.toolbar_buttons.iter().zip(tb_labels.iter()).enumerate()
        {
            let btn_idx = ids.sidebar_buttons.len() + ti;
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
            painter.rect(
                Rect::new(r.x, r.y, r.width, r.height),
                RectStyle::solid(theme_color(resolved.background))
                    .radius(theme.radii.md)
                    .border(
                        1.0,
                        theme_color(if st.hovered {
                            colors.accent
                        } else {
                            colors.border
                        }),
                    ),
            );
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
            painter.text_primitive(build_text_primitive(
                &mut self.fonts,
                &mut self.atlas,
                label_text,
                ([tb_ox, tb_oy], 13.0),
                resolved.foreground,
                context.scale_factor,
                theme.typography.line_height,
            ));
        }

        if let Some(sv_rect) = snapshot.get(ids.scroll_view) {
            let clip = [sv_rect.x, sv_rect.y, sv_rect.width, sv_rect.height];
            self.viewport_clip = clip;
            let mut btn_idx = ids.sidebar_buttons.len() + ids.toolbar_buttons.len();
            if let Some((page_widget, page_layout)) = gallery_page_nodes(&description, &root) {
                let clip_rect = Rect::new(clip[0], clip[1], clip[2], clip[3]);
                let mut clip_guard = SceneClipGuard::push(&mut painter, clip_rect);
                render_content_painter(
                    clip_guard.painter(),
                    page_widget,
                    page_layout,
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
        }

        let mut scene = painter.finish_checked().expect("valid gallery scene");
        self.search.paint(
            &mut scene,
            &description,
            &root,
            &snapshot,
            &theme,
            context.scale_factor,
            self.scroll,
            &mut self.fonts,
            &mut self.atlas,
        );
        self.paint_snackbar(&mut scene, &context, &theme);
        self.sync_hit_layer();
        scene
    }
}

// ── Layout IDs ───────────────────────────────────────────────────────────────

/// Use only a complete binding; a failed reconcile or partial binding keeps
/// the entire frame on the original sequential layout IDs.
fn layout_root(
    description: &WidgetNode<GalleryMessage>,
    binding: Option<&RetainedBinding<'_>>,
    context: &WidgetLayoutContext,
) -> LayoutNode {
    match binding {
        Some(binding) if binding.mismatches() == 0 => {
            description.to_layout_with_binding(binding, context)
        }
        _ => description.to_layout_with_context(NodeId::new(1), context),
    }
}

struct RetainedLayoutState {
    tree: RetainedTree,
    failure_logged: bool,
}

impl RetainedLayoutState {
    fn new() -> Self {
        Self {
            tree: RetainedTree::new(),
            failure_logged: false,
        }
    }

    fn log_failure(&mut self, reason: std::fmt::Arguments<'_>) {
        if !self.failure_logged {
            eprintln!("AcmeUI Gallery: {reason}; using sequential layout IDs");
            self.failure_logged = true;
        }
    }

    fn accept_binding(&mut self, mismatches: usize) -> bool {
        if mismatches == 0 {
            self.failure_logged = false;
            true
        } else {
            self.log_failure(format_args!("retained binding has {mismatches} mismatches"));
            false
        }
    }
}

fn retained_layout_root(
    description: &WidgetNode<GalleryMessage>,
    retained: &mut RetainedLayoutState,
    context: &WidgetLayoutContext,
) -> LayoutNode {
    let roots = std::slice::from_ref(description);
    let binding = match retained.tree.reconcile_roots(&widget_to_views(roots)) {
        Ok(_) => {
            let binding = bind_retained(roots, &retained.tree);
            retained
                .accept_binding(binding.mismatches())
                .then_some(binding)
        }
        Err(error) => {
            retained.log_failure(format_args!("retained reconcile failed: {error:?}"));
            None
        }
    };
    layout_root(description, binding.as_ref(), context)
}

struct GalleryNodeIds {
    sidebar: NodeId,
    sidebar_label: NodeId,
    sidebar_buttons: Vec<NodeId>,
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
        sidebar_buttons: sb.children.iter().skip(2).map(|child| child.id).collect(),
        toolbar: tb.id,
        toolbar_buttons: [tb.children[0].id, tb.children[1].id, tb.children[2].id],
        scroll_view: ca.children[1].id,
    }
}

fn gallery_page_nodes<'a>(
    widget: &'a WidgetNode<GalleryMessage>,
    layout: &'a LayoutNode,
) -> Option<(&'a WidgetNode<GalleryMessage>, &'a LayoutNode)> {
    let widget_root = widget.children();
    let widget_content = widget_root.get(1)?;
    let widget_content_children = widget_content.children();
    let widget_scroll = widget_content_children.get(1)?;
    let widget_scroll_children = widget_scroll.children();
    let page_widget = widget_scroll_children.first()?;

    let page_layout = layout.children.get(1)?.children.get(1)?.children.first()?;
    Some((page_widget, page_layout))
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
    for button in sb.children.iter_mut().skip(2) {
        button.style.width = Length::px(SIDEBAR_WIDTH - 24.0);
        button.style.height = Length::px(38.0);
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

/// RAII clip scope that guarantees `PopClip` is balanced with `PushClip`.
struct SceneClipGuard<'a, 'b> {
    painter: &'a mut Painter<'b>,
}

impl<'a, 'b> SceneClipGuard<'a, 'b> {
    fn push(painter: &'a mut Painter<'b>, clip_rect: Rect<acme_core::Logical>) -> Self {
        painter.scene_mut().push(DrawCommand::PushClip(clip_rect));
        Self { painter }
    }

    fn painter(&mut self) -> &mut Painter<'b> {
        self.painter
    }
}

impl Drop for SceneClipGuard<'_, '_> {
    fn drop(&mut self) {
        self.painter.scene_mut().push(DrawCommand::PopClip);
    }
}

#[allow(clippy::too_many_arguments)]
fn render_content_painter(
    painter: &mut Painter<'_>,
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
    let row_index = collapsible::row_action(widget).and_then(|_| {
        snapshot.get(layout.id).map(|_| {
            let index = *btn_idx;
            *btn_idx += 1;
            index
        })
    });
    // Styled rows and stacks include switch tracks, card media and avatars.
    if let WidgetNode::Row(container) | WidgetNode::Stack(container) = widget {
        if let (Some(background), Some(rect)) =
            (container.style.background, snapshot.get(layout.id))
        {
            let fill = match row_index {
                Some(index) if pressed == Some(index) => {
                    if background == acme_widgets::style::ColorToken::DangerSoft {
                        colors.danger_soft
                    } else {
                        colors.ghost_pressed
                    }
                }
                Some(index) if hovered == Some(index) => {
                    if background == acme_widgets::style::ColorToken::DangerSoft {
                        colors.danger_soft
                    } else {
                        colors.ghost_hover
                    }
                }
                _ => background.resolve(theme),
            };
            painter.rect(
                Rect::new(rect.x, rect.y - scroll_y, rect.width, rect.height),
                RectStyle::solid(theme_color(fill))
                    .radius(container.style.border_radius.unwrap_or(0.0)),
            );
        }
    }
    match widget {
        WidgetNode::Label(l) => {
            if l.font_size.is_some_and(|fs| fs <= 0.0) {
                return;
            }
            if let Some(rect) = snapshot.get(layout.id) {
                let fs = l.font_size.unwrap_or(theme.typography.body);
                let y_text = rect.y - scroll_y;
                let text_color = l
                    .style
                    .text_color
                    .map(|token| token.resolve(theme))
                    .or(l.color)
                    .unwrap_or(colors.foreground);

                let mut display_text = l.text.as_str();
                let wrap = match l.wrap {
                    acme_widgets::LabelWrap::None => acme_text::TextWrap::None,
                    acme_widgets::LabelWrap::Word => acme_text::TextWrap::Word,
                    acme_widgets::LabelWrap::Character => acme_text::TextWrap::Glyph,
                };
                let wrapped = l.wrap != acme_widgets::LabelWrap::None;
                let alignment = l.style.text_align.unwrap_or(TextAlign::Left);
                let aligned = !wrapped && alignment != TextAlign::Left;
                let mut text_x = rect.x + if wrapped || aligned { 0.0 } else { 4.0 };
                // Color swatch label: "\x01{color_name}|{display_text}"
                if let Some((color_name, display)) = l
                    .text
                    .strip_prefix('\x01')
                    .and_then(|rest| rest.split_once('|'))
                {
                    if let Some(swatch_color) = resolve_swatch_color(theme, color_name) {
                        let swatch_size = fs * 0.85;
                        let swatch_x = rect.x + 4.0;
                        let swatch_y = y_text + (fs * 1.5 - swatch_size).max(0.0) / 2.0;
                        painter.rect(
                            Rect::new(swatch_x, swatch_y, swatch_size, swatch_size),
                            RectStyle::solid(theme_color(swatch_color))
                                .radius(theme.radii.sm.min(swatch_size / 2.0)),
                        );
                        display_text = display;
                        text_x = swatch_x + swatch_size + 8.0;
                    }
                }
                let shaped = fonts.shape(
                    display_text,
                    &TextStyle {
                        font_size: fs,
                        line_height: l.line_height.unwrap_or(fs * theme.typography.line_height),
                        ..TextStyle::default()
                    },
                    TextConstraints {
                        max_width: wrapped.then_some((rect.width - (text_x - rect.x)).max(0.0)),
                        wrap,
                    },
                    scale,
                );
                let prepared = fonts.prepare(&shaped, atlas);
                let primitive = text_primitive_from_prepared(
                    [text_x, y_text + shaped.baseline],
                    theme_color(text_color),
                    prepared,
                );
                if aligned {
                    let remaining = (rect.width - shaped.width).max(0.0);
                    let aligned_x = rect.x
                        + if alignment == TextAlign::Center {
                            remaining / 2.0
                        } else {
                            remaining
                        };
                    let x = rect.x.max(clip[0]);
                    let y = y_text.max(clip[1]);
                    let right = (rect.x + rect.width).min(clip[0] + clip[2]);
                    let bottom = (y_text + rect.height).min(clip[1] + clip[3]);
                    let aligned_primitive = TextPrimitive {
                        origin: Point::new(aligned_x, y_text + shaped.baseline),
                        ..primitive
                    };
                    painter.with_clip(
                        Rect::new(x, y, (right - x).max(0.0), (bottom - y).max(0.0)),
                        |painter| {
                            painter.text_primitive(aligned_primitive);
                        },
                    );
                } else {
                    painter.text_primitive(primitive);
                }
            }
        }
        WidgetNode::Button(btn) => {
            if let Some(rect) = snapshot.get(layout.id) {
                let y = rect.y - scroll_y;
                let st = button_state_for_render(btn, btn_idx, hovered, pressed);
                let resolved = btn.resolve_style(theme, st);
                painter.rect(
                    Rect::new(rect.x, y, rect.width, rect.height),
                    RectStyle::solid(theme_color(resolved.background))
                        .radius(btn.style.border_radius.unwrap_or(theme.radii.md))
                        .border(
                            btn.style.border_width.unwrap_or(1.0),
                            theme_color(resolved.border),
                        ),
                );
                let shaped = fonts.shape(
                    &btn.label,
                    &TextStyle {
                        font_size: theme.typography.label,
                        line_height: theme.typography.label * theme.typography.line_height,
                        ..TextStyle::default()
                    },
                    TextConstraints {
                        max_width: None,
                        wrap: acme_text::TextWrap::None,
                    },
                    scale,
                );
                let prepared = fonts.prepare(&shaped, atlas);
                let btn_clip = intersect_clip([rect.x, y, rect.width, rect.height], clip);
                let origin = [
                    rect.x + (rect.width - shaped.width).max(0.0) / 2.0,
                    y + (rect.height - shaped.height).max(0.0) / 2.0 + shaped.baseline,
                ];
                painter.with_clip(
                    Rect::new(btn_clip[0], btn_clip[1], btn_clip[2], btn_clip[3]),
                    |painter| {
                        painter.text_primitive(text_primitive_from_prepared(
                            origin,
                            theme_color(resolved.foreground),
                            prepared,
                        ));
                    },
                );
            }
        }
        WidgetNode::Separator(_) => {
            if let Some(rect) = snapshot.get(layout.id) {
                let y = rect.y - scroll_y;
                painter.fill_rect(
                    Rect::new(rect.x, y, rect.width, 1.0),
                    theme_color(colors.border),
                );
            }
        }
        WidgetNode::Card(c) => {
            if let Some(rect) = snapshot.get(layout.id) {
                let y = rect.y - scroll_y;
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
                let radius = c.border_radius.unwrap_or(theme.radii.sm);
                if c.variant == CardVariant::Elevated {
                    paint_card_shadow_painter(
                        painter,
                        [rect.x, y, rect.width, rect.height],
                        radius,
                        theme.shadows.sm,
                    );
                }
                painter.rect(
                    Rect::new(rect.x, y, rect.width, rect.height),
                    RectStyle::solid(theme_color(bg))
                        .radius(radius)
                        .border(c.style.border_width.unwrap_or(1.0), theme_color(border)),
                );
            }
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                render_content_painter(
                    painter, w, l, snapshot, theme, scale, scroll_y, clip, btn_idx, hovered,
                    pressed, fonts, atlas,
                );
            }
        }
        WidgetNode::Canvas(c) => {
            if let Some(rect) = snapshot.get(layout.id)
                && rect.width > 0.0
                && rect.height > 0.0
            {
                if c.activate().is_some() {
                    *btn_idx += 1;
                }
                let canvas_rect = Rect::new(rect.x, rect.y - scroll_y, rect.width, rect.height);
                let clip_rect = Rect::new(clip[0], clip[1], clip[2], clip[3]);
                // Keep Painter's active clip stack synchronized while preserving outer PushClip retention.
                painter.with_clip(clip_rect, |painter| {
                    paint_canvas_with_theme(c, canvas_rect, theme, painter);
                });
            }
        }
        WidgetNode::TextInput(ti) => {
            if ti.key.as_str() == "gallery_search__input" {
                return;
            }
            if let Some(rect) = snapshot.get(layout.id) {
                let y = rect.y - scroll_y;
                painter.rect(
                    Rect::new(rect.x, y, rect.width, rect.height),
                    RectStyle::solid(theme_color(colors.surface))
                        .radius(theme.radii.md)
                        .border(1.0, theme_color(colors.input)),
                );
                let (text, color) = if ti.value.is_empty() {
                    (
                        ti.placeholder.as_deref().unwrap_or(""),
                        colors.muted_foreground,
                    )
                } else {
                    (ti.value.as_str(), colors.foreground)
                };
                if !text.is_empty() {
                    let fs = theme.typography.body;
                    let shaped = fonts.shape(
                        text,
                        &TextStyle {
                            font_size: fs,
                            line_height: fs * theme.typography.line_height,
                            ..TextStyle::default()
                        },
                        TextConstraints::default(),
                        scale,
                    );
                    let prepared = fonts.prepare(&shaped, atlas);
                    let text_clip = intersect_clip([rect.x, y, rect.width, rect.height], clip);
                    let origin = [
                        rect.x + theme.spacing.px2,
                        y + (rect.height - shaped.height).max(0.0) / 2.0 + shaped.baseline,
                    ];
                    painter.with_clip(
                        Rect::new(text_clip[0], text_clip[1], text_clip[2], text_clip[3]),
                        |painter| {
                            painter.text_primitive(text_primitive_from_prepared(
                                origin,
                                theme_color(color),
                                prepared,
                            ));
                        },
                    );
                }
            }
        }
        WidgetNode::Tooltip(t) => {
            render_content_painter(
                painter, &t.child, layout, snapshot, theme, scale, scroll_y, clip, btn_idx,
                hovered, pressed, fonts, atlas,
            );
        }
        WidgetNode::Popover(p) => {
            render_content_painter(
                painter,
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
                render_content_painter(
                    painter, w, l, snapshot, theme, scale, scroll_y, clip, btn_idx, hovered,
                    pressed, fonts, atlas,
                );
            }
        }
    }
}

#[cfg(test)]
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
    let row_index = collapsible::row_action(widget).and_then(|_| {
        snapshot.get(layout.id).map(|_| {
            let index = *btn_idx;
            *btn_idx += 1;
            index
        })
    });
    // Styled rows and stacks include switch tracks, card media and avatars.
    if let WidgetNode::Row(container) | WidgetNode::Stack(container) = widget {
        if let (Some(background), Some(rect)) =
            (container.style.background, snapshot.get(layout.id))
        {
            let fill = match row_index {
                Some(index) if pressed == Some(index) => {
                    if background == acme_widgets::style::ColorToken::DangerSoft {
                        colors.danger_soft
                    } else {
                        colors.ghost_pressed
                    }
                }
                Some(index) if hovered == Some(index) => {
                    if background == acme_widgets::style::ColorToken::DangerSoft {
                        colors.danger_soft
                    } else {
                        colors.ghost_hover
                    }
                }
                _ => background.resolve(theme),
            };
            frame.clipped_quads.push(ClippedQuad {
                quad: quad_rect(
                    [rect.x, rect.y - scroll_y, rect.width, rect.height],
                    fill,
                    container.style.border_radius.unwrap_or(0.0),
                    0.0,
                    fill,
                ),
                clip,
            });
        }
    }
    match widget {
        WidgetNode::Label(l) => {
            if let Some(rect) = snapshot.get(layout.id) {
                let fs = l.font_size.unwrap_or(theme.typography.body);
                let y_text = rect.y - scroll_y;
                let text_color = l
                    .style
                    .text_color
                    .map(|token| token.resolve(theme))
                    .or(l.color)
                    .unwrap_or(colors.foreground);

                let mut display_text = l.text.as_str();
                // Wrapped text must use the same content width as layout.
                // Keep the existing inset for unwrapped labels and swatches.
                let wrap = match l.wrap {
                    acme_widgets::LabelWrap::None => acme_text::TextWrap::None,
                    acme_widgets::LabelWrap::Word => acme_text::TextWrap::Word,
                    acme_widgets::LabelWrap::Character => acme_text::TextWrap::Glyph,
                };
                let wrapped = l.wrap != acme_widgets::LabelWrap::None;
                let alignment = l.style.text_align.unwrap_or(TextAlign::Left);
                let aligned = !wrapped && alignment != TextAlign::Left;
                let mut text_x = rect.x + if wrapped || aligned { 0.0 } else { 4.0 };
                // Color swatch label: "\x01{color_name}|{display_text}"
                if let Some((color_name, display)) = l
                    .text
                    .strip_prefix('\x01')
                    .and_then(|rest| rest.split_once('|'))
                {
                    if let Some(swatch_color) = resolve_swatch_color(theme, color_name) {
                        let swatch_size = fs * 0.85;
                        let swatch_x = rect.x + 4.0;
                        let swatch_y = y_text + (fs * 1.5 - swatch_size).max(0.0) / 2.0;
                        frame.clipped_quads.push(ClippedQuad {
                            quad: quad_rect(
                                [swatch_x, swatch_y, swatch_size, swatch_size],
                                swatch_color,
                                theme.radii.sm.min(swatch_size / 2.0),
                                0.0,
                                swatch_color,
                            ),
                            clip,
                        });
                        display_text = display;
                        text_x = swatch_x + swatch_size + 8.0;
                    }
                }
                let shaped = fonts.shape(
                    display_text,
                    &TextStyle {
                        font_size: fs,
                        line_height: l.line_height.unwrap_or(fs * theme.typography.line_height),
                        ..TextStyle::default()
                    },
                    TextConstraints {
                        max_width: wrapped.then_some((rect.width - (text_x - rect.x)).max(0.0)),
                        wrap,
                    },
                    scale,
                );
                // Single-line alignment uses the existing style contract.
                // Clip aligned text to its content box (e.g. avatar initials).
                let text_clip = if aligned {
                    let remaining = (rect.width - shaped.width).max(0.0);
                    text_x = rect.x
                        + if alignment == TextAlign::Center {
                            remaining / 2.0
                        } else {
                            remaining
                        };
                    let x = rect.x.max(clip[0]);
                    let y = y_text.max(clip[1]);
                    let right = (rect.x + rect.width).min(clip[0] + clip[2]);
                    let bottom = (y_text + rect.height).min(clip[1] + clip[3]);
                    [x, y, (right - x).max(0.0), (bottom - y).max(0.0)]
                } else {
                    clip
                };
                // Prepared glyph offsets are relative to the first baseline;
                // the layout rectangle supplies the top of the label instead.
                frame.text.push(TextRun {
                    prepared: fonts.prepare(&shaped, atlas),
                    origin: [text_x, y_text + shaped.baseline],
                    color: rgba(text_color),
                    clip: Some(text_clip),
                });
            }
        }
        WidgetNode::Button(btn) => {
            if let Some(rect) = snapshot.get(layout.id) {
                let y = rect.y - scroll_y;
                let st = button_state_for_render(btn, btn_idx, hovered, pressed);
                let resolved = btn.resolve_style(theme, st);
                frame.clipped_quads.push(ClippedQuad {
                    quad: quad_rect(
                        [rect.x, y, rect.width, rect.height],
                        resolved.background,
                        btn.style.border_radius.unwrap_or(theme.radii.md),
                        btn.style.border_width.unwrap_or(1.0),
                        resolved.border,
                    ),
                    clip,
                });
                let shaped = fonts.shape(
                    &btn.label,
                    &TextStyle {
                        font_size: theme.typography.label,
                        line_height: theme.typography.label * theme.typography.line_height,
                        ..TextStyle::default()
                    },
                    TextConstraints {
                        max_width: None,
                        wrap: acme_text::TextWrap::None,
                    },
                    scale,
                );
                frame.text.push(TextRun {
                    prepared: fonts.prepare(&shaped, atlas),
                    origin: [
                        rect.x + (rect.width - shaped.width).max(0.0) / 2.0,
                        y + (rect.height - shaped.height).max(0.0) / 2.0 + shaped.baseline,
                    ],
                    color: rgba(resolved.foreground),
                    clip: Some(intersect_clip([rect.x, y, rect.width, rect.height], clip)),
                });
            }
        }
        WidgetNode::Separator(_) => {
            if let Some(rect) = snapshot.get(layout.id) {
                let y = rect.y - scroll_y;
                frame.clipped_quads.push(ClippedQuad {
                    quad: quad_rect(
                        [rect.x, y, rect.width, 1.0],
                        colors.border,
                        0.0,
                        0.0,
                        colors.border,
                    ),
                    clip,
                });
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
                if c.variant == CardVariant::Elevated {
                    paint_card_shadow(
                        frame,
                        [rect.x, y, rect.width, rect.height],
                        radius,
                        theme.shadows.sm,
                        clip,
                    );
                }
                frame.clipped_quads.push(ClippedQuad {
                    quad: quad_rect(
                        [rect.x, y, rect.width, rect.height],
                        bg,
                        radius,
                        c.style.border_width.unwrap_or(1.0),
                        border,
                    ),
                    clip,
                });
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

fn button_state_for_render(
    button: &acme_widgets::Button<GalleryMessage>,
    interactive_index: &mut usize,
    hovered: Option<usize>,
    pressed: Option<usize>,
) -> ButtonState {
    let current = button.activate().map(|_| {
        let index = *interactive_index;
        *interactive_index += 1;
        index
    });
    ButtonState {
        hovered: current.is_some_and(|index| hovered == Some(index)),
        pressed: current.is_some_and(|index| pressed == Some(index)),
        focused: false,
    }
}

// ── Hit Region Collection ────────────────────────────────────────────────────

#[cfg(test)]
fn collect_hit_regions(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &acme_layout::LayoutSnapshot,
    scrolled: bool,
    result: &mut Vec<HitRegion>,
) {
    collect_hit_regions_with_nodes(widget, layout, snapshot, scrolled, result, None);
}

fn collect_hit_regions_with_nodes(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &acme_layout::LayoutSnapshot,
    scrolled: bool,
    result: &mut Vec<HitRegion>,
    mut nodes: Option<&mut Vec<NodeId>>,
) {
    if let Some(message) = collapsible::row_action(widget) {
        if let Some(rect) = snapshot.get(layout.id) {
            result.push(HitRegion {
                rect: [rect.x, rect.y, rect.width, rect.height],
                message,
                scrolled,
            });
            if let Some(nodes) = nodes.as_deref_mut() {
                nodes.push(layout.id);
            }
        }
    }
    match widget {
        WidgetNode::Button(btn) => {
            if let Some(msg) = btn.activate() {
                if let Some(rect) = snapshot.get(layout.id) {
                    result.push(HitRegion {
                        rect: [rect.x, rect.y, rect.width, rect.height],
                        message: *msg,
                        scrolled,
                    });
                    if let Some(nodes) = nodes.as_deref_mut() {
                        nodes.push(layout.id);
                    }
                }
            }
        }
        WidgetNode::Canvas(c) => {
            if let Some(msg) = c.activate() {
                if let Some(rect) = snapshot.get(layout.id) {
                    if rect.width > 0.0 && rect.height > 0.0 {
                        result.push(HitRegion {
                            rect: [rect.x, rect.y, rect.width, rect.height],
                            message: *msg,
                            scrolled,
                        });
                        if let Some(nodes) = nodes.as_deref_mut() {
                            nodes.push(layout.id);
                        }
                    }
                }
            }
        }
        WidgetNode::ScrollView(_) => {
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                collect_hit_regions_with_nodes(w, l, snapshot, true, result, nodes.as_deref_mut());
            }
        }
        WidgetNode::Tooltip(t) => {
            collect_hit_regions_with_nodes(&t.child, layout, snapshot, scrolled, result, nodes);
        }
        WidgetNode::Popover(p) => {
            collect_hit_regions_with_nodes(
                &p.children[0],
                layout,
                snapshot,
                scrolled,
                result,
                nodes,
            );
        }
        _ => {
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                collect_hit_regions_with_nodes(
                    w,
                    l,
                    snapshot,
                    scrolled,
                    result,
                    nodes.as_deref_mut(),
                );
            }
        }
    }
}

fn hit_layer_fingerprint(
    regions: &[HitRegion],
    scroll: f32,
    viewport_clip: [f32; 4],
    snackbar_bounds: Option<[f32; 4]>,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    regions.len().hash(&mut hasher);
    scroll.to_bits().hash(&mut hasher);
    for v in viewport_clip {
        v.to_bits().hash(&mut hasher);
    }
    if let Some(sb) = snackbar_bounds {
        1u8.hash(&mut hasher);
        for v in sb {
            v.to_bits().hash(&mut hasher);
        }
    } else {
        0u8.hash(&mut hasher);
    }
    for hr in regions {
        for v in hr.rect {
            v.to_bits().hash(&mut hasher);
        }
        hr.scrolled.hash(&mut hasher);
    }
    hasher.finish()
}

fn build_hit_layer(
    regions: &[HitRegion],
    nodes: &[NodeId],
    scroll: f32,
    viewport_clip: [f32; 4],
    snackbar_bounds: Option<[f32; 4]>,
) -> HitTestLayer {
    let mut layer = HitTestLayer::new();
    let clip_rect = Rect::new(
        viewport_clip[0],
        viewport_clip[1],
        viewport_clip[2],
        viewport_clip[3],
    );
    let mut overlay_indices = Vec::new();
    let use_nodes = nodes.len() == regions.len();
    for (idx, hr) in regions.iter().enumerate() {
        if snackbar::is_overlay_action(hr.message) {
            overlay_indices.push(idx);
            continue;
        }
        let id = if use_nodes {
            nodes[idx]
        } else {
            NodeId::new(100_000_000 + idx as u64)
        };
        let spec = HitRegionSpec::rect(
            id,
            Rect::new(hr.rect[0], hr.rect[1], hr.rect[2], hr.rect[3]),
        )
        .cursor(CursorHint::Pointer)
        .tag(idx as u64);
        if hr.scrolled {
            layer.with_scroll_viewport(clip_rect, 0.0, scroll, |layer| {
                layer.push_region(spec);
            });
        } else {
            layer.push_region(spec);
        }
    }
    if let Some(sb) = snackbar_bounds {
        layer.push_scrim(
            NodeId::new(u64::MAX - 1),
            Rect::new(sb[0], sb[1], sb[2], sb[3]),
            HitZLayer::Overlay,
            false,
        );
    }
    if !overlay_indices.is_empty() {
        layer.with_z_layer(HitZLayer::Overlay, |layer| {
            for idx in overlay_indices {
                let hr = &regions[idx];
                let id = if use_nodes {
                    nodes[idx]
                } else {
                    NodeId::new(100_000_000 + idx as u64)
                };
                layer.push_region(
                    HitRegionSpec::rect(
                        id,
                        Rect::new(hr.rect[0], hr.rect[1], hr.rect[2], hr.rect[3]),
                    )
                    .cursor(CursorHint::Pointer)
                    .tag(idx as u64),
                );
            }
        });
    }
    layer
}

// ── Render Helpers ───────────────────────────────────────────────────────────

fn intersect_clip(rect: [f32; 4], clip: [f32; 4]) -> [f32; 4] {
    let x = rect[0].max(clip[0]);
    let y = rect[1].max(clip[1]);
    [
        x,
        y,
        ((rect[0] + rect[2]).min(clip[0] + clip[2]) - x).max(0.0),
        ((rect[1] + rect[3]).min(clip[1] + clip[3]) - y).max(0.0),
    ]
}

fn paint_card_shadow_painter(
    painter: &mut Painter<'_>,
    rect: [f32; 4],
    radius: f32,
    shadow: acme_theme::ShadowDef,
) {
    for layer in (1..=4).rev() {
        let spread = shadow.spread + shadow.blur * layer as f32 / 8.0;
        let mut color = shadow.color;
        color.alpha /= 4.0;
        painter.rect(
            Rect::new(
                rect[0] + shadow.offset_x - spread,
                rect[1] + shadow.offset_y - spread,
                rect[2] + 2.0 * spread,
                rect[3] + 2.0 * spread,
            ),
            RectStyle::solid(theme_color(color)).radius(radius + spread),
        );
    }
}

fn build_text_primitive(
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
    text: &str,
    geometry: ([f32; 2], f32),
    color: ThemeColor,
    scale: f32,
    line_height_ratio: f32,
) -> TextPrimitive {
    let (origin, size) = geometry;
    let style = TextStyle {
        font_size: size,
        line_height: size * line_height_ratio,
        ..TextStyle::default()
    };
    let layout = fonts.shape(text, &style, TextConstraints::default(), scale);
    let prepared = fonts.prepare(&layout, atlas);
    text_primitive_from_prepared(origin, theme_color(color), prepared)
}

fn text_primitive_from_prepared(
    origin: [f32; 2],
    color: Color,
    prepared: PreparedText,
) -> TextPrimitive {
    let glyphs = prepared
        .glyphs
        .iter()
        .map(|glyph| GlyphDraw {
            x: glyph.x as f32,
            y: glyph.y as f32,
            width: glyph.width as f32,
            height: glyph.height as f32,
            atlas_x: glyph.atlas_x,
            atlas_y: glyph.atlas_y,
            atlas_page: glyph.atlas_page,
            format: match glyph.format {
                acme_text::AtlasFormat::Alpha8 => GlyphFormat::Alpha8,
                acme_text::AtlasFormat::Rgba8 => GlyphFormat::Rgba8,
            },
        })
        .collect();
    let uploads = prepared
        .uploads
        .iter()
        .map(|region| AtlasUpload {
            page: region.page,
            origin: [region.x, region.y],
            size: [region.width, region.height],
            format: match region.format {
                acme_text::AtlasFormat::Alpha8 => GlyphFormat::Alpha8,
                acme_text::AtlasFormat::Rgba8 => GlyphFormat::Rgba8,
            },
            pixels: region.pixels.clone(),
        })
        .collect();
    TextPrimitive {
        origin: Point::new(origin[0], origin[1]),
        color,
        glyphs,
        uploads,
    }
}

/// A bounded layered approximation, using the theme's elevation token. This
/// Gallery path emits ordinary clipped quads; it is not a renderer blur effect.
#[cfg(test)]
fn paint_card_shadow(
    frame: &mut Frame,
    rect: [f32; 4],
    radius: f32,
    shadow: acme_theme::ShadowDef,
    clip: [f32; 4],
) {
    for layer in (1..=4).rev() {
        let spread = shadow.spread + shadow.blur * layer as f32 / 8.0;
        let mut color = shadow.color;
        color.alpha /= 4.0;
        frame.clipped_quads.push(ClippedQuad {
            quad: quad_rect(
                [
                    rect[0] + shadow.offset_x - spread,
                    rect[1] + shadow.offset_y - spread,
                    rect[2] + 2.0 * spread,
                    rect[3] + 2.0 * spread,
                ],
                color,
                radius + spread,
                0.0,
                color,
            ),
            clip,
        });
    }
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

#[cfg(test)]
fn rgba(color: ThemeColor) -> [f32; 4] {
    [color.red, color.green, color.blue, color.alpha]
}

#[cfg(test)]
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

#[cfg(test)]
mod retained_tests;

#[cfg(test)]
mod mobile_card_tests;

#[cfg(test)]
mod mobile_progress_tests;

#[cfg(test)]
mod mobile_loader_tests;

#[cfg(test)]
mod tests {
    use super::*;

    fn layout_context() -> WidgetLayoutContext {
        WidgetLayoutContext {
            body_font_size: 14.0,
            body_line_height: 20.0,
            label_font_size: 13.0,
            control_height: 36.0,
            scale_factor: 1.0,
        }
    }

    #[test]
    fn sidebar_ids_follow_category_catalog() {
        let gallery = Gallery::new();
        let description = gallery.description();
        let root = description.to_layout_with_context(NodeId::new(1), &layout_context());
        let ids = extract_gallery_ids(&root);
        let (page_widget, page_layout) = gallery_page_nodes(&description, &root).unwrap();

        assert_eq!(ids.sidebar_buttons.len(), CATEGORIES.len());
        assert_eq!(gallery.selected_category, CATEGORIES.len() - 1);
        assert!(matches!(page_widget, WidgetNode::Column(_)));
        assert_eq!(page_layout.id, root.children[1].children[1].children[0].id);
    }

    #[test]
    fn disabled_button_renders_without_consuming_hit_index() {
        let WidgetNode::Button(disabled) = WidgetNode::from(
            mobile_button("disabled", "Disabled")
                .disabled(true)
                .on_press(GalleryMessage::DpiInfo),
        ) else {
            panic!("expected Button");
        };
        let mut index = 11;

        let state = button_state_for_render(&disabled, &mut index, Some(11), Some(11));

        assert_eq!(state, ButtonState::default());
        assert_eq!(index, 11);
    }

    #[test]
    fn interactive_button_consumes_hit_index() {
        let WidgetNode::Button(enabled) =
            WidgetNode::from(mobile_button("enabled", "Enabled").on_press(GalleryMessage::DpiInfo))
        else {
            panic!("expected Button");
        };
        let mut index = 11;

        let state = button_state_for_render(&enabled, &mut index, Some(11), None);

        assert!(state.hovered);
        assert!(!state.pressed);
        assert_eq!(index, 12);
    }

    fn mobile_hits(gallery: &Gallery) -> Vec<HitRegion> {
        let widget = gallery.mobile_page();
        let layout = widget.to_layout_with_context(NodeId::new(1), &layout_context());
        let snapshot = LayoutEngine::new()
            .compute(&layout, (900.0, 1600.0))
            .unwrap();
        let mut hits = Vec::new();
        collect_hit_regions(&widget, &layout, &snapshot, false, &mut hits);
        hits
    }

    #[test]
    fn mobile_controls_dispatch_and_rebuild_their_state() {
        let mut gallery = Gallery::new();
        for message in [
            GalleryMessage::MobileToggle(true),
            GalleryMessage::MobileValue(4),
            GalleryMessage::MobileSelected(2),
        ] {
            gallery.button_info = mobile_hits(&gallery);
            let index = gallery
                .button_info
                .iter()
                .position(|hit| hit.message == message)
                .expect("mobile control has a laid-out hit area");
            let [x, y, w, h] = gallery.button_info[index].rect;
            assert!(h >= 44.0);
            gallery.cursor = (x + w / 2.0, y + h / 2.0);
            assert_eq!(gallery.hit(), Some(index));
            assert!(gallery.activate(index));
        }
        assert!(gallery.mobile_checked);
        assert_eq!(gallery.mobile_value, 4);
        assert_eq!(gallery.mobile_selected, 2);
        let hits = mobile_hits(&gallery);
        assert!(
            hits.iter()
                .any(|h| h.message == GalleryMessage::MobileToggle(false))
        );
        assert!(
            hits.iter()
                .any(|h| h.message == GalleryMessage::MobileValue(6))
        );
    }

    #[test]
    fn mobile_stepper_limits_do_not_leave_ghost_hit_indices() {
        let mut gallery = Gallery::new();
        let full_hit_count = mobile_hits(&gallery).len();
        for value in [0, 10] {
            gallery.mobile_value = value;
            let hits = mobile_hits(&gallery);
            let values: Vec<i64> = hits
                .iter()
                .filter_map(|hit| match hit.message {
                    GalleryMessage::MobileValue(value) => Some(value),
                    _ => None,
                })
                .collect();
            assert_eq!(values, if value == 0 { vec![2] } else { vec![8] });
            assert_eq!(hits.len() + 1, full_hit_count);
        }
    }

    #[test]
    fn mobile_toggle_frame_contains_themed_track_and_thumb() {
        let mut gallery = Gallery::new();
        for dark in [false, true] {
            gallery.dark = dark;
            let theme = gallery.current_theme();
            for checked in [false, true] {
                let widget =
                    WidgetNode::from(mobile_toggle::<GalleryMessage>("Wi-Fi").checked(checked));
                let layout = widget.to_layout_with_context(NodeId::new(1), &layout_context());
                let snapshot = LayoutEngine::new()
                    .compute(&layout, (400.0, 100.0))
                    .unwrap();
                let mut frame = Frame::default();
                render_content(
                    &mut frame,
                    &widget,
                    &layout,
                    &snapshot,
                    &theme,
                    1.0,
                    0.0,
                    [0.0, 0.0, 400.0, 100.0],
                    &mut 0,
                    None,
                    None,
                    &mut gallery.fonts,
                    &mut gallery.atlas,
                );
                let track = frame
                    .clipped_quads
                    .iter()
                    .map(|q| &q.quad)
                    .find(|q| q.rect[2..] == [52.0, 32.0])
                    .expect("switch track must be painted, not just laid out");
                let thumb = frame
                    .clipped_quads
                    .iter()
                    .map(|q| &q.quad)
                    .find(|q| q.rect[2..] == [24.0, 24.0])
                    .expect("switch thumb must be painted");
                assert_eq!(
                    track.color,
                    rgba(if checked {
                        theme.colors.primary
                    } else {
                        theme.colors.muted
                    })
                );
                assert_eq!(thumb.color, rgba(theme.colors.surface));
                assert_eq!(
                    thumb.rect[0] - track.rect[0],
                    if checked { 24.0 } else { 4.0 }
                );
            }
        }
    }

    #[test]
    fn mobile_toggle_paint_keeps_the_scroll_view_clip() {
        let mut gallery = Gallery::new();
        let widget = WidgetNode::from(mobile_toggle::<GalleryMessage>("Wi-Fi"));
        let layout = widget.to_layout_with_context(NodeId::new(1), &layout_context());
        let snapshot = LayoutEngine::new()
            .compute(&layout, (400.0, 100.0))
            .unwrap();
        let theme = gallery.current_theme();
        for scroll in [12.0, 100.0] {
            let mut frame = Frame::default();
            let clip = [0.0, 10.0, 400.0, 50.0];
            render_content(
                &mut frame,
                &widget,
                &layout,
                &snapshot,
                &theme,
                1.0,
                scroll,
                clip,
                &mut 0,
                None,
                None,
                &mut gallery.fonts,
                &mut gallery.atlas,
            );
            assert_eq!(
                frame.clipped_quads.len(),
                3,
                "host, track and thumb need a clip even when fully scrolled out"
            );
            assert!(frame.clipped_quads.iter().all(|q| q.clip == clip));
            assert!(
                !frame
                    .quads
                    .iter()
                    .any(|q| q.rect[2..] == [52.0, 32.0] || q.rect[2..] == [24.0, 24.0])
            );
        }
    }

    fn pointer_click_message(g: &mut Gallery, width: f32, height: f32, message: GalleryMessage) {
        g.frame(FrameContext {
            window: WindowId(1),
            logical_width: width,
            logical_height: height,
            scale_factor: 1.0,
        });
        let hr = g
            .button_info
            .iter()
            .find(|h| h.message == message)
            .expect("expected target hit region");
        if hr.scrolled {
            g.scroll = (hr.rect[1] - 120.0).clamp(0.0, g.max_scroll);
            g.frame(FrameContext {
                window: WindowId(1),
                logical_width: width,
                logical_height: height,
                scale_factor: 1.0,
            });
        }
        let hr = g
            .button_info
            .iter()
            .find(|h| h.message == message)
            .expect("expected target hit region after scroll");
        let y_off = if hr.scrolled { g.scroll } else { 0.0 };
        let (x, y) = (
            hr.rect[0] + hr.rect[2] / 2.0,
            hr.rect[1] + hr.rect[3] / 2.0 - y_off,
        );
        g.event(PlatformEvent::PointerMoved {
            window: WindowId(1),
            x,
            y,
        });
        assert!(g.event(PlatformEvent::PointerButton {
            window: WindowId(1),
            pressed: true,
            x,
            y,
            button: 0,
            pointer: 0,
            shift: false,
            ctrl: false,
            alt: false,
        }));
        assert!(g.event(PlatformEvent::PointerButton {
            window: WindowId(1),
            pressed: false,
            x,
            y,
            button: 0,
            pointer: 0,
            shift: false,
            ctrl: false,
            alt: false,
        }));
    }

    #[test]
    fn stage3_painter_emits_canvas_commands_for_charts_and_foundations() {
        let mut g = Gallery::new();
        // Charts page: BarChart, LineChart, Gauge, Sparkline all emit Canvas draw commands via Painter.
        g.selected_category = 6;
        let scene = g.frame(FrameContext {
            window: WindowId(1),
            logical_width: 1100.0,
            logical_height: 900.0,
            scale_factor: 1.0,
        });
        let charts_clips = scene
            .commands()
            .iter()
            .filter(|cmd| matches!(cmd, DrawCommand::PushClip(_)))
            .count();
        let charts_quads = scene
            .commands()
            .iter()
            .filter(|cmd| matches!(cmd, DrawCommand::Quad(_)))
            .count();
        assert!(
            charts_clips >= 5,
            "Charts page should push scroll clip + 4 Canvas with_clip scopes, got {charts_clips}"
        );
        assert!(charts_quads > 15);

        // Foundations page: ProgressRing and QRCode emit Canvas commands + live_region skips visible text.
        g.selected_category = 1;
        let foundations_scene = g.frame(FrameContext {
            window: WindowId(1),
            logical_width: 1100.0,
            logical_height: 900.0,
            scale_factor: 1.0,
        });
        let foundations_clips = foundations_scene
            .commands()
            .iter()
            .filter(|cmd| matches!(cmd, DrawCommand::PushClip(_)))
            .count();
        assert!(
            foundations_clips >= 3,
            "Foundations page should include ProgressRing + QRCode Canvas clips, got {foundations_clips}"
        );
    }

    #[test]
    fn stage3_high_contrast_theme_pack_selection_works() {
        let prev_pref = std::fs::read(THEME_PREF_FILE).ok();
        let mut g = Gallery::new();
        g.theme_pack = "default".into();
        let default_primary = g.current_theme().colors.primary;

        g.selected_category = 0; // Design System
        pointer_click_message(&mut g, 1100.0, 900.0, GalleryMessage::SelectHighContrast);
        assert_eq!(g.theme_pack, "high-contrast");
        let hc_theme = g.current_theme();
        assert!(hc_theme.meets_wcag_aa());
        assert_ne!(hc_theme.colors.primary, default_primary);

        // CycleTheme reaches "high-contrast" across the pack list.
        g.theme_pack = "default".into();
        let mut visited = Vec::new();
        for _ in 0..=available_themes().len() {
            g.cycle_theme();
            visited.push(g.theme_pack.clone());
        }
        assert!(visited.iter().any(|p| p == "high-contrast"));

        if let Some(bytes) = prev_pref {
            let _ = std::fs::write(THEME_PREF_FILE, bytes);
        } else {
            let _ = std::fs::remove_file(THEME_PREF_FILE);
        }
    }

    #[test]
    fn stage3_hit_layer_routes_canvas_clicks_and_respects_scroll_and_scrim_clip() {
        let mut g = Gallery::new();

        // 1. Interactive Canvas click routing: Charts interactive Canvas + Drawer close.
        g.selected_category = 6; // Charts
        assert_eq!(g.canvas_pings, 0);
        pointer_click_message(&mut g, 1100.0, 900.0, GalleryMessage::CanvasPing);
        assert_eq!(g.canvas_pings, 1);

        g.selected_category = 4; // Overlay
        pointer_click_message(&mut g, 1100.0, 900.0, GalleryMessage::DrawerToggle);
        assert!(g.drawer_open);
        pointer_click_message(&mut g, 1100.0, 900.0, GalleryMessage::DrawerClose);
        assert!(!g.drawer_open);

        // 2. Inputs page interactive builders (DatePicker, Combobox, SliderMarks).
        g.selected_category = 2; // Inputs
        pointer_click_message(&mut g, 1100.0, 900.0, GalleryMessage::DatePickerToggle);
        assert!(g.date_picker_open);
        pointer_click_message(&mut g, 1100.0, 900.0, GalleryMessage::ComboboxToggle);
        assert!(g.combobox_open);
        pointer_click_message(&mut g, 1100.0, 900.0, GalleryMessage::SliderMarkSelect(75));
        assert!((g.slider_value - 75.0).abs() < f32::EPSILON);

        // 3. Mobile page P4 builders (MobileTabBar, MobileDrawer, MobileCarousel).
        g.selected_category = CATEGORIES.len() - 1;
        pointer_click_message(&mut g, 1100.0, 900.0, GalleryMessage::MobileTabSelect(2));
        assert_eq!(g.mobile_tab_index, 2);
        pointer_click_message(&mut g, 1100.0, 900.0, GalleryMessage::MobileCarouselNext);
        assert_eq!(g.mobile_carousel_index, 1);
        pointer_click_message(&mut g, 1100.0, 900.0, GalleryMessage::MobileDrawerToggle);
        assert!(g.mobile_drawer_open);
        pointer_click_message(&mut g, 1100.0, 900.0, GalleryMessage::MobileDrawerClose);
        assert!(!g.mobile_drawer_open);

        // 4. Scroll viewport clipping + snackbar scrim blocking via HitTestLayer.
        g.scroll = 0.0;
        g.frame(FrameContext {
            window: WindowId(1),
            logical_width: 1100.0,
            logical_height: 700.0,
            scale_factor: 1.0,
        });
        let offscreen_below = g
            .button_info
            .iter()
            .find(|h| h.scrolled && h.rect[1] > 760.0)
            .expect("mobile page has buttons below 760px");
        g.event(PlatformEvent::PointerMoved {
            window: WindowId(1),
            x: offscreen_below.rect[0] + 10.0,
            y: offscreen_below.rect[1] + 10.0,
        });
        assert_eq!(
            g.hit(),
            None,
            "HitTestLayer must reject scrolled targets below viewport clip"
        );

        // Trigger snackbar scrim and verify click-through under scrim bounds is blocked.
        g.snackbar.show(std::time::Instant::now(), true);
        g.frame(FrameContext {
            window: WindowId(1),
            logical_width: 1100.0,
            logical_height: 700.0,
            scale_factor: 1.0,
        });
        let sb = g.snackbar.bounds.expect("snackbar bounds painted");
        // Point near top-left edge inside snackbar card scrim (away from Action/Dismiss buttons).
        g.event(PlatformEvent::PointerMoved {
            window: WindowId(1),
            x: sb[0] + 4.0,
            y: sb[1] + 4.0,
        });
        assert_eq!(
            g.hit(),
            None,
            "snackbar scrim must block underlying page hits"
        );
    }
}

#[cfg(test)]
mod mobile_search_tests;

#[cfg(test)]
mod mobile_chip_tests;

#[cfg(test)]
mod mobile_avatar_tests;

#[cfg(test)]
mod mobile_banner_tests;

#[cfg(test)]
mod mobile_notification_tests;

#[cfg(test)]
mod mobile_list_item_tests;

#[cfg(test)]
mod mobile_action_tests;

#[cfg(test)]
mod mobile_sheet_handle_tests;

#[cfg(test)]
mod mobile_contract_tests;

#[cfg(test)]
mod fab_tests;

#[cfg(test)]
mod snackbar_tests;

#[cfg(test)]
mod swipe_actions_tests;

#[cfg(test)]
mod style_sizing_tests;

#[cfg(test)]
mod speed_dial_tests;

#[cfg(test)]
mod about_dialog_tests;

#[cfg(test)]
mod bottom_navigation_tests;

#[cfg(test)]
mod page_header_tests;

#[cfg(test)]
mod tour_tests;

#[cfg(test)]
mod toolbar_tests;

#[cfg(test)]
mod fullscreen_tests;

#[cfg(test)]
mod anchor_tests;

#[cfg(test)]
mod collapsible_tests;

#[cfg(test)]
mod notification_tests;
