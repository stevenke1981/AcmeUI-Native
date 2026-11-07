//! Minimal external consumer of the AcmeUI-Native SDK.
//!
//! The UI is built through the same public path the Gallery apps use:
//! `WidgetNode` tree -> `LayoutNode` -> `LayoutEngine::compute_with_text`
//! -> `acme_render_wgpu::Frame` -> `scene_from_frame`. Whether the window is
//! presented by the GPU or the CPU renderer is decided by `acme_platform::run`
//! from `ACME_RENDER_BACKEND`, not by this crate.
#![forbid(unsafe_code)]

use acme_core::{NodeId, Scene};
use acme_layout::{LayoutEngine, LayoutNode, LayoutRect, LayoutSnapshot, WidgetLayoutContext};
use acme_platform::{Application, FrameContext, WindowConfig, WindowId, run};
use acme_render_wgpu::{Frame, Quad, TextRun, scene_from_frame};
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextStyle};
use acme_theme::{Theme, ThemeColor};
use acme_ui::prelude::*;

const WINDOW_WIDTH: f64 = 640.0;
const WINDOW_HEIGHT: f64 = 420.0;
const ATLAS_SIZE: u32 = 2048;
const CONTROL_HEIGHT: f32 = 36.0;
const CARD_RADIUS: f32 = 8.0;
const BUTTON_RADIUS: f32 = 6.0;
const BORDER_WIDTH: f32 = 1.0;

/// Application state: text system, glyph atlas, layout engine and theme.
pub struct ConsumerApp {
    fonts: FontSystem,
    atlas: GlyphAtlas,
    layout: LayoutEngine,
    theme: Theme,
}

impl Default for ConsumerApp {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsumerApp {
    pub fn new() -> Self {
        Self {
            fonts: FontSystem::new(),
            atlas: GlyphAtlas::new(ATLAS_SIZE, ATLAS_SIZE),
            layout: LayoutEngine::new(),
            theme: Theme::light(),
        }
    }

    /// Builds one frame of the UI as a [`Scene`] without opening a window.
    pub fn build_scene(
        &mut self,
        width: f32,
        height: f32,
        scale_factor: f32,
    ) -> Result<Scene, Box<dyn std::error::Error>> {
        let view = description();
        let context = WidgetLayoutContext {
            body_font_size: self.theme.typography.body,
            body_line_height: self.theme.typography.body * self.theme.typography.line_height,
            label_font_size: self.theme.typography.label,
            control_height: CONTROL_HEIGHT,
            scale_factor,
        };
        let root = view.to_layout_with_context(NodeId::new(1), &context);
        let snapshot =
            self.layout
                .compute_with_text(&root, (width, height), &mut self.fonts, scale_factor)?;
        let mut painter = Painter {
            snapshot: &snapshot,
            theme: &self.theme,
            fonts: &mut self.fonts,
            atlas: &mut self.atlas,
            scale_factor,
            frame: Frame {
                clear: rgba(self.theme.colors.background),
                ..Frame::default()
            },
        };
        painter.paint(&view, &root);
        Ok(scene_from_frame(&painter.frame))
    }
}

impl Application for ConsumerApp {
    fn window_config(&self) -> WindowConfig {
        WindowConfig {
            title: "AcmeUI SDK consumer".into(),
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        }
    }

    fn on_gpu_recovered(&mut self, _window: WindowId) {
        self.atlas.clear();
    }

