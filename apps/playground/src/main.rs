//! AcmeUI Native Playground — interactive widget testing application.
//!
//! Demonstrates all available widget types with theme toggling, keyboard
//! navigation, CJK/emoji rendering, and real-time state display.
#![forbid(unsafe_op_in_unsafe_fn)]

use acme_layout::{Edges, LayoutEngine, LayoutKind, LayoutStyle, Length, Overflow};
use acme_platform::{Application, FrameContext, PlatformEvent, PlatformKey, WindowConfig};
use acme_render_wgpu::{ClippedQuad, Frame, Quad, TextRun};
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextStyle};
use acme_theme::{Theme, ThemeColor};
use acme_widgets::{
    ButtonState, ButtonVariant, WidgetNode, button, card, column, label, row, separator,
};

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
            atlas: GlyphAtlas::new(2048, 2048),
            layout: LayoutEngine::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Widget tree
    // -----------------------------------------------------------------------
    // ID mapping (pre-order traversal of to_layout):
    //
    //   1: Column(root)          "playground"
    //     2: Row(toolbar)
    //       3: Button(theme)              ← index 0
    //       4: Label(status)
    //     5: Label(title)
    //     6: Label(subtitle)
    //     7: Separator
    //     8: Column(content)      scrollable
    //       9: Label(section-hdr) "▸ Button Variants"
    //      10: Row(btn-row)
    //        11: Button(primary)          ← index 1
    //        12: Button(secondary)        ← index 2
    //        13: Button(ghost)            ← index 3
    //        14: Button(danger)           ← index 4
    //      15: Label(section-hdr) "▸ Interactive Demo"
    //      16: Row(demo-row)
    //        17: Button(clicker)          ← index 5
    //        18: Button(reset)            ← index 6
    //        19: Button(toggle-danger)    ← index 7
    //      20: Label(section-hdr) "▸ Card Container"
    //      21: Card(demo-card)
    //        22: Label(card-0)
    //        23: Label(card-1)
    //        24: Label(card-2)
    //      25: Label(section-hdr) "▸ CJK & Emoji"
    //      26: Label(cjk-1)
    //      27: Label(cjk-2)
    //      28: Label(korean)
    //      29: Label(emoji)
    //      30: Label(mixed)
    //      31: Separator
    //      32: Label(state-footer)
    //
    // Interactive button IDs: 3, 11, 12, 13, 14, 17, 18, 19

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

// ---------------------------------------------------------------------------
// Application trait implementation
// ---------------------------------------------------------------------------

