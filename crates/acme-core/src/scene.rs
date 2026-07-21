use crate::{Color, Logical, Point, Radius, Rect};

#[derive(Clone, Debug, PartialEq)]
pub struct PreparedGlyph {
    pub atlas_page: u32,
    pub atlas_rect: [u32; 4],
    pub position: Point<Logical>,
    pub color: Color,
}
#[derive(Clone, Debug, PartialEq)]
pub struct AtlasUpload {
    pub page: u32,
    pub origin: [u32; 2],
    pub size: [u32; 2],
    pub alpha: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum PaintCommand {
    SolidRect {
        rect: Rect<Logical>,
        color: Color,
    },
    RoundedRect {
        rect: Rect<Logical>,
        radius: Radius,
        color: Color,
    },
    Border {
        rect: Rect<Logical>,
        width: f32,
        radius: Radius,
        color: Color,
    },
    Text {
        glyphs: Vec<PreparedGlyph>,
    },
    PushClip(Rect<Logical>),
    PopClip,
}
/// Backend-neutral ordered draw command (logical pixels).
#[derive(Clone, Debug, PartialEq)]
pub enum DrawCommand {
    /// Solid rect with optional rounded corners and border.
    Quad(QuadPrimitive),
    /// Text run with pre-rasterized glyphs and atlas uploads.
    Text(TextPrimitive),
    /// Push a rectangular clip region.
    PushClip(Rect<Logical>),
    /// Pop back to previous clip region.
    PopClip,
    /// Begin a composited layer with given parameters.
    BeginLayer(LayerParams),
    /// End the current composited layer.
    EndLayer,
}

/// A quad primitive: solid fill + optional rounded corner + optional border.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuadPrimitive {
    /// Bounding rectangle in logical pixels.
    pub rect: Rect<Logical>,
    /// Fill color.
    pub color: Color,
    /// Corner radius in logical pixels (0.0 = sharp).
    pub radius: f32,
    /// Border width in logical pixels (0.0 = no border).
    pub border_width: f32,
    /// Border color.
    pub border_color: Color,
}

/// A text primitive with pre-rasterized glyphs and atlas data.
#[derive(Clone, Debug, PartialEq)]
pub struct TextPrimitive {
    /// Baseline origin in logical pixels.
    pub origin: Point<Logical>,
    /// Text color.
    pub color: Color,
    /// Positioned glyph draws.
    pub glyphs: Vec<GlyphDraw>,
    /// Atlas upload data needed by this text run.
    pub uploads: Vec<AtlasUpload>,
}

/// A single positioned glyph reference into an atlas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphDraw {
    /// Logical x position.
    pub x: f32,
    /// Logical y position.
    pub y: f32,
    /// Logical width.
    pub width: f32,
    /// Logical height.
    pub height: f32,
    /// Atlas x offset in pixels.
    pub atlas_x: u32,
    /// Atlas y offset in pixels.
    pub atlas_y: u32,
    /// Glyph pixel format.
    pub format: GlyphFormat,
}

/// Glyph atlas pixel format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlyphFormat {
    /// 8-bit alpha mask.
    Alpha8,
    /// 32-bit RGBA color.
    Rgba8,
}

