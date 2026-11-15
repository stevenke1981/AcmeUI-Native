//! Immediate-mode 2D painter building balanced [`Scene`] display lists.

use crate::{
    Color, DrawCommand, ImageFit, ImagePrimitive, LayerParams, Logical, Point, QuadPrimitive, Rect,
    Rgba8Image, Scene, SceneError, TextPrimitive,
};

/// Maximum number of segments emitted by a single diagonal [`Painter::line`] or
/// [`Painter::arc_segments`] call.
const MAX_LINE_SEGMENTS: usize = 512;

/// Axis-alignment epsilon in logical pixels for fast exact quad lines.
const AXIS_EPSILON: f32 = 1e-4;

/// Alias matching immediate-mode image fit terminology.
pub type ImageScaleMode = ImageFit;

/// Visual style parameters for [`Painter::rect`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectStyle {
    /// Interior fill color.
    pub fill: Color,
    /// Corner radius in logical pixels (`0.0` = sharp corners).
    pub radius: f32,
    /// Border thickness in logical pixels (`0.0` = no border).
    pub border_width: f32,
    /// Border stroke color.
    pub border_color: Color,
}

impl Default for RectStyle {
    fn default() -> Self {
        Self {
            fill: Color::TRANSPARENT,
            radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

impl RectStyle {
    /// Create a sharp, borderless style with the given fill color.
    pub const fn solid(color: Color) -> Self {
        Self {
            fill: color,
            radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }

    /// Set the corner radius in logical pixels (non-finite or negative values become `0.0`).
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = sanitize_non_neg(radius);
        self
    }

    /// Set the border width and color (non-finite or negative widths become `0.0`).
    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.border_width = sanitize_non_neg(width);
        self.border_color = color;
        self
    }
}

enum SceneTarget<'a> {
    Owned(Scene),
    Borrowed(&'a mut Scene),
}

impl SceneTarget<'_> {
    fn as_ref(&self) -> &Scene {
        match self {
            Self::Owned(scene) => scene,
            Self::Borrowed(scene) => scene,
        }
    }

    fn as_mut(&mut self) -> &mut Scene {
        match self {
            Self::Owned(scene) => scene,
            Self::Borrowed(scene) => scene,
        }
    }
}

/// Immediate-mode painter emitting [`DrawCommand`]s into an owned or borrowed [`Scene`].
///
/// Tracks nested translation offsets, intersected clip regions, and composited
/// opacity layers, guaranteeing symmetric `PushClip`/`PopClip` and
/// `BeginLayer`/`EndLayer` pairs on [`Painter::flush`], [`Painter::finish`],
/// and [`Drop`].
pub struct Painter<'a> {
    target: SceneTarget<'a>,
    offset_stack: Vec<(f32, f32)>,
    clip_stack: Vec<Rect<Logical>>,
    clip_depth: usize,
    layer_depth: usize,
}

impl Default for Painter<'static> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Painter<'a> {
    /// Create a painter owning an empty [`Scene`] with a transparent clear color.
    pub fn new() -> Self {
        Self::from_scene(Scene::new())
    }

    /// Create a painter owning an empty [`Scene`] with the given clear color.
    pub fn with_clear(clear: Color) -> Self {
        Self::from_scene(Scene::with_clear(clear))
    }

    /// Create a painter that appends into an owned [`Scene`].
    pub fn from_scene(scene: Scene) -> Self {
        Self {
            target: SceneTarget::Owned(scene),
            offset_stack: Vec::new(),
            clip_stack: Vec::new(),
            clip_depth: 0,
            layer_depth: 0,
        }
    }

    /// Create a painter borrowing an existing [`Scene`] mutably.
    pub fn on(scene: &'a mut Scene) -> Self {
        Self {
            target: SceneTarget::Borrowed(scene),
            offset_stack: Vec::new(),
            clip_stack: Vec::new(),
            clip_depth: 0,
            layer_depth: 0,
        }
    }

    /// Borrow the underlying [`Scene`].
    pub fn scene(&self) -> &Scene {
        self.target.as_ref()
    }

    /// Mutably borrow the underlying [`Scene`].
    pub fn scene_mut(&mut self) -> &mut Scene {
        self.target.as_mut()
    }

    /// Current active translation offset depth.
    pub fn offset_depth(&self) -> usize {
        self.offset_stack.len()
    }

    /// Current active clip stack depth pushed by this painter.
    pub fn clip_depth(&self) -> usize {
        self.clip_depth
    }

    /// Current active layer stack depth pushed by this painter.
    pub fn layer_depth(&self) -> usize {
        self.layer_depth
    }

    // -----------------------------------------------------------------------
    // Scoped state stacks
    // -----------------------------------------------------------------------

    /// Push a relative translation offset `(dx, dy)` in logical pixels.
    ///
    /// Non-finite components are sanitized to `0.0`.
    pub fn push_offset(&mut self, dx: f32, dy: f32) {
        let (cx, cy) = self.current_offset();
        let nx = sanitize_finite(cx + sanitize_finite(dx));
        let ny = sanitize_finite(cy + sanitize_finite(dy));
        self.offset_stack.push((nx, ny));
    }

