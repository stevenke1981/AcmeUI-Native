use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutEngine, WidgetLayoutContext};
use acme_widgets::retained_binding::bind_retained;
use acme_widgets::style::ColorToken;
use acme_widgets::view_bridge::widget_to_views;

#[derive(Clone, Debug, PartialEq)]
enum Msg {}

fn keyed<'a>(node: &'a WidgetNode<Msg>, key: &str) -> &'a WidgetNode<Msg> {
    fn find<'a>(node: &'a WidgetNode<Msg>, key: &str) -> Option<&'a WidgetNode<Msg>> {
        if node.key().is_some_and(|k| k.as_str() == key) {
            Some(node)
        } else {
            node.children().iter().find_map(|child| find(child, key))
        }
    }
    find(node, key).expect("progress part must exist")
}

#[test]
fn defaults_and_optional_label_preserve_readable_percentage_and_theme_tokens() {
    let builder = mobile_progress::<Msg>(1.0, 4.0);
    assert_eq!(builder.id.as_str(), "mobile_progress");
    assert_eq!((builder.value, builder.total), (1.0, 4.0));
    assert!(builder.label.is_none());
    for description in [None, Some("下載進度")] {
        let mut builder = mobile_progress(1.0, 4.0).key("download");
        if let Some(text) = description {
            builder = builder.label(text);
        }
        let node: WidgetNode<Msg> = builder.into();
        assert!(matches!(node, WidgetNode::Column(_)));
        let children = node.children();
        assert_eq!(children.len(), 2 + usize::from(description.is_some()));
        if let Some(text) = description {
            let WidgetNode::Label(label) = &children[0] else {
                panic!("description must precede the track");
            };
            assert_eq!(label.text, text);
            assert!(label.color.is_none());
        }
        let WidgetNode::Label(label) = children.last().unwrap() else {
            panic!("percentage must be separate from the thin track");
        };
        assert_eq!(label.text, "25%");
        assert!(label.color.is_none());
        for (key, token) in [
            ("download__track", ColorToken::Muted),
            ("download__fill", ColorToken::Primary),
        ] {
            let WidgetNode::Row(part) = keyed(&node, key) else {
                panic!("expected styled row");
            };
            assert_eq!(part.style.background, Some(token));
            assert!(part.message.is_none());
        }
    }
}

#[test]
fn numeric_boundaries_produce_finite_clamped_fractions_even_after_public_field_edits() {
    assert!(
        mobile_progress::<Msg>(f32::NAN, 100.0)
            .fraction()
            .is_finite()
    );
    for (value, total, expected) in [
        (0.0, 100.0, 0.0),
        (25.0, 100.0, 0.25),
        (200.0, 100.0, 1.0),
        (-10.0, 100.0, 0.0),
        (0.5, 0.0, 0.5),
        (0.5, -4.0, 0.5),
        (0.5, f32::NAN, 0.5),
        (0.5, f32::INFINITY, 0.5),
        (0.5, f32::NEG_INFINITY, 0.5),
        (f32::NAN, 100.0, 0.0),
        (f32::INFINITY, 100.0, 1.0),
        (f32::NEG_INFINITY, 100.0, 0.0),
        (f32::MAX, f32::MIN_POSITIVE, 1.0),
        (f32::MAX, f32::MAX, 1.0),
    ] {
        for edit_fields in [false, true] {
            let mut builder = mobile_progress::<Msg>(value, total);
            if edit_fields {
                builder.value = value;
                builder.total = total;
            }
            assert_eq!(builder.fraction(), expected, "value={value}, total={total}");
            let node: WidgetNode<Msg> = builder.into();
            let WidgetNode::Label(label) = node.children().last().unwrap() else {
                panic!("expected percentage");
            };
            assert_eq!(label.text, format!("{}%", (expected * 100.0) as u32));
        }
    }
}

#[test]
fn actual_layout_scales_fill_with_available_width_and_keeps_text_outside_the_track() {
    let context = WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 20.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: 1.0,
    };
    for width in [120.0, 240.0, 400.0] {
        for fraction in [0.0, 0.01, 0.25, 0.75, 1.0] {
            for description in [false, true] {
                let mut builder = mobile_progress(fraction * 100.0, 100.0).key("job");
                if description {
                    builder = builder.label("Downloading");
                }
                let node: WidgetNode<Msg> = builder.into();
                let layout = node.to_layout_with_context(NodeId::new(1), &context);
                let snapshot = LayoutEngine::new()
                    .compute(&layout, (width, 200.0))
                    .unwrap();
                let outer = snapshot.get(layout.id).unwrap();
                let track_layout = &layout.children[usize::from(description)];
                let track = snapshot.get(track_layout.id).unwrap();
                let fill = snapshot.get(track_layout.children[0].id).unwrap();
                assert_eq!(outer.width, width);
                assert_eq!(track.width, width - 16.0);
                assert_eq!(track.x, outer.x + 8.0);
                assert_eq!((track.height, fill.height), (6.0, 6.0));
                // Taffy's default layout rounds final edges to logical pixels.
                assert_eq!(fill.width, (track.width * fraction).round());
                assert_eq!((fill.x, fill.y), (track.x, track.y));
                let percent = snapshot.get(layout.children.last().unwrap().id).unwrap();
                assert!(percent.y >= track.y + track.height + 4.0);
                if description {
                    let label = snapshot.get(layout.children[0].id).unwrap();
                    assert!(track.y >= label.y + label.height + 4.0);
                }
                assert!(fill.x + fill.width <= track.x + track.width + 0.001);
            }
        }
    }
}

#[test]
fn optional_label_and_value_changes_preserve_track_and_fill_identity() {
    let mut tree = RetainedTree::default();
    let mut previous = None;
    for (label, value) in [(false, 0.0), (true, 25.0), (false, 100.0)] {
        let mut builder = mobile_progress(value, 100.0).key("job");
        if label {
            builder = builder.label("Progress");
        }
        let node: WidgetNode<Msg> = builder.into();
        tree.reconcile_roots(&widget_to_views(std::slice::from_ref(&node)))
            .unwrap();
        let binding = bind_retained(std::slice::from_ref(&node), &tree);
        assert_eq!(binding.mismatches(), 0);
        let ids = ["job", "job__track", "job__fill"]
            .map(|key| binding.id_for(keyed(&node, key)).unwrap());
        if let Some(previous) = previous {
            assert_eq!(ids, previous);
        }
        previous = Some(ids);
    }
}
