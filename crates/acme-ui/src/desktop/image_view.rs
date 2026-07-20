//! ImageView — an image display component with fit modes.
//!
//! Since actual image rendering is not available at the widget-description
//! layer, this serves as a placeholder container with alt text.

use crate::*;

/// How the image content fits within its bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageFit {
    /// Fit within bounds, maintain aspect ratio.
    #[default]
    Contain,
    /// Cover bounds, may crop.
    Cover,
    /// Stretch to fill.
    Fill,
    /// Original size, no scaling.
    None,
}

/// Builder for an ImageView component.
pub struct ImageViewBuilder<M> {
    pub id: WidgetKey,
    pub label: String,
    pub fit: ImageFit,
    pub width: f32,
    pub height: f32,
    pub description: Option<String>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a new ImageView builder.
pub fn image_view<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    label: impl Into<String>,
) -> ImageViewBuilder<M> {
    ImageViewBuilder::<M> {
        id: id.into(),
        label: label.into(),
        fit: ImageFit::default(),
        width: 200.0,
        height: 200.0,
        description: None,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> ImageViewBuilder<M> {
    /// Set the image fit mode.
    pub fn fit(mut self, value: ImageFit) -> Self {
        self.fit = value;
        self
    }

    /// Set the viewport width in pixels.
    pub fn width(mut self, value: f32) -> Self {
        self.width = value;
        self
    }

    /// Set the viewport height in pixels.
    pub fn height(mut self, value: f32) -> Self {
        self.height = value;
        self
    }

    /// Set the accessible description text.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }
}

impl<M: Clone + 'static> From<ImageViewBuilder<M>> for WidgetNode<M> {
    fn from(b: ImageViewBuilder<M>) -> Self {
        let mut c = card::<M>()
            .variant(CardVariant::Outlined)
            .child(label::<M>(b.label));

        if let Some(desc) = b.description {
            c = c.child(label::<M>(desc));
        }

        // Wrap the card in a container with the specified viewport dimensions
        column::<M>()
            .key(b.id)
            .width(b.width)
            .height(b.height)
            .child(c.build())
            .build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutKind, Length};

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn image_view_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = image_view("iv", "Placeholder").into();
        let layout = node.to_layout(NodeId::new(1));
        // Container Column wrapping a Card
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert_eq!(layout.style.width, Length::px(200.0));
        assert_eq!(layout.style.height, Length::px(200.0));
    }

    #[test]
    fn image_view_builder_defaults() {
        let iv = image_view::<TestMsg>("iv", "Alt text");
        assert_eq!(iv.label, "Alt text");
        assert_eq!(iv.fit, ImageFit::Contain);
        assert!((iv.width - 200.0).abs() < f32::EPSILON);
        assert!((iv.height - 200.0).abs() < f32::EPSILON);
        assert!(iv.description.is_none());
    }

    #[test]
    fn image_view_field_setters_work() {
        let iv = image_view::<TestMsg>("iv", "Photo")
            .fit(ImageFit::Cover)
            .width(400.0)
            .height(300.0)
            .description("A photo");

        assert_eq!(iv.fit, ImageFit::Cover);
        assert!((iv.width - 400.0).abs() < f32::EPSILON);
        assert!((iv.height - 300.0).abs() < f32::EPSILON);
        assert_eq!(iv.description, Some("A photo".to_string()));
    }
}