/// Parameters for a composited layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayerParams {
    /// Opacity multiplier (0.0 – 1.0).
    pub opacity: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    commands: Vec<PaintCommand>,
    draw_commands: Vec<DrawCommand>,
}
impl Scene {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&mut self, c: PaintCommand) {
        self.commands.push(c)
    }
    pub fn commands(&self) -> &[PaintCommand] {
        &self.commands
    }
    /// Push a [`DrawCommand`] onto the ordered display list.
    pub fn push_draw(&mut self, cmd: DrawCommand) {
        self.draw_commands.push(cmd);
    }
    /// Return all draw commands in order.
    pub fn draw_commands(&self) -> &[DrawCommand] {
        &self.draw_commands
    }
}
#[derive(Clone, Debug, Default)]
pub struct ClipStack {
    stack: Vec<Rect<Logical>>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipError {
    Underflow,
    Unbalanced,
}
impl ClipStack {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&mut self, rect: Rect<Logical>) -> Option<Rect<Logical>> {
        let clipped = if let Some(p) = self.stack.last() {
            p.intersect(&rect)
        } else {
            Some(rect)
        };
        self.stack
            .push(clipped.unwrap_or_else(|| Rect::new(0.0, 0.0, 0.0, 0.0)));
        clipped
    }
    pub fn pop(&mut self) -> Result<Rect<Logical>, ClipError> {
        self.stack.pop().ok_or(ClipError::Underflow)
    }
    pub fn current(&self) -> Option<Rect<Logical>> {
        self.stack.last().copied()
    }
    pub fn finish(self) -> Result<(), ClipError> {
        if self.stack.is_empty() {
            Ok(())
        } else {
            Err(ClipError::Unbalanced)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_clips_intersect_and_balance() {
        let mut s = ClipStack::new();
        s.push(Rect::new(0., 0., 10., 10.));
        assert_eq!(
            s.push(Rect::new(5., 5., 10., 10.)),
            Some(Rect::new(5., 5., 5., 5.))
        );
        s.pop().unwrap();
        s.pop().unwrap();
        assert_eq!(s.finish(), Ok(()));
    }
    #[test]
    fn clip_underflow_fails() {
        assert_eq!(ClipStack::new().pop(), Err(ClipError::Underflow));
    }
    #[test]
    fn empty_nested_clip_keeps_parent_depth() {
        let mut stack = ClipStack::new();
        stack.push(Rect::new(0.0, 0.0, 2.0, 2.0));
        assert_eq!(stack.push(Rect::new(5.0, 5.0, 1.0, 1.0)), None);
        stack.pop().unwrap();
        assert_eq!(stack.current(), Some(Rect::new(0.0, 0.0, 2.0, 2.0)));
    }
    #[test]
    fn draw_quad_command() {
        let mut scene = Scene::new();
        let quad = QuadPrimitive {
            rect: Rect::new(0.0, 0.0, 100.0, 50.0),
            color: Color::rgba(1.0, 0.0, 0.0, 1.0),
            radius: 4.0,
            border_width: 2.0,
            border_color: Color::rgba(0.0, 0.0, 0.0, 1.0),
        };
        scene.push_draw(DrawCommand::Quad(quad));
        assert_eq!(scene.draw_commands().len(), 1);
        assert_eq!(
            scene.draw_commands()[0],
            DrawCommand::Quad(QuadPrimitive {
                rect: Rect::new(0.0, 0.0, 100.0, 50.0),
                color: Color::rgba(1.0, 0.0, 0.0, 1.0),
                radius: 4.0,
                border_width: 2.0,
                border_color: Color::rgba(0.0, 0.0, 0.0, 1.0),
            })
        );
    }
    #[test]
    fn draw_text_command() {
        let mut scene = Scene::new();
        let text = TextPrimitive {
            origin: Point::new(10.0, 20.0),
            color: Color::rgba(0.0, 0.0, 0.0, 1.0),
            glyphs: vec![GlyphDraw {
                x: 10.0,
                y: 20.0,
                width: 8.0,
                height: 12.0,
                atlas_x: 0,
                atlas_y: 0,
                format: GlyphFormat::Alpha8,
            }],
            uploads: vec![],
        };
        scene.push_draw(DrawCommand::Text(text));
        assert_eq!(scene.draw_commands().len(), 1);
    }
    #[test]
    fn draw_order_preserved() {
        let mut scene = Scene::new();
        scene.push_draw(DrawCommand::PushClip(Rect::new(0.0, 0.0, 400.0, 300.0)));
        scene.push_draw(DrawCommand::Quad(QuadPrimitive {
            rect: Rect::new(10.0, 10.0, 50.0, 50.0),
            color: Color::TRANSPARENT,
            radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }));
        scene.push_draw(DrawCommand::PopClip);
        let cmds = scene.draw_commands();
        assert_eq!(cmds.len(), 3);
        assert!(matches!(cmds[0], DrawCommand::PushClip(_)));
        assert!(matches!(cmds[1], DrawCommand::Quad(_)));
        assert!(matches!(cmds[2], DrawCommand::PopClip));
    }
}
