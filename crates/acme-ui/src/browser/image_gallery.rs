//! ImageGallery component — a grid-based image gallery layout.
//!
//! Renders as a Column with a header and a flex-wrap Row of thumbnail
//! containers. Each image item shows a label and caption.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// An image item in the gallery.
#[derive(Clone, Debug)]
pub struct GalleryImage {
    pub label: String,
    pub caption: Option<String>,
    pub aspect_ratio: f32,
}

impl GalleryImage {
    /// Create a gallery image entry.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            caption: None,
            aspect_ratio: 1.0,
        }
    }

    /// Set the image caption text.
    pub fn caption(mut self, value: impl Into<String>) -> Self {
        self.caption = Some(value.into());
        self
    }

    /// Set the width/height aspect ratio (e.g. 1.0 = square, 1.5 = landscape).
    pub fn aspect_ratio(mut self, value: f32) -> Self {
        self.aspect_ratio = value.max(0.25);
        self
    }
}

/// Builder for an image gallery grid.
pub struct ImageGalleryBuilder<M> {
    pub id: WidgetKey,
    pub images: Vec<GalleryImage>,
    pub columns: usize,
    pub gap: f32,
    pub thumbnail_size: f32,
    pub show_captions: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create an image gallery builder.
pub fn image_gallery<M: Clone + 'static>(id: impl Into<WidgetKey>) -> ImageGalleryBuilder<M> {
    ImageGalleryBuilder {
        id: id.into(),
        images: vec![],
        columns: 4,
        gap: 8.0,
        thumbnail_size: 120.0,
        show_captions: true,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> ImageGalleryBuilder<M> {
    /// Add an image to the gallery.
    pub fn image(mut self, entry: GalleryImage) -> Self {
        self.images.push(entry);
        self
    }

    /// Set the number of thumbnail columns.
    pub fn columns(mut self, value: usize) -> Self {
        self.columns = value.max(1);
        self
    }

    /// Set the gap between thumbnails.
    pub fn gap(mut self, value: f32) -> Self {
        self.gap = value;
        self
    }

    /// Set the thumbnail base size in pixels.
    pub fn thumbnail_size(mut self, value: f32) -> Self {
        self.thumbnail_size = value;
        self
    }

    /// Show/hide captions below thumbnails.
    pub fn show_captions(mut self, value: bool) -> Self {
        self.show_captions = value;
        self
    }

    /// Build the image gallery widget.
    pub fn build(self) -> WidgetNode<M> {
        let id_prefix = self.id.as_str().to_string();

        let mut col = crate::column::<M>()
            .key(WidgetKey::new(format!("{}_gallery", &id_prefix)))
            .gap(self.gap);

        // Header
        let header = format!("Image Gallery — {} images", self.images.len());
        col = col.child(crate::label_with_size::<M>(header, 14.0));

        // Gallery grid
        let mut grid = crate::row::<M>()
            .key(format!("{}_grid", id_prefix).as_str())
            .gap(self.gap);

        for (i, img) in self.images.iter().enumerate() {
            let thumb_height = self.thumbnail_size / img.aspect_ratio;

            let mut thumb_col = crate::column::<M>()
                .key(format!("{}_thumb_{}", id_prefix, i).as_str())
                .gap(4.0);

            // Thumbnail placeholder using a Card inside a sized container
            let thumb = crate::card::<M>()
                .variant(crate::CardVariant::Muted)
                .padding(4.0)
                .child(crate::label_builder(&img.label).font_size(11.0).build())
                .build();

            // Wrap thumbnail in a sized column to simulate width/height
            let sized_thumb = crate::column::<M>()
                .width(self.thumbnail_size)
                .height(thumb_height)
                .child(thumb)
                .build();

            thumb_col = thumb_col.child(sized_thumb);

            // Optional caption
            if self.show_captions
                && let Some(cap) = &img.caption
            {
                thumb_col = thumb_col.child(
                    crate::label_builder(cap)
                        .font_size(11.0)
                        .color(crate::ThemeColor::rgb(120, 120, 120))
                        .build(),
                );
            }

            grid = grid.child(thumb_col.build());
        }

        col = col.child(grid.build());
        col.build()
    }
}

impl<M: Clone + 'static> From<ImageGalleryBuilder<M>> for WidgetNode<M> {
    fn from(b: ImageGalleryBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutEngine, WidgetLayoutContext};

    fn test_context() -> WidgetLayoutContext {
        WidgetLayoutContext {
            body_font_size: 16.0,
            body_line_height: 22.0,
            label_font_size: 14.0,
            control_height: 32.0,
            scale_factor: 1.0,
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn image_gallery_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = image_gallery("gal").build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let snapshot = LayoutEngine::new()
            .compute(&layout, (800.0, 600.0))
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.height > 0.0, "gallery height should be > 0");
    }

    #[test]
    fn image_gallery_with_images() {
        let node: WidgetNode<TestMsg> = image_gallery("gal")
            .image(GalleryImage::new("Photo 1").caption("Sunset"))
            .image(GalleryImage::new("Photo 2").aspect_ratio(1.5))
            .columns(3)
            .build();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column")
        };
        // header + grid = 2
        assert_eq!(col.children.len(), 2);
        let WidgetNode::Row(grid) = &col.children[1] else {
            panic!("expected Row for grid")
        };
        assert_eq!(grid.children.len(), 2);
    }

    #[test]
    fn image_gallery_captions_hidden() {
        let node: WidgetNode<TestMsg> = image_gallery("gal")
            .image(GalleryImage::new("Photo 1"))
            .show_captions(false)
            .build();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column")
        };
        let WidgetNode::Row(grid) = &col.children[1] else {
            panic!("expected Row")
        };
        let WidgetNode::Column(thumb_col) = &grid.children[0] else {
            panic!("expected Column")
        };
        // sized container only, no caption
        assert_eq!(thumb_col.children.len(), 1);
    }

    #[test]
    fn image_gallery_builder_defaults() {
        let g = image_gallery::<TestMsg>("gal");
        assert_eq!(g.columns, 4);
        assert!(g.show_captions);
        assert!(g.images.is_empty());
        assert!((g.gap - 8.0).abs() < f32::EPSILON);
        assert!((g.thumbnail_size - 120.0).abs() < f32::EPSILON);
    }

    #[test]
    fn image_gallery_columns_min_one() {
        let g = image_gallery::<TestMsg>("gal").columns(0);
        assert_eq!(g.columns, 1);
    }

    #[test]
    fn gallery_image_aspect_ratio_min() {
        let img = GalleryImage::new("x").aspect_ratio(0.0);
        assert!((img.aspect_ratio - 0.25).abs() < f32::EPSILON);
    }
}
