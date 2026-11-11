use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_layout::{Edges, LayoutEngine, WidgetLayoutContext};
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

fn sample(orientation: DataListOrientation, density: DataListDensity) -> WidgetNode<()> {
    data_list()
        .key("profile")
        .orientation(orientation)
        .density(density)
        .item(DataListItem::new("Name", "陳小明"))
        .item(DataListItem::new("Role", "Developer"))
        .into()
}

fn context(scale: f32) -> WidgetLayoutContext {
    WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 20.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: scale,
    }
}

#[test]
fn default_density_preserves_existing_content_spacing_and_empty_behavior() {
    assert_eq!(data_list::<()>().density, DataListDensity::Default);
    for orientation in [
        DataListOrientation::Vertical,
        DataListOrientation::Horizontal,
    ] {
        let node = sample(orientation, DataListDensity::Default);
        let WidgetNode::Column(root) = &node else {
            panic!("Column")
        };
        assert_eq!(root.gap, 8.0);
        assert_eq!(root.padding, Edges::all(4.0));
        assert!(root.message.is_none());
        assert!(root.style.background.is_none());
        assert_eq!(root.children.len(), 2);
        let pair = match &root.children[0] {
            WidgetNode::Row(pair) if orientation == DataListOrientation::Horizontal => pair,
            WidgetNode::Column(pair) if orientation == DataListOrientation::Vertical => pair,
            _ => panic!("requested orientation"),
        };
        assert_eq!(
            pair.gap,
            if orientation == DataListOrientation::Horizontal {
                12.0
            } else {
                2.0
            }
        );
        let text: Vec<_> = pair
            .children
            .iter()
            .map(|node| match node {
                WidgetNode::Label(label) => {
                    assert!(label.color.is_none());
                    label.text.as_str()
                }
                _ => panic!("Label"),
            })
            .collect();
        assert_eq!(text, ["Name", "陳小明"]);
    }
    let mut builder = data_list::<()>().density(DataListDensity::Relaxed);
    builder.density = DataListDensity::Compact;
    let empty: WidgetNode<()> = builder.into();
    assert!(empty.children().is_empty());
    assert_eq!(empty.key().unwrap().as_str(), "data_list");
    let node: WidgetNode<()> = data_list().item(DataListItem::new("", "")).into();
    assert_eq!(node.children()[0].children().len(), 2);
}

#[test]
fn densities_change_real_extent_without_overlapping_or_losing_content() {
    let mut fonts = acme_text::FontSystem::new();
    for orientation in [
        DataListOrientation::Vertical,
        DataListOrientation::Horizontal,
    ] {
        for scale in [1.0, 1.25, 2.0] {
            for width in [240.0, 400.0] {
                let mut heights = Vec::new();
                for density in [
                    DataListDensity::Compact,
                    DataListDensity::Default,
                    DataListDensity::Relaxed,
                ] {
                    let node = crate::column()
                        .width(width)
                        .child(sample(orientation, density))
                        .build();
                    let layout = node.to_layout_with_context(NodeId::new(1), &context(scale));
                    let snapshot = LayoutEngine::new()
                        .compute_with_text(&layout, (width, 600.0), &mut fonts, scale)
                        .unwrap();
                    let list = &layout.children[0];
                    let outer = snapshot.get(list.id).unwrap();
                    heights.push(outer.height);
                    let first = snapshot.get(list.children[0].id).unwrap();
                    let second = snapshot.get(list.children[1].id).unwrap();
                    assert!(second.y > first.y + first.height);
                    assert!(outer.width <= width && outer.height.is_finite());
                    for item in &list.children {
                        let rect = snapshot.get(item.id).unwrap();
                        assert!(rect.x >= outer.x && rect.x + rect.width <= outer.x + outer.width);
                        assert!(
                            rect.y >= outer.y && rect.y + rect.height <= outer.y + outer.height
                        );
                        for label in &item.children {
                            let text = snapshot.get(label.id).unwrap();
                            assert!(text.width > 0.0 && text.height > 0.0);
                        }
                    }
                }
                assert!(
                    heights[0] < heights[1] && heights[1] < heights[2],
                    "{orientation:?} {scale}: {heights:?}"
                );
            }
        }
    }
}

#[test]
fn density_changes_and_keyed_instance_reorders_keep_all_retained_ids() {
    for orientation in [
        DataListOrientation::Vertical,
        DataListOrientation::Horizontal,
    ] {
        let mut tree = RetainedTree::new();
        let mut saved = None;
        for (density, reverse) in [
            (DataListDensity::Default, false),
            (DataListDensity::Compact, true),
            (DataListDensity::Relaxed, false),
        ] {
            let mut roots = vec![
                sample(orientation, density),
                data_list()
                    .key("other")
                    .item(DataListItem::new("Other", "Value"))
                    .into(),
            ];
            if reverse {
                roots.reverse();
            }
            tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
            let binding = bind_retained(&roots, &tree);
            assert_eq!(binding.mismatches(), 0);
            let node = &roots[usize::from(reverse)];
            let layout = node.to_layout_with_binding(&binding, &context(1.0));
            fn ids(node: &acme_layout::LayoutNode, out: &mut Vec<NodeId>) {
                out.push(node.id);
                for child in &node.children {
                    ids(child, out);
                }
            }
            let mut current = Vec::new();
            ids(&layout, &mut current);
            assert_eq!(current.len(), 7);
            if let Some(previous) = &saved {
                assert_eq!(&current, previous);
            } else {
                saved = Some(current);
            }
        }
    }
}