    /// Pop the most recent translation offset, if any.
    pub fn pop_offset(&mut self) {
        self.offset_stack.pop();
    }

    /// Return the current cumulative translation offset `(x, y)`.
    pub fn current_offset(&self) -> (f32, f32) {
        self.offset_stack.last().copied().unwrap_or((0.0, 0.0))
    }

    /// Execute `f` with a temporary relative translation offset `(dx, dy)`.
    pub fn with_offset<R>(&mut self, dx: f32, dy: f32, f: impl FnOnce(&mut Self) -> R) -> R {
        struct OffsetGuard<'p, 'a> {
            painter: &'p mut Painter<'a>,
        }
        impl Drop for OffsetGuard<'_, '_> {
            fn drop(&mut self) {
                self.painter.pop_offset();
            }
        }
        self.push_offset(dx, dy);
        let guard = OffsetGuard { painter: self };
        f(guard.painter)
    }

    /// Push a clip rectangle in local (un-offset) coordinates.
    ///
    /// The rectangle is translated by [`Self::current_offset`], sanitized, and
    /// intersected with the current top clip. If the intersection is empty or
    /// disjoint, a zero-sized `Rect::new(0.0, 0.0, 0.0, 0.0)` clip is still
    /// pushed and emitted so `PushClip`/`PopClip` depth always stays symmetric.
    pub fn push_clip(&mut self, rect: Rect<Logical>) {
        let translated = self.offset_rect(rect);
        let non_empty_translated = if has_positive_area(translated) {
            Some(translated)
        } else {
            None
        };
        let effective = match (self.clip_stack.last(), non_empty_translated) {
            (Some(top), Some(r)) => top
                .intersect(&r)
                .unwrap_or_else(|| Rect::new(0.0, 0.0, 0.0, 0.0)),
            (None, Some(r)) => r,
            (_, None) => Rect::new(0.0, 0.0, 0.0, 0.0),
        };
        self.clip_stack.push(effective);
        self.clip_depth += 1;
        self.target.as_mut().push(DrawCommand::PushClip(effective));
    }

    /// Pop the most recent clip rectangle pushed by this painter, if any.
    pub fn pop_clip(&mut self) {
        if self.clip_depth > 0 {
            self.clip_stack.pop();
            self.clip_depth -= 1;
            self.target.as_mut().push(DrawCommand::PopClip);
        }
    }

    /// Return the current effective clip rectangle in world logical coordinates.
    pub fn current_clip(&self) -> Option<Rect<Logical>> {
        self.clip_stack.last().copied()
    }

    /// Execute `f` within a temporary clip rectangle in local coordinates.
    pub fn with_clip<R>(&mut self, rect: Rect<Logical>, f: impl FnOnce(&mut Self) -> R) -> R {
        struct ClipGuard<'p, 'a> {
            painter: &'p mut Painter<'a>,
        }
        impl Drop for ClipGuard<'_, '_> {
            fn drop(&mut self) {
                self.painter.pop_clip();
            }
        }
        self.push_clip(rect);
        let guard = ClipGuard { painter: self };
        f(guard.painter)
    }

    /// Return `true` if `rect` (in local un-offset coordinates) has positive area
    /// and intersects the current active clip, if any.
    pub fn is_visible(&self, rect: Rect<Logical>) -> bool {
        let translated = self.offset_rect(rect);
        self.is_translated_visible(translated)
    }

    /// Begin a composited opacity layer (`opacity` clamped to `[0.0, 1.0]`).
    pub fn push_layer(&mut self, opacity: f32) {
        self.push_layer_params(LayerParams { opacity });
    }

    /// Begin a composited layer with [`LayerParams`].
    pub fn push_layer_params(&mut self, params: LayerParams) {
        let clean = LayerParams {
            opacity: sanitize_opacity(params.opacity),
        };
        self.layer_depth += 1;
        self.target.as_mut().push(DrawCommand::BeginLayer(clean));
    }

    /// End the most recent composited layer pushed by this painter, if any.
    pub fn pop_layer(&mut self) {
        if self.layer_depth > 0 {
            self.layer_depth -= 1;
            self.target.as_mut().push(DrawCommand::EndLayer);
        }
    }

    /// Execute `f` inside a temporary opacity layer.
    pub fn with_layer<R>(&mut self, opacity: f32, f: impl FnOnce(&mut Self) -> R) -> R {
        self.with_layer_params(LayerParams { opacity }, f)
    }

    /// Execute `f` inside a temporary composited layer with [`LayerParams`].
    pub fn with_layer_params<R>(
        &mut self,
        params: LayerParams,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        struct LayerGuard<'p, 'a> {
            painter: &'p mut Painter<'a>,
        }
        impl Drop for LayerGuard<'_, '_> {
            fn drop(&mut self) {
                self.painter.pop_layer();
            }
        }
        self.push_layer_params(params);
        let guard = LayerGuard { painter: self };
        f(guard.painter)
    }

