//! AcmeUI Native Playground — interactive widget testing application.
//!
//! Demonstrates all available widget types with theme toggling, keyboard
//! navigation, CJK/emoji rendering, and real-time state display.
#![forbid(unsafe_op_in_unsafe_fn)]

use std::time::Instant;

use acme_core::{
    AtlasUpload, ClickDetector, CursorHint, GlyphDraw, GlyphFormat, HitRegionSpec, HitTestLayer,
    NodeId, Painter, Point, PointerOutcome, Rect, RetainedTree, Scene, TextPrimitive,
};
use acme_layout::{Edges, LayoutEngine, LayoutKind, LayoutNode, LayoutStyle, Length, Overflow};
use acme_platform::{
    Application, CursorIcon, FrameContext, PlatformEvent, PlatformKey, WindowConfig, WindowId,
    WindowRequest,
};
use acme_text::{AtlasFormat, FontSystem, GlyphAtlas, TextConstraints, TextStyle};
use acme_theme::packs::theme_by_name;
use acme_theme::{Theme, ThemeColor, ThemeMode};
use acme_widgets::painter_bridge::{layout_rect_to_logical, paint_canvas_with_theme, theme_color};
use acme_widgets::retained_binding::{RetainedBinding, bind_retained};
use acme_widgets::view_bridge::widget_to_views;
use acme_widgets::{
    ButtonState, ButtonVariant, WidgetNode, button, canvas, card, column, label, row, separator,
};

#[cfg(test)]
mod retained_tests;

const INTERACTIVE_COUNT: usize = 9;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();
    acme_platform::run(Playground::new())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Message type
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
enum PlaygroundMessage {
    ToggleTheme,
    ClickPrimary,
    ClickSecondary,
    ClickGhost,
    ClickDanger,
    IncrementCounter,
    ResetCounter,
    ToggleDangerMode,
    ToggleHighContrast,
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

struct Playground {
    dark: bool,
    high_contrast: bool,
    focused: usize,
    cursor: (f32, f32),
    scroll: f32,
    max_scroll: f32,
    /// Interactive node IDs emitted in `frame()`: 8 buttons + 1 interactive canvas.
    interactive_ids: [NodeId; INTERACTIVE_COUNT],
    /// Spatial hit-testing layer built each frame (fixed header + scroll-clipped viewport).
    hit_layer: HitTestLayer,
    /// Stateful click/hover/drag recognizer bound to `hit_layer`.
    click_detector: ClickDetector,
    /// Last applied pointer cursor hint to deduplicate host requests.
    cursor_hint: CursorHint,
    /// Pending host window requests drained by `Application::take_window_requests`.
    pending_window_requests: Vec<WindowRequest>,
    /// Application start timestamp for multi-click timing.
    started_at: Instant,
    /// Number of times the counter button has been clicked.
    click_count: u32,
    /// Whether danger mode is active.
    danger_mode: bool,
    fonts: FontSystem,
    atlas: GlyphAtlas,
    layout: LayoutEngine,
    retained: RetainedTree,
    retained_error_logged: bool,
    #[cfg(test)]
    last_frame_layout: Option<([NodeId; 8], acme_layout::LayoutSnapshot)>,
}

impl Playground {
    fn new() -> Self {
        Self {
            dark: false,
            high_contrast: false,
            focused: 0,
            cursor: (0.0, 0.0),
            scroll: 0.0,
            max_scroll: 0.0,
            interactive_ids: [NodeId::new(1); INTERACTIVE_COUNT],
            hit_layer: HitTestLayer::new(),
            click_detector: ClickDetector::new(),
            cursor_hint: CursorHint::Default,
            pending_window_requests: Vec::new(),
            started_at: Instant::now(),
            click_count: 0,
            danger_mode: false,
            fonts: FontSystem::new(),
            atlas: GlyphAtlas::default(),
            layout: LayoutEngine::new(),
            retained: RetainedTree::new(),
            retained_error_logged: false,
            #[cfg(test)]
            last_frame_layout: None,
        }
    }

    fn current_theme(&self) -> Theme {
        let mode = if self.dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        if self.high_contrast {
            theme_by_name("high-contrast", mode).unwrap_or_else(|| {
                if self.dark {
                    Theme::dark()
                } else {
                    Theme::light()
                }
            })
        } else if self.dark {
            Theme::dark()
        } else {
            Theme::light()
        }
    }

    fn contrast_canvas(high_contrast: bool, danger_mode: bool) -> WidgetNode<PlaygroundMessage> {
        canvas()
            .key("contrast-canvas")
            .size(320.0, 56.0)
            .accessible_label(if high_contrast {
                "High Contrast Canvas Toggle (ON)"
            } else {
                "High Contrast Canvas Toggle (OFF)"
            })
            .cursor(CursorHint::Pointer)
            .on_click(PlaygroundMessage::ToggleHighContrast)
            .paint(move |painter, rect, theme| {
                let colors = theme.colors;
                let bg = theme_color(if high_contrast {
                    colors.surface_elevated
                } else {
                    colors.surface
                });
                let border = theme_color(if danger_mode {
                    colors.danger
                } else if high_contrast {
                    colors.ring
                } else {
                    colors.border
                });
                painter.rounded_rect_bordered(rect, theme.radii.md, bg, 1.5, border);

                let rx = rect.origin.x.get();
                let ry = rect.origin.y.get();
                let rw = rect.size.width.get();
                let rh = rect.size.height.get();
                if rw < 48.0 || rh < 24.0 {
                    return;
                }

                // Semantic palette swatches rendered directly through immediate Painter calls.
                let swatches = [
                    colors.primary,
                    colors.secondary,
                    colors.success,
                    colors.warning,
                    colors.danger,
                    colors.accent,
                ];
                let bar_w = 18.0;
                let bar_gap = 6.0;
                let base_x = rx + 14.0;
                let center_y = ry + rh * 0.5;
                for (i, swatch) in swatches.iter().enumerate() {
                    let bx = base_x + i as f32 * (bar_w + bar_gap);
                    if bx + bar_w > rx + rw - 16.0 {
                        break;
                    }
                    let bar_h = 16.0 + ((i + usize::from(high_contrast)) % 3) as f32 * 6.0;
                    painter.rounded_rect(
                        Rect::new(bx, center_y - bar_h * 0.5, bar_w, bar_h),
                        theme.radii.sm,
                        theme_color(*swatch),
                    );
                }
                let indicator = theme_color(if high_contrast {
                    colors.success
                } else {
                    colors.primary
                });
                painter.circle(Point::new(rx + rw - 24.0, center_y), 7.0, indicator);
            })
            .build()
    }

