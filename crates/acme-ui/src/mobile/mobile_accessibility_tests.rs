//! Standalone public-API tests; see REPORT-P5-mobile.md for the rustc command.
//! This avoids adding a dev-dependency to the component crate.

use acme_accessibility::{AccNode, build_tree_update};
use acme_core::NodeId;
use acme_layout::{LayoutEngine, WidgetLayoutContext};
use acme_ui::WidgetNode;
use acme_ui::mobile::{mobile_segmented, mobile_stepper, mobile_toggle};

fn walk(widget: WidgetNode<()>) -> Vec<AccNode> {
    let context = WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 20.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: 1.0,
    };
    let layout = widget.to_layout_with_context(NodeId::new(1), &context);
    let snapshot = LayoutEngine::new()
        .compute(&layout, (600.0, 300.0))
        .unwrap();
    build_tree_update(&widget, &snapshot, None, None)
        .nodes
        .into_iter()
        .map(|(_, node)| node)
        .collect()
}

#[test]
fn toggle_exposes_a_button_name_state_and_disabled_flag() {
    for (checked, state) in [(false, "Off"), (true, "On")] {
        let nodes = walk(
            mobile_toggle("Wi-Fi")
                .checked(checked)
                .disabled(true)
                .on_toggle(|_| ())
                .into(),
        );
        let button = nodes
            .iter()
            .find(|n| n.label() == Some(&format!("Wi-Fi · {state}")))
            .unwrap();
        assert_eq!(format!("{:?}", button.role()), "Button");
        assert!(button.is_disabled());
        assert!(button.bounds().unwrap().y1 - button.bounds().unwrap().y0 >= 44.0);
    }
}

#[test]
fn stepper_exposes_the_value_and_limit_button_state() {
    let nodes = walk(mobile_stepper(5).min(5).max(10).on_change(|_| ()).into());
    assert!(nodes.iter().any(|n| n.label() == Some("5")));
    let minus = nodes.iter().find(|n| n.label() == Some("−")).unwrap();
    let plus = nodes.iter().find(|n| n.label() == Some("+")).unwrap();
    assert_eq!(format!("{:?}", minus.role()), "Button");
    assert_eq!(format!("{:?}", plus.role()), "Button");
    assert!(minus.is_disabled());
    assert!(!plus.is_disabled());
}

#[test]
fn segmented_exposes_selection_in_its_name_and_disabled_state() {
    let nodes = walk(
        mobile_segmented(vec!["Day", "Week"])
            .selected_index(1)
            .disabled(true)
            .on_select(|_| ())
            .into(),
    );
    for name in ["Day", "▣ Week (selected)"] {
        let button = nodes.iter().find(|n| n.label() == Some(name)).unwrap();
        assert_eq!(format!("{:?}", button.role()), "Button");
        assert!(button.is_disabled());
    }
}
