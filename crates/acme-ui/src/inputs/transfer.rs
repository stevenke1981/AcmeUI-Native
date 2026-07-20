//! Transfer component — two-column transfer list.
//!
//! Renders a Row with a source Column, a middle button Column (▶ / ◀), and a
//! target Column. Items can be visually marked as selected.

use acme_core::WidgetKey;
use acme_widgets::*;

/// An item in the transfer list.
#[derive(Clone, Debug)]
pub struct TransferItem {
    pub label: String,
    pub value: String,
    pub disabled: bool,
}

/// Builder for a Transfer component.
pub struct TransferBuilder<M> {
    pub id: WidgetKey,
    pub source: Vec<TransferItem>,
    pub target: Vec<TransferItem>,
    pub source_selected: Vec<String>,
    pub target_selected: Vec<String>,
    pub on_change: Option<M>,
}

/// Create a new Transfer builder.
pub fn transfer<M: Clone + 'static>(id: impl Into<WidgetKey>) -> TransferBuilder<M> {
    TransferBuilder {
        id: id.into(),
        source: vec![],
        target: vec![],
        source_selected: vec![],
        target_selected: vec![],
        on_change: None,
    }
}

/// Create a transfer list item.
pub fn transfer_item(label: impl Into<String>, value: impl Into<String>) -> TransferItem {
    TransferItem {
        label: label.into(),
        value: value.into(),
        disabled: false,
    }
}

impl TransferItem {
    /// Mark the item as disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
}

impl<M: Clone + 'static> TransferBuilder<M> {
    /// Add a source-side item.
    pub fn source_item(mut self, item: TransferItem) -> Self {
        self.source.push(item);
        self
    }

    /// Add a target-side item.
    pub fn target_item(mut self, item: TransferItem) -> Self {
        self.target.push(item);
        self
    }

    /// Set the source-side selected values.
    pub fn source_selected(mut self, values: Vec<String>) -> Self {
        self.source_selected = values;
        self
    }

    /// Set the target-side selected values.
    pub fn target_selected(mut self, values: Vec<String>) -> Self {
        self.target_selected = values;
        self
    }

    /// Set the message dispatched when items are transferred.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

/// Render a list of items inside a Card.
fn render_item_list<M: Clone + 'static>(
    items: &[TransferItem],
    selected: &[String],
    header: &str,
    count: usize,
) -> WidgetNode<M> {
    let header = label::<M>(format!("{} ({})", header, count));
    let mut list_col = column::<M>().gap(2.0).child(header);

    for item in items {
        let is_selected = selected.contains(&item.value);
        let item_card = card::<M>()
            .padding(6.0)
            .variant(if is_selected {
                CardVariant::Interactive
            } else {
                CardVariant::Plain
            })
            .child(label::<M>(item.label.clone()));
        list_col = list_col.child(item_card);
    }

    let panel = card::<M>()
        .variant(CardVariant::Outlined)
        .padding(8.0)
        .child(list_col);

    panel.build()
}

impl<M: Clone + 'static> From<TransferBuilder<M>> for WidgetNode<M> {
    fn from(b: TransferBuilder<M>) -> Self {
        let source_panel = render_item_list::<M>(
            &b.source,
            &b.source_selected,
            "Source",
            b.source.len(),
        );

        let middle = column::<M>()
            .gap(8.0)
            .child(button::<M>(
                format!("{}-to", b.id.as_str()).as_str(),
                "▶",
            ))
            .child(button::<M>(
                format!("{}-from", b.id.as_str()).as_str(),
                "◀",
            ));

        let target_panel = render_item_list::<M>(
            &b.target,
            &b.target_selected,
            "Target",
            b.target.len(),
        );

        row::<M>()
            .key(b.id)
            .gap(8.0)
            .child(source_panel)
            .child(middle)
            .child(target_panel)
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
    use acme_layout::LayoutKind;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn transfer_builder_defaults() {
        let t = transfer::<TestMsg>("t");
        assert!(t.source.is_empty());
        assert!(t.target.is_empty());
        assert!(t.source_selected.is_empty());
        assert!(t.target_selected.is_empty());
    }

    #[test]
    fn transfer_renders_row_with_three_panels() {
        let node: WidgetNode<TestMsg> = transfer("t")
            .source_item(transfer_item("Item 1", "1"))
            .source_item(transfer_item("Item 2", "2"))
            .target_item(transfer_item("Item A", "a"))
            .into();
        let WidgetNode::Row(row) = &node else {
            panic!("expected Row variant");
        };
        // source panel + middle buttons + target panel = 3 children
        assert_eq!(row.children.len(), 3);
    }

    #[test]
    fn transfer_source_item_count_in_header() {
        let node: WidgetNode<TestMsg> = transfer("t")
            .source_item(transfer_item("A", "a"))
            .source_item(transfer_item("B", "b"))
            .target_item(transfer_item("C", "c"))
            .into();
        let WidgetNode::Row(row) = &node else {
            panic!("expected Row");
        };
        // Source panel contains header "Source (2)"
        let WidgetNode::Card(source_card) = &row.children[0] else {
            panic!("expected Card for source panel");
        };
        let WidgetNode::Column(source_col) = &source_card.children[0] else {
            panic!("expected Column inside source Card");
        };
        let WidgetNode::Label(header) = &source_col.children[0] else {
            panic!("expected Label header");
        };
        assert_eq!(header.text, "Source (2)");
    }

    #[test]
    fn transfer_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = transfer("t")
            .source_item(transfer_item("X", "x"))
            .target_item(transfer_item("Y", "y"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert!(!layout.children.is_empty());
    }
}