    // -----------------------------------------------------------------------
    // Widget tree
    // -----------------------------------------------------------------------
    // IDs are computed by extract_playground_ids() from the LayoutNode tree.
    // See PlaygroundNodeIds for the structural mapping.

    fn build_tree(&self) -> WidgetNode<PlaygroundMessage> {
        column()
            .key("playground")
            .gap(12.0)
            .padding(20.0)
            // Toolbar
            .child(
                row()
                    .key("toolbar")
                    .gap(12.0)
                    .child(
                        button(
                            "theme",
                            if self.dark {
                                "☀️ Light Mode"
                            } else {
                                "🌙 Dark Mode"
                            },
                        )
                        .on_click(PlaygroundMessage::ToggleTheme),
                    )
                    .child(label(format!(
                        "Theme: {}  Clicks: {}  Danger: {}",
                        if self.dark { "Dark" } else { "Light" },
                        self.click_count,
                        if self.danger_mode { "ON" } else { "OFF" },
                    ))),
            )
            .child(label("AcmeUI Native Playground"))
            .child(label("Interactive Widget Testing · 繁體中文 🎨"))
            .child(separator())
            // Scrollable content
            .child(
                column()
                    .key("content")
                    .gap(12.0)
                    // ── Section 1: Button Variants ──
                    .child(label("▸ Button Variants"))
                    .child(
                        row()
                            .key("btn-row")
                            .gap(8.0)
                            .child(
                                button("primary", "Primary")
                                    .primary()
                                    .on_click(PlaygroundMessage::ClickPrimary),
                            )
                            .child(
                                button("secondary", "Secondary")
                                    .on_click(PlaygroundMessage::ClickSecondary),
                            )
                            .child(
                                button("ghost", "Ghost")
                                    .variant(ButtonVariant::Ghost)
                                    .on_click(PlaygroundMessage::ClickGhost),
                            )
                            .child(
                                button("danger", "Danger")
                                    .variant(ButtonVariant::Danger)
                                    .on_click(PlaygroundMessage::ClickDanger),
                            ),
                    )
                    // ── Section 2: Interactive Demo ──
                    .child(label("▸ Interactive Demo"))
                    .child(
                        row()
                            .key("demo-row")
                            .gap(8.0)
                            .child(
                                button("clicker", format!("Clicks: {}", self.click_count))
                                    .primary()
                                    .on_click(PlaygroundMessage::IncrementCounter),
                            )
                            .child(
                                button("reset", "Reset Counter")
                                    .variant(ButtonVariant::Danger)
                                    .on_click(PlaygroundMessage::ResetCounter),
                            )
                            .child(
                                button(
                                    "toggle-danger",
                                    if self.danger_mode {
                                        "⚠ Danger ON"
                                    } else {
                                        "Safe Mode"
                                    },
                                )
                                .variant(if self.danger_mode {
                                    ButtonVariant::Danger
                                } else {
                                    ButtonVariant::Ghost
                                })
                                .on_click(PlaygroundMessage::ToggleDangerMode),
                            ),
                    )
                    // ── Section 3: Card Container ──
                    .child(label("▸ Card Container"))
                    .child(
                        card()
                            .key("demo-card")
                            .gap(8.0)
                            .padding(16.0)
                            .child(label(
                                "Card — a rounded surface with column layout and padding.",
                            ))
                            .child(label(
                                "Cards can contain any widgets: labels, buttons, rows, columns.",
                            ))
                            .child(label(
                                "Supports nesting, gap spacing, and semantic padding.",
                            )),
                    )
                    // ── Section 4: CJK & Emoji ──
                    .child(label("▸ CJK & Emoji Rendering"))
                    .child(label("繁體中文：系統介面與文字渲染測試 🀄"))
                    .child(label("日本語：システムインターフェース 🗾"))
                    .child(label("한국어: 사용자 인터페이스 테스트 🎯"))
                    .child(label("Emoji: 🚀🎨🙂🎉🔥💯⭐🧪✨🎭👋🌟😊🎮"))
                    .child(label("Mixed: Hello 你好 こんにちは 🌍 123 ABC"))
                    // ── Footer ──
                    .child(separator())
                    .child(label(format!(
                        "State: {}  Clicks: {}  Danger: {}",
                        if self.dark {
                            "🌙 Dark"
                        } else {
                            "☀️ Light"
                        },
                        self.click_count,
                        if self.danger_mode {
                            "⚠ ON"
                        } else {
                            "✓ OFF"
                        },
                    )))
                    // ── Section 5: Canvas & High Contrast ──
                    .child(label("▸ Canvas & High Contrast"))
                    .child(Self::contrast_canvas(self.high_contrast, self.danger_mode)),
            )
            .build()
    }

    /// Keep the Playground's legacy layout styles and measurement behavior,
    /// replacing only IDs when the complete description binds successfully.
    fn layout_root(&mut self, description: &WidgetNode<PlaygroundMessage>) -> LayoutNode {
        let mut root = description.to_layout(NodeId::new(1));
        let views = widget_to_views(std::slice::from_ref(description));
        if let Err(error) = self.retained.reconcile_roots(&views) {
            if !self.retained_error_logged {
                tracing::warn!(
                    "Playground retained reconcile failed, using sequential layout IDs: {error:?}"
                );
                self.retained_error_logged = true;
            }
            return root;
        }

        let binding = bind_retained(std::slice::from_ref(description), &self.retained);
        if binding.mismatches() == 0 && assign_retained_ids(description, &binding, &mut root) {
            self.retained_error_logged = false;
            return root;
        }
        if !self.retained_error_logged {
            tracing::warn!(
                "Playground retained binding/layout mismatch, using sequential layout IDs"
            );
            self.retained_error_logged = true;
        }
        // Assignment can discover a mismatch after visiting earlier siblings.
        // Rebuild so a failed frame never mixes retained and sequential IDs.
        description.to_layout(NodeId::new(1))
    }

    // -----------------------------------------------------------------------
    // Hit testing
    // -----------------------------------------------------------------------

