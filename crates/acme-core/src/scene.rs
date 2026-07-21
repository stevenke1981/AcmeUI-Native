use crate::{Color, Logical, Point, Radius, Rect};

// ---------------------------------------------------------------------------
// Primitives
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct PreparedGlyph {
    pub atlas_page: u32,
    pub atlas_rect: [u32; 4],
    pub position: Point<Logical>,
    pub color: Color,
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

// ---------------------------------------------------------------------------
// AtlasUpload with RGBA support
// ---------------------------------------------------------------------------

/// Pixel data for an atlas page region.
#[derive(Clone, Debug, PartialEq)]
pub struct AtlasUpload {
    /// Atlas page index.
    pub page: u32,
    /// Pixel origin (x, y) within the atlas page.
    pub origin: [u32; 2],
    /// Pixel size (width, height) of the upload region.
    pub size: [u32; 2],
    /// Pixel format of the data.
    pub format: GlyphFormat,
    /// Raw pixel data (size[0] × size[1] × format.bytes_per_pixel() bytes).
    pub pixels: Vec<u8>,
}

// ---------------------------------------------------------------------------
// PaintCommand — legacy, maps onto DrawCommand
// ---------------------------------------------------------------------------

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

impl PaintCommand {
    /// Convert this legacy paint command into one or more [`DrawCommand`]s.
    pub fn into_draw_commands(self) -> Vec<DrawCommand> {
        match self {
            PaintCommand::SolidRect { rect, color } => {
                vec![DrawCommand::Quad(QuadPrimitive {
                    rect,
                    color,
                    radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                })]
            }
            PaintCommand::RoundedRect {
                rect,
                radius,
                color,
            } => {
                vec![DrawCommand::Quad(QuadPrimitive {
                    rect,
                    color,
                    radius: radius_to_f32(radius),
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                })]
            }
            PaintCommand::Border {
                rect,
                width,
                radius,
                color,
            } => vec![DrawCommand::Quad(QuadPrimitive {
                rect,
                color: Color::TRANSPARENT,
                radius: radius_to_f32(radius),
                border_width: width,
                border_color: color,
            })],
            PaintCommand::Text { glyphs: _ } => {
                // Text PaintCommand loses glyph detail; pushed as an empty text primitive.
                vec![DrawCommand::Text(TextPrimitive {
                    origin: Point::new(0.0, 0.0),
                    color: Color::rgba(0.0, 0.0, 0.0, 1.0),
                    glyphs: vec![],
                    uploads: vec![],
                })]
            }
            PaintCommand::PushClip(r) => vec![DrawCommand::PushClip(r)],
            PaintCommand::PopClip => vec![DrawCommand::PopClip],
        }
    }
}

fn radius_to_f32(r: Radius) -> f32 {
    r.0
}

// ---------------------------------------------------------------------------
// Scene — single-vector draw-command list with clear colour
// ---------------------------------------------------------------------------

/// Error type for scene validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneError {
    /// More `PopClip` than `PushClip` encountered.
    ClipUnderflow,
    /// Unmatched `PushClip` commands remain at end of scene.
    ClipUnbalanced,
}

/// An ordered display list with a clear colour.
///
/// `Scene` stores a single `Vec<DrawCommand>` and a clear-colour.
/// Use [`push`](Scene::push) to append [`DrawCommand`] values directly.
/// The deprecated [`push_paint`](Scene::push_paint) helper converts legacy
/// [`PaintCommand`] values.
#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    /// Background clear colour.
    pub clear: Color,
    commands: Vec<DrawCommand>,
}

impl Scene {
    /// Create an empty scene with a transparent clear colour.
    pub fn new() -> Self {
        Self {
            clear: Color::TRANSPARENT,
            commands: Vec::new(),
        }
    }

    /// Create an empty scene with the given clear colour.
    pub fn with_clear(clear: Color) -> Self {
        Self {
            clear,
            commands: Vec::new(),
        }
    }

    /// The scene's background clear colour.
    pub fn clear_color(&self) -> Color {
        self.clear
    }

    /// Push a [`DrawCommand`] onto the ordered display list.
    pub fn push(&mut self, cmd: DrawCommand) {
        self.commands.push(cmd);
    }

    /// Return all draw commands in order.
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// Deprecated: push a [`PaintCommand`] (converts to [`DrawCommand`] internally).
    #[deprecated(note = "use push(DrawCommand) instead")]
    pub fn push_paint(&mut self, cmd: PaintCommand) {
        self.commands.extend(cmd.into_draw_commands());
    }

