//! Custom immediate-mode drawing surface widget.

use crate::WidgetNode;
use acme_core::{CursorHint, Logical, Painter, Rect, WidgetKey};
use acme_layout::{LayoutKind, LayoutStyle, Length};
use acme_theme::Theme;
use std::fmt;
use std::sync::Arc;

/// Callback signature used by [`Canvas`] to emit draw commands using a semantic [`Theme`].
pub type CanvasPaintFn = dyn Fn(&mut Painter<'_>, Rect<Logical>, &Theme) + Send + Sync;

fn noop_paint(_painter: &mut Painter<'_>, _rect: Rect<Logical>, _theme: &Theme) {}

/// Immediate-mode custom-drawn leaf widget.
pub struct Canvas<M> {
    pub id: Option<WidgetKey>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub layout: LayoutStyle,
    pub paint: Arc<CanvasPaintFn>,
    pub on_click: Option<M>,
    pub accessible_label: Option<String>,
    pub cursor: Option<CursorHint>,
    pub disabled: bool,
}

impl<M: Clone> Clone for Canvas<M> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            width: self.width,
            height: self.height,
            layout: self.layout.clone(),
            paint: Arc::clone(&self.paint),
            on_click: self.on_click.clone(),
            accessible_label: self.accessible_label.clone(),
            cursor: self.cursor,
            disabled: self.disabled,
        }
    }
}

impl<M: fmt::Debug> fmt::Debug for Canvas<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Canvas")
            .field("id", &self.id)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("layout", &self.layout)
            .field("paint", &"<fn>")
            .field("on_click", &self.on_click)
            .field("accessible_label", &self.accessible_label)
            .field("cursor", &self.cursor)
            .field("disabled", &self.disabled)
            .finish()
    }
}

impl<M: PartialEq> PartialEq for Canvas<M> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.width == other.width
            && self.height == other.height
            && self.layout == other.layout
            && Arc::ptr_eq(&self.paint, &other.paint)
            && self.on_click == other.on_click
            && self.accessible_label == other.accessible_label
            && self.cursor == other.cursor
            && self.disabled == other.disabled
    }
}

impl<M> Canvas<M> {
    /// Return the optional widget key assigned to this canvas.
    pub fn key(&self) -> Option<&WidgetKey> {
        self.id.as_ref()
    }

    /// Return the activation message if enabled and clickable.
    pub fn activate(&self) -> Option<&M> {
        if self.disabled {
            None
        } else {
            self.on_click.as_ref()
        }
    }

    /// Invoke the paint callback into `painter` for `rect` using `theme`.
    pub fn paint_into(&self, painter: &mut Painter<'_>, rect: Rect<Logical>, theme: &Theme) {
        (self.paint)(painter, rect, theme);
    }

    /// Compute the effective [`LayoutStyle`] passed into `acme-layout`.
    pub fn effective_layout(&self) -> LayoutStyle {
        let mut style = self.layout.clone();
        style.kind = LayoutKind::Leaf;
        if let Some(w) = self.width
            && matches!(style.width, Length::Auto)
        {
            style.width = Length::px(crate::finite(w));
        }
        if let Some(h) = self.height
            && matches!(style.height, Length::Auto)
        {
            style.height = Length::px(crate::finite(h));
        }
        style
    }
}

/// Builder for constructing a [`Canvas`] widget.
pub struct CanvasBuilder<M> {
    canvas: Canvas<M>,
}

impl<M: Clone> Clone for CanvasBuilder<M> {
    fn clone(&self) -> Self {
        Self {
            canvas: self.canvas.clone(),
        }
    }
}

impl<M: fmt::Debug> fmt::Debug for CanvasBuilder<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CanvasBuilder")
            .field("canvas", &self.canvas)
            .finish()
    }
}

/// Create a new [`CanvasBuilder`].
pub fn canvas<M>() -> CanvasBuilder<M> {
    CanvasBuilder {
        canvas: Canvas {
            id: None,
            width: None,
            height: None,
            layout: LayoutStyle::default(),
            paint: Arc::new(noop_paint),
            on_click: None,
            accessible_label: None,
            cursor: None,
            disabled: false,
        },
    }
}