    #[cfg(test)]
    fn hit(&self) -> Option<usize> {
        let hit = self
            .hit_layer
            .hit_test(Point::new(self.cursor.0, self.cursor.1))?;
        usize::try_from(hit.tag)
            .ok()
            .filter(|&index| index < INTERACTIVE_COUNT)
    }

    fn interactive_index_of(&self, id: NodeId) -> Option<usize> {
        self.interactive_ids
            .iter()
            .position(|&node_id| node_id == id)
    }

    fn elapsed_secs(&self) -> f64 {
        self.started_at.elapsed().as_secs_f64()
    }

    fn sync_cursor(&mut self) {
        let next = self.click_detector.cursor();
        if next != self.cursor_hint {
            self.cursor_hint = next;
            self.pending_window_requests
                .push(WindowRequest::SetCursor(map_cursor_hint(next)));
        }
    }

    fn activate(&mut self, index: usize) -> bool {
        match index {
            0 => self.dark = !self.dark,
            1 | 5 => self.click_count = self.click_count.wrapping_add(1),
            2 | 3 => { /* visual feedback only */ }
            4 | 7 => self.danger_mode = !self.danger_mode,
            6 => {
                self.click_count = 0;
                self.danger_mode = false;
            }
            8 => self.high_contrast = !self.high_contrast,
            _ => return false,
        }
        true
    }

    // -----------------------------------------------------------------------
    // Text helper
    // -----------------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    fn add_text(
        &mut self,
        painter: &mut Painter<'_>,
        text: &str,
        origin: [f32; 2],
        font_size: f32,
        color: ThemeColor,
        scale: f32,
    ) {
        let style = TextStyle {
            font_size,
            line_height: font_size * 1.35,
            ..TextStyle::default()
        };
        let layout = self
            .fonts
            .shape(text, &style, TextConstraints::default(), scale);
        let prepared = self.fonts.prepare(&layout, &mut self.atlas);

        let uploads = prepared
            .uploads
            .into_iter()
            .map(|region| AtlasUpload {
                page: region.page,
                origin: [region.x, region.y],
                size: [region.width, region.height],
                format: map_atlas_format(region.format),
                pixels: region.pixels,
            })
            .collect();
        let glyphs = prepared
            .glyphs
            .into_iter()
            .map(|glyph| GlyphDraw {
                x: glyph.x as f32,
                y: glyph.y as f32,
                width: glyph.width as f32,
                height: glyph.height as f32,
                atlas_x: glyph.atlas_x,
                atlas_y: glyph.atlas_y,
                atlas_page: glyph.atlas_page,
                format: map_atlas_format(glyph.format),
            })
            .collect();

        painter.text_primitive(TextPrimitive {
            origin: Point::new(origin[0], origin[1]),
            color: theme_color(color),
            glyphs,
            uploads,
        });
    }
}

/// Reassign IDs only for the one-to-one widgets used by `build_tree`.
/// Unsupported widgets or an incomplete layout shape use the sequential path.
fn assign_retained_ids(
    widget: &WidgetNode<PlaygroundMessage>,
    binding: &RetainedBinding<'_>,
    layout: &mut LayoutNode,
) -> bool {
    let children = match widget {
        WidgetNode::Row(container) | WidgetNode::Column(container) => &container.children[..],
        WidgetNode::Card(card) => &card.children[..],
        WidgetNode::Button(_)
        | WidgetNode::Label(_)
        | WidgetNode::Separator(_)
        | WidgetNode::Canvas(_) => &[],
        _ => return false,
    };
    let Some(id) = binding.id_for(widget) else {
        return false;
    };
    if children.len() != layout.children.len() {
        return false;
    }
    layout.id = id;
    children
        .iter()
        .zip(&mut layout.children)
        .all(|(child, layout)| assign_retained_ids(child, binding, layout))
}

/// Apply the existing frame style values to the actual `build_tree` structure.
fn apply_playground_styles(root: &mut LayoutNode, width: f32, height: f32) {
    // Root column: fill the window.
    root.style = LayoutStyle {
        kind: LayoutKind::Column,
        width: Length::px(width),
        height: Length::px(height),
        padding: Edges::all(20.0),
        gap: 12.0,
        ..LayoutStyle::default()
    };

    let body_width = (width - 40.0).max(200.0);
    let header_height = 44.0;

    // children[0]: Toolbar row
    root.children[0].style = LayoutStyle {
        kind: LayoutKind::Row,
        width: Length::px(body_width),
        height: Length::px(header_height),
        gap: 12.0,
        ..LayoutStyle::default()
    };
    // Toolbar's first child: Theme button
    root.children[0].children[0].style = LayoutStyle {
        width: Length::px(170.0),
        height: Length::px(header_height),
        ..LayoutStyle::default()
    };

    // children[1]: Title
    root.children[1].style.width = Length::px(body_width);
    // children[2]: Subtitle
    root.children[2].style.width = Length::px(body_width);
    // children[3]: Separator
    root.children[3].style = LayoutStyle {
        width: Length::px(body_width),
        height: Length::px(1.0),
        ..LayoutStyle::default()
    };

    // children[4]: Scrollable content column
    root.children[4].style = LayoutStyle {
        kind: LayoutKind::Column,
        width: Length::px(body_width),
        min_height: Length::px(0.0),
        flex_grow: 1.0,
        flex_shrink: 1.0,
        overflow: Overflow::Scroll,
        gap: 12.0,
        ..LayoutStyle::default()
    };

    // Style scroll-area children.
    let content = &mut root.children[4];

    // Section headers (children 0, 2, 4, 6)
    for &i in &[0usize, 2, 4, 6] {
        if i < content.children.len() {
            content.children[i].style.width = Length::px(body_width);
        }
    }

    // Button row 1 (child 1): btn-row.
    if content.children.len() > 1 {
        content.children[1].style = LayoutStyle {
            kind: LayoutKind::Row,
            width: Length::px(body_width),
            height: Length::px(header_height),
            gap: 8.0,
            ..LayoutStyle::default()
        };
        for btn in &mut content.children[1].children {
            btn.style = LayoutStyle {
                width: Length::px(120.0),
                height: Length::px(header_height),
                ..LayoutStyle::default()
            };
        }
    }

    // Button row 2 (child 3): demo-row.
    if content.children.len() > 3 {
        content.children[3].style = LayoutStyle {
            kind: LayoutKind::Row,
            width: Length::px(body_width),
            height: Length::px(header_height),
            gap: 8.0,
            ..LayoutStyle::default()
        };
        for btn in &mut content.children[3].children {
            btn.style = LayoutStyle {
                width: Length::px(140.0),
                height: Length::px(header_height),
                ..LayoutStyle::default()
            };
        }
    }

    // Card (child 5)
    if content.children.len() > 5 {
        content.children[5].style = LayoutStyle {
            kind: LayoutKind::Column,
            width: Length::px(body_width),
            padding: Edges::all(16.0),
            gap: 8.0,
            ..LayoutStyle::default()
        };
    }

    // CJK/Emoji labels (children 7-11)
    for &i in &[7usize, 8, 9, 10, 11] {
        if i < content.children.len() {
            content.children[i].style.width = Length::px(body_width);
        }
    }

    // Separator (child 12)
    if content.children.len() > 12 {
        content.children[12].style = LayoutStyle {
            width: Length::px(body_width),
            height: Length::px(1.0),
            ..LayoutStyle::default()
        };
    }

    // State footer (child 13)
    if content.children.len() > 13 {
        content.children[13].style.width = Length::px(body_width);
    }

    // Canvas section header (child 14) & interactive Canvas (child 15)
    if content.children.len() > 14 {
        content.children[14].style.width = Length::px(body_width);
    }
    if content.children.len() > 15 {
        content.children[15].style = LayoutStyle {
            width: Length::px(body_width),
            height: Length::px(56.0),
            ..LayoutStyle::default()
        };
    }
}