    /// Balance any unpopped clips and layers opened by this painter and clear
    /// its translation offset stack.
    pub fn flush(&mut self) {
        while self.clip_depth > 0 {
            self.pop_clip();
        }
        while self.layer_depth > 0 {
            self.pop_layer();
        }
        self.offset_stack.clear();
    }

    /// Flush all open state stacks and return the resulting [`Scene`].
    ///
    /// If constructed with [`Painter::on`], the borrowed `Scene` remains balanced
    /// and populated, and a clone of it is returned.
    pub fn finish(mut self) -> Scene {
        self.flush();
        match &mut self.target {
            SceneTarget::Owned(scene) => std::mem::take(scene),
            SceneTarget::Borrowed(scene) => (*scene).clone(),
        }
    }

    /// Flush all open state stacks, validate clip/layer balance, and return the [`Scene`].
    pub fn finish_checked(self) -> Result<Scene, SceneError> {
        let scene = self.finish();
        scene.validate()?;
        Ok(scene)
    }

    // -----------------------------------------------------------------------
    // Drawing primitives
    // -----------------------------------------------------------------------

    /// Draw a solid axis-aligned rectangle.
    pub fn fill_rect(&mut self, rect: Rect<Logical>, color: Color) {
        self.rect(rect, RectStyle::solid(color));
    }

    /// Draw an outlined axis-aligned rectangle with a transparent interior.
    pub fn stroke_rect(&mut self, rect: Rect<Logical>, border_width: f32, border_color: Color) {
        let border_width = sanitize_non_neg(border_width);
        if border_width <= 0.0 {
            return;
        }
        self.rect(
            rect,
            RectStyle {
                fill: Color::TRANSPARENT,
                radius: 0.0,
                border_width,
                border_color,
            },
        );
    }

    /// Draw a rectangle with full [`RectStyle`] (fill, corner radius, and border).
    pub fn rect(&mut self, rect: Rect<Logical>, style: RectStyle) {
        let translated = self.offset_rect(rect);
        if !self.is_translated_visible(translated) {
            return;
        }
        self.target.as_mut().push(DrawCommand::Quad(QuadPrimitive {
            rect: translated,
            color: style.fill,
            radius: sanitize_non_neg(style.radius),
            border_width: sanitize_non_neg(style.border_width),
            border_color: style.border_color,
        }));
    }

    /// Draw a filled rounded rectangle.
    pub fn rounded_rect(&mut self, rect: Rect<Logical>, radius: f32, color: Color) {
        self.rect(rect, RectStyle::solid(color).radius(radius));
    }

    /// Draw a filled rounded rectangle with an inset border.
    pub fn rounded_rect_bordered(
        &mut self,
        rect: Rect<Logical>,
        radius: f32,
        fill: Color,
        border_width: f32,
        border_color: Color,
    ) {
        self.rect(
            rect,
            RectStyle::solid(fill)
                .radius(radius)
                .border(border_width, border_color),
        );
    }

    /// Draw a filled circle centered at `center` with the given `radius`.
    pub fn circle(&mut self, center: Point<Logical>, radius: f32, color: Color) {
        let r = sanitize_non_neg(radius);
        if r <= 0.0 {
            return;
        }
        let cx = sanitize_finite(center.x.get());
        let cy = sanitize_finite(center.y.get());
        let bounds = Rect::new(cx - r, cy - r, r * 2.0, r * 2.0);
        self.rounded_rect(bounds, r, color);
    }

    /// Draw a filled circle with a circular inset stroke.
    pub fn circle_outline(
        &mut self,
        center: Point<Logical>,
        radius: f32,
        stroke_width: f32,
        fill: Color,
        stroke_color: Color,
    ) {
        let r = sanitize_non_neg(radius);
        if r <= 0.0 {
            return;
        }
        let sw = sanitize_non_neg(stroke_width);
        let cx = sanitize_finite(center.x.get());
        let cy = sanitize_finite(center.y.get());
        let bounds = Rect::new(cx - r, cy - r, r * 2.0, r * 2.0);
        self.rounded_rect_bordered(bounds, r, fill, sw, stroke_color);
    }

    /// Draw an unfilled circular ring with outer radius `outer_radius` and stroke `thickness`.
    pub fn ring(
        &mut self,
        center: Point<Logical>,
        outer_radius: f32,
        thickness: f32,
        color: Color,
    ) {
        let r = sanitize_non_neg(outer_radius);
        let t = sanitize_non_neg(thickness);
        if r <= 0.0 || t <= 0.0 {
            return;
        }
        self.circle_outline(center, r, t.min(r), Color::TRANSPARENT, color);
    }

