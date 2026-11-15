//! Minimal external consumer of the AcmeUI-Native SDK.
//!
//! The UI is built through the public immediate-mode SDK path:
//! `WidgetNode` tree -> `LayoutNode` -> `LayoutEngine::compute_with_text`
//! -> `acme_core::Painter` & `acme_core::HitTestLayer` / `ClickDetector`.
//! Whether the window is presented by the GPU or the CPU renderer is decided by
//! `acme_platform::run` from `ACME_RENDER_BACKEND`, not by this crate.
#![forbid(unsafe_code)]

use std::time::Instant;

use acme_core::{
    AtlasUpload, ClickDetector, CursorHint, GlyphDraw, GlyphFormat, HitRegionSpec, HitTestLayer,
    NodeId, Painter, Point, PointerOutcome, Scene, TextPrimitive,
};
use acme_layout::{LayoutEngine, LayoutNode, LayoutRect, LayoutSnapshot, WidgetLayoutContext};
use acme_platform::{
    Application, CursorIcon, FrameContext, PlatformEvent, WindowConfig, WindowId, WindowRequest,
    run,
};
use acme_text::{AtlasFormat, FontSystem, GlyphAtlas, TextConstraints, TextStyle};
use acme_theme::{Theme, ThemeColor};
use acme_ui::ButtonState;
use acme_ui::painter_bridge::{layout_rect_to_logical, theme_color};
use acme_ui::prelude::*;

const WINDOW_WIDTH: f64 = 640.0;
const WINDOW_HEIGHT: f64 = 420.0;
const CONTROL_HEIGHT: f32 = 36.0;
const CARD_RADIUS: f32 = 8.0;
const BUTTON_RADIUS: f32 = 6.0;
const BORDER_WIDTH: f32 = 1.0;

#[derive(Clone, Copy, Debug)]
enum ConsumerMessage {
    Continue,
}

/// Application state: text system, glyph atlas, layout engine, theme, and hit state.
pub struct ConsumerApp {
    fonts: FontSystem,
    atlas: GlyphAtlas,
    layout: LayoutEngine,
    theme: Theme,
    hit_layer: HitTestLayer,
    click_detector: ClickDetector,
    cursor_hint: CursorHint,
    pending_window_requests: Vec<WindowRequest>,
    started_at: Instant,
    action_clicks: u32,
    button_id: Option<NodeId>,
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
            atlas: GlyphAtlas::default(),
            layout: LayoutEngine::new(),
            theme: Theme::light(),
            hit_layer: HitTestLayer::new(),
            click_detector: ClickDetector::new(),
            cursor_hint: CursorHint::Default,
            pending_window_requests: Vec::new(),
            started_at: Instant::now(),
            action_clicks: 0,
            button_id: None,
        }
    }

    fn sync_cursor(&mut self) {
        let next = self.click_detector.cursor();
        if next != self.cursor_hint {
            self.cursor_hint = next;
            self.pending_window_requests
                .push(WindowRequest::SetCursor(map_cursor_hint(next)));
        }
    }

    /// Builds one frame of the UI as a [`Scene`] without opening a window,
    /// and populates [`HitTestLayer`] for pointer routing.
    pub fn build_scene(
        &mut self,
        width: f32,
        height: f32,
        scale_factor: f32,
    ) -> Result<Scene, Box<dyn std::error::Error>> {
        self.atlas.begin_frame();
        let view = description(self.action_clicks);
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

        self.hit_layer.clear();
        self.button_id = None;
        let hovered_id = self.click_detector.hovered();
        let pressed_id = self.click_detector.pressed();
        let mut painter = Painter::with_clear(theme_color(self.theme.colors.background));

        let mut renderer = SceneRenderer {
            snapshot: &snapshot,
            theme: &self.theme,
            fonts: &mut self.fonts,
            atlas: &mut self.atlas,
            scale_factor,
            hovered_id,
            pressed_id,
            painter: &mut painter,
            hit_layer: &mut self.hit_layer,
            button_id: &mut self.button_id,
        };
        renderer.paint(&view, &root);

        painter
            .finish_checked()
            .map_err(|err| format!("invalid scene: {err:?}").into())
    }
}

