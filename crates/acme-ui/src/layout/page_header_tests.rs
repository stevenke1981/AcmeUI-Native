use super::*;
use crate::WidgetNode;
use acme_core::{FocusManager, RetainedTree};
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

fn header(key: &str, description: bool, version: usize) -> WidgetNode<usize> {
    let mut builder = page_header(key, "文件 · Document")
        .action(HeaderAction::primary(
            format!("{key}:save"),
            "Save",
            version,
        ))
        .action(HeaderAction::secondary(
            format!("{key}:reset"),
            "Reset",
            version + 1,
        ));
    if description {
        builder = builder.description("載入中的說明 · Subtitle");
    }
    builder.into()
}

#[test]
fn description_changes_and_header_reorders_preserve_action_identity_and_focus() {
    let mut tree = RetainedTree::new();
    let mut focus = FocusManager::new();
    let mut saved = None;
    for (description, reverse, version) in [
        (false, false, 10),
        (true, true, 20),
        (false, true, 30),
        (true, false, 40),
    ] {
        let mut roots = vec![
            header("a", description, version),
            header("b", !description, version),
        ];
        if reverse {
            roots.reverse();
        }
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let ids = ["a", "b"].map(|key| {
            let root = roots
                .iter()
                .find(|n| n.key().unwrap().as_str() == key)
                .unwrap();
            let actions = root.children().last().unwrap();
            let buttons = actions.children();
            for (index, child) in buttons.iter().enumerate() {
                let WidgetNode::Button(button) = child else {
                    panic!("action Button");
                };
                assert_eq!(button.activate(), Some(&(version + index)));
                assert_eq!(
                    button.variant,
                    if index == 0 {
                        crate::ButtonVariant::Primary
                    } else {
                        crate::ButtonVariant::Secondary
                    }
                );
            }
            [
                binding.id_for(root).unwrap(),
                binding.id_for(actions).unwrap(),
                binding.id_for(&buttons[0]).unwrap(),
                binding.id_for(&buttons[1]).unwrap(),
            ]
        });
        focus.set_order(
            ids.iter()
                .flat_map(|ids| ids[2..].iter())
                .map(|&id| (id, tree.get(id).unwrap().focusable)),
        );
        if let Some(saved) = saved {
            assert_eq!(
                focus.focused(),
                Some(ids[0][2]),
                "subtitle updates must retain focus"
            );
            assert_eq!(ids, saved);
        } else {
            assert!(focus.request_focus(ids[0][2]));
        }
        assert_ne!(ids[0][2], ids[1][2]);
        saved = Some(ids);
    }
}

#[test]
fn no_actions_and_passive_actions_preserve_existing_composition() {
    let empty: WidgetNode<()> = page_header("empty", "Title").into();
    assert_eq!(empty.children().len(), 1);
    let passive: WidgetNode<()> = page_header("passive", "Title")
        .action(HeaderAction {
            key: "read".into(),
            label: "Read".into(),
            on_click: None,
            primary: false,
        })
        .into();
    let WidgetNode::Button(button) = &passive.children()[1].children()[0] else {
        panic!("passive Button");
    };
    assert!(button.activate().is_none());
    assert_eq!(button.label, "Read");
}
