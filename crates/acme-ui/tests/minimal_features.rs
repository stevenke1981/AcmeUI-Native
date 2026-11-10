//! External consumer coverage, also run with one feature and no defaults.

#[cfg(feature = "inputs")]
#[test]
fn inputs_exposes_its_icon_dependency_and_preserves_activation() {
    use acme_ui::{IconName, WidgetNode, inputs::icon_button};
    let node: WidgetNode<&str> = icon_button("search", IconName::Search)
        .on_click("search")
        .into();
    let WidgetNode::Button(button) = node else {
        panic!("Button")
    };
    assert!(!button.label.is_empty());
    assert_eq!(button.activate(), Some(&"search"));
}

#[cfg(feature = "layout")]
#[test]
fn layout_exposes_form_validation_without_extra_feature_flags() {
    use acme_ui::{
        WidgetNode, label,
        layout::{form, form_field},
    };
    let node = form::<()>("account")
        .field(
            form_field("name", "Name")
                .required(true)
                .content(label("Alice"))
                .error("Required"),
        )
        .build();
    fn labels<'a>(node: &'a WidgetNode<()>, out: &mut Vec<&'a str>) {
        if let WidgetNode::Label(label) = node {
            out.push(&label.text);
        }
        for child in node.children() {
            labels(child, out);
        }
    }
    let mut text = Vec::new();
    labels(&node, &mut text);
    assert!(text.contains(&"Name *"));
    assert!(text.contains(&"Alice"));
    assert!(text.contains(&"Required"));
}