impl Application for ConsumerApp {
    fn window_config(&self) -> WindowConfig {
        WindowConfig {
            title: "AcmeUI SDK consumer".into(),
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            ..WindowConfig::default()
        }
    }

    fn take_window_requests(&mut self, _window: WindowId) -> Vec<WindowRequest> {
        std::mem::take(&mut self.pending_window_requests)
    }

    fn event(&mut self, event: PlatformEvent) -> bool {
        match event {
            PlatformEvent::PointerMoved { x, y, .. } => {
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
                let point = Point::new(x, y);
                let time_secs = self.started_at.elapsed().as_secs_f64();
                let outcomes = self.click_detector.on_pointer_button(
                    &self.hit_layer,
                    button,
                    pressed,
                    point,
                    time_secs,
                );
                let mut activated = false;
                for outcome in &outcomes {
                    if let PointerOutcome::Clicked { id, button: 0, .. } = *outcome
                        && self.button_id == Some(id)
                    {
                        self.action_clicks = self.action_clicks.wrapping_add(1);
                        activated = true;
                    }
                }
                if !pressed {
                    let _ = self.click_detector.on_pointer_move(&self.hit_layer, point);
                }
                self.sync_cursor();
                activated || !outcomes.is_empty()
            }
            PlatformEvent::Resized { .. } => true,
            _ => false,
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

/// The declarative UI: a template root, a card with one label, and an interactive button.
fn description(action_clicks: u32) -> WidgetNode<ConsumerMessage> {
    default_template::<ConsumerMessage>("SDK consumer")
        .subtitle("Built from path dependencies on the AcmeUI-Native crates")
        .child(
            card::<ConsumerMessage>()
                .child(label(format!(
                    "Rendered through Painter & HitTestLayer (clicks: {action_clicks})"
                )))
                .build(),
        )
        .child(
            button::<ConsumerMessage>("primary-action", "Continue")
                .primary()
                .on_click(ConsumerMessage::Continue),
        )
        .build()
}

#[derive(Clone, Copy)]
enum Align {
    Start,
    Center,
}

/// Walks the widget tree alongside its layout and emits [`Painter`] primitives
/// and [`HitTestLayer`] regions.
struct SceneRenderer<'a, 'p> {
    snapshot: &'a LayoutSnapshot,
    theme: &'a Theme,
    fonts: &'a mut FontSystem,
    atlas: &'a mut GlyphAtlas,
    scale_factor: f32,
    hovered_id: Option<NodeId>,
    pressed_id: Option<NodeId>,
    painter: &'a mut Painter<'p>,
    hit_layer: &'a mut HitTestLayer,
    button_id: &'a mut Option<NodeId>,
}

impl SceneRenderer<'_, '_> {
    fn paint(&mut self, widget: &WidgetNode<ConsumerMessage>, layout: &LayoutNode) {
        let Some(&rect) = self.snapshot.get(layout.id) else {
            return;
        };
        let colors = self.theme.colors;
        match widget {
            WidgetNode::Column(container) | WidgetNode::Row(container) => {
                self.paint_children(&container.children, &layout.children);
            }
            WidgetNode::Card(card) => {
                self.painter.rounded_rect_bordered(
                    layout_rect_to_logical(rect),
                    CARD_RADIUS,
                    theme_color(colors.surface),
                    BORDER_WIDTH,
                    theme_color(colors.border),
                );
                self.paint_children(&card.children, &layout.children);
            }
            WidgetNode::Label(text) => {
                let size = text.font_size.unwrap_or(self.theme.typography.body);
                self.draw_text(&text.text, rect, size, colors.foreground, Align::Start);
            }
            WidgetNode::Button(button) => {
                let state = ButtonState {
                    hovered: self.hovered_id == Some(layout.id),
                    pressed: self.pressed_id == Some(layout.id),
                    focused: false,
                };
                let resolved = button.resolve_style(self.theme, state);
                let logical_rect = layout_rect_to_logical(rect);
                self.painter.rounded_rect_bordered(
                    logical_rect,
                    BUTTON_RADIUS,
                    theme_color(resolved.background),
                    BORDER_WIDTH,
                    theme_color(resolved.border),
                );
                let fg = if button.disabled {
                    colors.disabled_text
                } else {
                    resolved.foreground
                };
                self.draw_text(
                    &button.label,
                    rect,
                    self.theme.typography.label,
                    fg,
                    Align::Center,
                );
                self.hit_layer.push_region(
                    HitRegionSpec::rect(layout.id, logical_rect)
                        .enabled(!button.disabled)
                        .cursor(if button.disabled {
                            CursorHint::NotAllowed
                        } else {
                            CursorHint::Pointer
                        })
                        .tag(1),
                );
                *self.button_id = Some(layout.id);
            }
            _ => {}
        }
    }

    fn paint_children(&mut self, widgets: &[WidgetNode<ConsumerMessage>], layouts: &[LayoutNode]) {
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

        self.painter.text_primitive(TextPrimitive {
            origin: Point::new(x, y),
            color: theme_color(color),
            glyphs,
            uploads,
        });
    }
}

fn map_atlas_format(format: AtlasFormat) -> GlyphFormat {
    match format {
        AtlasFormat::Alpha8 => GlyphFormat::Alpha8,
        AtlasFormat::Rgba8 => GlyphFormat::Rgba8,
    }
}

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

    #[test]
    fn hit_layer_and_click_detector_route_button_clicks_and_drag_cancel() {
        let mut app = ConsumerApp::new();
        let _ = app
            .build_scene(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32, 1.0)
            .expect("scene should build");
        let btn_id = app.button_id.expect("button hit region registered");
        let entry = app
            .hit_layer
            .to_flat_entries()
            .into_iter()
            .find(|e| e.id == btn_id)
            .expect("button entry in hit layer");
        let cx = entry.bounds.origin.x.get() + entry.bounds.size.width.get() * 0.5;
        let cy = entry.bounds.origin.y.get() + entry.bounds.size.height.get() * 0.5;

        assert!(app.event(PlatformEvent::PointerMoved {
            window: WindowId(0),
            x: cx,
            y: cy,
        }));
        assert_eq!(app.click_detector.hovered(), Some(btn_id));
        assert!(
            app.take_window_requests(WindowId(0))
                .contains(&WindowRequest::SetCursor(CursorIcon::Pointer))
        );

        assert!(app.event(PlatformEvent::PointerButton {
            window: WindowId(0),
            pressed: true,
            x: cx,
            y: cy,
            button: 0,
            pointer: 1,
            shift: false,
            ctrl: false,
            alt: false,
        }));
        assert!(app.event(PlatformEvent::PointerButton {
            window: WindowId(0),
            pressed: false,
            x: cx,
            y: cy,
            button: 0,
            pointer: 1,
            shift: false,
            ctrl: false,
            alt: false,
        }));
        assert_eq!(app.action_clicks, 1);

        // Drag out cancels click activation.
        assert!(app.event(PlatformEvent::PointerButton {
            window: WindowId(0),
            pressed: true,
            x: cx,
            y: cy,
            button: 0,
            pointer: 1,
            shift: false,
            ctrl: false,
            alt: false,
        }));
        assert!(app.event(PlatformEvent::PointerMoved {
            window: WindowId(0),
            x: cx + 200.0,
            y: cy + 200.0,
        }));
        assert!(app.event(PlatformEvent::PointerButton {
            window: WindowId(0),
            pressed: false,
            x: cx + 200.0,
            y: cy + 200.0,
            button: 0,
            pointer: 1,
            shift: false,
            ctrl: false,
            alt: false,
        }));
        assert_eq!(app.action_clicks, 1);
    }
}
