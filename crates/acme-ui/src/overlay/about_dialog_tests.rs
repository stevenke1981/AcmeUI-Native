use super::*;
use acme_core::RetainedTree;
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

fn close(node: &WidgetNode<u8>) -> &Button<u8> {
    let WidgetNode::Card(card) = node else {
        panic!("existing card root");
    };
    let WidgetNode::Button(button) = card.children[0].children().last().unwrap() else {
        panic!("Close Button");
    };
    button
}

#[test]
fn converted_close_carries_current_message_and_preserves_passive_fallback() {
    for message in [None, Some(7), Some(9)] {
        let mut builder = about_dialog("about", "程式")
            .version("1.0")
            .open(true)
            .description("Description")
            .credits("Credits");
        if let Some(message) = message {
            builder = builder.on_close(message);
        }
        let node: WidgetNode<u8> = builder.into();
        let button = close(&node);
        assert_eq!(button.activate().copied(), message);
        assert_eq!(button.key.as_str(), "about_close");
        assert_eq!(button.label, "Close");
        let content = &node.children()[0];
        assert_eq!(content.key().unwrap().as_str(), "about");
        assert!(matches!(&content.children()[1],WidgetNode::Label(label) if label.text=="程式"));
    }
}

#[test]
fn close_identity_survives_optional_description_and_credits_changes() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for optional in [false, true, false] {
        let mut builder = about_dialog("about", "App").open(true).on_close(1_u8);
        if optional {
            builder = builder.description("說明").credits("Credits");
        }
        let roots = [WidgetNode::from(builder)];
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let child = roots[0].children()[0].children().last().unwrap();
        let id = binding.id_for(child).unwrap();
        if let Some(saved) = saved {
            assert_eq!(id, saved);
        }
        saved = Some(id);
        assert_eq!(close(&roots[0]).activate(), Some(&1));
    }
}