    /// Draw a line segment from `from` to `to` with the given `thickness`.
    ///
    /// Horizontal and vertical lines emit a single exact [`QuadPrimitive`].
    /// Diagonal lines emit deterministic circle segments capped at `<= 512`
    /// segments so CPU and wgpu backends render identically without shader
    /// changes.
    pub fn line(&mut self, from: Point<Logical>, to: Point<Logical>, thickness: f32, color: Color) {
        let t = sanitize_non_neg(thickness);
        if t <= 0.0 {
            return;
        }
        let x0 = sanitize_finite(from.x.get());
        let y0 = sanitize_finite(from.y.get());
        let x1 = sanitize_finite(to.x.get());
        let y1 = sanitize_finite(to.y.get());
        let dx = x1 - x0;
        let dy = y1 - y0;
        let half = t * 0.5;

        if dx.abs() <= AXIS_EPSILON && dy.abs() <= AXIS_EPSILON {
            return;
        }

        if dy.abs() <= AXIS_EPSILON {
            let min_x = x0.min(x1);
            let mid_y = (y0 + y1) * 0.5;
            self.fill_rect(Rect::new(min_x, mid_y - half, dx.abs(), t), color);
            return;
        }

        if dx.abs() <= AXIS_EPSILON {
            let mid_x = (x0 + x1) * 0.5;
            let min_y = y0.min(y1);
            self.fill_rect(Rect::new(mid_x - half, min_y, t, dy.abs()), color);
            return;
        }

        let len = dx.hypot(dy);
        if !len.is_finite() || len <= AXIS_EPSILON {
            return;
        }

        // Bounding-box cull before emitting diagonal circle segments.
        let bounds = Rect::new(
            x0.min(x1) - half,
            y0.min(y1) - half,
            dx.abs() + t,
            dy.abs() + t,
        );
        if !self.is_visible(bounds) {
            return;
        }

        let step = (t * 0.5).max(0.5);
        let raw_count = (len / step).ceil();
        let count = if raw_count.is_finite() && raw_count >= 1.0 {
            ((raw_count as usize) + 1).clamp(2, MAX_LINE_SEGMENTS)
        } else {
            MAX_LINE_SEGMENTS
        };
        let denom = (count - 1) as f32;
        for i in 0..count {
            let u = i as f32 / denom;
            let cx = x0 + dx * u;
            let cy = y0 + dy * u;
            self.circle(Point::new(cx, cy), half, color);
        }
    }

    /// Draw connected line segments through `points`.
    pub fn polyline(&mut self, points: &[Point<Logical>], thickness: f32, color: Color) {
        if points.len() < 2 || sanitize_non_neg(thickness) <= 0.0 {
            return;
        }
        for seg in points.windows(2) {
            self.line(seg[0], seg[1], thickness, color);
        }
    }

    /// Draw a closed polygon outline connecting `points` back to `points[0]`.
    pub fn polygon_outline(&mut self, points: &[Point<Logical>], thickness: f32, color: Color) {
        if points.len() < 2 || sanitize_non_neg(thickness) <= 0.0 {
            return;
        }
        self.polyline(points, thickness, color);
        if points.len() >= 3 {
            self.line(points[points.len() - 1], points[0], thickness, color);
        }
    }

    /// Draw an arc approximated by `steps` chord segments (capped at `<= 512` steps).
    #[allow(clippy::too_many_arguments)]
    pub fn arc_segments(
        &mut self,
        center: Point<Logical>,
        radius: f32,
        start_angle_rad: f32,
        sweep_angle_rad: f32,
        thickness: f32,
        color: Color,
        steps: usize,
    ) {
        let r = sanitize_non_neg(radius);
        let t = sanitize_non_neg(thickness);
        if r <= 0.0
            || t <= 0.0
            || steps == 0
            || !start_angle_rad.is_finite()
            || !sweep_angle_rad.is_finite()
            || sweep_angle_rad.abs() <= 1e-6
        {
            return;
        }

        let cx = sanitize_finite(center.x.get());
        let cy = sanitize_finite(center.y.get());
        let outer = r + t * 0.5;
        let bounds = Rect::new(cx - outer, cy - outer, outer * 2.0, outer * 2.0);
        if !self.is_visible(bounds) {
            return;
        }

        let steps = steps.min(MAX_LINE_SEGMENTS);
        let mut prev = Point::new(
            cx + r * start_angle_rad.cos(),
            cy + r * start_angle_rad.sin(),
        );
        for i in 1..=steps {
            let angle = start_angle_rad + sweep_angle_rad * (i as f32 / steps as f32);
            let next = Point::new(cx + r * angle.cos(), cy + r * angle.sin());
            self.line(prev, next, t, color);
            prev = next;
        }
    }

    /// Draw an RGBA8 image fitted with [`ImageFit::Contain`] inside `rect`.
    pub fn image(&mut self, rect: Rect<Logical>, image: Rgba8Image) {
        self.image_fit(rect, image, ImageFit::Contain);
    }

    /// Draw an RGBA8 image inside `rect` using the specified [`ImageFit`].
    pub fn image_fit(&mut self, rect: Rect<Logical>, image: Rgba8Image, fit: ImageFit) {
        let translated = self.offset_rect(rect);
        if !self.is_translated_visible(translated) {
            return;
        }
        self.target
            .as_mut()
            .push(DrawCommand::Image(ImagePrimitive {
                image,
                rect: translated,
                fit,
            }));
    }