impl Application for Playground {
    fn window_config(&self) -> WindowConfig {
        WindowConfig {
            title: "AcmeUI Native Playground".into(),
            width: 1080.0,
            height: 720.0,
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

    fn frame(&mut self, context: FrameContext) -> Frame {
        let width = context.logical_width;
        let height = context.logical_height;
        let theme = if self.dark {
            Theme::dark()
        } else {
            Theme::light()
        };
        let colors = theme.colors;

        // ── 1. Build and style the layout tree ──
        let mut next_id = 1;
        let mut root = self.build_tree().to_layout(&mut next_id);

        // Root column — fill the window
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

        // ── Style scroll-area children ──
        let content = &mut root.children[4];

        // Section headers (children 0, 4, 7, 10)
        for &i in &[0usize, 4, 7, 10] {
            if i < content.children.len() {
                content.children[i].style.width = Length::px(body_width);
            }
        }

        // Button row 1 (child 1) — btn-row
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

        // Button row 2 (child 5) — demo-row
        if content.children.len() > 5 {
            content.children[5].style = LayoutStyle {
                kind: LayoutKind::Row,
                width: Length::px(body_width),
                height: Length::px(header_height),
                gap: 8.0,
                ..LayoutStyle::default()
            };
            for btn in &mut content.children[5].children {
                btn.style = LayoutStyle {
                    width: Length::px(140.0),
                    height: Length::px(header_height),
                    ..LayoutStyle::default()
                };
            }
        }

        // Card (child 8)
        if content.children.len() > 8 {
            content.children[8].style = LayoutStyle {
                kind: LayoutKind::Column,
                width: Length::px(body_width),
                padding: Edges::all(16.0),
                gap: 8.0,
                ..LayoutStyle::default()
            };
        }

        // CJK/Emoji labels (children 11–14)
        for &i in &[11usize, 12, 13, 14, 15] {
            if i < content.children.len() {
                content.children[i].style.width = Length::px(body_width);
            }
        }

        // Separator (child 16)
        if content.children.len() > 16 {
            content.children[16].style = LayoutStyle {
                width: Length::px(body_width),
                height: Length::px(1.0),
                ..LayoutStyle::default()
            };
        }

        // State footer (child 17)
        if content.children.len() > 17 {
            content.children[17].style.width = Length::px(body_width);
        }

        // ── 2. Compute layout ──
        let snapshot = self
            .layout
            .compute(&root, (width, height))
            .expect("finite Playground viewport");

        // ── 3. Extract button hit-test rects ──
        let button_ids = [3u64, 11, 12, 13, 14, 17, 18, 19];
        for (index, id) in button_ids.into_iter().enumerate() {
            if let Some(rect) = snapshot.get(id) {
                self.buttons[index] = [rect.x, rect.y, rect.width, rect.height];
            }
        }

        // ── 4. Scroll metrics ──
        if let Some(metrics) = snapshot.scroll_metrics(8) {
            self.max_scroll = (metrics.content_height - metrics.viewport_height).max(0.0);
        }
        self.scroll = self.scroll.clamp(0.0, self.max_scroll);

        // ── 5. Build frame ──
        let mut frame = Frame {
            clear: rgba(colors.background),
            ..Frame::default()
        };

        // ── Toolbar background ──
        if let Some(header_rect) = snapshot.get(2) {
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
                theme.typography.label_size,
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
        // Title (ID 5)
        if let Some(title_rect) = snapshot.get(5) {
            self.add_text(
                &mut frame,
                "AcmeUI Native Playground",
                [title_rect.x, title_rect.y + 2.0],
                22.0,
                colors.text,
                context.scale_factor,
                None,
            );
        }

        // Subtitle (ID 6)
        if let Some(sub_rect) = snapshot.get(6) {
            self.add_text(
                &mut frame,
                "Interactive Widget Testing · 繁體中文 🎨",
                [sub_rect.x, sub_rect.y + 2.0],
                theme.typography.body_size,
                colors.text_muted,
                context.scale_factor,
                None,
            );
        }

        // Status label in toolbar (ID 4)
        if let Some(status_rect) = snapshot.get(4) {
            self.add_text(
                &mut frame,
                &format!(
                    "Theme: {}  Clicks: {}  Danger: {}",
                    if self.dark { "Dark" } else { "Light" },
                    self.click_count,
                    if self.danger_mode { "ON" } else { "OFF" },
                ),
                [status_rect.x, status_rect.y + 13.0],
                theme.typography.label_size,
                colors.text_muted,
                context.scale_factor,
                None,
            );
        }

        // Separator (ID 7)
        if let Some(sep_rect) = snapshot.get(7) {
            frame.quads.push(Quad::solid(
                [sep_rect.x, sep_rect.y, sep_rect.width, 1.0],
                rgba(colors.border),
            ));
        }

        // ── Scroll viewport ──
        if let Some(vp) = snapshot.get(8) {
            let clip = [vp.x, vp.y, vp.width, vp.height];
            let scroll = self.scroll;

            // Section headers
            let section_ids = [
                (9u64, "▸ Button Variants"),
                (15, "▸ Interactive Demo"),
                (20, "▸ Card Container"),
                (25, "▸ CJK & Emoji Rendering"),
            ];
            for (sid, text) in &section_ids {
                if let Some(r) = snapshot.get(*sid) {
                    let y = r.y - scroll;
                    self.add_text(
                        &mut frame,
                        text,
                        [r.x + 4.0, y + 2.0],
                        theme.typography.body_size,
                        colors.text,
                        context.scale_factor,
                        Some(clip),
                    );
                }
            }

            // Card background (ID 21)
            if let Some(card_rect) = snapshot.get(21) {
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
                // Card content labels (IDs 22, 23, 24)
                for cid in [22u64, 23, 24] {
                    if let Some(cr) = snapshot.get(cid) {
                        let cy = cr.y - scroll;
                        self.add_text(
                            &mut frame,
                            match cid {
                                22 => "Card — a rounded surface with column layout and padding.",
                                23 => {
                                    "Cards can contain any widgets: labels, buttons, rows, columns."
                                }
                                24 => "Supports nesting, gap spacing, and semantic padding.",
                                _ => "",
                            },
                            [cr.x + 4.0, cy + 2.0],
                            theme.typography.body_size,
                            colors.text,
                            context.scale_factor,
                            Some(clip),
                        );
                    }
                }
            }

            // CJK & Emoji labels (IDs 26-30)
            let cjk_emoji_texts = [
                (26u64, "繁體中文：系統介面與文字渲染測試 🀄"),
                (27, "日本語：システムインターフェース 🗾"),
                (28, "한국어: 사용자 인터페이스 테스트 🎯"),
                (29, "Emoji: 🚀🎨🙂🎉🔥💯⭐🧪✨🎭👋🌟😊🎮"),
                (30, "Mixed: Hello 你好 こんにちは 🌍 123 ABC"),
            ];
            for (cid, text) in &cjk_emoji_texts {
                if let Some(r) = snapshot.get(*cid) {
                    let y = r.y - scroll;
                    self.add_text(
                        &mut frame,
                        text,
                        [r.x + 4.0, y + 2.0],
                        theme.typography.body_size,
                        colors.text,
                        context.scale_factor,
                        Some(clip),
                    );
                }
            }

            // Separator in content area (ID 31)
            if let Some(sep2) = snapshot.get(31) {
                let y = sep2.y - scroll;
                frame.clipped_quads.push(ClippedQuad {
                    quad: Quad::solid([sep2.x, y, sep2.width, 1.0], rgba(colors.border)),
                    clip,
                });
            }

            // State footer (ID 32)
            if let Some(footer) = snapshot.get(32) {
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
                    theme.typography.label_size,
                    colors.text_muted,
                    context.scale_factor,
                    Some(clip),
                );
            }
        }

        frame
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn rgba(color: ThemeColor) -> [f32; 4] {
    [color.red, color.green, color.blue, color.alpha]
}
