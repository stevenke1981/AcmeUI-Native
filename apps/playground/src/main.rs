//! AcmeUI Native Playground — interactive widget testing application.
//!
//! Demonstrates all available widget types with theme toggling, keyboard
//! navigation, CJK/emoji rendering, and real-time state display.
#![forbid(unsafe_op_in_unsafe_fn)]

use acme_core::{NodeId, RetainedTree};
use acme_layout::{Edges, LayoutEngine, LayoutKind, LayoutNode, LayoutStyle, Length, Overflow};
use acme_platform::{
    Application, FrameContext, PlatformEvent, PlatformKey, WindowConfig, WindowId,
};
use acme_render_wgpu::{ClippedQuad, Frame, Quad, TextRun, scene_from_frame};
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextStyle};
use acme_theme::{Theme, ThemeColor};
use acme_widgets::retained_binding::{RetainedBinding, bind_retained};
use acme_widgets::view_bridge::widget_to_views;
use acme_widgets::{
    ButtonState, ButtonVariant, WidgetNode, button, card, column, label, row, separator,
};

#[cfg(test)]
mod retained_tests;

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
}

// ---------------------------------------------------------------------------
// Application state
// ---------------------------------------------------------------------------

struct Playground {
    dark: bool,
    hovered: Option<usize>,
    pressed: Option<usize>,
    focused: usize,
    cursor: (f32, f32),
    scroll: f32,
    max_scroll: f32,
    /// Position rectangles for the 8 interactive buttons.
    buttons: [[f32; 4]; 8],
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
            hovered: None,
            pressed: None,
            focused: 0,
            cursor: (0.0, 0.0),
            scroll: 0.0,
            max_scroll: 0.0,
            buttons: [[0.0; 4]; 8],
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
                    ))),
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

    fn hit(&self) -> Option<usize> {
        self.buttons.iter().position(|rect| {
            self.cursor.0 >= rect[0]
                && self.cursor.0 <= rect[0] + rect[2]
                && self.cursor.1 >= rect[1]
                && self.cursor.1 <= rect[1] + rect[3]
        })
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
        frame: &mut Frame,
        text: &str,
        origin: [f32; 2],
        font_size: f32,
        color: ThemeColor,
        scale: f32,
        clip: Option<[f32; 4]>,
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
        frame.text.push(TextRun {
            prepared,
            origin,
            color: rgba(color),
            clip,
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
        WidgetNode::Button(_) | WidgetNode::Label(_) | WidgetNode::Separator(_) => &[],
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
        flex_grow: 1.0,
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
    /// Section header node IDs: Button Variants, Interactive Demo, Card, CJK
    section_headers: [NodeId; 4],
    /// CJK & Emoji label IDs (5 labels)
    cjk_labels: [NodeId; 5],
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
///         └── Label(footer) (32)
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
            content_col.children[0].id, // 9
            content_col.children[2].id, // 15
            content_col.children[4].id, // 20
            content_col.children[6].id, // 25
        ],
        cjk_labels: [
            content_col.children[7].id,  // 26
            content_col.children[8].id,  // 27
            content_col.children[9].id,  // 28
            content_col.children[10].id, // 29
            content_col.children[11].id, // 30
        ],
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
            min_width: None,
            min_height: None,
            position: None,
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
                        .filter(|value| Some(*value) == self.hit());
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
                self.focused = if shift {
                    (self.focused + 7) % 8
                } else {
                    (self.focused + 1) % 8
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

    fn frame(&mut self, context: FrameContext) -> acme_core::Scene {
        let width = context.logical_width;
        let height = context.logical_height;
        let theme = if self.dark {
            Theme::dark()
        } else {
            Theme::light()
        };
        let colors = theme.colors;

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

        // ── 3. Extract button hit-test rects ──
        for (index, btn_id) in ids.button_ids.into_iter().enumerate() {
            if let Some(rect) = snapshot.get(btn_id) {
                self.buttons[index] = [rect.x, rect.y, rect.width, rect.height];
            }
        }

        // ── 4. Scroll metrics ──
        if let Some(metrics) = snapshot.scroll_metrics(ids.content) {
            self.max_scroll = (metrics.content_height - metrics.viewport_height).max(0.0);
        }
        self.scroll = self.scroll.clamp(0.0, self.max_scroll);

        // ── 5. Build frame ──
        let mut frame = Frame {
            clear: rgba(colors.background),
            ..Frame::default()
        };

        // ── Toolbar background ──
        if let Some(header_rect) = snapshot.get(ids.toolbar) {
            frame.quads.push(Quad {
                rect: [
                    header_rect.x,
                    header_rect.y,
                    header_rect.width,
                    header_rect.height,
                ],
                color: rgba(colors.surface),
                radius: theme.radii.lg,
                border_width: 1.0,
                border_color: rgba(colors.border),
            });
        }

        // ── Render interactive buttons by index ──
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

        for (index, rect) in self.buttons.into_iter().enumerate() {
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
                hovered: self.hovered == Some(index),
                pressed: self.pressed == Some(index),
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

            // Pre-compute label text (must outlive the button builder)
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

            // Build a temporary button for style resolution
            let builder = button::<PlaygroundMessage>(key, label_text.as_str())
                .variant(effective_variant)
                .disabled(disabled);
            let resolved = builder.resolve_style(&theme, state);

            // Button quad
            frame.quads.push(Quad {
                rect,
                color: rgba(resolved.background),
                radius: theme.radii.md,
                border_width: if state.focused { 3.0 } else { 1.0 },
                border_color: rgba(if state.focused {
                    resolved.focus
                } else {
                    resolved.border
                }),
            });

            // Button label text
            self.add_text(
                &mut frame,
                &label_text,
                [rect[0] + 14.0, rect[1] + 13.0],
                theme.typography.label,
                if disabled {
                    colors.disabled_text
                } else {
                    resolved.foreground
                },
                context.scale_factor,
                None,
            );
        }

        // ── Fixed header text ──
        if let Some(title_rect) = snapshot.get(ids.title) {
            self.add_text(
                &mut frame,
                "AcmeUI Native Playground",
                [title_rect.x, title_rect.y + 2.0],
                22.0,
                colors.foreground,
                context.scale_factor,
                None,
            );
        }

        if let Some(sub_rect) = snapshot.get(ids.subtitle) {
            self.add_text(
                &mut frame,
                "Interactive Widget Testing · 繁體中文 🎨",
                [sub_rect.x, sub_rect.y + 2.0],
                theme.typography.body,
                colors.muted_foreground,
                context.scale_factor,
                None,
            );
        }

        if let Some(status_rect) = snapshot.get(ids.status) {
            self.add_text(
                &mut frame,
                &format!(
                    "Theme: {}  Clicks: {}  Danger: {}",
                    if self.dark { "Dark" } else { "Light" },
                    self.click_count,
                    if self.danger_mode { "ON" } else { "OFF" },
                ),
                [status_rect.x, status_rect.y + 13.0],
                theme.typography.label,
                colors.muted_foreground,
                context.scale_factor,
                None,
            );
        }

        if let Some(sep_rect) = snapshot.get(ids.sep1) {
            frame.quads.push(Quad::solid(
                [sep_rect.x, sep_rect.y, sep_rect.width, 1.0],
                rgba(colors.border),
            ));
        }

        // ── Scroll viewport ──
        if let Some(vp) = snapshot.get(ids.content) {
            let clip = [vp.x, vp.y, vp.width, vp.height];
            let scroll = self.scroll;

            // Section headers
            let section_texts = [
                (ids.section_headers[0], "▸ Button Variants"),
                (ids.section_headers[1], "▸ Interactive Demo"),
                (ids.section_headers[2], "▸ Card Container"),
                (ids.section_headers[3], "▸ CJK & Emoji Rendering"),
            ];
            for (sid, text) in &section_texts {
                if let Some(r) = snapshot.get(*sid) {
                    let y = r.y - scroll;
                    self.add_text(
                        &mut frame,
                        text,
                        [r.x + 4.0, y + 2.0],
                        theme.typography.body,
                        colors.foreground,
                        context.scale_factor,
                        Some(clip),
                    );
                }
            }

            // Card background
            if let Some(card_rect) = snapshot.get(ids.card) {
                let y = card_rect.y - scroll;
                frame.clipped_quads.push(ClippedQuad {
                    quad: Quad {
                        rect: [card_rect.x, y, card_rect.width, card_rect.height],
                        color: rgba(colors.surface),
                        radius: theme.radii.lg,
                        border_width: 1.0,
                        border_color: rgba(colors.border),
                    },
                    clip,
                });
                // Card content labels
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
                        let cy = cr.y - scroll;
                        self.add_text(
                            &mut frame,
                            text,
                            [cr.x + 4.0, cy + 2.0],
                            theme.typography.body,
                            colors.foreground,
                            context.scale_factor,
                            Some(clip),
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
                    let y = r.y - scroll;
                    self.add_text(
                        &mut frame,
                        text,
                        [r.x + 4.0, y + 2.0],
                        theme.typography.body,
                        colors.foreground,
                        context.scale_factor,
                        Some(clip),
                    );
                }
            }

            // Separator in content area
            if let Some(sep2) = snapshot.get(ids.sep2) {
                let y = sep2.y - scroll;
                frame.clipped_quads.push(ClippedQuad {
                    quad: Quad::solid([sep2.x, y, sep2.width, 1.0], rgba(colors.border)),
                    clip,
                });
            }

            // State footer
            if let Some(footer) = snapshot.get(ids.footer) {
                let y = footer.y - scroll;
                self.add_text(
                    &mut frame,
                    &format!(
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
                    ),
                    [footer.x + 4.0, y + 2.0],
                    theme.typography.label,
                    colors.muted_foreground,
                    context.scale_factor,
                    Some(clip),
                );
            }
        }

        scene_from_frame(&frame)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn rgba(color: ThemeColor) -> [f32; 4] {
    [color.red, color.green, color.blue, color.alpha]
}
