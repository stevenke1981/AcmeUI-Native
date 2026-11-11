use super::*;
use acme_core::NodeId;
use acme_layout::{LayoutEngine, WidgetLayoutContext};

#[test]
fn spinner_tones_use_semantic_tokens_without_actions() {
    for (tone, token) in [
        (crate::Tone::Neutral, ColorToken::Foreground),
        (crate::Tone::Primary, ColorToken::Primary),
        (crate::Tone::Success, ColorToken::Success),
        (crate::Tone::Warning, ColorToken::Warning),
        (crate::Tone::Danger, ColorToken::Danger),
        (crate::Tone::Info, ColorToken::Info),
    ] {
        let node = spinner::<()>().tone(tone).build();
        for dot in node.children() {
            let WidgetNode::Row(dot) = dot else {
                panic!("dot surface");
            };
            assert_eq!(dot.style.background, Some(token));
            assert!(dot.message.is_none());
        }
    }
}

#[test]
fn spinner_rotates_periodically_and_reduced_motion_has_a_fixed_pose() {
    let build = |ms, reduced| {
        spinner::<()>()
            .animation(Duration::from_millis(ms), reduced)
            .build()
    };
    let initial = build(0, false);
    assert_ne!(initial, build(250, false));
    assert_eq!(initial, build(1000, false));
    assert_eq!(initial, build(250, true));
    assert_eq!(
        initial,
        spinner::<()>().animation(Duration::MAX, true).build()
    );
    assert_eq!(initial.children().len(), 7);
    let WidgetNode::Row(top) = &initial.children()[0] else {
        panic!("head");
    };
    let quarter = build(250, false);
    let WidgetNode::Row(right) = &quarter.children()[0] else {
        panic!("head");
    };
    let top_margin = top.style.margin.unwrap();
    let right_margin = right.style.margin.unwrap();
    assert!(right_margin.left > top_margin.left && right_margin.top > top_margin.top);
    let maximum = spinner::<()>().animation(Duration::MAX, false).build();
    for dot in maximum.children() {
        let WidgetNode::Row(dot) = dot else {
            panic!("dot");
        };
        let margin = dot.style.margin.unwrap();
        assert!(margin.left.is_finite() && margin.top.is_finite());
    }
}

#[test]
fn spinner_dots_stay_inside_fixed_bounds_at_every_size_and_phase() {
    let context = WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 20.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: 1.0,
    };
    for size in [
        crate::ControlSize::Xs,
        crate::ControlSize::Sm,
        crate::ControlSize::Md,
        crate::ControlSize::Lg,
        crate::ControlSize::Xl,
    ] {
        let mut previous = None;
        for ms in [0, 33, 125, 250, 499, 750, 999] {
            let widget = spinner::<()>()
                .size(size)
                .animation(Duration::from_millis(ms), false)
                .build();
            let layout = widget.to_layout_with_context(NodeId::new(1), &context);
            let snapshot = LayoutEngine::new()
                .compute(&layout, (120.0, 120.0))
                .unwrap();
            let outer = snapshot.get(layout.id).unwrap();
            let bounds = (outer.width, outer.height);
            assert!(outer.width > 0.0 && outer.width == outer.height);
            if let Some(previous) = previous {
                assert_eq!(bounds, previous);
            }
            previous = Some(bounds);
            for dot in &layout.children {
                let rect = snapshot.get(dot.id).unwrap();
                assert!(rect.width > 0.0 && rect.height > 0.0);
                assert!(rect.x >= outer.x && rect.y >= outer.y);
                assert!(rect.x + rect.width <= outer.x + outer.width);
                assert!(rect.y + rect.height <= outer.y + outer.height);
            }
        }
    }
}