// ---------------------------------------------------------------------------
// Pre-computed node IDs for the Playground's widget tree
// ---------------------------------------------------------------------------

/// IDs read from the styled frame layout, retained or sequential.
struct PlaygroundNodeIds {
    /// Toolbar row
    toolbar: NodeId,
    /// Status label inside toolbar
    status: NodeId,
    /// Title label
    title: NodeId,
    /// Subtitle label
    subtitle: NodeId,
    /// Toolbar separator
    sep1: NodeId,
    /// Content column (scrollable)
    content: NodeId,
    /// Position of buttons in the button_ids array
    button_ids: [NodeId; 8],
    /// Card node
    card: NodeId,
    /// Card children
    card_children: [NodeId; 3],
    /// Separator inside content
    sep2: NodeId,
    /// Footer label
    footer: NodeId,
    /// Section header node IDs: Button Variants, Interactive Demo, Card, CJK, Canvas
    section_headers: [NodeId; 5],
    /// CJK & Emoji label IDs (5 labels)
    cjk_labels: [NodeId; 5],
    /// Interactive high-contrast Canvas node ID
    canvas: NodeId,
}

/// Extract structural IDs from the layout root.
///
/// Root layout structure from `build_tree()`:
///   Column root (sequential IDs shown; retained IDs may differ)
///     ├── Row("toolbar") (2)
///     │   ├── Button("theme") (3)   ← button index 0
///     │   └── Label(status) (4)
///     ├── Label(title) (5)
///     ├── Label(subtitle) (6)
///     ├── Separator (7)
///     └── Column("content") (8)  [scrollable]
///         ├── Label(section-1) (9)
///         ├── Row("btn-row") (10)
///         │   ├── Button("primary") (11)   ← button index 1
///         │   ├── Button("secondary") (12) ← button index 2
///         │   ├── Button("ghost") (13)     ← button index 3
///         │   └── Button("danger") (14)    ← button index 4
///         ├── Label(section-2) (15)
///         ├── Row("demo-row") (16)
///         │   ├── Button("clicker") (17)   ← button index 5
///         │   ├── Button("reset") (18)     ← button index 6
///         │   └── Button("toggle-danger") (19) ← button index 7
///         ├── Label(section-3) (20)
///         ├── Card("demo-card") (21)
///         │   ├── Label(card-0) (22)
///         │   ├── Label(card-1) (23)
///         │   └── Label(card-2) (24)
///         ├── Label(section-4) (25)
///         ├── Label(cjk-1) (26)
///         ├── Label(cjk-2) (27)
///         ├── Label(korean) (28)
///         ├── Label(emoji) (29)
///         ├── Label(mixed) (30)
///         ├── Separator (31)
///         ├── Label(footer) (32)
///         ├── Label(section-5) (33)
///         └── Canvas("contrast-canvas") (34)  ← interactive index 8
fn extract_playground_ids(root: &LayoutNode) -> PlaygroundNodeIds {
    let toolbar = &root.children[0];
    let content_col = &root.children[4];
    PlaygroundNodeIds {
        toolbar: toolbar.id,
        status: toolbar.children[1].id,
        title: root.children[1].id,
        subtitle: root.children[2].id,
        sep1: root.children[3].id,
        content: content_col.id,
        button_ids: [
            toolbar.children[0].id,                 // 3
            content_col.children[1].children[0].id, // 11
            content_col.children[1].children[1].id, // 12
            content_col.children[1].children[2].id, // 13
            content_col.children[1].children[3].id, // 14
            content_col.children[3].children[0].id, // 17
            content_col.children[3].children[1].id, // 18
            content_col.children[3].children[2].id, // 19
        ],
        card: content_col.children[5].id, // 21
        card_children: [
            content_col.children[5].children[0].id, // 22
            content_col.children[5].children[1].id, // 23
            content_col.children[5].children[2].id, // 24
        ],
        sep2: content_col.children[12].id,   // 31
        footer: content_col.children[13].id, // 32
        section_headers: [
            content_col.children[0].id,  // 9
            content_col.children[2].id,  // 15
            content_col.children[4].id,  // 20
            content_col.children[6].id,  // 25
            content_col.children[14].id, // 33
        ],
        cjk_labels: [
            content_col.children[7].id,  // 26
            content_col.children[8].id,  // 27
            content_col.children[9].id,  // 28
            content_col.children[10].id, // 29
            content_col.children[11].id, // 30
        ],
        canvas: content_col.children[15].id, // 34
    }
}

