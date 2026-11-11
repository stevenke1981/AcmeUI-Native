use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Next(usize),
    Previous,
    Finish,
}

fn builder(current: usize) -> TourBuilder<Message> {
    tour()
        .key("tour")
        .step(TourStep::new("歡迎").description("開始使用"))
        .step(TourStep::new("Actions"))
        .step(TourStep::new("Done").description("Finish the tour"))
        .current(current)
        .on_prev(Message::Previous)
        .on_next(Message::Next(current))
        .on_finish(Message::Finish)
}

#[test]
fn maximum_current_does_not_overflow_or_expose_navigation() {
    let node: WidgetNode<Message> = builder(usize::MAX).into();
    assert!(node.children().is_empty());
    assert_eq!(node.key().unwrap().as_str(), "tour");
}

#[test]
fn empty_and_out_of_range_steps_have_no_phantom_previous_or_finish_actions() {
    for current in [0, 1, usize::MAX] {
        let empty: WidgetNode<Message> = tour()
            .current(current)
            .on_prev(Message::Previous)
            .on_next(Message::Next(0))
            .on_finish(Message::Finish)
            .into();
        assert!(empty.children().is_empty());
    }
    let invalid: WidgetNode<Message> = builder(3).into();
    assert!(invalid.children().is_empty());
}

#[test]
fn valid_steps_preserve_content_and_current_typed_navigation_messages() {
    for current in 0..3 {
        let node: WidgetNode<Message> = builder(current).into();
        assert!(
            matches!(&node.children()[0], WidgetNode::Label(l) if l.text == format!("Step {}/3", current + 1))
        );
        assert!(
            matches!(&node.children()[1], WidgetNode::Label(l) if l.text == ["歡迎", "Actions", "Done"][current])
        );
        assert_eq!(node.children().len(), if current == 1 { 3 } else { 4 });
        if current != 1 {
            assert!(
                matches!(&node.children()[2], WidgetNode::Label(l) if l.text == if current == 0 { "開始使用" } else { "Finish the tour" })
            );
        }
        let navigation = node.children().last().unwrap();
        let actions = navigation
            .children()
            .iter()
            .map(|child| {
                let WidgetNode::Button(button) = child else {
                    panic!("navigation Button");
                };
                (button.label.as_str(), button.activate().unwrap().clone())
            })
            .collect::<Vec<_>>();
        let mut expected = Vec::new();
        if current > 0 {
            expected.push(("Previous", Message::Previous));
        }
        expected.push(if current == 2 {
            ("Finish", Message::Finish)
        } else {
            ("Next", Message::Next(current))
        });
        assert_eq!(actions, expected);
    }
    let passive: WidgetNode<Message> = tour().step(TourStep::new("Read only")).into();
    assert!(passive.children().last().unwrap().children().is_empty());
}

#[test]
fn keyed_card_identity_and_finite_layout_survive_valid_invalid_transitions() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    let mut fonts = acme_text::FontSystem::new();
    for current in [0, usize::MAX, 3, 1, 2] {
        let roots = vec![WidgetNode::from(builder(current))];
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let id = binding.id_for(&roots[0]).unwrap();
        if let Some(saved) = saved {
            assert_eq!(id, saved);
        }
        saved = Some(id);
        let layout = roots[0].to_layout(NodeId::new(1));
        let snapshot = acme_layout::LayoutEngine::new()
            .compute_with_text(&layout, (320.0, 400.0), &mut fonts, 1.0)
            .unwrap();
        let rect = snapshot.get(layout.id).unwrap();
        assert!(
            rect.width.is_finite()
                && rect.height.is_finite()
                && rect.width > 0.0
                && rect.height > 0.0
        );
    }
}
