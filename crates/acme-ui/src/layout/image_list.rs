//! ImageList — image grid/masonry layout.
//! Aligns with MUI Image List component.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Image list variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageListVariant {
    #[default]
    Standard,
    Woven,
    Masonry,
    Quilted,
}

/// A single image item.
#[derive(Clone, Debug)]
pub struct ImageItem {
    pub src: String,
    pub alt: String,
    pub cols: u32,
    pub rows: u32,
}

impl ImageItem {
    pub fn new(src: impl Into<String>, alt: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            alt: alt.into(),
            cols: 1,
            rows: 1,
        }
    }

    pub fn cols(mut self, value: u32) -> Self {
        self.cols = value;
        self
    }

    pub fn rows(mut self, value: u32) -> Self {
        self.rows = value;
        self
    }
}

/// Builder for an image list.
pub struct ImageListBuilder<M> {
    pub id: WidgetKey,
    pub variant: ImageListVariant,
    pub cols: u32,
    pub gap: f32,
    pub items: Vec<ImageItem>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create an image list builder.
pub fn image_list<M: Clone + 'static>() -> ImageListBuilder<M> {
    ImageListBuilder {
        id: WidgetKey::from("image_list"),
        variant: ImageListVariant::default(),
        cols: 3,
        gap: 4.0,
        items: Vec::new(),
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> ImageListBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn variant(mut self, value: ImageListVariant) -> Self {
        self.variant = value;
        self
    }

    pub fn cols(mut self, value: u32) -> Self {
        self.cols = value.max(1);
        self
    }

    pub fn gap(mut self, value: f32) -> Self {
        self.gap = value;
        self
    }

    pub fn item(mut self, item: ImageItem) -> Self {
        self.items.push(item);
        self
    }
}

impl<M: Clone + 'static> From<ImageListBuilder<M>> for WidgetNode<M> {
    fn from(b: ImageListBuilder<M>) -> Self {
        let mut grid = crate::column::<M>().key(b.id).gap(b.gap).padding(b.gap);
        for item in &b.items {
            let cell = crate::stack::<M>()
                .child(crate::label(format!("[{}]", item.alt)))
                .build();
            grid = grid.child(cell);
        }
        grid.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn image_list_produces_column() {
        let node: WidgetNode<Msg> = image_list()
            .item(ImageItem::new("img1.png", "Image 1"))
            .into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn image_list_child_count_matches_items() {
        let node: WidgetNode<Msg> = image_list()
            .item(ImageItem::new("a.png", "A"))
            .item(ImageItem::new("b.png", "B"))
            .item(ImageItem::new("c.png", "C"))
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        assert_eq!(c.children.len(), 3);
    }

    #[test]
    fn image_list_default_cols() {
        let b = image_list::<Msg>();
        assert_eq!(b.cols, 3);
    }

    #[test]
    fn image_list_variant() {
        let b = image_list::<Msg>().variant(ImageListVariant::Masonry);
        assert_eq!(b.variant, ImageListVariant::Masonry);
    }
}