impl Playground {
    #[allow(clippy::too_many_arguments)]
    fn paint_button(
        &mut self,
        painter: &mut Painter<'_>,
        theme: &Theme,
        index: usize,
        node_id: NodeId,
        rect: Rect<acme_core::Logical>,
        scale_factor: f32,
        hovered_id: Option<NodeId>,
        pressed_id: Option<NodeId>,
    ) {
        let button_variants = [
            ButtonVariant::Primary,   // 0: theme toggle
            ButtonVariant::Primary,   // 1: primary click
            ButtonVariant::Secondary, // 2: secondary click
            ButtonVariant::Ghost,     // 3: ghost click
            ButtonVariant::Danger,    // 4: danger click
            ButtonVariant::Primary,   // 5: increment counter
            ButtonVariant::Danger,    // 6: reset counter
            ButtonVariant::Secondary, // 7: toggle danger mode
        ];
        let button_labels = [
            if self.dark {
                "☀️ Light Mode"
            } else {
                "🌙 Dark Mode"
            },
            "Primary",
            "Secondary",
            "Ghost",
            "Danger",
            "",
            "Reset Counter",
            "",
        ];

        let effective_variant = if index == 7 {
            if self.danger_mode {
                ButtonVariant::Danger
            } else {
                ButtonVariant::Ghost
            }
        } else {
            button_variants[index]
        };

        let state = ButtonState {
            hovered: hovered_id == Some(node_id),
            pressed: pressed_id == Some(node_id),
            focused: self.focused == index,
        };
        let disabled = index == 5 && self.click_count >= 999;

        let key = match index {
            0 => "theme",
            1 => "primary",
            2 => "secondary",
            3 => "ghost",
            4 => "danger",
            5 => "clicker",
            6 => "reset",
            7 => "toggle-danger",
            _ => "",
        };

        let label_text: String = match index {
            5 => format!("Clicks: {}", self.click_count),
            7 => {
                if self.danger_mode {
                    "⚠ Danger ON".to_string()
                } else {
                    "Safe Mode".to_string()
                }
            }
            _ => button_labels[index].to_string(),
        };

        let builder = button::<PlaygroundMessage>(key, label_text.as_str())
            .variant(effective_variant)
            .disabled(disabled);
        let resolved = builder.resolve_style(theme, state);

        painter.rounded_rect_bordered(
            rect,
            theme.radii.md,
            theme_color(resolved.background),
            if state.focused { 3.0 } else { 1.0 },
            theme_color(if state.focused {
                resolved.focus
            } else {
                resolved.border
            }),
        );

        self.add_text(
            painter,
            &label_text,
            [rect.origin.x.get() + 14.0, rect.origin.y.get() + 13.0],
            theme.typography.label,
            resolved.foreground,
            scale_factor,
        );
    }
}

// ---------------------------------------------------------------------------
// Application trait implementation
// ---------------------------------------------------------------------------

impl Application for Playground {
    fn window_config(&self) -> WindowConfig {
        WindowConfig {
            title: "AcmeUI Native Playground".into(),
            width: 1080.0,
            height: 720.0,
            ..WindowConfig::default()
        }
    }

    fn take_window_requests(&mut self, _window: WindowId) -> Vec<WindowRequest> {
        std::mem::take(&mut self.pending_window_requests)
    }

    fn event(&mut self, event: PlatformEvent) -> bool {
        match event {
            PlatformEvent::PointerMoved { x, y, .. } => {
                self.cursor = (x, y);
                let point = Point::new(x, y);
                let prev_hovered = self.click_detector.hovered();
                let prev_dragging = self.click_detector.is_dragging();
                let outcomes = self.click_detector.on_pointer_move(&self.hit_layer, point);
                self.sync_cursor();
                !outcomes.is_empty()
                    || self.click_detector.hovered() != prev_hovered
                    || self.click_detector.is_dragging() != prev_dragging
            }
            PlatformEvent::PointerLeft { .. } | PlatformEvent::CursorLeft { .. } => {
                let outcomes = self.click_detector.on_pointer_leave();
                self.sync_cursor();
                !outcomes.is_empty()
            }
            PlatformEvent::PointerButton {
                pressed,
                x,
                y,
                button,
                ..
            } => {
                self.cursor = (x, y);
                let point = Point::new(x, y);
                let outcomes = self.click_detector.on_pointer_button(
                    &self.hit_layer,
                    button,
                    pressed,
                    point,
                    self.elapsed_secs(),
                );
                let mut activated = false;
                for outcome in &outcomes {
                    if let PointerOutcome::Clicked { id, button: 0, .. } = *outcome
                        && let Some(index) = self.interactive_index_of(id)
                    {
                        self.focused = index;
                        activated |= self.activate(index);
                    }
                }
                if !pressed {
                    let _ = self.click_detector.on_pointer_move(&self.hit_layer, point);
                }
                self.sync_cursor();
                activated || !outcomes.is_empty()
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
                self.focused = if shift {
                    (self.focused + INTERACTIVE_COUNT - 1) % INTERACTIVE_COUNT
                } else {
                    (self.focused + 1) % INTERACTIVE_COUNT
                };
                true
            }
            PlatformEvent::Key {
                key: PlatformKey::Enter | PlatformKey::Space,
                pressed: true,
                ..
            } => self.activate(self.focused),
            PlatformEvent::Resized { .. } => true,
            _ => false,
        }
    }

    fn on_gpu_recovered(&mut self, _window: WindowId) {
        // The renderer rebuilt empty GPU atlases after device loss; drop the CPU
        // atlas so the next frame re-uploads every glyph instead of referencing
        // blank texture regions.
        self.atlas.clear();
    }

