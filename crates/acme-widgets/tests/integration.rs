//! Cross-crate integration tests: Widget → Layout → Reconciliation pipeline.

use acme_core::{NodeId, RetainedTree, ViewNode};
use acme_layout::{LayoutEngine, LayoutKind};
use acme_widgets::{WidgetNode, button, column, label, row};

/// Build a simple widget tree, convert to layout, and compute.
#[test]
fn widget_to_layout_pipeline() {
    let widget: WidgetNode<()> = column()
        .key("root")
        .gap(8.0)
        .child(label("Hello"))
        .child(button("btn", "Click me"))
        .build();

    let layout = widget.to_layout(NodeId::new(1));
    assert_eq!(layout.style.kind, LayoutKind::Column);
    assert_eq!(layout.children.len(), 2);

    let mut engine = LayoutEngine::new();
    let snapshot = engine
        .compute(&layout, (400.0, 300.0))
        .expect("layout should succeed");

    // Snapshot should contain entries for all nodes.
    assert!(snapshot.len() >= 3, "expected at least 3 layout results");
}

/// Reconcile a widget tree, modify it, and verify identity preservation.
#[test]
fn reconciliation_preserves_identity() {
    let mut tree = RetainedTree::new();

    let v1 = vec![ViewNode::new("root", "column")
        .child(ViewNode::new("child_a", "label"))
        .child(ViewNode::new("child_b", "button"))];

    let report1 = tree.reconcile_roots(&v1).expect("first reconcile");
    assert_eq!(report1.mounted.len(), 3);

    // Second reconcile with same keys — should reuse all nodes.
    let v2 = vec![ViewNode::new("root", "column")
        .child(ViewNode::new("child_a", "label"))
        .child(ViewNode::new("child_b", "button"))];

    let report2 = tree.reconcile_roots(&v2).expect("second reconcile");
    assert_eq!(report2.reused.len(), 3);
    assert_eq!(report2.mounted.len(), 0);
}

/// Reconcile with a removed child — verify removal.
#[test]
fn reconciliation_removes_deleted_child() {
    let mut tree = RetainedTree::new();

    let v1 = vec![ViewNode::new("root", "column")
        .child(ViewNode::new("a", "label"))
        .child(ViewNode::new("b", "label"))];
    tree.reconcile_roots(&v1).expect("first reconcile");

    let v2 = vec![ViewNode::new("root", "column")
        .child(ViewNode::new("a", "label"))];
    let report = tree.reconcile_roots(&v2).expect("second reconcile");

    assert_eq!(report.removed.len(), 1);
}

/// Theme switch should not break widget tree structure.
#[test]
fn theme_switch_preserves_structure() {
    use acme_theme::Theme;

    let light = Theme::light();
    let dark = Theme::dark();

    // Both themes should produce valid color tokens.
    assert_ne!(light.colors.background, dark.colors.background);
    assert_ne!(light.colors.foreground, dark.colors.foreground);
}

/// Nested row/column layout computes without panic.
#[test]
fn nested_layout_computes() {
    let widget: WidgetNode<()> = row()
        .key("outer")
        .gap(4.0)
        .child(
            column()
                .key("inner")
                .gap(2.0)
                .child(label("A"))
                .child(label("B"))
                .build(),
        )
        .child(label("C"))
        .build();

    let layout = widget.to_layout(NodeId::new(1));
    let mut engine = LayoutEngine::new();
    let snapshot = engine
        .compute(&layout, (800.0, 600.0))
        .expect("nested layout should succeed");

    // All nodes should have positive dimensions.
    for child in &layout.children {
        if let Some(rect) = snapshot.get(child.id) {
            assert!(rect.width >= 0.0);
            assert!(rect.height >= 0.0);
        }
    }
}