    /// Offset `text.origin` by [`Self::current_offset`] and emit a [`DrawCommand::Text`]
    /// if the primitive contains glyphs or atlas uploads.
    pub fn text_primitive(&mut self, mut text: TextPrimitive) {
        if text.glyphs.is_empty() && text.uploads.is_empty() {
            return;
        }
        let (dx, dy) = self.current_offset();
        text.origin = Point::new(
            sanitize_finite(text.origin.x.get() + dx),
            sanitize_finite(text.origin.y.get() + dy),
        );
        self.target.as_mut().push(DrawCommand::Text(text));
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn offset_rect(&self, rect: Rect<Logical>) -> Rect<Logical> {
        let (dx, dy) = self.current_offset();
        let x = sanitize_finite(sanitize_finite(rect.origin.x.get()) + dx);
        let y = sanitize_finite(sanitize_finite(rect.origin.y.get()) + dy);
        let w = sanitize_non_neg(rect.size.width.get());
        let h = sanitize_non_neg(rect.size.height.get());
        Rect::new(x, y, w, h)
    }

    fn is_translated_visible(&self, translated: Rect<Logical>) -> bool {
        if !has_positive_area(translated) {
            return false;
        }
        match self.clip_stack.last() {
            Some(clip) => clip.intersect(&translated).is_some(),
            None => true,
        }
    }
}

impl Drop for Painter<'_> {
    fn drop(&mut self) {
        self.flush();
    }
}

