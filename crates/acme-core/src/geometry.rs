use std::{
    fmt,
    marker::PhantomData,
    ops::{Add, Sub},
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Logical;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Physical;
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Local;

#[derive(Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct Pixels<U> {
    value: f32,
    unit: PhantomData<U>,
}

pub type LogicalPx = Pixels<Logical>;
pub type PhysicalPx = Pixels<Physical>;

impl<U> Pixels<U> {
    pub fn new(value: f32) -> Self {
        Self {
            value: finite(value),
            unit: PhantomData,
        }
    }
    pub fn get(&self) -> f32 {
        self.value
    }
}
impl<U> fmt::Debug for Pixels<U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}
impl<U> Add for Pixels<U> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.value + rhs.value)
    }
}
impl<U> Sub for Pixels<U> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.value - rhs.value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScaleFactor(f32);
impl ScaleFactor {
    pub fn new(value: f32) -> Option<Self> {
        (value.is_finite() && value > 0.0).then_some(Self(value))
    }
    pub fn get(self) -> f32 {
        self.0
    }
    pub fn logical_to_physical(self, value: LogicalPx) -> PhysicalPx {
        PhysicalPx::new(value.get() * self.0)
    }
    pub fn physical_to_logical(self, value: PhysicalPx) -> LogicalPx {
        LogicalPx::new(value.get() / self.0)
    }
    pub fn round_physical(self, value: LogicalPx) -> PhysicalPx {
        PhysicalPx::new((value.get() * self.0).round())
    }
}
impl Default for ScaleFactor {
    fn default() -> Self {
        Self(1.0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point<U> {
    pub x: Pixels<U>,
    pub y: Pixels<U>,
}
impl<U> Point<U> {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x: Pixels::new(x),
            y: Pixels::new(y),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size<U> {
    pub width: Pixels<U>,
    pub height: Pixels<U>,
}
impl<U> Size<U> {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width: Pixels::new(width.max(0.0)),
            height: Pixels::new(height.max(0.0)),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect<U> {
    pub origin: Point<U>,
    pub size: Size<U>,
}
impl<U> Rect<U> {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }
    pub fn contains(&self, point: Point<U>) -> bool {
        point.x.get() >= self.origin.x.get()
            && point.y.get() >= self.origin.y.get()
            && point.x.get() < self.origin.x.get() + self.size.width.get()
            && point.y.get() < self.origin.y.get() + self.size.height.get()
    }
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let x = self.origin.x.get().max(other.origin.x.get());
        let y = self.origin.y.get().max(other.origin.y.get());
        let right = (self.origin.x.get() + self.size.width.get())
            .min(other.origin.x.get() + other.size.width.get());
        let bottom = (self.origin.y.get() + self.size.height.get())
            .min(other.origin.y.get() + other.size.height.get());
        (right > x && bottom > y).then(|| Self::new(x, y, right - x, bottom - y))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets<U> {
    pub top: Pixels<U>,
    pub right: Pixels<U>,
    pub bottom: Pixels<U>,
    pub left: Pixels<U>,
}
impl<U> Insets<U> {
    pub fn all(value: f32) -> Self {
        Self {
            top: Pixels::new(value),
            right: Pixels::new(value),
            bottom: Pixels::new(value),
            left: Pixels::new(value),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Radius(pub f32);
impl Radius {
    pub fn new(value: f32) -> Self {
        Self(finite(value).max(0.0))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
impl Color {
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: channel(r),
            g: channel(g),
            b: channel(b),
            a: channel(a),
        }
    }
}
fn finite(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}
fn channel(value: f32) -> f32 {
    finite(value).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dpi_round_trip() {
        let s = ScaleFactor::new(1.25).unwrap();
        let x = LogicalPx::new(12.0);
        assert_eq!(s.physical_to_logical(s.logical_to_physical(x)), x);
        assert_eq!(s.round_physical(LogicalPx::new(1.0)).get(), 1.0);
    }
    #[test]
    fn invalid_values_normalize() {
        assert!(ScaleFactor::new(0.0).is_none());
        assert_eq!(LogicalPx::new(f32::NAN).get(), 0.0);
        assert_eq!(
            Color::rgba(-1.0, 2.0, f32::NAN, 1.0),
            Color::rgba(0.0, 1.0, 0.0, 1.0)
        );
    }
    #[test]
    fn rectangles_are_half_open_and_intersect() {
        let a = Rect::<Logical>::new(0.0, 0.0, 10.0, 10.0);
        assert!(a.contains(Point::new(9.0, 9.0)));
        assert!(!a.contains(Point::new(10.0, 5.0)));
        assert_eq!(
            a.intersect(&Rect::new(5.0, 5.0, 10.0, 10.0)),
            Some(Rect::new(5.0, 5.0, 5.0, 5.0))
        );
    }
}