impl<M> CanvasBuilder<M> {
    /// Assign a reconciliation key to the canvas.
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.canvas.id = Some(key.into());
        self
    }

    /// Set the explicit logical width of the canvas.
    pub fn width(mut self, value: f32) -> Self {
        let clean = crate::finite(value);
        self.canvas.width = Some(clean);
        self.canvas.layout.width = Length::px(clean);
        self
    }

    /// Set the explicit logical height of the canvas.
    pub fn height(mut self, value: f32) -> Self {
        let clean = crate::finite(value);
        self.canvas.height = Some(clean);
        self.canvas.layout.height = Length::px(clean);
        self
    }

    /// Set both explicit logical width and height.
    pub fn size(self, width: f32, height: f32) -> Self {
        self.width(width).height(height)
    }

    /// Override the underlying [`LayoutStyle`].
    pub fn layout(mut self, mut style: LayoutStyle) -> Self {
        style.kind = LayoutKind::Leaf;
        if let Length::Px(w) = style.width {
            self.canvas.width = Some(w);
        }
        if let Length::Px(h) = style.height {
            self.canvas.height = Some(h);
        }
        self.canvas.layout = style;
        self
    }

    /// Provide the immediate-mode paint closure receiving `&mut Painter`, `Rect<Logical>`, and `&Theme`.
    pub fn paint<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut Painter<'_>, Rect<Logical>, &Theme) + Send + Sync + 'static,
    {
        self.canvas.paint = Arc::new(f);
        self
    }

    /// Provide a pre-wrapped `Arc` paint callback.
    pub fn paint_arc(mut self, f: Arc<CanvasPaintFn>) -> Self {
        self.canvas.paint = f;
        self
    }

    /// Attach a click message emitted when the canvas is clicked while enabled.
    pub fn on_click(mut self, message: M) -> Self {
        self.canvas.on_click = Some(message);
        self
    }

    /// Set an accessibility label exposed through AccessKit.
    pub fn accessible_label(mut self, label: impl Into<String>) -> Self {
        self.canvas.accessible_label = Some(label.into());
        self
    }

    /// Override the pointer cursor hint shown when hovering the canvas.
    pub fn cursor(mut self, cursor: CursorHint) -> Self {
        self.canvas.cursor = Some(cursor);
        self
    }

    /// Mark the canvas as disabled (blocks interaction and accessibility activation).
    pub fn disabled(mut self, value: bool) -> Self {
        self.canvas.disabled = value;
        self
    }

    /// Consume the builder and return the [`Canvas`] struct directly.
    pub fn into_canvas(self) -> Canvas<M> {
        self.canvas
    }

    /// Consume the builder and produce a [`WidgetNode::Canvas`].
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::Canvas(self.canvas)
    }
}

impl<M> From<CanvasBuilder<M>> for WidgetNode<M> {
    fn from(builder: CanvasBuilder<M>) -> Self {
        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::LayoutEngine;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Tapped,
    }

    #[test]
    fn canvas_builder_sets_fields_and_converts() {
        let node: WidgetNode<Msg> = canvas::<Msg>()
            .key("chart")
            .size(160.0, 80.0)
            .accessible_label("Sparkline chart")
            .cursor(CursorHint::Pointer)
            .on_click(Msg::Tapped)
            .disabled(false)
            .into();

        let WidgetNode::Canvas(c) = &node else {
            panic!("expected WidgetNode::Canvas");
        };
        assert_eq!(c.key().map(WidgetKey::as_str), Some("chart"));
        assert_eq!(c.width, Some(160.0));
        assert_eq!(c.height, Some(80.0));
        assert_eq!(c.accessible_label.as_deref(), Some("Sparkline chart"));
        assert_eq!(c.cursor, Some(CursorHint::Pointer));
        assert_eq!(c.activate(), Some(&Msg::Tapped));
        assert!(node.children().is_empty());
    }

    #[test]
    fn canvas_disabled_blocks_activate() {
        let node = canvas::<Msg>().on_click(Msg::Tapped).disabled(true).build();
        let WidgetNode::Canvas(c) = node else {
            panic!("expected Canvas");
        };
        assert!(c.disabled);
        assert_eq!(c.activate(), None);
    }

    #[test]
    fn canvas_paint_invokes_with_theme_and_produces_commands() {
        let calls = Arc::new(AtomicUsize::new(0));
        let calls_ref = Arc::clone(&calls);

        let node = canvas::<Msg>()
            .size(100.0, 50.0)
            .paint(move |painter, rect, theme| {
                calls_ref.fetch_add(1, Ordering::SeqCst);
                let color = acme_core::Color::rgba(
                    theme.colors.primary.red,
                    theme.colors.primary.green,
                    theme.colors.primary.blue,
                    theme.colors.primary.alpha,
                );
                painter.fill_rect(rect, color);
            })
            .build();

        let WidgetNode::Canvas(c) = node else {
            panic!("expected Canvas");
        };
        let theme = Theme::dark();
        let mut painter = Painter::new();
        c.paint_into(&mut painter, Rect::new(0.0, 0.0, 100.0, 50.0), &theme);
        let scene = painter.finish();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(!scene.commands().is_empty());
    }

    #[test]
    fn canvas_layout_computes_non_zero_rect() {
        let node = canvas::<Msg>().key("c").size(120.0, 44.0).build();
        let layout = node.to_layout(NodeId::new(1));
        let snapshot = LayoutEngine::new()
            .compute(&layout, (400.0, 300.0))
            .unwrap();
        let rect = snapshot.get(layout.id).expect("canvas rect");
        assert_eq!(rect.width, 120.0);
        assert_eq!(rect.height, 44.0);
    }

    #[test]
    fn canvas_debug_clone_and_partial_eq_work() {
        let built = canvas::<Msg>().key("k").size(20.0, 10.0).build();
        let cloned = built.clone();
        assert_eq!(built, cloned);
        assert!(format!("{cloned:?}").contains("<fn>"));
    }
}
