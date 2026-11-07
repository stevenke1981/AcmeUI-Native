//! ROADMAP 014: AccessKit request mapping and the TreeItem press point.

use accesskit::{Action, ActionData, ActionRequest, Role, ScrollHint};
use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutEngine, LayoutNode, LayoutSnapshot};
use acme_platform::{PlatformEvent, PlatformKey};
use acme_widgets::retained_binding::bind_retained;
use acme_widgets::view_bridge::widget_to_views;
use acme_widgets::{TreeNode, WidgetNode, button, column, label, row, tree};

use crate::{AccessibilityAction, AccessibilityAdapter, from_access_id, to_access_id};

#[derive(Clone, Debug, PartialEq)]
struct Msg;

fn request(action: Action, target: u64, data: Option<ActionData>) -> ActionRequest {
    ActionRequest {
        action,
        target: accesskit::NodeId(target),
        data,
    }
}

fn id(raw: u64) -> NodeId {
    NodeId::new(raw)
}

// ------------------------------------------------------------------
// Request mapping
// ------------------------------------------------------------------

#[test]
fn access_id_round_trips_including_virtual_ids() {
    for raw in [0, 1, 42, (1u64 << 48) + 3, 1 << 56, (1 << 56) + 9, u64::MAX] {
        assert_eq!(from_access_id(to_access_id(id(raw))), id(raw), "{raw}");
        assert_eq!(to_access_id(from_access_id(accesskit::NodeId(raw))).0, raw);
    }
}

#[test]
fn requests_map_to_actions_with_the_exact_target() {
    let virtual_id = (1u64 << 56) + 5;
    let cases = [
        (Action::Focus, None, AccessibilityAction::Focus(id(7))),
        (Action::Click, None, AccessibilityAction::Click(id(7))),
        (Action::Expand, None, AccessibilityAction::Expand(id(7))),
        (Action::Collapse, None, AccessibilityAction::Collapse(id(7))),
        (
            Action::ScrollIntoView,
            Some(ActionData::ScrollHint(ScrollHint::TopEdge)),
            AccessibilityAction::ScrollIntoView(id(7)),
        ),
        (
            Action::SetValue,
            Some(ActionData::Value("hi".into())),
            AccessibilityAction::SetValue(id(7), "hi".to_string()),
        ),
    ];
    for (action, data, expected) in cases {
        assert_eq!(
            AccessibilityAction::from_request(&request(action, 7, data.clone())),
            Some(expected),
            "{action:?}"
        );
    }
    assert_eq!(
        AccessibilityAction::from_request(&request(Action::Click, virtual_id, None)),
        Some(AccessibilityAction::Click(id(virtual_id)))
    );
}

#[test]
fn set_value_needs_a_string_and_unsupported_actions_are_none() {
    for data in [None, Some(ActionData::NumericValue(1.0))] {
        assert_eq!(
            AccessibilityAction::from_request(&request(Action::SetValue, 1, data)),
            None
        );
    }
    for action in [Action::Blur, Action::Increment, Action::ShowTooltip] {
        assert_eq!(
            AccessibilityAction::from_request(&request(action, 1, None)),
            None,
            "{action:?}"
        );
    }
}

#[test]
fn handle_request_composes_mapping_and_routing() {
    let adapter = AccessibilityAdapter::new(3);
    let events = adapter.handle_request(&request(Action::Focus, 9, None));
    assert_eq!(
        events,
        adapter.route_action(&AccessibilityAction::Focus(id(9)))
    );
    assert!(!events.is_empty());
    assert!(
        adapter
            .handle_request(&request(Action::Blur, 9, None))
            .is_empty()
    );
}

// ------------------------------------------------------------------
// Press point
// ------------------------------------------------------------------

/// A row that starts with a checkbox. This repository has no `Checkbox`
/// widget, so a button labelled "Select" stands in for it at the row start.
fn checkbox_row() -> WidgetNode<Msg> {
    row::<Msg>()
        .child(button::<Msg>("select", "Select").on_click(Msg))
        .child(label::<Msg>("Item"))
        .build()
}

