use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutEngine, WidgetLayoutContext};
use acme_widgets::retained_binding::bind_retained;
use acme_widgets::view_bridge::widget_to_views;

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Submit,
    Clear,
    Edited,
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
fn search_contains_a_real_focusable_input_and_themed_rounded_surface() {
    let node: WidgetNode<Msg> = mobile_search("Search · 搜尋").key("query").into();
    let WidgetNode::Row(row) = &node else {
        panic!("search Row")
    };
    assert!(
        row.style.background.is_some(),
        "search needs a themed surface"
    );
    assert!(row.style.border_radius.is_some_and(|r| r > 0.0));
    let WidgetNode::TextInput(input) = &row.children[1] else {
        panic!("editable TextInput, not placeholder Label")
    };
    assert_eq!(input.key.as_str(), "query__input");
    assert_eq!(input.placeholder.as_deref(), Some("Search · 搜尋"));
    let views = widget_to_views(&[node]);
    assert!(views[0].children[1].focusable);
}

#[test]
fn keyed_search_inputs_survive_optional_actions_and_instance_reordering() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for reverse in [false, true, false] {
        let mut a = mobile_search("A").key("a");
        if reverse {
            a = a.on_clear(Msg::Clear).on_submit(Msg::Submit);
        }
        let a: WidgetNode<Msg> = a.into();
        let b: WidgetNode<Msg> = mobile_search("B").key("b").on_clear(Msg::Clear).into();
        let roots = if reverse { vec![b, a] } else { vec![a, b] };
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let mut ids = Vec::new();
        for key in ["a", "b"] {
            let node = roots
                .iter()
                .find(|n| n.key().unwrap().as_str() == key)
                .unwrap();
            let layout = node.to_layout_with_binding(&binding, &context());
            assert!(matches!(node.children()[1], WidgetNode::TextInput(_)));
            ids.push(layout.children[1].id);
            if let Some(WidgetNode::Button(clear)) = node.children().get(2) {
                assert_eq!(clear.key.as_str(), format!("{key}__clear"));
                assert_eq!(clear.activate(), Some(&Msg::Clear));
            }
        }
        if let Some(previous) = &saved {
            assert_eq!(&ids, previous);
        } else {
            saved = Some(ids);
        }
    }
}

#[test]
fn input_fills_remaining_width_without_overlapping_optional_actions() {
    let mut fonts = acme_text::FontSystem::new();
    for width in [240.0, 400.0, 640.0] {
        for clear in [false, true] {
            let mut builder = mobile_search("搜尋關鍵字")
                .key("query")
                .on_submit(Msg::Submit);
            if clear {
                builder = builder.on_clear(Msg::Clear);
            }
            let node: WidgetNode<Msg> = builder.into();
            let layout = node.to_layout_with_context(NodeId::new(1), &context());
            let snapshot = LayoutEngine::new()
                .compute_with_text(&layout, (width, 100.0), &mut fonts, 1.0)
                .unwrap();
            let r = snapshot.get(layout.id).unwrap();
            let input = snapshot.get(layout.children[1].id).unwrap();
            assert!(input.width >= 40.0, "usable input width: {input:?}");
            assert!(input.height >= 44.0);
            assert!(r.width <= width);
            for pair in layout.children.windows(2) {
                let a = snapshot.get(pair[0].id).unwrap();
                let b = snapshot.get(pair[1].id).unwrap();
                assert!(a.x + a.width <= b.x);
            }
            let last = snapshot.get(layout.children.last().unwrap().id).unwrap();
            assert!(last.x + last.width <= r.x + r.width);
        }
    }
}

#[test]
fn controlled_values_and_input_markers_do_not_submit_or_clear() {
    let defaults = mobile_search::<Msg>("Find");
    assert_eq!(defaults.id.as_str(), "mobile_search");
    assert!(defaults.value.is_empty() && defaults.on_input.is_none());
    for query in ["", "coffee", "搜尋👨‍👩‍👧‍👦é"] {
        let mut builder = mobile_search("Find")
            .value(query)
            .on_input(Msg::Edited)
            .on_submit(Msg::Submit)
            .on_clear(Msg::Clear);
        builder.placeholder = "尋找".into();
        let node: WidgetNode<Msg> = builder.into();
        let WidgetNode::Row(row) = &node else {
            panic!("row")
        };
        let WidgetNode::TextInput(input) = &row.children[1] else {
            panic!("input")
        };
        assert_eq!(input.value, query);
        assert_eq!(input.placeholder.as_deref(), Some("尋找"));
        assert_eq!(input.message, Some(Msg::Edited));
        assert_eq!(row.message, Some(Msg::Submit));
        for (index, action) in [(0, Msg::Submit), (2, Msg::Clear)] {
            let WidgetNode::Button(button) = &row.children[index] else {
                panic!("action")
            };
            assert_eq!(button.activate(), Some(&action));
        }
    }
}
