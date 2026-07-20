use acme_layout::{Edges, LayoutEngine, LayoutKind, LayoutNode, LayoutStyle, Length, Overflow};
use acme_platform::{Application, FrameContext, PlatformEvent, PlatformKey, WindowConfig};
use acme_render_wgpu::{ClippedQuad, Frame, Quad, TextRun};
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextStyle};
use acme_theme::{Theme, ThemeColor};
use acme_widgets::{WidgetNode, button, column, label, row, scroll_view};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();
    acme_platform::run(Gallery::new())?;
    Ok(())
}

struct Gallery {
    dark: bool,
    hovered: Option<usize>,
    pressed: Option<usize>,
    focused: usize,
    cursor: (f32, f32),
    scroll: f32,
    max_scroll: f32,
    buttons: [[f32; 4]; 3],
    fonts: FontSystem,
    atlas: GlyphAtlas,
    layout: LayoutEngine,
}

#[derive(Clone, Copy)]
enum GalleryMessage {
    ToggleTheme,
    FocusDemo,
    DpiInfo,
}

impl Gallery {
    fn new() -> Self {
        Self {
            dark: false,
            hovered: None,
            pressed: None,
            focused: 0,
            cursor: (0.0, 0.0),
            scroll: 0.0,
            max_scroll: 0.0,
            buttons: [[0.0; 4]; 3],
            fonts: FontSystem::new(),
            atlas: GlyphAtlas::new(2048, 2048),
            layout: LayoutEngine::new(),
        }
    }

    fn description(&self) -> WidgetNode<GalleryMessage> {
        let mut samples = scroll_view("samples").viewport_height(320.0);
        for index in 0..9 {
            samples = samples.child(label(format!(
                "範例 {:02} — CJK fallback 與彩色 emoji 🚀",
                index + 1
            )));
        }
        column()
            .key("gallery")
            .gap(16.0)
            .padding(28.0)
            .child(label("AcmeUI Native Gallery — 真實文字渲染 🙂"))
            .child(
                row()
                    .gap(16.0)
                    .child(
                        button("theme", "切換主題")
                            .primary()
                            .on_click(GalleryMessage::ToggleTheme),
                    )
                    .child(button("focus", "鍵盤焦點").on_click(GalleryMessage::FocusDemo))
                    .child(button("dpi", "DPI 安全").on_click(GalleryMessage::DpiInfo)),
            )
            .child(samples.build())
            .build()
    }

    fn hit(&self) -> Option<usize> {
        self.buttons.iter().position(|rect| {
            self.cursor.0 >= rect[0]
                && self.cursor.0 <= rect[0] + rect[2]
                && self.cursor.1 >= rect[1]
                && self.cursor.1 <= rect[1] + rect[3]
        })
    }

    fn activate(&mut self, index: usize) -> bool {
        let WidgetNode::Column(root) = self.description() else {
            return false;
        };
        let Some(WidgetNode::Row(button_row)) = root.children.get(1) else {
            return false;
        };
        let Some(WidgetNode::Button(button)) = button_row.children.get(index) else {
            return false;
        };
        let message = button.activate().copied();
        match message {
            Some(GalleryMessage::ToggleTheme) => self.dark = !self.dark,
            Some(GalleryMessage::FocusDemo | GalleryMessage::DpiInfo) => {}
            None => return false,
        }
        true
    }

