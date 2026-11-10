use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutEngine, WidgetLayoutContext};
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Clicked,
}
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
fn fab_has_elevated_surface_and_one_typed_action() {
    let defaults = float_button::<Msg>();
    assert_eq!(defaults.id.as_str(), "float_button");
    assert_eq!(defaults.size, FloatButtonSize::Normal);
    assert_eq!(defaults.shape, FloatButtonShape::Circle);
    assert_eq!(defaults.placement, FloatButtonPlacement::BottomRight);
    assert!(defaults.icon.is_none() && defaults.label.is_none() && defaults.on_click.is_none());
    for size in [
        FloatButtonSize::Mini,
        FloatButtonSize::Normal,
        FloatButtonSize::Extended,
    ] {
        for shape in [FloatButtonShape::Circle, FloatButtonShape::Square] {
            for icon in [None, Some(crate::IconName::Plus)] {
                for label in [None, Some(""), Some("新增 é👨‍👩‍👧‍👦")] {
                    for message in [None, Some(Msg::Clicked)] {
                        let mut builder = float_button().key("add").size(size).shape(shape);
                        builder.icon = icon;
                        builder.label = label.map(String::from);
                        builder.on_click = message.clone();
                        let node: WidgetNode<Msg> = builder.into();
                        let WidgetNode::Card(card) = &node else {
                            panic!("elevated Card")
                        };
                        assert_eq!(card.variant, crate::CardVariant::Elevated);
                        assert!(card.background_color.is_none());
                        assert_eq!(card.children.len(), 1);
                        let WidgetNode::Button(button) = &card.children[0] else {
                            panic!("Button")
                        };
                        assert_eq!(button.key.as_str(), "add__action");
                        assert_eq!(button.activate(), message.as_ref());
                        assert_eq!(button.variant, crate::ButtonVariant::Primary);
                        let height = if size == FloatButtonSize::Mini {
                            40.0
                        } else {
                            56.0
                        };
                        assert_eq!(
                            button.style.border_radius,
                            Some(if shape == FloatButtonShape::Circle {
                                height / 2.0
                            } else {
                                0.0
                            })
                        );
                        assert_eq!(button.style.border_radius, card.border_radius);
                        let expected = match (icon, label.filter(|s| !s.is_empty())) {
                            (Some(i), Some(s)) if size == FloatButtonSize::Extended => {
                                format!("{} {s}", i.char())
                            }
                            (Some(i), _) => i.char().into(),
                            (None, Some(s)) => s.into(),
                            _ => "+".into(),
                        };
                        assert_eq!(button.label, expected);
                        let views = widget_to_views(&[node]);
                        assert!(!views[0].focusable && views[0].children[0].focusable);
                    }
                }
            }
        }
    }
}
#[test]
fn fab_sizes_are_real_and_long_extended_labels_stay_bounded() {
    let mut fonts = acme_text::FontSystem::new();
    for width in [120.0, 240.0, 600.0] {
        for size in [
            FloatButtonSize::Mini,
            FloatButtonSize::Normal,
            FloatButtonSize::Extended,
        ] {
            let node: WidgetNode<Msg> = crate::row()
                .width(width)
                .child(
                    float_button()
                        .size(size)
                        .label("A very long action · 請新增項目並繼續下一步")
                        .on_click(Msg::Clicked),
                )
                .build();
            let layout = node.to_layout_with_context(NodeId::new(1), &context());
            let snapshot = LayoutEngine::new()
                .compute_with_text(&layout, (width, 300.0), &mut fonts, 1.0)
                .unwrap();
            let surface = snapshot.get(layout.children[0].id).unwrap();
            let action = snapshot.get(layout.children[0].children[0].id).unwrap();
            let expected_height = if size == FloatButtonSize::Mini {
                40.0
            } else {
                56.0
            };
            assert_eq!(surface.height, expected_height);
            assert_eq!(action.height, expected_height);
            assert_eq!(surface.width, action.width);
            assert!(action.width >= expected_height && action.width <= width);
            if size != FloatButtonSize::Extended {
                assert_eq!(action.width, expected_height);
            }
        }
    }
}
#[test]
fn fab_surface_and_action_identity_survive_size_content_and_reorder() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for changed in [false, true, false] {
        let a: WidgetNode<Msg> = float_button()
            .key("a")
            .size(if changed {
                FloatButtonSize::Extended
            } else {
                FloatButtonSize::Mini
            })
            .icon(crate::IconName::Plus)
            .label(if changed { "新增" } else { "Add" })
            .on_click(Msg::Clicked)
            .into();
        let b: WidgetNode<Msg> = float_button().key("b").into();
        let roots = if changed { vec![b, a] } else { vec![a, b] };
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let ids = ["a", "b"].map(|key| {
            let root = roots
                .iter()
                .find(|n| n.key().unwrap().as_str() == key)
                .unwrap();
            [
                binding.id_for(root).unwrap(),
                binding.id_for(&root.children()[0]).unwrap(),
            ]
        });
        assert_ne!(ids[0][1], ids[1][1]);
        if let Some(previous) = saved {
            assert_eq!(ids, previous);
        }
        saved = Some(ids);
    }
}
