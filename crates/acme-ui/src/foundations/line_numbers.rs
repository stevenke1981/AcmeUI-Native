//! LineNumbers component — code line number gutter.
//!
//! Renders as a Column of right-aligned line number labels.
//! The highlighted line gets an accent background.

use acme_core::WidgetKey;
use acme_widgets::*;
use std::marker::PhantomData;

/// Builder for a LineNumbers component.
pub struct LineNumbersBuilder<M> {
    pub id: WidgetKey,
    pub line_count: usize,
    pub start_line: usize,
    pub highlighted_line: Option<usize>,
    pub width: f32,
    _phantom: PhantomData<M>,
}

/// Create a new LineNumbers builder.
pub fn line_numbers<M: Clone + 'static>(id: impl Into<WidgetKey>) -> LineNumbersBuilder<M> {
    LineNumbersBuilder::<M> {
        id: id.into(),
        line_count: 0,
        start_line: 1,
        highlighted_line: None,
        width: 40.0,
        _phantom: PhantomData,
    }
}

impl<M: Clone + 'static> LineNumbersBuilder<M> {
    /// Set the number of lines to display.
    pub fn line_count(mut self, value: usize) -> Self {
        self.line_count = value;
        self
    }

    /// Set the starting line number (default 1).
    pub fn start_line(mut self, value: usize) -> Self {
        self.start_line = value;
        self
    }

    /// Set the line number to highlight.
    pub fn highlighted_line(mut self, value: usize) -> Self {
        self.highlighted_line = Some(value);
        self
    }

    /// Set the width of the gutter (default 40.0).
    pub fn width(mut self, value: f32) -> Self {
        self.width = value;
        self
    }
}

impl<M: Clone + 'static> From<LineNumbersBuilder<M>> for WidgetNode<M> {
    fn from(b: LineNumbersBuilder<M>) -> Self {
        let mut col = column::<M>().key(b.id).width(b.width);

        for i in 0..b.line_count {
            let line_num = b.start_line + i;
            let is_highlighted = b.highlighted_line == Some(line_num);

            let line_str = format!("{:>4}", line_num);

            if is_highlighted {
                // Highlighted line: wrap label in a Card with accent background
                let entry = card::<M>()
                    .variant(CardVariant::Interactive)
                    .child(label::<M>(line_str));
                col = col.child(entry);
            } else {
                col = col.child(label::<M>(line_str));
            }
        }

        col.build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::LayoutKind;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn line_numbers_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = line_numbers("ln1").line_count(10).into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert_eq!(layout.children.len(), 10);
    }

    #[test]
    fn line_numbers_builder_defaults() {
        let l = line_numbers::<TestMsg>("l");
        assert_eq!(l.line_count, 0);
        assert_eq!(l.start_line, 1);
        assert!(l.highlighted_line.is_none());
        assert_eq!(l.width, 40.0);
    }

    #[test]
    fn line_numbers_shows_line_numbers() {
        let node: WidgetNode<TestMsg> = line_numbers("ln").line_count(3).into();
        let WidgetNode::Column(container) = &node else {
            panic!("expected Column");
        };
        assert_eq!(container.children.len(), 3);
        // Each child should be a Label
        for (i, child) in container.children.iter().enumerate() {
            let WidgetNode::Label(lbl) = child else {
                panic!("expected Label at position {}", i);
            };
            let expected = format!("{:>4}", i + 1);
            assert_eq!(lbl.text, expected);
        }
    }

    #[test]
    fn line_numbers_highlighted_line() {
        let node: WidgetNode<TestMsg> = line_numbers("ln").line_count(5).highlighted_line(3).into();
        let WidgetNode::Column(container) = &node else {
            panic!("expected Column");
        };
        assert_eq!(container.children.len(), 5);
        // Line 3 is at index 2 (lines 1-5, start_line=1)
        let WidgetNode::Card(card) = &container.children[2] else {
            panic!("expected Card for highlighted line");
        };
        assert_eq!(card.variant, CardVariant::Interactive);
        // Non-highlighted lines should be plain Labels
        let WidgetNode::Label(_) = &container.children[0] else {
            panic!("expected Label for non-highlighted line");
        };
        let WidgetNode::Label(_) = &container.children[1] else {
            panic!("expected Label for non-highlighted line");
        };
    }

    #[test]
    fn line_numbers_custom_start_line() {
        let node: WidgetNode<TestMsg> = line_numbers("ln").line_count(3).start_line(10).into();
        let WidgetNode::Column(container) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Label(lbl_first) = &container.children[0] else {
            panic!("expected Label");
        };
        assert_eq!(lbl_first.text, "  10");
        let WidgetNode::Label(lbl_last) = &container.children[2] else {
            panic!("expected Label");
        };
        assert_eq!(lbl_last.text, "  12");
    }
}