    /// Validate the scene's clip stack is balanced.
    ///
    /// Returns `Ok(())` if every `PushClip` has a matching `PopClip` and no
    /// `PopClip` appears without a preceding `PushClip`.
    pub fn validate(&self) -> Result<(), SceneError> {
        let mut clips = 0usize;
        for cmd in &self.commands {
            match cmd {
                DrawCommand::PushClip(_) => clips += 1,
                DrawCommand::PopClip => {
                    clips = clips.checked_sub(1).ok_or(SceneError::ClipUnderflow)?;
                }
                _ => {}
            }
        }
        if clips != 0 {
            return Err(SceneError::ClipUnbalanced);
        }
        Ok(())
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
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
// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- ClipStack tests (unchanged) ---

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

    // --- Scene tests ---

    #[test]
    fn scene_default_clear_is_transparent() {
        let s = Scene::new();
        assert_eq!(s.clear_color(), Color::TRANSPARENT);
    }

    #[test]
    fn scene_with_clear_sets_clear() {
        let c = Color::rgba(0.1, 0.2, 0.3, 1.0);
        let s = Scene::with_clear(c);
        assert_eq!(s.clear_color(), c);
    }

    #[test]
    fn push_and_commands_roundtrip() {
        let mut scene = Scene::new();
        scene.push(DrawCommand::Quad(QuadPrimitive {
            rect: Rect::new(0.0, 0.0, 100.0, 50.0),
            color: Color::rgba(1.0, 0.0, 0.0, 1.0),
            radius: 4.0,
            border_width: 2.0,
            border_color: Color::rgba(0.0, 0.0, 0.0, 1.0),
        }));
        assert_eq!(scene.commands().len(), 1);
        assert_eq!(
            scene.commands()[0],
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
        scene.push(DrawCommand::Text(text));
        assert_eq!(scene.commands().len(), 1);
    }

    #[test]
    fn draw_order_preserved() {
        let mut scene = Scene::new();
        scene.push(DrawCommand::PushClip(Rect::new(0.0, 0.0, 400.0, 300.0)));
        scene.push(DrawCommand::Quad(QuadPrimitive {
            rect: Rect::new(10.0, 10.0, 50.0, 50.0),
            color: Color::TRANSPARENT,
            radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }));
        scene.push(DrawCommand::PopClip);
        let cmds = scene.commands();
        assert_eq!(cmds.len(), 3);
        assert!(matches!(cmds[0], DrawCommand::PushClip(_)));
        assert!(matches!(cmds[1], DrawCommand::Quad(_)));
        assert!(matches!(cmds[2], DrawCommand::PopClip));
    }

    // --- Scene validation tests ---

    #[test]
    fn validate_balanced_ok() {
        let mut scene = Scene::new();
        scene.push(DrawCommand::PushClip(Rect::new(0.0, 0.0, 100.0, 100.0)));
        scene.push(DrawCommand::Quad(QuadPrimitive {
            rect: Rect::new(10.0, 10.0, 50.0, 50.0),
            color: Color::rgba(1.0, 0.0, 0.0, 1.0),
            radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }));
        scene.push(DrawCommand::PopClip);
        assert_eq!(scene.validate(), Ok(()));
    }

    #[test]
    fn validate_clip_underflow_fails() {
        let mut scene = Scene::new();
        scene.push(DrawCommand::PopClip);
        assert_eq!(scene.validate(), Err(SceneError::ClipUnderflow));
    }

    #[test]
    fn validate_clip_unbalanced_fails() {
        let mut scene = Scene::new();
        scene.push(DrawCommand::PushClip(Rect::new(0.0, 0.0, 100.0, 100.0)));
        // No matching PopClip
        assert_eq!(scene.validate(), Err(SceneError::ClipUnbalanced));
    }

    #[test]
    fn validate_empty_scene_ok() {
        let scene = Scene::new();
        assert_eq!(scene.validate(), Ok(()));
    }

    // --- PaintCommand → DrawCommand conversion ---

    #[test]
    fn paint_solid_rect_converts_to_quad() {
        let cmd = PaintCommand::SolidRect {
            rect: Rect::new(0.0, 0.0, 50.0, 50.0),
            color: Color::rgba(0.5, 0.5, 0.5, 1.0),
        };
        let draw_cmds = cmd.into_draw_commands();
        assert_eq!(draw_cmds.len(), 1);
        assert_eq!(
            draw_cmds[0],
            DrawCommand::Quad(QuadPrimitive {
                rect: Rect::new(0.0, 0.0, 50.0, 50.0),
                color: Color::rgba(0.5, 0.5, 0.5, 1.0),
                radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            })
        );
    }

    #[test]
    fn paint_clip_converts_directly() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        let push = PaintCommand::PushClip(r).into_draw_commands();
        let pop = PaintCommand::PopClip.into_draw_commands();
        assert_eq!(push, vec![DrawCommand::PushClip(r)]);
        assert_eq!(pop, vec![DrawCommand::PopClip]);
    }

    // --- AtlasUpload RGBA support ---

    #[test]
    fn atlas_upload_rgba_roundtrip() {
        let pixels: Vec<u8> = vec![
            255, 0, 0, 255, // red pixel
            0, 255, 0, 255, // green pixel
            0, 0, 255, 255, // blue pixel
            255, 255, 0, 255, // yellow pixel
        ];
        let upload = AtlasUpload {
            page: 0,
            origin: [0, 0],
            size: [2, 2],
            format: GlyphFormat::Rgba8,
            pixels: pixels.clone(),
        };
        assert_eq!(upload.pixels.len(), 16);
        assert_eq!(upload.pixels, pixels);
    }
}
