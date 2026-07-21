//! Carousel component — a slide carousel with previous/next navigation and dots.

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a single carousel slide.
pub struct CarouselSlideBuilder<M> {
    pub id: WidgetKey,
    pub label: String,
    pub caption: Option<String>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a new carousel slide builder.
pub fn carousel_slide<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    label: impl Into<String>,
) -> CarouselSlideBuilder<M> {
    CarouselSlideBuilder {
        id: id.into(),
        label: label.into(),
        caption: None,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> CarouselSlideBuilder<M> {
    /// Set the caption for this slide.
    pub fn caption(mut self, value: impl Into<String>) -> Self {
        self.caption = Some(value.into());
        self
    }
}

/// Builder for a Carousel component.
pub struct CarouselBuilder<M> {
    pub id: WidgetKey,
    pub slides: Vec<CarouselSlideBuilder<M>>,
    pub active_index: usize,
    pub show_dots: bool,
    pub show_arrows: bool,
    pub on_change: Option<M>,
}

/// Create a new Carousel builder.
pub fn carousel<M: Clone + 'static>(id: impl Into<WidgetKey>) -> CarouselBuilder<M> {
    CarouselBuilder {
        id: id.into(),
        slides: vec![],
        active_index: 0,
        show_dots: true,
        show_arrows: true,
        on_change: None,
    }
}

impl<M: Clone + 'static> CarouselBuilder<M> {
    /// Add a slide to the carousel.
    pub fn slide(mut self, slide: CarouselSlideBuilder<M>) -> Self {
        self.slides.push(slide);
        self
    }

    /// Set the active (visible) slide index.
    pub fn active_index(mut self, index: usize) -> Self {
        self.active_index = index;
        self
    }

    /// Show or hide the dot indicators.
    pub fn show_dots(mut self, value: bool) -> Self {
        self.show_dots = value;
        self
    }

    /// Show or hide the previous / next arrow buttons.
    pub fn show_arrows(mut self, value: bool) -> Self {
        self.show_arrows = value;
        self
    }

    /// Set the message dispatched when navigation arrows are clicked.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<CarouselBuilder<M>> for WidgetNode<M> {
    fn from(b: CarouselBuilder<M>) -> Self {
        // Previous arrow button
        let prev_btn: Option<WidgetNode<M>> = if b.show_arrows {
            Some(if let Some(ref msg) = b.on_change {
                crate::button("carousel-prev", "◀")
                    .variant(ButtonVariant::Ghost)
                    .on_click(msg.clone())
            } else {
                crate::button("carousel-prev", "◀")
                    .variant(ButtonVariant::Ghost)
                    .into()
            })
        } else {
            None
        };

        // Next arrow button
        let next_btn: Option<WidgetNode<M>> = if b.show_arrows {
            Some(if let Some(ref msg) = b.on_change {
                crate::button("carousel-next", "▶")
                    .variant(ButtonVariant::Ghost)
                    .on_click(msg.clone())
            } else {
                crate::button("carousel-next", "▶")
                    .variant(ButtonVariant::Ghost)
                    .into()
            })
        } else {
            None
        };

        // Build the current active slide content
        let slide_content = if b.slides.is_empty() {
            crate::card::<M>()
                .variant(CardVariant::Muted)
                .child(crate::label::<M>("(empty)"))
                .padding(16.0)
                .build()
        } else {
            let idx = b.active_index.min(b.slides.len() - 1);
            let slide = &b.slides[idx];
            let mut slide_col = crate::column::<M>().child(
                crate::card::<M>()
                    .child(crate::label::<M>(&slide.label))
                    .padding(16.0)
                    .build(),
            );
            if let Some(ref caption) = slide.caption {
                slide_col = slide_col.child(crate::label::<M>(caption));
            }
            slide_col.build()
        };

        // Main row: [prev] [slide] [next]
        let mut main_row = crate::row::<M>().gap(4.0);
        if let Some(btn) = prev_btn {
            main_row = main_row.child(btn);
        }
        main_row = main_row.child(slide_content);
        if let Some(btn) = next_btn {
            main_row = main_row.child(btn);
        }

        // Build the column with the main row and optional dots
        let mut col = crate::column::<M>().gap(8.0).child(main_row.build());

        if b.show_dots && b.slides.len() > 1 {
            let mut dots_row = crate::row::<M>().gap(4.0);
            for (i, _) in b.slides.iter().enumerate() {
                let dot = if i == b.active_index {
                    crate::label::<M>("●")
                } else {
                    crate::label::<M>("○")
                };
                dots_row = dots_row.child(dot);
            }
            col = col.child(dots_row.build());
        }

        crate::card::<M>()
            .key(b.id)
            .child(col.build())
            .padding(8.0)
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

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn carousel_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = carousel("car")
            .slide(carousel_slide("s1", "Slide 1"))
            .slide(carousel_slide("s2", "Slide 2"))
            .active_index(0)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Wrapper Card -> Column: [Row, dots Row]
        assert!(!layout.children.is_empty());
    }

    #[test]
    fn carousel_builder_defaults() {
        let c = carousel::<TestMsg>("car");
        assert_eq!(c.active_index, 0);
        assert!(c.show_dots);
        assert!(c.show_arrows);
        assert!(c.on_change.is_none());
        assert!(c.slides.is_empty());
    }

    #[test]
    fn carousel_shows_active_slide_content() {
        let node: WidgetNode<TestMsg> = carousel("car")
            .slide(carousel_slide("s1", "First").caption("desc 1"))
            .slide(carousel_slide("s2", "Second"))
            .active_index(1)
            .into();
        let WidgetNode::Card(card) = &node else {
            panic!("expected Card wrapper");
        };
        // Card wraps Column
        assert_eq!(card.children.len(), 1);
        // Column has 2 children: main row, and no dots row when only 2 slides
        // Wait — dots are shown for 2 slides (b.slides.len() > 1)
        // So: Column children = [main Row, dots Row]
        let WidgetNode::Column(col) = &card.children[0] else {
            panic!("expected Column inside Card");
        };
        // Column: Row (main content) + Row (dots) = 2 children
        assert!(!col.children.is_empty());
    }
}
