use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutEngine, WidgetLayoutContext};
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
fn square_is_independent_of_elevation_outline_and_setter_order() {
    assert!(!paper::<()>().square);
    for elevation in [
        PaperElevation::E0,
        PaperElevation::E1,
        PaperElevation::E2,
        PaperElevation::E4,
        PaperElevation::E8,
        PaperElevation::E16,
        PaperElevation::E24,
    ] {
        for square in [false, true] {
            for outlined in [false, true] {
                let mut builder = paper::<()>()
                    .key("surface")
                    .square(!square)
                    .elevation(elevation)
                    .outlined(outlined)
                    .child(crate::label("內容"));
                assert_eq!(builder.square, !square);
                builder.square = square;
                let node: WidgetNode<()> = builder.into();
                let WidgetNode::Card(card) = node else {
                    panic!("Card")
                };
                assert_eq!(card.key.unwrap().as_str(), "surface");
                assert_eq!(
                    card.border_radius,
                    Some(if square { 0.0 } else { elevation.radius() })
                );
                assert_eq!(
                    card.variant,
                    if outlined {
                        acme_widgets::CardVariant::Outlined
                    } else {
                        acme_widgets::CardVariant::Elevated
                    }
                );
                assert_eq!(card.padding, acme_layout::Edges::all(16.0));
                assert_eq!(card.children.len(), 1);
                assert!(card.background_color.is_none());
            }
        }
    }
    let node: WidgetNode<()> = paper()
        .elevation(PaperElevation::E8)
        .square(true)
        .square(false)
        .into();
    let WidgetNode::Card(card) = node else {
        panic!("Card")
    };
    assert_eq!(card.border_radius, Some(12.0));
}

#[test]
fn changing_corners_keeps_real_layout_and_retained_content_identity() {
    let mut tree = RetainedTree::new();
    let mut fonts = acme_text::FontSystem::new();
    let mut saved = None;
    for square in [false, true, false] {
        let node: WidgetNode<()> = paper()
            .key("surface")
            .square(square)
            .elevation(PaperElevation::E8)
            .child(crate::label("Paper 方形"))
            .into();
        let roots = [node];
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let layout = roots[0].to_layout_with_binding(&binding, &context());
        let ids = (layout.id, layout.children[0].id);
        if let Some(old) = saved {
            assert_eq!(ids, old);
        } else {
            saved = Some(ids);
        }
        for width in [240.0, 400.0] {
            let parent = crate::column().width(width).child(roots[0].clone()).build();
            let layout = parent.to_layout_with_context(NodeId::new(1), &context());
            let snapshot = LayoutEngine::new()
                .compute_with_text(&layout, (width, 300.0), &mut fonts, 1.0)
                .unwrap();
            let card = snapshot.get(layout.children[0].id).unwrap();
            let text = snapshot.get(layout.children[0].children[0].id).unwrap();
            assert_eq!(card.width, width);
            assert_eq!((text.x, text.y), (card.x + 16.0, card.y + 16.0));
            assert!(text.width <= card.width - 32.0);
        }
    }
}
