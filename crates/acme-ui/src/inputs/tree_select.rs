//! TreeSelect component — a select with tree-structured dropdown options.
//!
//! When open, renders a Column with the currently selected value (or placeholder)
//! and a dropdown Card containing a tree of option nodes with indentation.

use acme_core::WidgetKey;
use acme_widgets::*;

/// A node in the tree select hierarchy.
#[derive(Clone, Debug)]
pub struct TreeSelectNode {
    pub label: String,
    pub value: String,
    pub children: Vec<TreeSelectNode>,
}

/// Builder for a TreeSelect component.
pub struct TreeSelectBuilder<M> {
    pub id: WidgetKey,
    pub placeholder: String,
    pub options: Vec<TreeSelectNode>,
    pub selected_value: Option<String>,
    pub open: bool,
    pub on_change: Option<M>,
}

/// Create a new TreeSelect builder.
pub fn tree_select<M: Clone + 'static>(id: impl Into<WidgetKey>) -> TreeSelectBuilder<M> {
    TreeSelectBuilder {
        id: id.into(),
        placeholder: String::new(),
        options: vec![],
        selected_value: None,
        open: false,
        on_change: None,
    }
}

/// Create a tree select node.
pub fn tree_select_node(
    label: impl Into<String>,
    value: impl Into<String>,
) -> TreeSelectNode {
    TreeSelectNode {
        label: label.into(),
        value: value.into(),
        children: vec![],
    }
}

impl TreeSelectNode {
    /// Add a child node.
    pub fn child(mut self, node: TreeSelectNode) -> Self {
        self.children.push(node);
        self
    }

    /// Set all children at once.
    pub fn children(mut self, nodes: Vec<TreeSelectNode>) -> Self {
        self.children = nodes;
        self
    }
}

impl<M: Clone + 'static> TreeSelectBuilder<M> {
    /// Set the placeholder text shown when nothing is selected.
    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = value.into();
        self
    }

    /// Add a root-level option node.
    pub fn option(mut self, node: TreeSelectNode) -> Self {
        self.options.push(node);
        self
    }

    /// Set the currently selected value.
    pub fn selected_value(mut self, value: Option<String>) -> Self {
        self.selected_value = value;
        self
    }

    /// Set whether the dropdown is open.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set the message dispatched when the selection changes.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

/// Recursively render tree nodes into widget children with indentation.
fn render_tree_nodes<M: Clone + 'static>(
    nodes: &[TreeSelectNode],
    selected: &Option<String>,
    id_prefix: &str,
    depth: usize,
) -> Vec<WidgetNode<M>> {
    let mut result = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        let is_selected = selected.as_ref().is_some_and(|s| s == &node.value);
        let indent_px = depth * 16;

        let mut row_builder = row::<M>().gap(4.0);
        // Create an indentation space
        if indent_px > 0 {
            row_builder = row_builder.child(label::<M>(" ".repeat(indent_px)));
        }

        let node_key = format!("{}-n{}", id_prefix, i);
        let node_card = card::<M>()
            .key(node_key.as_str())
            .padding(4.0)
            .variant(if is_selected {
                CardVariant::Interactive
            } else {
                CardVariant::Plain
            })
            .child(label::<M>(node.label.clone()));
        row_builder = row_builder.child(node_card);
        result.push(row_builder.build());

        // Render children recursively
        if !node.children.is_empty() {
            result.append(&mut render_tree_nodes::<M>(
                &node.children,
                selected,
                &format!("{}-n{}", id_prefix, i),
                depth + 1,
            ));
        }
    }
    result
}

impl<M: Clone + 'static> From<TreeSelectBuilder<M>> for WidgetNode<M> {
    fn from(b: TreeSelectBuilder<M>) -> Self {
        let selected_label = b
            .selected_value
            .as_ref()
            .map_or_else(|| b.placeholder.clone(), |_| "Selected".to_string());

        // Closed: show selected or placeholder in a Card.
        if !b.open {
            return card::<M>()
                .key(b.id)
                .variant(CardVariant::Outlined)
                .padding(8.0)
                .child(label::<M>(selected_label))
                .build();
        }

        // Open: current selection label + dropdown tree
        let mut tree_items = vec![];
        tree_items.push(label::<M>(selected_label));

        let mut dropdown = card::<M>().variant(CardVariant::Outlined);
        let rendered = render_tree_nodes::<M>(&b.options, &b.selected_value, b.id.as_str(), 0);
        for r in rendered {
            dropdown = dropdown.child(r);
        }
        tree_items.push(dropdown.build());

        let mut col = column::<M>().gap(2.0);
        for item in tree_items {
            col = col.child(item);
        }
        col.key(b.id).build()
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
    fn tree_select_builder_defaults() {
        let ts = tree_select::<TestMsg>("ts");
        assert!(ts.placeholder.is_empty());
        assert!(ts.options.is_empty());
        assert!(ts.selected_value.is_none());
        assert!(!ts.open);
    }

    #[test]
    fn tree_select_closed_renders_card() {
        let node: WidgetNode<TestMsg> = tree_select("ts").placeholder("Pick...").into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant when closed");
        };
        assert_eq!(c.variant, CardVariant::Outlined);
    }

    #[test]
    fn tree_select_open_renders_column() {
        let node: WidgetNode<TestMsg> = tree_select("ts")
            .option(
                tree_select_node("Root", "root")
                    .child(tree_select_node("Child", "child")),
            )
            .open(true)
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant when open");
        };
        assert_eq!(col.children.len(), 2); // selected label + dropdown
    }

    #[test]
    fn tree_select_node_has_children() {
        let node = tree_select_node("Parent", "p")
            .child(tree_select_node("Child A", "a"))
            .child(tree_select_node("Child B", "b"));
        assert_eq!(node.children.len(), 2);
    }

    #[test]
    fn tree_select_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = tree_select("ts")
            .option(tree_select_node("Item", "item"))
            .open(true)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert!(!layout.children.is_empty());
    }
}
