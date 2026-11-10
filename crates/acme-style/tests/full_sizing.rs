use acme_core::NodeId;
use acme_layout::{Edges, LayoutEngine, LayoutNode, LayoutStyle, Length};
use acme_style::{Style, Styled, h, w};

#[derive(Default)]
struct BoxStyle(Style);

impl Styled for BoxStyle {
    fn style(&self) -> &Style {
        &self.0
    }

    fn style_mut(&mut self) -> &mut Style {
        &mut self.0
    }
}

#[test]
fn full_width_root_matches_the_viewport() {
    let mut style = LayoutStyle::column();
    BoxStyle::default()
        .w_full()
        .h(Length::px(40.0))
        .0
        .apply_to_layout(&mut style);
    let root = LayoutNode::leaf(NodeId::new(1), style);
    let snapshot = LayoutEngine::new().compute(&root, (240.0, 120.0)).unwrap();
    assert_eq!(snapshot.get(root.id).unwrap().width, 240.0);
}

#[test]
fn full_helpers_fill_the_parent_content_box_in_both_dimensions() {
    for (width, height) in [(240.0, 120.0), (400.0, 200.0), (800.0, 600.0)] {
        for full in [
            BoxStyle::default().w_full().h_full(),
            BoxStyle::default().w(w::full()).h(h::full()),
            BoxStyle::default().w(w::screen()).h(h::screen()),
        ] {
            let mut style = LayoutStyle::default();
            full.flex_shrink(0.0).0.apply_to_layout(&mut style);
            let child = LayoutNode::leaf(NodeId::new(2), style);
            let parent = LayoutNode::container(
                NodeId::new(1),
                LayoutStyle {
                    width: Length::px(width),
                    height: Length::px(height),
                    padding: Edges::all(12.0),
                    ..LayoutStyle::column()
                },
                vec![child],
            );
            let snapshot = LayoutEngine::new()
                .compute(&parent, (1000.0, 800.0))
                .unwrap();
            let rect = snapshot.get(NodeId::new(2)).unwrap();
            assert_eq!((rect.x, rect.y), (12.0, 12.0));
            assert_eq!((rect.width, rect.height), (width - 24.0, height - 24.0));
        }
    }
}
