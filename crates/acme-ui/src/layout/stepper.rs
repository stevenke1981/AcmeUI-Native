//! Stepper component — a horizontal row of step labels with connectors.

use crate::WidgetNode;

/// Builder for a stepper.
pub struct StepperBuilder<M> {
    pub steps: Vec<String>,
    pub active_step: usize,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a stepper builder.
pub fn stepper<M: Clone + 'static>() -> StepperBuilder<M> {
    StepperBuilder {
        steps: vec![],
        active_step: 0,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone> StepperBuilder<M> {
    /// Add a step label.
    pub fn step(mut self, label: impl Into<String>) -> Self {
        self.steps.push(label.into());
        self
    }

    /// Set the active step index.
    pub fn active(mut self, index: usize) -> Self {
        self.active_step = index;
        self
    }

    /// Build the widget node tree.
    pub fn build(self) -> WidgetNode<M> {
        let mut row = crate::row::<M>().gap(8.0);
        for (i, step_label) in self.steps.iter().enumerate() {
            if i > 0 {
                row = row.child(crate::separator::<M>());
            }
            let text = if i == self.active_step {
                format!("▶ {step_label}")
            } else {
                step_label.clone()
            };
            row = row.child(crate::label::<M>(text));
        }
        row.build()
    }
}

impl<M: Clone + 'static> From<StepperBuilder<M>> for WidgetNode<M> {
    fn from(b: StepperBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WidgetNode;
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn stepper_has_non_zero_layout_rect() {
        let node: WidgetNode<Msg> = stepper::<Msg>()
            .step("Step 1")
            .step("Step 2")
            .step("Step 3")
            .active(1)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Row: step1 + sep + step2 + sep + step3 = 5 children
        assert_eq!(layout.children.len(), 5);
    }

    #[test]
    fn stepper_single_step() {
        let node: WidgetNode<Msg> = stepper::<Msg>().step("Only").into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.children.len(), 1);
    }

    #[test]
    fn stepper_builds_row() {
        let node: WidgetNode<Msg> = stepper::<Msg>().step("A").step("B").build();
        assert!(matches!(node, WidgetNode::Row(_)));
    }
}