    fn add_text(
        &mut self,
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

impl Application for Gallery {
    fn window_config(&self) -> WindowConfig {
        WindowConfig {
            title: "AcmeUI Native Gallery".into(),
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
                    (self.focused + 2) % 3
                } else {
                    (self.focused + 1) % 3
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
        let viewport_height = (context.logical_height - 260.0).max(120.0);
        let mut next_id = 1;
        let mut root = self.description().to_layout(&mut next_id);
        apply_gallery_styles(
            &mut root,
            context.logical_width,
            context.logical_height,
            viewport_height,
        );
        let snapshot = self
            .layout
            .compute(&root, (context.logical_width, context.logical_height))
            .expect("finite Gallery viewport");
        let theme = if self.dark {
            Theme::dark()
        } else {
            Theme::light()
        };
        let colors = theme.colors;
        let header = *snapshot.get(2).expect("header layout");
        let viewport = *snapshot.get(7).expect("viewport layout");
        self.max_scroll = snapshot
            .scroll_metrics(7)
            .map(|metrics| (metrics.content_height - metrics.viewport_height).max(0.0))
            .unwrap_or(0.0);
        self.scroll = self.scroll.clamp(0.0, self.max_scroll);
        for (index, id) in [4, 5, 6].into_iter().enumerate() {
            let rect = *snapshot.get(id).expect("button layout");
            self.buttons[index] = [rect.x, rect.y, rect.width, rect.height];
        }

        let mut frame = Frame {
            clear: rgba(colors.background),
            ..Frame::default()
        };
        frame
            .quads
            .push(panel(header, colors.surface, colors.border, theme.radii.lg));
        self.add_text(
            &mut frame,
            "AcmeUI Native Gallery",
            ([header.x + 24.0, header.y + 20.0], 25.0),
            colors.text,
            context.scale_factor,
            None,
        );
        self.add_text(
            &mut frame,
            "English · 繁體中文 · Emoji 🎨🙂 · cosmic-text 0.14",
            ([header.x + 24.0, header.y + 60.0], 17.0),
            colors.text_muted,
            context.scale_factor,
            None,
        );

        let labels = ["切換主題", "Tab 鍵盤焦點", "DPI 100–200%"];
        let keys = ["theme", "focus", "dpi"];
        for (index, rect) in self.buttons.into_iter().enumerate() {
            let state = acme_widgets::ButtonState {
                hovered: self.hovered == Some(index),
                pressed: self.pressed == Some(index),
                focused: self.focused == index,
            };
            let builder = if index == 0 {
                button::<GalleryMessage>("theme", labels[index]).primary()
            } else {
                button::<GalleryMessage>(keys[index], labels[index])
            };
            let resolved = builder.resolve_style(&theme, state);
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
            self.add_text(
                &mut frame,
                labels[index],
                (
                    [rect[0] + 18.0, rect[1] + 14.0],
                    theme.typography.label_size,
                ),
                resolved.foreground,
                context.scale_factor,
                None,
            );
        }

        frame.quads.push(panel(
            viewport,
            colors.surface,
            colors.border,
            theme.radii.lg,
        ));
        let clip = [
            viewport.x + 1.0,
            viewport.y + 1.0,
            viewport.width - 2.0,
            viewport.height - 2.0,
        ];
        for index in 0..9 {
            let sample = *snapshot.get(8 + index as u64).expect("sample layout");
            let y = sample.y - self.scroll;
            let item = Quad {
                rect: [sample.x + 18.0, y + 8.0, sample.width - 36.0, 56.0],
                color: rgba(if index % 2 == 0 {
                    colors.surface_hover
                } else {
                    colors.surface
                }),
                radius: theme.radii.md,
                border_width: 1.0,
                border_color: rgba(colors.border),
            };
            frame.clipped_quads.push(ClippedQuad { quad: item, clip });
            self.add_text(
                &mut frame,
                &format!("範例 {:02} — CJK fallback 與彩色 emoji 🚀", index + 1),
                ([sample.x + 34.0, y + 23.0], theme.typography.body_size),
                colors.text,
                context.scale_factor,
                Some(clip),
            );
        }
        frame
    }
}

fn apply_gallery_styles(root: &mut LayoutNode, width: f32, height: f32, viewport_height: f32) {
    root.style = LayoutStyle {
        kind: LayoutKind::Column,
        width: Length::px(width),
        height: Length::px(height),
        padding: Edges::all(28.0),
        gap: 18.0,
        ..LayoutStyle::default()
    };
    root.children[0].style.width = Length::px((width - 56.0).max(300.0));
    root.children[0].style.height = Length::px(106.0);
    root.children[1].style = LayoutStyle {
        kind: LayoutKind::Row,
        gap: 16.0,
        height: Length::px(52.0),
        ..LayoutStyle::default()
    };
    for button in &mut root.children[1].children {
        button.style.width = Length::px(220.0);
        button.style.height = Length::px(52.0);
    }
    root.children[2].style = LayoutStyle {
        kind: LayoutKind::Column,
        width: Length::px((width - 56.0).max(300.0)),
        height: Length::px(viewport_height),
        overflow: Overflow::Scroll,
        ..LayoutStyle::default()
    };
    for sample in &mut root.children[2].children {
        sample.style.width = Length::px((width - 92.0).max(260.0));
        sample.style.height = Length::px(72.0);
    }
}

fn rgba(color: ThemeColor) -> [f32; 4] {
    [color.red, color.green, color.blue, color.alpha]
}

fn panel(rect: acme_layout::LayoutRect, fill: ThemeColor, border: ThemeColor, radius: f32) -> Quad {
    Quad {
        rect: [rect.x, rect.y, rect.width, rect.height],
        color: rgba(fill),
        radius,
        border_width: 1.0,
        border_color: rgba(border),
    }
}