    fn frame(&mut self, context: FrameContext) -> Scene {
        let width = context.logical_width;
        let height = context.logical_height;
        let theme = self.current_theme();
        let colors = theme.colors;
        self.atlas.begin_frame();

        // ── 1. Build and style the layout tree ──
        let description = self.build_tree();
        let mut root = self.layout_root(&description);

        apply_playground_styles(&mut root, width, height);

        // ── 2. Compute layout ──
        let snapshot = self
            .layout
            .compute(&root, (width, height))
            .expect("finite Playground viewport");
        let ids = extract_playground_ids(&root);
        #[cfg(test)]
        {
            self.last_frame_layout = Some((ids.button_ids, snapshot.clone()));
        }

        for (index, btn_id) in ids.button_ids.into_iter().enumerate() {
            self.interactive_ids[index] = btn_id;
        }
        self.interactive_ids[8] = ids.canvas;

        // ── 3. Scroll metrics ──
        if let Some(metrics) = snapshot.scroll_metrics(ids.content) {
            self.max_scroll = (metrics.content_height - metrics.viewport_height).max(0.0);
        }
        self.scroll = self.scroll.clamp(0.0, self.max_scroll);

        // ── 4. Rebuild HitTestLayer (fixed header vs scroll-clipped viewport) ──
        self.hit_layer.clear();
        if let Some(header_btn_rect) = snapshot.get(ids.button_ids[0]) {
            self.hit_layer.push_region(
                HitRegionSpec::rect(ids.button_ids[0], layout_rect_to_logical(*header_btn_rect))
                    .enabled(true)
                    .cursor(CursorHint::Pointer)
                    .tag(0),
            );
        }
        if let Some(vp) = snapshot.get(ids.content) {
            let vp_rect = layout_rect_to_logical(*vp);
            self.hit_layer
                .with_scroll_viewport(vp_rect, 0.0, self.scroll, |layer| {
                    for (offset, btn_id) in ids.button_ids[1..].iter().enumerate() {
                        let index = offset + 1;
                        if let Some(rect) = snapshot.get(*btn_id) {
                            let enabled = !(index == 5 && self.click_count >= 999);
                            let cursor = if enabled {
                                CursorHint::Pointer
                            } else {
                                CursorHint::NotAllowed
                            };
                            layer.push_region(
                                HitRegionSpec::rect(*btn_id, layout_rect_to_logical(*rect))
                                    .enabled(enabled)
                                    .cursor(cursor)
                                    .tag(index as u64),
                            );
                        }
                    }
                    if let Some(canvas_rect) = snapshot.get(ids.canvas) {
                        layer.push_region(
                            HitRegionSpec::rect(ids.canvas, layout_rect_to_logical(*canvas_rect))
                                .enabled(true)
                                .cursor(CursorHint::Pointer)
                                .tag(8),
                        );
                    }
                });
        }

        let hovered_id = self.click_detector.hovered();
        let pressed_id = self.click_detector.pressed();

        // ── 5. Paint immediate scene via acme_core::Painter ──
        let mut painter = Painter::with_clear(theme_color(colors.background));

        // ── Fixed Header / Toolbar ──
        if let Some(header_rect) = snapshot.get(ids.toolbar) {
            painter.rounded_rect_bordered(
                layout_rect_to_logical(*header_rect),
                theme.radii.lg,
                theme_color(colors.surface),
                1.0,
                theme_color(colors.border),
            );
        }
        if let Some(btn_rect) = snapshot.get(ids.button_ids[0]) {
            self.paint_button(
                &mut painter,
                &theme,
                0,
                ids.button_ids[0],
                layout_rect_to_logical(*btn_rect),
                context.scale_factor,
                hovered_id,
                pressed_id,
            );
        }

        if let Some(title_rect) = snapshot.get(ids.title) {
            self.add_text(
                &mut painter,
                "AcmeUI Native Playground",
                [title_rect.x, title_rect.y + 2.0],
                22.0,
                colors.foreground,
                context.scale_factor,
            );
        }

        if let Some(sub_rect) = snapshot.get(ids.subtitle) {
            self.add_text(
                &mut painter,
                "Interactive Widget Testing · 繁體中文 🎨",
                [sub_rect.x, sub_rect.y + 2.0],
                theme.typography.body,
                colors.muted_foreground,
                context.scale_factor,
            );
        }

        if let Some(status_rect) = snapshot.get(ids.status) {
            self.add_text(
                &mut painter,
                &format!(
                    "Theme: {}{}  Clicks: {}  Danger: {}",
                    if self.dark { "Dark" } else { "Light" },
                    if self.high_contrast { " (HC)" } else { "" },
                    self.click_count,
                    if self.danger_mode { "ON" } else { "OFF" },
                ),
                [status_rect.x, status_rect.y + 13.0],
                theme.typography.label,
                colors.muted_foreground,
                context.scale_factor,
            );
        }

        if let Some(sep_rect) = snapshot.get(ids.sep1) {
            painter.fill_rect(
                Rect::new(sep_rect.x, sep_rect.y, sep_rect.width, 1.0),
                theme_color(colors.border),
            );
        }

        // ── Scroll viewport (clipped + offset) ──
        if let Some(vp) = snapshot.get(ids.content) {
            let vp_rect = layout_rect_to_logical(*vp);
            let scroll = self.scroll;

            painter.with_clip(vp_rect, |painter| {
                painter.with_offset(0.0, -scroll, |painter| {
                    // Scrollable interactive buttons (1..=7)
                    for (offset, btn_id) in ids.button_ids[1..].iter().enumerate() {
                        let index = offset + 1;
                        if let Some(btn_rect) = snapshot.get(*btn_id) {
                            self.paint_button(
                                painter,
                                &theme,
                                index,
                                *btn_id,
                                layout_rect_to_logical(*btn_rect),
                                context.scale_factor,
                                hovered_id,
                                pressed_id,
                            );
                        }
                    }

                    // Section headers
                    let section_texts = [
                        (ids.section_headers[0], "▸ Button Variants"),
                        (ids.section_headers[1], "▸ Interactive Demo"),
                        (ids.section_headers[2], "▸ Card Container"),
                        (ids.section_headers[3], "▸ CJK & Emoji Rendering"),
                        (ids.section_headers[4], "▸ Canvas & High Contrast"),
                    ];
                    for (sid, text) in &section_texts {
                        if let Some(r) = snapshot.get(*sid) {
                            self.add_text(
                                painter,
                                text,
                                [r.x + 4.0, r.y + 2.0],
                                theme.typography.body,
                                colors.foreground,
                                context.scale_factor,
                            );
                        }
                    }

                    // Card background and labels
                    if let Some(card_rect) = snapshot.get(ids.card) {
                        painter.rounded_rect_bordered(
                            layout_rect_to_logical(*card_rect),
                            theme.radii.lg,
                            theme_color(colors.surface),
                            1.0,
                            theme_color(colors.border),
                        );
                        let card_texts = [
                            (
                                ids.card_children[0],
                                "Card — a rounded surface with column layout and padding.",
                            ),
                            (
                                ids.card_children[1],
                                "Cards can contain any widgets: labels, buttons, rows, columns.",
                            ),
                            (
                                ids.card_children[2],
                                "Supports nesting, gap spacing, and semantic padding.",
                            ),
                        ];
                        for (cid, text) in &card_texts {
                            if let Some(cr) = snapshot.get(*cid) {
                                self.add_text(
                                    painter,
                                    text,
                                    [cr.x + 4.0, cr.y + 2.0],
                                    theme.typography.body,
                                    colors.foreground,
                                    context.scale_factor,
                                );
                            }
                        }
                    }

                    // CJK & Emoji labels
                    let cjk_emoji_texts = [
                        (ids.cjk_labels[0], "繁體中文：系統介面與文字渲染測試 🀄"),
                        (ids.cjk_labels[1], "日本語：システムインターフェース 🗾"),
                        (ids.cjk_labels[2], "한국어: 사용자 인터페이스 테스트 🎯"),
                        (ids.cjk_labels[3], "Emoji: 🚀🎨🙂🎉🔥💯⭐🧪✨🎭👋🌟😊🎮"),
                        (ids.cjk_labels[4], "Mixed: Hello 你好 こんにちは 🌍 123 ABC"),
                    ];
                    for (cid, text) in &cjk_emoji_texts {
                        if let Some(r) = snapshot.get(*cid) {
                            self.add_text(
                                painter,
                                text,
                                [r.x + 4.0, r.y + 2.0],
                                theme.typography.body,
                                colors.foreground,
                                context.scale_factor,
                            );
                        }
                    }

                    // Separator in content area
                    if let Some(sep2) = snapshot.get(ids.sep2) {
                        painter.fill_rect(
                            Rect::new(sep2.x, sep2.y, sep2.width, 1.0),
                            theme_color(colors.border),
                        );
                    }

                    // State footer
                    if let Some(footer) = snapshot.get(ids.footer) {
                        self.add_text(
                            painter,
                            &format!(
                                "State: {}  HC: {}  Clicks: {}  Danger: {}",
                                if self.dark {
                                    "🌙 Dark"
                                } else {
                                    "☀️ Light"
                                },
                                if self.high_contrast { "ON" } else { "OFF" },
                                self.click_count,
                                if self.danger_mode {
                                    "⚠ ON"
                                } else {
                                    "✓ OFF"
                                },
                            ),
                            [footer.x + 4.0, footer.y + 2.0],
                            theme.typography.label,
                            colors.muted_foreground,
                            context.scale_factor,
                        );
                    }

                    // Interactive high-contrast Canvas widget
                    if let Some(canvas_layout_rect) = snapshot.get(ids.canvas) {
                        let canvas_rect = layout_rect_to_logical(*canvas_layout_rect);
                        if let WidgetNode::Column(root_col) = &description
                            && let Some(WidgetNode::Column(content_col)) = root_col.children.get(4)
                            && let Some(WidgetNode::Canvas(canvas_widget)) =
                                content_col.children.get(15)
                        {
                            paint_canvas_with_theme(canvas_widget, canvas_rect, &theme, painter);
                        }
                        if self.focused == 8 || hovered_id == Some(ids.canvas) {
                            let ring_color = if self.focused == 8 {
                                theme_color(colors.ring)
                            } else {
                                theme_color(colors.primary)
                            };
                            painter.rect(
                                canvas_rect,
                                acme_core::RectStyle::default()
                                    .radius(theme.radii.md)
                                    .border(if self.focused == 8 { 3.0 } else { 2.0 }, ring_color),
                            );
                        }
                        let canvas_label = if self.high_contrast {
                            "Canvas: High Contrast ON (click or Space)"
                        } else {
                            "Canvas: Standard Theme (click or Space)"
                        };
                        painter.with_clip(canvas_rect, |painter| {
                            self.add_text(
                                painter,
                                canvas_label,
                                [
                                    canvas_rect.origin.x.get() + 168.0,
                                    canvas_rect.origin.y.get() + 19.0,
                                ],
                                theme.typography.label,
                                colors.foreground,
                                context.scale_factor,
                            );
                        });
                    }
                });
            });
        }

        painter.finish()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn map_cursor_hint(hint: CursorHint) -> CursorIcon {
    match hint {
        CursorHint::Default => CursorIcon::Default,
        CursorHint::Pointer => CursorIcon::Pointer,
        CursorHint::Text => CursorIcon::Text,
        CursorHint::Grab => CursorIcon::Grab,
        CursorHint::Grabbing => CursorIcon::Grabbing,
        CursorHint::ResizeEW => CursorIcon::ResizeHorizontal,
        CursorHint::ResizeNS => CursorIcon::ResizeVertical,
        CursorHint::NotAllowed => CursorIcon::NotAllowed,
    }
}

fn map_atlas_format(format: AtlasFormat) -> GlyphFormat {
    match format {
        AtlasFormat::Alpha8 => GlyphFormat::Alpha8,
        AtlasFormat::Rgba8 => GlyphFormat::Rgba8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::DrawCommand;

    fn ctx(width: f32, height: f32) -> FrameContext {
        FrameContext {
            window: WindowId(0),
            logical_width: width,
            logical_height: height,
            scale_factor: 1.0,
        }
    }

    fn move_pointer(app: &mut Playground, x: f32, y: f32) -> bool {
        app.event(PlatformEvent::PointerMoved {
            window: WindowId(0),
            x,
            y,
        })
    }

    fn press_pointer(app: &mut Playground, x: f32, y: f32, pressed: bool) -> bool {
        app.event(PlatformEvent::PointerButton {
            window: WindowId(0),
            pressed,
            x,
            y,
            button: 0,
            pointer: 1,
            shift: false,
            ctrl: false,
            alt: false,
        })
    }

    fn layout_rect(app: &Playground, id: NodeId) -> acme_layout::LayoutRect {
        let (_, snapshot) = app.last_frame_layout.as_ref().expect("frame layout stored");
        *snapshot.get(id).expect("node rect exists")
    }

    #[test]
    fn painter_emits_valid_scene_with_balanced_clips_and_high_contrast_canvas() {
        let mut app = Playground::new();
        let scene = app.frame(ctx(1080.0, 720.0));
        assert!(scene.validate().is_ok(), "emitted scene must validate");

        let commands = scene.commands();
        let push_clips = commands
            .iter()
            .filter(|cmd| matches!(cmd, DrawCommand::PushClip(_)))
            .count();
        let pop_clips = commands
            .iter()
            .filter(|cmd| matches!(cmd, DrawCommand::PopClip))
            .count();
        assert_eq!(push_clips, pop_clips);
        assert!(push_clips >= 2, "viewport and canvas clips expected");
        assert!(
            commands
                .iter()
                .any(|cmd| matches!(cmd, DrawCommand::Quad(_))),
            "expected Quad commands from Painter"
        );
        assert!(
            commands
                .iter()
                .any(|cmd| matches!(cmd, DrawCommand::Text(_))),
            "expected Text commands from Painter"
        );

        let default_clear = scene.clear_color();
        app.high_contrast = true;
        let hc_scene = app.frame(ctx(1080.0, 720.0));
        assert!(hc_scene.validate().is_ok());
        assert_ne!(hc_scene.clear_color(), default_clear);
    }

    #[test]
    fn disabled_clicker_paints_the_disabled_foreground() {
        let mut app = Playground::new();
        app.click_count = 999;
        let theme = app.current_theme();
        let scene = app.frame(ctx(1080.0, 720.0));
        let clicker = layout_rect(&app, app.interactive_ids[5]);
        let origin = Point::new(clicker.x + 14.0, clicker.y - app.scroll + 13.0);

        assert!(scene.commands().iter().any(|command| matches!(
            command,
            DrawCommand::Text(text) if text.origin == origin
                && text.color == theme_color(theme.colors.disabled_foreground)
        )));
    }

    #[test]
    fn scroll_clipped_hit_layer_and_click_detector_handle_header_viewport_and_cursor() {
        let mut app = Playground::new();
        // Compact height so `content` scrolls and clips `btn-row` when scrolled down.
        let _ = app.frame(ctx(800.0, 380.0));

        // 1. Fixed header theme button remains clickable even after scrolling down.
        app.scroll = 140.0;
        let _ = app.frame(ctx(800.0, 380.0));

        let header_btn = layout_rect(&app, app.interactive_ids[0]);
        let hx = header_btn.x + header_btn.width * 0.5;
        let hy = header_btn.y + header_btn.height * 0.5;

        assert!(move_pointer(&mut app, hx, hy));
        assert_eq!(app.click_detector.cursor(), CursorHint::Pointer);
        let requests = app.take_window_requests(WindowId(0));
        assert!(requests.contains(&WindowRequest::SetCursor(CursorIcon::Pointer)));

        assert!(!app.dark);
        assert!(press_pointer(&mut app, hx, hy, true));
        assert!(press_pointer(&mut app, hx, hy, false));
        assert!(app.dark, "clicking fixed header button toggles theme");

        // 2. Scrolled-off button in `btn-row` (`primary`, index 1) must be clipped out.
        let primary_btn = layout_rect(&app, app.interactive_ids[1]);
        let px_unscrolled = primary_btn.x + primary_btn.width * 0.5;
        let py_scrolled = (primary_btn.y - app.scroll) + primary_btn.height * 0.5;
        app.cursor = (px_unscrolled, py_scrolled);
        assert_eq!(
            app.hit(),
            None,
            "scrolled-off button must be clipped (primary={primary_btn:?}, scroll={}, max_scroll={})",
            app.scroll,
            app.max_scroll
        );

        assert!(!press_pointer(&mut app, px_unscrolled, py_scrolled, true));
        assert!(!press_pointer(&mut app, px_unscrolled, py_scrolled, false));
        assert_eq!(app.click_count, 0, "clipped button click must not activate");

        // 3. When scroll = 0, clicking `clicker` (index 5) increments counter and canvas toggles HC.
        app.scroll = 0.0;
        let _ = app.frame(ctx(1080.0, 720.0));
        let clicker_rect = layout_rect(&app, app.interactive_ids[5]);
        let cx = clicker_rect.x + clicker_rect.width * 0.5;
        let cy = clicker_rect.y + clicker_rect.height * 0.5;
        assert!(move_pointer(&mut app, cx, cy));
        assert!(press_pointer(&mut app, cx, cy, true));
        assert!(press_pointer(&mut app, cx, cy, false));
        assert_eq!(app.click_count, 1);

        // 4. Scroll down to bottom so the interactive Canvas enters the viewport and toggles HC.
        app.scroll = app.max_scroll;
        let _ = app.frame(ctx(1080.0, 720.0));
        let canvas_rect = layout_rect(&app, app.interactive_ids[8]);
        let cvx = canvas_rect.x + canvas_rect.width * 0.5;
        let cvy = (canvas_rect.y - app.scroll) + canvas_rect.height * 0.5;
        assert!(!app.high_contrast);
        assert!(move_pointer(&mut app, cvx, cvy));
        assert!(press_pointer(&mut app, cvx, cvy, true));
        assert!(press_pointer(&mut app, cvx, cvy, false));
        assert!(app.high_contrast, "clicking canvas toggles high contrast");
    }

    #[test]
    fn drag_out_cancels_click_and_keyboard_focus_cycles_all_controls() {
        let mut app = Playground::new();
        let _ = app.frame(ctx(1080.0, 720.0));

        let clicker = layout_rect(&app, app.interactive_ids[5]);
        let start_x = clicker.x + clicker.width * 0.5;
        let start_y = clicker.y + clicker.height * 0.5;

        // Press inside button, drag outside, release -> no click activation.
        assert!(press_pointer(&mut app, start_x, start_y, true));
        assert_eq!(app.click_detector.pressed(), Some(app.interactive_ids[5]));
        assert!(move_pointer(&mut app, start_x + 300.0, start_y));
        assert!(app.click_detector.is_dragging());
        assert!(press_pointer(&mut app, start_x + 300.0, start_y, false));
        assert_eq!(app.click_detector.pressed(), None);
        assert_eq!(app.click_count, 0, "drag-out release must cancel click");

        // Tab through interactive buttons to the canvas (index 8) and activate with Space.
        assert_eq!(app.focused, 0);
        for _ in 0..8 {
            assert!(app.event(PlatformEvent::Key {
                window: WindowId(0),
                key: PlatformKey::Tab,
                pressed: true,
                shift: false,
                ctrl: false,
                alt: false,
                meta: false,
                repeat: false,
                text: None,
                character: None,
            }));
        }
        assert_eq!(app.focused, 8);
        assert!(app.event(PlatformEvent::Key {
            window: WindowId(0),
            key: PlatformKey::Space,
            pressed: true,
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
            repeat: false,
            text: None,
            character: None,
        }));
        assert!(app.high_contrast);

        // Shift+Tab wraps backwards cleanly.
        assert!(app.event(PlatformEvent::Key {
            window: WindowId(0),
            key: PlatformKey::Tab,
            pressed: true,
            shift: true,
            ctrl: false,
            alt: false,
            meta: false,
            repeat: false,
            text: None,
            character: None,
        }));
        assert_eq!(app.focused, 7);
    }
}
