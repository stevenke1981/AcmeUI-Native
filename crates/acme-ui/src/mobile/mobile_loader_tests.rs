use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutEngine, WidgetLayoutContext};
use acme_widgets::LabelWrap;
use acme_widgets::retained_binding::bind_retained;
use acme_widgets::view_bridge::widget_to_views;

#[derive(Clone)]
enum Msg {}

#[test]
fn defaults_optional_messages_and_public_field_edits_preserve_the_indicator() {
    let defaults = mobile_loader::<Msg>();
    assert_eq!(defaults.id.as_str(), "mobile_loader");
    assert!(defaults.message.is_none());
    for message in [None, Some(""), Some("載入中…"), Some("Loading…")] {
        let mut builder = mobile_loader::<Msg>().key("request");
        // Public builder fields remain usable in addition to the chainable API.
        builder.message = message.map(String::from);
        let node: WidgetNode<Msg> = builder.into();
        let WidgetNode::Column(column) = &node else {
            panic!("loader must be a column");
        };
        assert_eq!(node.key().unwrap().as_str(), "request");
        assert_eq!(column.padding, acme_layout::Edges::all(24.0));
        assert_eq!(column.gap, 8.0);
        assert!(column.message.is_none());
        assert!(column.style.background.is_none());
        assert_eq!(column.children.len(), 1 + usize::from(message.is_some()));
        let WidgetNode::Label(indicator) = &column.children[0] else {
            panic!("expected foundation spinner indicator");
        };
        assert_eq!(indicator.text, "⟳");
        assert!(indicator.font_size.is_some_and(|size| size > 0.0));
        assert!(indicator.color.is_none());
        if let Some(text) = message {
            let WidgetNode::Label(label) = &column.children[1] else {
                panic!("message must follow the indicator");
            };
            assert_eq!(label.text, text);
            assert!(label.color.is_none());
        }
        assert!(
            widget_to_views(&[node])[0]
                .children
                .iter()
                .all(|v| !v.focusable)
        );
    }
}

#[test]
fn long_messages_wrap_inside_the_loader_at_narrow_and_wide_widths() {
    let mut fonts = acme_text::FontSystem::new();
    for width in [120.0, 240.0, 400.0] {
        for text in [
            "Loading the next page. Please wait while the request finishes.",
            "LoadingWithoutSpacesLoadingWithoutSpacesLoadingWithoutSpaces",
            "正在載入下一頁，請稍候，資料準備完成後將自動顯示。",
        ] {
            let node: WidgetNode<Msg> = mobile_loader().message(text).into();
            let WidgetNode::Label(message) = &node.children()[1] else {
                panic!("message label");
            };
            assert_eq!(message.wrap, LabelWrap::Character);
            let layout = node.to_layout_with_context(
                NodeId::new(1),
                &WidgetLayoutContext {
                    body_font_size: 14.0,
                    body_line_height: 20.0,
                    label_font_size: 13.0,
                    control_height: 36.0,
                    scale_factor: 1.0,
                },
            );
            let snapshot = LayoutEngine::new()
                .compute_with_text(&layout, (width, 1000.0), &mut fonts, 1.0)
                .unwrap();
            let outer = snapshot.get(layout.id).unwrap();
            let indicator = snapshot.get(layout.children[0].id).unwrap();
            let message = snapshot.get(layout.children[1].id).unwrap();
            assert!(outer.width <= width && outer.width > 48.0);
            assert_eq!(message.width, outer.width - 48.0);
            assert_eq!(message.x, outer.x + 24.0);
            assert!(message.y >= indicator.y + indicator.height + 8.0);
            assert!(message.y + message.height <= outer.y + outer.height - 24.0 + 1.0);
            if width == 120.0 {
                assert!(
                    message.height > 40.0,
                    "long message must occupy multiple lines"
                );
            }
        }
    }
}

#[test]
fn omission_and_message_changes_keep_loader_and_indicator_identity_when_reordered() {
    let mut tree = RetainedTree::default();
    let mut previous = None;
    for (message, reverse) in [
        (None, false),
        (Some("Loading"), true),
        (Some("載入中"), false),
        (None, true),
    ] {
        let mut first = mobile_loader::<Msg>().key("first");
        if let Some(text) = message {
            first = first.message(text);
        }
        let first = WidgetNode::from(first);
        let second = WidgetNode::from(mobile_loader::<Msg>().key("second").message("Waiting"));
        let roots = if reverse {
            vec![second, first]
        } else {
            vec![first, second]
        };
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let ids = ["first", "second"].map(|key| {
            let node = roots
                .iter()
                .find(|n| n.key().unwrap().as_str() == key)
                .unwrap();
            [
                binding.id_for(node).unwrap(),
                binding.id_for(&node.children()[0]).unwrap(),
            ]
        });
        assert_ne!(ids[0][0], ids[1][0]);
        assert_ne!(ids[0][1], ids[1][1]);
        if let Some(previous) = previous {
            assert_eq!(ids, previous);
        }
        previous = Some(ids);
    }
}
