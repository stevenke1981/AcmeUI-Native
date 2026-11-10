use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutEngine, LayoutKind, WidgetLayoutContext};
use acme_widgets::style::{ColorToken, Styled};
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};
fn context() -> WidgetLayoutContext {
    WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 20.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: 1.0,
    }
}
#[test]
fn defaults_and_numeric_public_widths_produce_visible_finite_fixed_bars() {
    let b = mobile_sheet_handle::<()>();
    assert_eq!(b.id.as_str(), "mobile_sheet_handle");
    assert_eq!(b.width, 36.0);
    for (width, expected) in [
        (36.0, 36.0),
        (48.0, 48.0),
        (0.0, 0.0),
        (-10.0, 0.0),
        (f32::NAN, 36.0),
        (f32::INFINITY, 36.0),
        (f32::NEG_INFINITY, 36.0),
    ] {
        let mut b = mobile_sheet_handle::<()>().key("handle");
        b.width = width;
        let node: WidgetNode<()> = b.into();
        let WidgetNode::Row(row) = &node else {
            panic!("Row")
        };
        assert!(row.children.is_empty() && row.message.is_none());
        assert_eq!(row.style.background, Some(ColorToken::MutedForeground));
        assert_eq!(row.style.border_radius, Some(2.0));
        let layout = node.to_layout_with_context(NodeId::new(1), &context());
        assert_eq!(layout.style.kind, LayoutKind::Row);
        let snapshot = LayoutEngine::new()
            .compute(&layout, (180.0, 100.0))
            .unwrap();
        let r = snapshot.get(layout.id).unwrap();
        assert_eq!((r.width, r.height), (expected, 4.0));
    }
}
#[test]
fn handle_keeps_its_size_beside_a_flexible_sibling() {
    for width in [48.0, 180.0, 320.0] {
        let node: WidgetNode<()> = crate::row()
            .width(width)
            .child(mobile_sheet_handle().width(36.0))
            .child(
                crate::row()
                    .w(acme_layout::Length::px(200.0))
                    .flex_shrink(1.0),
            )
            .build();
        let layout = node.to_layout_with_context(NodeId::new(1), &context());
        let snapshot = LayoutEngine::new()
            .compute(&layout, (width, 100.0))
            .unwrap();
        let r = snapshot.get(layout.children[0].id).unwrap();
        assert_eq!((r.width, r.height), (36.0, 4.0));
    }
}
#[test]
fn width_changes_and_keyed_reorder_keep_retained_handle_ids() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for changed in [false, true, false] {
        let a: WidgetNode<()> = mobile_sheet_handle()
            .key("a")
            .width(if changed { 48.0 } else { 36.0 })
            .into();
        let b: WidgetNode<()> = mobile_sheet_handle().key("b").into();
        let roots = if changed { vec![b, a] } else { vec![a, b] };
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let ids = ["a", "b"].map(|key| {
            binding
                .id_for(
                    roots
                        .iter()
                        .find(|n| n.key().unwrap().as_str() == key)
                        .unwrap(),
                )
                .unwrap()
        });
        if let Some(previous) = saved {
            assert_eq!(ids, previous);
        }
        saved = Some(ids);
    }
}