/// A tree whose rows hold a button, one row starts with a checkbox, and a
/// plain button follows the tree.
fn root() -> WidgetNode<Msg> {
    let item = |key: &'static str| TreeNode::new(key, button::<Msg>(key, "Open").on_click(Msg));
    let mail = tree::<Msg>("mail")
        .child(item("a").child(item("b")).expanded(true))
        .child(item("c"))
        .child(TreeNode::new("d", checkbox_row()));
    column::<Msg>()
        .child(mail.build())
        .child(button::<Msg>("tail", "Tail").on_click(Msg))
        .build()
}

fn snapshot_of(layout: &LayoutNode) -> LayoutSnapshot {
    LayoutEngine::new()
        .compute(layout, (400.0, 300.0))
        .expect("layout should succeed")
}

/// Runs `check` with an adapter on each ID path.
fn adapters(check: impl Fn(&AccessibilityAdapter)) {
    let root = root();
    let layout = root.to_layout(NodeId::new(1));
    let mut adapter = AccessibilityAdapter::new(1);
    adapter.update(&root, &snapshot_of(&layout));
    check(&adapter);

    let mut retained = RetainedTree::new();
    let roots = std::slice::from_ref(&root);
    retained
        .reconcile_roots(&widget_to_views(roots))
        .expect("reconcile");
    let binding = bind_retained(roots, &retained);
    let ctx = acme_layout::WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 21.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: 1.0,
    };
    let layout = root.to_layout_with_binding(&binding, &ctx);
    let mut adapter = AccessibilityAdapter::new(1);
    adapter.update_with_binding(&root, &snapshot_of(&layout), &binding);
    check(&adapter);
}

fn node_ids(adapter: &AccessibilityAdapter, role: Role) -> Vec<NodeId> {
    let tree = adapter.tree_ref().expect("tree");
    tree.nodes
        .iter()
        .filter(|(_, n)| n.role() == role)
        .map(|(i, _)| from_access_id(*i))
        .collect()
}

fn bounds(adapter: &AccessibilityAdapter, node: NodeId) -> accesskit::Rect {
    let tree = adapter.tree_ref().expect("tree");
    let target = to_access_id(node);
    tree.nodes
        .iter()
        .find(|(i, _)| *i == target)
        .and_then(|(_, n)| n.bounds())
        .expect("node has bounds")
}

fn press_of(events: &[PlatformEvent]) -> (f32, f32) {
    events
        .iter()
        .find_map(|e| match e {
            PlatformEvent::PointerButton {
                x,
                y,
                pressed: true,
                ..
            } => Some((*x, *y)),
            _ => None,
        })
        .expect("a pointer press")
}

#[test]
fn tree_item_press_is_at_the_trailing_edge_not_the_centre_or_leading_content() {
    adapters(|adapter| {
        let items = node_ids(adapter, Role::TreeItem);
        assert_eq!(items.len(), 4);
        for item in items {
            let b = bounds(adapter, item);
            let centre_x = ((b.x0 + b.x1) * 0.5) as f32;
            let centre_y = ((b.y0 + b.y1) * 0.5) as f32;
            for action in [
                AccessibilityAction::Click(item),
                AccessibilityAction::Activate(item),
                AccessibilityAction::Expand(item),
                AccessibilityAction::Collapse(item),
            ] {
                let (x, y) = press_of(&adapter.route_action(&action));
                // Content (a checkbox at the start, a button at the centre)
                // occupies the leading half; the press is in the trailing part.
                assert!(x > centre_x && x < b.x1 as f32, "{action:?}: x {x}");
                assert_eq!(x, (b.x1 - 2.0_f64.min((b.x1 - b.x0) / 2.0)) as f32);
                assert_eq!(y, centre_y, "{action:?}");
            }
        }
    });
}

#[test]
fn ordinary_button_click_still_presses_the_centre() {
    adapters(|adapter| {
        let tail = *node_ids(adapter, Role::Button).last().unwrap();
        let b = bounds(adapter, tail);
        let centre = (((b.x0 + b.x1) * 0.5) as f32, ((b.y0 + b.y1) * 0.5) as f32);
        let events = adapter.route_action(&AccessibilityAction::Click(tail));
        assert_eq!(press_of(&events), centre);
        assert!(!events.iter().any(|e| matches!(
            e,
            PlatformEvent::Key {
                key: PlatformKey::ArrowRight,
                ..
            }
        )));
    });
}
