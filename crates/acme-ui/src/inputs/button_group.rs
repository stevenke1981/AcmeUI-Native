//! ButtonGroup component.
//!
//! A horizontal or vertical group of buttons with connected borders.
//! The first button gets left/top border radius, the last gets right/bottom,
//! and middle buttons have no radius between them — common in toolbars.
//!
//! Each item is rendered as a [`Card`] containing a [`Button`].
//! Selected items use [`CardVariant::Interactive`]; unselected use
//! [`CardVariant::Outlined`].

use acme_core::WidgetKey;
use acme_widgets::*;

/// Orientation for the button group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    /// Arrange buttons left-to-right.
    #[default]
    Horizontal,
    /// Arrange buttons top-to-bottom.
    Vertical,
}

/// Internal item stored in the builder before conversion.
#[derive(Clone, Debug)]
struct ButtonGroupItem<M> {
    label: String,
    on_click: M,
}

/// Builder for a ButtonGroup component.
///
/// Convert into [`WidgetNode<M>`] via `From` / `.into()`.
pub struct ButtonGroupBuilder<M> {
    pub id: WidgetKey,
    items: Vec<ButtonGroupItem<M>>,
    pub selected: usize,
    pub orientation: Orientation,
    pub compact: bool,
}

/// Create a new ButtonGroup builder.
pub fn button_group<M: Clone + 'static>(id: impl Into<WidgetKey>) -> ButtonGroupBuilder<M> {
    ButtonGroupBuilder {
        id: id.into(),
        items: vec![],
        selected: 0,
        orientation: Orientation::default(),
        compact: false,
    }
}

impl<M: Clone + 'static> ButtonGroupBuilder<M> {
    /// Add a button item with a label and the message dispatched on click.
    pub fn item(mut self, label: impl Into<String>, on_click: M) -> Self {
        self.items.push(ButtonGroupItem {
            label: label.into(),
            on_click,
        });
        self
    }

    /// Set the index of the currently selected item (default `0`).
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    /// Set the layout orientation (default [`Orientation::Horizontal`]).
    pub fn orientation(mut self, value: Orientation) -> Self {
        self.orientation = value;
        self
    }

    /// Enable tighter spacing variant.
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }
}

impl<M: Clone + 'static> From<ButtonGroupBuilder<M>> for WidgetNode<M> {
    fn from(b: ButtonGroupBuilder<M>) -> Self {
        let id_str = b.id.as_str().to_string();
        let item_count = b.items.len();
        let gap = if b.compact { 2.0 } else { 4.0 };
        let card_padding = if b.compact { 2.0 } else { 4.0 };

        // Clamp selected index to valid range.
        let selected = if item_count == 0 {
            0
        } else {
            b.selected.min(item_count - 1)
        };

        let mut container = match b.orientation {
            Orientation::Horizontal => row::<M>().key(b.id.clone()).gap(gap),
            Orientation::Vertical => column::<M>().key(b.id).gap(gap),
        };

        for (i, item) in b.items.iter().enumerate() {
            let is_selected = i == selected;
            let btn_key = format!("{id_str}_{i}");

            // Border radius: first gets left/top, last gets right/bottom,
            // middle items have no radius between them.
            let radius = if item_count == 0 || item_count == 1 || i == 0 || i == item_count - 1 {
                4.0
            } else {
                0.0
            };

            // Build a Button (handles label rendering & click message).
            let btn = button::<M>(btn_key.as_str(), item.label.as_str());
            let btn_node = btn.on_click(item.on_click.clone());

            // Wrap in a Card for the visual container variant and border radius.
            let card = card()
                .variant(if is_selected {
                    CardVariant::Interactive
                } else {
                    CardVariant::Outlined
                })
                .border_radius(radius)
                .padding(card_padding)
                .child(btn_node)
                .build();

            container = container.child(card);
        }

        container.build()
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
    enum TestMsg {
        ClickA,
        ClickB,
        ClickC,
    }

    #[test]
    fn button_group_builds_row() {
        let node: WidgetNode<TestMsg> = button_group("bg")
            .item("A", TestMsg::ClickA)
            .item("B", TestMsg::ClickB)
            .item("C", TestMsg::ClickC)
            .into();

        // Default orientation is Horizontal → Row
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        assert_eq!(container.children.len(), 3);

        // Each child must be a Card (visual container for the button)
        for child in &container.children {
            assert!(
                matches!(child, WidgetNode::Card(_)),
                "each item should be a Card"
            );
        }
    }

    #[test]
    fn button_group_selected_item() {
        let node: WidgetNode<TestMsg> = button_group("bg")
            .item("A", TestMsg::ClickA)
            .item("B", TestMsg::ClickB)
            .selected(1)
            .into();

        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        assert_eq!(container.children.len(), 2);

        // Index 0 → unselected (Outlined)
        let WidgetNode::Card(card0) = &container.children[0] else {
            panic!("expected Card at index 0");
        };
        assert_eq!(
            card0.variant,
            CardVariant::Outlined,
            "unselected item should be Outlined"
        );

        // Index 1 → selected (Interactive)
        let WidgetNode::Card(card1) = &container.children[1] else {
            panic!("expected Card at index 1");
        };
        assert_eq!(
            card1.variant,
            CardVariant::Interactive,
            "selected item should be Interactive"
        );
    }

    #[test]
    fn button_group_compact() {
        let node: WidgetNode<TestMsg> = button_group("bg")
            .item("A", TestMsg::ClickA)
            .item("B", TestMsg::ClickB)
            .compact()
            .into();

        // Compact reduces the Row gap from 4.0 → 2.0
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert_eq!(layout.style.gap, 2.0, "compact gap should be 2.0");
    }

    #[test]
    fn button_group_vertical() {
        let node: WidgetNode<TestMsg> = button_group("bg")
            .item("A", TestMsg::ClickA)
            .item("B", TestMsg::ClickB)
            .orientation(Orientation::Vertical)
            .into();

        let WidgetNode::Column(container) = &node else {
            panic!("expected Column for vertical orientation");
        };
        assert_eq!(container.children.len(), 2);
    }

    #[test]
    fn button_group_from_trait() {
        let bg = button_group::<TestMsg>("bg").item("A", TestMsg::ClickA);
        let node: WidgetNode<TestMsg> = bg.into();
        // Default orientation → Row
        assert!(
            matches!(node, WidgetNode::Row(_)),
            "From impl should produce a Row by default"
        );
    }

    #[test]
    fn button_group_default_selected_is_zero() {
        let bg = button_group::<TestMsg>("bg")
            .item("A", TestMsg::ClickA)
            .item("B", TestMsg::ClickB);
        assert_eq!(bg.selected, 0, "default selected index should be 0");
    }
}
