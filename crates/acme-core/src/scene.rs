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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    commands: Vec<PaintCommand>,
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
}