fn sanitize_finite(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

fn sanitize_non_neg(value: f32) -> f32 {
    sanitize_finite(value).max(0.0)
}

fn sanitize_opacity(value: f32) -> f32 {
    sanitize_finite(value).clamp(0.0, 1.0)
}

fn has_positive_area(rect: Rect<Logical>) -> bool {
    rect.size.width.get() > 0.0 && rect.size.height.get() > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AtlasUpload, GlyphDraw, GlyphFormat, ImageKey};

    const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    fn sample_image() -> Rgba8Image {
        Rgba8Image::new(ImageKey::new(7), 1, 2, 2, vec![255; 16]).unwrap()
    }

    #[test]
    fn rect_style_builders_sanitize_values() {
        let style = RectStyle::solid(RED).radius(-5.0).border(f32::NAN, BLUE);
        assert_eq!(style.fill, RED);
        assert_eq!(style.radius, 0.0);
        assert_eq!(style.border_width, 0.0);
        assert_eq!(style.border_color, BLUE);

        let valid = RectStyle::default().radius(6.0).border(2.0, BLUE);
        assert_eq!(valid.fill, Color::TRANSPARENT);
        assert_eq!(valid.radius, 6.0);
        assert_eq!(valid.border_width, 2.0);
    }

    #[test]
    fn offsets_accumulate_pop_and_sanitize_non_finite() {
        let mut p = Painter::new();
        assert_eq!(p.current_offset(), (0.0, 0.0));
        p.push_offset(10.0, 20.0);
        p.push_offset(f32::NAN, f32::INFINITY);
        assert_eq!(p.current_offset(), (10.0, 20.0));

        p.with_offset(-3.0, 5.0, |p| {
            assert_eq!(p.current_offset(), (7.0, 25.0));
            p.fill_rect(Rect::new(1.0, 2.0, 10.0, 10.0), RED);
        });
        assert_eq!(p.current_offset(), (10.0, 20.0));

        p.pop_offset();
        p.pop_offset();
        p.pop_offset(); // safe underflow
        assert_eq!(p.current_offset(), (0.0, 0.0));

        let scene = p.finish_checked().unwrap();
        assert_eq!(
            scene.commands(),
            &[DrawCommand::Quad(QuadPrimitive {
                rect: Rect::new(8.0, 27.0, 10.0, 10.0),
                color: RED,
                radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            })]
        );
    }

    #[test]
    fn nested_clip_intersections_respect_offsets_and_stay_symmetric() {
        let mut p = Painter::new();
        p.push_offset(10.0, 10.0);
        p.push_clip(Rect::new(0.0, 0.0, 100.0, 100.0));
        assert_eq!(p.current_clip(), Some(Rect::new(10.0, 10.0, 100.0, 100.0)));

        p.with_clip(Rect::new(50.0, 50.0, 80.0, 80.0), |p| {
            // World: [60, 140) x [60, 140) intersected with [10, 110) x [10, 110)
            assert_eq!(p.current_clip(), Some(Rect::new(60.0, 60.0, 50.0, 50.0)));
            p.fill_rect(Rect::new(55.0, 55.0, 10.0, 10.0), RED);
        });
        assert_eq!(p.current_clip(), Some(Rect::new(10.0, 10.0, 100.0, 100.0)));
        p.pop_clip();
        assert_eq!(p.current_clip(), None);

        let scene = p.finish_checked().unwrap();
        assert!(matches!(
            scene.commands(),
            [
                DrawCommand::PushClip(_),
                DrawCommand::PushClip(_),
                DrawCommand::Quad(_),
                DrawCommand::PopClip,
                DrawCommand::PopClip,
            ]
        ));
    }

    #[test]
    fn disjoint_and_negative_clips_push_zero_clip_and_cull_children() {
        let mut p = Painter::new();
        p.push_clip(Rect::new(0.0, 0.0, 50.0, 50.0));
        p.with_clip(Rect::new(200.0, 200.0, 40.0, 40.0), |p| {
            assert_eq!(p.current_clip(), Some(Rect::new(0.0, 0.0, 0.0, 0.0)));
            assert!(!p.is_visible(Rect::new(10.0, 10.0, 20.0, 20.0)));
            assert!(!p.is_visible(Rect::new(210.0, 210.0, 10.0, 10.0)));
            p.fill_rect(Rect::new(10.0, 10.0, 20.0, 20.0), RED);
        });
        assert_eq!(p.current_clip(), Some(Rect::new(0.0, 0.0, 50.0, 50.0)));

        p.with_clip(Rect::new(0.0, 0.0, -10.0, 20.0), |p| {
            assert_eq!(p.current_clip(), Some(Rect::new(0.0, 0.0, 0.0, 0.0)));
            p.fill_rect(Rect::new(0.0, 0.0, 5.0, 5.0), BLUE);
        });

        let scene = p.finish_checked().unwrap();
        // No Quad commands were emitted inside disjoint/negative clips, and PushClip/PopClip are symmetric.
        assert_eq!(
            scene
                .commands()
                .iter()
                .filter(|c| matches!(c, DrawCommand::Quad(_)))
                .count(),
            0
        );
        assert_eq!(
            scene
                .commands()
                .iter()
                .filter(|c| matches!(c, DrawCommand::PushClip(_)))
                .count(),
            3
        );
        assert_eq!(
            scene
                .commands()
                .iter()
                .filter(|c| matches!(c, DrawCommand::PopClip))
                .count(),
            3
        );
    }

    #[test]
    fn zero_sized_and_outside_primitives_are_culled() {
        let mut p = Painter::new();
        p.push_clip(Rect::new(10.0, 10.0, 50.0, 50.0));
        p.fill_rect(Rect::new(10.0, 10.0, 0.0, 10.0), RED);
        p.fill_rect(Rect::new(10.0, 10.0, -5.0, 10.0), RED);
        p.fill_rect(Rect::new(60.0, 10.0, 10.0, 10.0), RED); // touching right edge, outside half-open
        p.stroke_rect(Rect::new(15.0, 15.0, 10.0, 10.0), 0.0, BLUE);
        p.circle(Point::new(20.0, 20.0), 0.0, RED);
        p.ring(Point::new(20.0, 20.0), 10.0, 0.0, RED);
        p.image(Rect::new(100.0, 100.0, 20.0, 20.0), sample_image());
        p.pop_clip();

        let scene = p.finish_checked().unwrap();
        assert_eq!(scene.commands().len(), 2); // only PushClip + PopClip
    }

    #[test]
    fn opacity_layers_clamp_and_balance() {
        let mut p = Painter::with_clear(BLUE);
        p.with_layer(1.5, |p| {
            p.with_layer_params(LayerParams { opacity: f32::NAN }, |p| {
                p.fill_rect(Rect::new(0.0, 0.0, 10.0, 10.0), RED);
            });
        });

        let scene = p.finish_checked().unwrap();
        assert_eq!(scene.clear_color(), BLUE);
        assert_eq!(
            scene.commands(),
            &[
                DrawCommand::BeginLayer(LayerParams { opacity: 1.0 }),
                DrawCommand::BeginLayer(LayerParams { opacity: 0.0 }),
                DrawCommand::Quad(QuadPrimitive {
                    rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                    color: RED,
                    radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }),
                DrawCommand::EndLayer,
                DrawCommand::EndLayer,
            ]
        );
    }

    #[test]
    fn finish_auto_balances_unpopped_clips_layers_and_offsets() {
        let mut p = Painter::new();
        p.push_offset(10.0, 10.0);
        p.push_clip(Rect::new(0.0, 0.0, 100.0, 100.0));
        p.push_layer(0.5);
        p.fill_rect(Rect::new(5.0, 5.0, 20.0, 20.0), RED);

        let scene = p.finish_checked().unwrap();
        assert_eq!(scene.validate(), Ok(()));
        assert_eq!(scene.commands().len(), 5);
    }

    #[test]
    fn borrowed_painter_on_scene_populates_and_balances_caller_scene() {
        let mut scene = Scene::with_clear(BLUE);
        {
            let mut p = Painter::on(&mut scene);
            p.push_clip(Rect::new(0.0, 0.0, 50.0, 50.0));
            p.push_layer(0.8);
            p.fill_rect(Rect::new(0.0, 0.0, 10.0, 10.0), RED);
            let cloned = p.finish();
            assert_eq!(cloned.validate(), Ok(()));
            assert_eq!(cloned, scene);
        }
        assert_eq!(scene.validate(), Ok(()));
        assert_eq!(scene.commands().len(), 5);
    }

    #[test]
    fn borrowed_painter_drop_flushes_open_stacks() {
        let mut scene = Scene::new();
        {
            let mut p = Painter::on(&mut scene);
            p.push_clip(Rect::new(0.0, 0.0, 40.0, 40.0));
            p.push_layer(0.4);
            p.fill_rect(Rect::new(1.0, 1.0, 5.0, 5.0), RED);
        }
        assert_eq!(scene.validate(), Ok(()));
        assert_eq!(scene.commands().len(), 5);
    }

    #[test]
    fn from_scene_appends_to_existing_commands() {
        let mut base = Scene::new();
        base.push(DrawCommand::Quad(QuadPrimitive {
            rect: Rect::new(0.0, 0.0, 5.0, 5.0),
            color: BLUE,
            radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }));
        let mut p = Painter::from_scene(base);
        p.fill_rect(Rect::new(10.0, 10.0, 5.0, 5.0), RED);
        let out = p.finish_checked().unwrap();
        assert_eq!(out.commands().len(), 2);
    }

    #[test]
    fn rounded_rect_and_bordered_emit_expected_quads() {
        let mut p = Painter::new();
        p.rounded_rect(Rect::new(0.0, 0.0, 40.0, 20.0), 8.0, RED);
        p.rounded_rect_bordered(Rect::new(10.0, 10.0, 50.0, 30.0), 6.0, RED, 2.0, BLUE);
        let scene = p.finish_checked().unwrap();
        assert_eq!(
            scene.commands(),
            &[
                DrawCommand::Quad(QuadPrimitive {
                    rect: Rect::new(0.0, 0.0, 40.0, 20.0),
                    color: RED,
                    radius: 8.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }),
                DrawCommand::Quad(QuadPrimitive {
                    rect: Rect::new(10.0, 10.0, 50.0, 30.0),
                    color: RED,
                    radius: 6.0,
                    border_width: 2.0,
                    border_color: BLUE,
                }),
            ]
        );
    }

    #[test]
    fn circles_outlines_and_rings_emit_circular_quads() {
        let mut p = Painter::new();
        p.circle(Point::new(20.0, 30.0), 10.0, RED);
        p.circle_outline(Point::new(50.0, 50.0), 12.0, 3.0, RED, BLUE);
        p.ring(Point::new(80.0, 80.0), 15.0, 4.0, BLUE);
        p.ring(Point::new(120.0, 80.0), 10.0, 25.0, BLUE); // thickness clamped to outer_radius

        let scene = p.finish_checked().unwrap();
        assert_eq!(
            scene.commands(),
            &[
                DrawCommand::Quad(QuadPrimitive {
                    rect: Rect::new(10.0, 20.0, 20.0, 20.0),
                    color: RED,
                    radius: 10.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }),
                DrawCommand::Quad(QuadPrimitive {
                    rect: Rect::new(38.0, 38.0, 24.0, 24.0),
                    color: RED,
                    radius: 12.0,
                    border_width: 3.0,
                    border_color: BLUE,
                }),
                DrawCommand::Quad(QuadPrimitive {
                    rect: Rect::new(65.0, 65.0, 30.0, 30.0),
                    color: Color::TRANSPARENT,
                    radius: 15.0,
                    border_width: 4.0,
                    border_color: BLUE,
                }),
                DrawCommand::Quad(QuadPrimitive {
                    rect: Rect::new(110.0, 70.0, 20.0, 20.0),
                    color: Color::TRANSPARENT,
                    radius: 10.0,
                    border_width: 10.0,
                    border_color: BLUE,
                }),
            ]
        );
    }

    #[test]
    fn axis_aligned_lines_emit_single_exact_quad() {
        let mut p = Painter::new();
        p.line(Point::new(100.0, 20.0), Point::new(10.0, 20.0), 4.0, RED);
        p.line(Point::new(30.0, 50.0), Point::new(30.0, 10.0), 2.0, BLUE);
        let scene = p.finish_checked().unwrap();
        assert_eq!(
            scene.commands(),
            &[
                DrawCommand::Quad(QuadPrimitive {
                    rect: Rect::new(10.0, 18.0, 90.0, 4.0),
                    color: RED,
                    radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }),
                DrawCommand::Quad(QuadPrimitive {
                    rect: Rect::new(29.0, 10.0, 2.0, 40.0),
                    color: BLUE,
                    radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }),
            ]
        );
    }

    #[test]
    fn diagonal_line_is_bounded_and_culled_when_outside() {
        let mut p = Painter::new();
        p.line(Point::new(0.0, 0.0), Point::new(5000.0, 5000.0), 2.0, RED);
        let count = p.scene().commands().len();
        assert!((2..=MAX_LINE_SEGMENTS).contains(&count));

        let mut culled = Painter::new();
        culled.push_clip(Rect::new(0.0, 0.0, 50.0, 50.0));
        culled.line(
            Point::new(200.0, 200.0),
            Point::new(400.0, 300.0),
            2.0,
            BLUE,
        );
        culled.pop_clip();
        assert_eq!(culled.finish_checked().unwrap().commands().len(), 2);
    }

    #[test]
    fn polyline_and_polygon_outline_connect_segments() {
        let mut p = Painter::new();
        p.polyline(
            &[
                Point::new(0.0, 0.0),
                Point::new(20.0, 0.0),
                Point::new(20.0, 20.0),
            ],
            2.0,
            RED,
        );
        assert_eq!(p.scene().commands().len(), 2);

        p.polygon_outline(
            &[
                Point::new(10.0, 10.0),
                Point::new(30.0, 10.0),
                Point::new(30.0, 30.0),
                Point::new(10.0, 30.0),
            ],
            2.0,
            BLUE,
        );
        // 2 from polyline + 4 closed edges from rectangle polygon_outline
        assert_eq!(p.finish_checked().unwrap().commands().len(), 6);
    }

    #[test]
    fn arc_segments_draws_steps_and_culls_cleanly() {
        let mut p = Painter::new();
        p.arc_segments(
            Point::new(50.0, 50.0),
            20.0,
            0.0,
            std::f32::consts::FRAC_PI_2,
            2.0,
            RED,
            8,
        );
        let drawn = p.finish_checked().unwrap();
        assert!(!drawn.commands().is_empty());

        let mut culled = Painter::new();
        culled.push_clip(Rect::new(0.0, 0.0, 10.0, 10.0));
        culled.arc_segments(
            Point::new(100.0, 100.0),
            10.0,
            0.0,
            std::f32::consts::PI,
            2.0,
            RED,
            16,
        );
        culled.pop_clip();
        assert_eq!(culled.finish_checked().unwrap().commands().len(), 2);
    }

    #[test]
    fn images_and_text_primitives_apply_offsets_and_fit_mode() {
        let img = sample_image();
        let mut p = Painter::new();
        p.push_offset(10.0, 20.0);
        p.image(Rect::new(1.0, 2.0, 30.0, 40.0), img.clone());
        p.image_fit(
            Rect::new(5.0, 6.0, 30.0, 40.0),
            img.clone(),
            ImageScaleMode::Fill,
        );
        p.text_primitive(TextPrimitive {
            origin: Point::new(3.0, 4.0),
            color: RED,
            glyphs: vec![GlyphDraw {
                x: 0.0,
                y: 0.0,
                width: 8.0,
                height: 12.0,
                atlas_x: 0,
                atlas_y: 0,
                atlas_page: 0,
                format: GlyphFormat::Alpha8,
            }],
            uploads: Vec::<AtlasUpload>::new(),
        });
        // Empty text primitive should be ignored
        p.text_primitive(TextPrimitive {
            origin: Point::new(1.0, 1.0),
            color: RED,
            glyphs: vec![],
            uploads: vec![],
        });
        p.pop_offset();

        let scene = p.finish_checked().unwrap();
        assert_eq!(scene.commands().len(), 3);
        assert!(matches!(
            &scene.commands()[0],
            DrawCommand::Image(ImagePrimitive {
                rect,
                fit: ImageFit::Contain,
                ..
            }) if *rect == Rect::new(11.0, 22.0, 30.0, 40.0)
        ));
        assert!(matches!(
            &scene.commands()[1],
            DrawCommand::Image(ImagePrimitive {
                rect,
                fit: ImageFit::Fill,
                ..
            }) if *rect == Rect::new(15.0, 26.0, 30.0, 40.0)
        ));
        assert!(matches!(
            &scene.commands()[2],
            DrawCommand::Text(TextPrimitive { origin, .. })
                if *origin == Point::new(13.0, 24.0)
        ));
    }

    #[test]
    fn finish_checked_reports_preexisting_underflow_on_borrowed_scene() {
        let mut bad = Scene::new();
        bad.push(DrawCommand::PopClip);
        let p = Painter::on(&mut bad);
        assert_eq!(p.finish_checked(), Err(SceneError::ClipUnderflow));
    }

    #[test]
    fn flush_is_idempotent_and_scoped_helpers_return_values() {
        let mut p = Painter::new();
        let val = p.with_offset(5.0, 10.0, |p| {
            p.with_clip(Rect::new(0.0, 0.0, 50.0, 50.0), |p| {
                p.with_layer(0.75, |p| {
                    p.stroke_rect(Rect::new(2.0, 2.0, 20.0, 20.0), 2.0, RED);
                    123usize
                })
            })
        });
        assert_eq!(val, 123);
        assert_eq!(p.offset_depth(), 0);
        assert_eq!(p.clip_depth(), 0);
        assert_eq!(p.layer_depth(), 0);
        p.flush();
        p.flush();
        let scene = p.finish_checked().unwrap();
        assert_eq!(scene.commands().len(), 5);
    }
}
