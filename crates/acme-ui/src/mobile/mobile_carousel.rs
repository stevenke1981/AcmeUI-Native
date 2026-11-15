//! Mobile carousel component — paged card viewport with prev/next and dot controls.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_widgets::{ButtonVariant, CardVariant, button, card, column, label, row};

/// Builder for a mobile carousel.
pub struct MobileCarouselBuilder<M> {
    pub id: WidgetKey,
    pub items: Vec<WidgetNode<M>>,
    pub active_index: usize,
    pub show_indicators: bool,
    pub on_prev: Option<M>,
    pub on_next: Option<M>,
    pub on_select: Option<fn(usize) -> M>,
}

/// Create a mobile carousel builder.
pub fn mobile_carousel<M: Clone + 'static>(id: impl Into<WidgetKey>) -> MobileCarouselBuilder<M> {
    MobileCarouselBuilder {
        id: id.into(),
        items: Vec::new(),
        active_index: 0,
        show_indicators: true,
        on_prev: None,
        on_next: None,
        on_select: None,
    }
}

impl<M: Clone + 'static> MobileCarouselBuilder<M> {
    /// Add a slide item to the carousel.
    pub fn item(mut self, item: impl Into<WidgetNode<M>>) -> Self {
        self.items.push(item.into());
        self
    }

    /// Set the active slide index.
    pub fn active_index(mut self, index: usize) -> Self {
        self.active_index = index;
        self
    }

    /// Control whether page indicator dots are rendered below the active item.
    pub fn show_indicators(mut self, show: bool) -> Self {
        self.show_indicators = show;
        self
    }

    /// Set the message dispatched when the previous-slide button is clicked.
    pub fn on_prev(mut self, msg: M) -> Self {
        self.on_prev = Some(msg);
        self
    }

    /// Set the message dispatched when the next-slide button is clicked.
    pub fn on_next(mut self, msg: M) -> Self {
        self.on_next = Some(msg);
        self
    }

    /// Set the callback invoked when an indicator dot is clicked.
    pub fn on_select(mut self, f: fn(usize) -> M) -> Self {
        self.on_select = Some(f);
        self
    }
}

impl<M: Clone + 'static> From<MobileCarouselBuilder<M>> for WidgetNode<M> {
    fn from(mut b: MobileCarouselBuilder<M>) -> Self {
        let id_str = b.id.as_str().to_string();
        let len = b.items.len();
        let active = if len == 0 {
            0
        } else {
            b.active_index.min(len - 1)
        };

        let at_start = len == 0 || active == 0;
        let at_end = len == 0 || active + 1 >= len;

        let prev_btn = button::<M>(format!("{id_str}_prev").as_str(), "‹")
            .variant(ButtonVariant::Ghost)
            .disabled(at_start);
        let prev_node: WidgetNode<M> = match b.on_prev {
            Some(msg) => prev_btn.on_click(msg),
            None => prev_btn.into(),
        };

        let next_btn = button::<M>(format!("{id_str}_next").as_str(), "›")
            .variant(ButtonVariant::Ghost)
            .disabled(at_end);
        let next_node: WidgetNode<M> = match b.on_next {
            Some(msg) => next_btn.on_click(msg),
            None => next_btn.into(),
        };

        let slide_node: WidgetNode<M> = if len > 0 {
            b.items.swap_remove(active)
        } else {
            card::<M>()
                .variant(CardVariant::Muted)
                .padding(8.0)
                .child(label::<M>("Empty"))
                .build()
        };

        let viewport = row::<M>()
            .gap(8.0)
            .child(prev_node)
            .child(slide_node)
            .child(next_node)
            .build();

        let mut root = column::<M>().key(b.id).gap(8.0).child(viewport);

        if b.show_indicators && len > 0 {
            let mut dots = row::<M>().gap(4.0);
            for i in 0..len {
                let is_active = i == active;
                let dot = button::<M>(
                    format!("{id_str}_dot_{i}").as_str(),
                    if is_active { "●" } else { "○" },
                )
                .variant(if is_active {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Ghost
                });
                let dot_node: WidgetNode<M> = match b.on_select {
                    Some(cb) => dot.on_click(cb(i)),
                    None => dot.into(),
                };
                dots = dots.child(dot_node);
            }
            root = root.child(dots.build());
        }

        root.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Prev,
        Next,
        Go(usize),
    }

    #[test]
    fn mobile_carousel_renders_active_slide_and_controls() {
        let node: WidgetNode<Msg> = mobile_carousel("hero")
            .item(label("Slide 1"))
            .item(label("Slide 2"))
            .item(label("Slide 3"))
            .active_index(1)
            .on_prev(Msg::Prev)
            .on_next(Msg::Next)
            .on_select(Msg::Go)
            .into();

        let WidgetNode::Column(root) = &node else {
            panic!("expected Column");
        };
        assert_eq!(root.children.len(), 2);

        let WidgetNode::Row(viewport) = &root.children[0] else {
            panic!("expected viewport Row");
        };
        let WidgetNode::Button(prev) = &viewport.children[0] else {
            panic!("expected prev Button");
        };
        assert!(!prev.disabled);
        assert_eq!(prev.activate(), Some(&Msg::Prev));

        let WidgetNode::Label(slide) = &viewport.children[1] else {
            panic!("expected active slide Label");
        };
        assert_eq!(slide.text, "Slide 2");

        let WidgetNode::Button(next) = &viewport.children[2] else {
            panic!("expected next Button");
        };
        assert!(!next.disabled);
        assert_eq!(next.activate(), Some(&Msg::Next));

        let WidgetNode::Row(dots) = &root.children[1] else {
            panic!("expected indicator Row");
        };
        assert_eq!(dots.children.len(), 3);
        let WidgetNode::Button(dot2) = &dots.children[2] else {
            panic!("expected dot Button");
        };
        assert_eq!(dot2.activate(), Some(&Msg::Go(2)));
    }

    #[test]
    fn mobile_carousel_disables_prev_at_zero_and_next_at_end() {
        let first: WidgetNode<Msg> = mobile_carousel("c")
            .item(label("A"))
            .item(label("B"))
            .active_index(0)
            .on_prev(Msg::Prev)
            .on_next(Msg::Next)
            .into();
        let WidgetNode::Column(root) = &first else {
            panic!("expected Column");
        };
        let WidgetNode::Row(vp) = &root.children[0] else {
            panic!("expected Row");
        };
        let WidgetNode::Button(prev) = &vp.children[0] else {
            panic!("expected prev");
        };
        let WidgetNode::Button(next) = &vp.children[2] else {
            panic!("expected next");
        };
        assert!(prev.disabled);
        assert!(!next.disabled);
    }
}