    fn frame(&mut self, context: FrameContext) -> Scene {
        match self.build_scene(
            context.logical_width,
            context.logical_height,
            context.scale_factor,
        ) {
            Ok(scene) => scene,
            Err(error) => {
                eprintln!("acme-sdk-consumer: failed to build frame: {error}");
                Scene::new()
            }
        }
    }
}

/// The declarative UI: a template root, a card with one label, and a button.
/// The button is visual only; this example does not route click messages.
fn description() -> WidgetNode<()> {
    default_template::<()>("SDK consumer")
        .subtitle("Built from path dependencies on the AcmeUI-Native crates")
        .child(
            card::<()>()
                .child(label("Rendered through the public SDK API"))
                .build(),
        )
        .child(button::<()>("primary-action", "Continue").primary())
        .build()
}

#[derive(Clone, Copy)]
enum Align {
    Start,
    Center,
}

/// Walks the widget tree alongside its layout and emits renderer primitives.
struct Painter<'a> {
    snapshot: &'a LayoutSnapshot,
    theme: &'a Theme,
    fonts: &'a mut FontSystem,
    atlas: &'a mut GlyphAtlas,
    scale_factor: f32,
    frame: Frame,
}

impl Painter<'_> {
    fn paint(&mut self, widget: &WidgetNode<()>, layout: &LayoutNode) {
        let Some(&rect) = self.snapshot.get(layout.id) else {
            return;
        };
        let colors = self.theme.colors;
        match widget {
            WidgetNode::Column(container) | WidgetNode::Row(container) => {
                self.paint_children(&container.children, &layout.children);
            }
            WidgetNode::Card(card) => {
                self.frame
                    .quads
                    .push(quad(rect, colors.surface, colors.border, CARD_RADIUS));
                self.paint_children(&card.children, &layout.children);
            }
            WidgetNode::Label(text) => {
                let size = text.font_size.unwrap_or(self.theme.typography.body);
                self.draw_text(&text.text, rect, size, colors.foreground, Align::Start);
            }
            WidgetNode::Button(button) => {
                self.frame
                    .quads
                    .push(quad(rect, colors.primary, colors.primary, BUTTON_RADIUS));
                self.draw_text(
                    &button.label,
                    rect,
                    self.theme.typography.label,
                    colors.primary_foreground,
                    Align::Center,
                );
            }
            _ => {}
        }
    }

    fn paint_children(&mut self, widgets: &[WidgetNode<()>], layouts: &[LayoutNode]) {
        for (widget, layout) in widgets.iter().zip(layouts) {
            self.paint(widget, layout);
        }
    }

    fn draw_text(
        &mut self,
        content: &str,
        rect: LayoutRect,
        font_size: f32,
        color: ThemeColor,
        align: Align,
    ) {
        let style = TextStyle {
            font_size,
            line_height: font_size * self.theme.typography.line_height,
            ..TextStyle::default()
        };
        let measured = self
            .fonts
            .measure(content, &style, TextConstraints::default());
        let x = match align {
            Align::Start => rect.x,
            Align::Center => rect.x + (rect.width - measured.width) / 2.0,
        };
        let y = rect.y + (rect.height - measured.height) / 2.0 + measured.baseline;
        let layout = self.fonts.shape(
            content,
            &style,
            TextConstraints::default(),
            self.scale_factor,
        );
        let prepared = self.fonts.prepare(&layout, self.atlas);
        self.frame.text.push(TextRun {
            prepared,
            origin: [x, y],
            color: rgba(color),
            clip: None,
        });
    }
}

fn quad(rect: LayoutRect, fill: ThemeColor, border: ThemeColor, radius: f32) -> Quad {
    Quad {
        rect: [rect.x, rect.y, rect.width, rect.height],
        color: rgba(fill),
        radius,
        border_width: BORDER_WIDTH,
        border_color: rgba(border),
    }
}

fn rgba(color: ThemeColor) -> [f32; 4] {
    [color.red, color.green, color.blue, color.alpha]
}

fn main() {
    if let Err(error) = run(ConsumerApp::new()) {
        eprintln!("acme-sdk-consumer: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_scene_is_non_empty_without_opening_a_window() {
        let mut app = ConsumerApp::new();

        let scene = app
            .build_scene(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32, 1.0)
            .expect("the consumer frame should lay out");

        assert!(!scene.commands().is_empty(), "scene has no draw commands");
        scene.validate().expect("scene should be valid");
    }
}
