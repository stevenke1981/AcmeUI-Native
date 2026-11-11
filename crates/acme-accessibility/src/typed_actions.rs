use crate::{AccessibilityAction, NodeId, RetainedBinding, WidgetNode};
use acme_widgets::retained_binding::is_synthetic;

impl AccessibilityAction {
    /// Resolve Click/Activate to the exact current widget's typed message.
    ///
    /// Use a complete binding from the same current description, successfully
    /// reconciled before binding. Unknown, synthetic, unbound, passive and
    /// disabled targets return None, as do non-activation actions. Dispatch a
    /// returned message once; this method creates no pointer events and never
    /// bubbles to a parent or activates a child at the target's coordinates.
    /// Existing `route_action` remains available for platform-event consumers.
    pub fn activation_message<M: Clone>(
        &self,
        root: &WidgetNode<M>,
        binding: &RetainedBinding<'_>,
    ) -> Option<M> {
        let (Self::Click(target) | Self::Activate(target)) = self else {
            return None;
        };
        if binding.mismatches() != 0 || is_synthetic(*target) {
            return None;
        }
        find_message(root, binding, *target)
    }
}

fn find_message<M: Clone>(
    node: &WidgetNode<M>,
    binding: &RetainedBinding<'_>,
    target: NodeId,
) -> Option<M> {
    if binding.id_for(node) == Some(target) {
        return match node {
            WidgetNode::Row(c) | WidgetNode::Column(c) | WidgetNode::Stack(c) => c.message.clone(),
            WidgetNode::Button(button) => button.activate().cloned(),
            _ => None,
        };
    }
    if let WidgetNode::Tree(tree) = node {
        // Tree content participates in retained binding even though the public
        // children() slice represents only ordinary widget children.
        tree.children
            .iter()
            .find_map(|row| find_message(&row.content, binding, target))
    } else {
        node.children()
            .iter()
            .find_map(|child| find_message(child, binding, target))
    }
}

pub(crate) fn append_passive_labels<M>(
    node: &WidgetNode<M>,
    label_override: Option<&str>,
    out: &mut String,
) {
    match node {
        WidgetNode::Label(label) => {
            let text = label_override.unwrap_or(&label.text).trim();
            if !text.is_empty() {
                if !out.is_empty() {
                    out.push(' ');
                }
                out.push_str(text);
            }
        }
        WidgetNode::Row(c) | WidgetNode::Column(c) | WidgetNode::Stack(c)
            if c.message.is_none() =>
        {
            for child in &c.children {
                append_passive_labels(child, label_override, out);
            }
        }
        WidgetNode::Card(card) => {
            for child in &card.children {
                append_passive_labels(child, label_override, out);
            }
        }
        // Nested controls keep their own names/actions; do not consume them
        // into the parent's accessible name.
        _ => {}
    }
}
