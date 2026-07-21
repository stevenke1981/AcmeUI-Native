//! Tour — guided tour / onboarding steps overlay.
//! Aligns with Ant Design Tour component.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// A single tour step.
#[derive(Clone, Debug)]
pub struct TourStep {
    pub title: String,
    pub description: Option<String>,
    pub target: Option<String>,
}

impl TourStep {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            target: None,
        }
    }

    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub fn target(mut self, selector: impl Into<String>) -> Self {
        self.target = Some(selector.into());
        self
    }
}

/// Builder for a guided tour.
pub struct TourBuilder<M> {
    pub id: WidgetKey,
    pub steps: Vec<TourStep>,
    pub current: usize,
    pub on_next: Option<M>,
    pub on_prev: Option<M>,
    pub on_finish: Option<M>,
}

/// Create a tour builder.
pub fn tour<M: Clone + 'static>() -> TourBuilder<M> {
    TourBuilder {
        id: WidgetKey::from("tour"),
        steps: Vec::new(),
        current: 0,
        on_next: None,
        on_prev: None,
        on_finish: None,
    }
}

impl<M: Clone + 'static> TourBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn step(mut self, step: TourStep) -> Self {
        self.steps.push(step);
        self
    }

    pub fn current(mut self, index: usize) -> Self {
        self.current = index;
        self
    }

    pub fn on_next(mut self, msg: M) -> Self {
        self.on_next = Some(msg);
        self
    }

    pub fn on_prev(mut self, msg: M) -> Self {
        self.on_prev = Some(msg);
        self
    }

    pub fn on_finish(mut self, msg: M) -> Self {
        self.on_finish = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<TourBuilder<M>> for WidgetNode<M> {
    fn from(b: TourBuilder<M>) -> Self {
        let step = b.steps.get(b.current);
        let mut card = crate::card::<M>()
            .key(b.id)
            .variant(acme_widgets::CardVariant::Elevated)
            .padding(16.0)
            .gap(8.0);

        if let Some(step) = step {
            card = card.child(crate::label(format!(
                "Step {}/{}",
                b.current + 1,
                b.steps.len()
            )));
            card = card.child(crate::label(step.title.clone()));
            if let Some(desc) = &step.description {
                card = card.child(crate::label(desc.clone()));
            }
        }

        // Navigation buttons
        let mut nav_row = crate::row::<M>().gap(8.0);
        if b.current > 0 {
            if let Some(msg) = b.on_prev {
                nav_row = nav_row.child(crate::button("tour_prev", "Previous").on_click(msg));
            }
        }
        let is_last = b.current + 1 >= b.steps.len();
        if is_last {
            if let Some(msg) = b.on_finish {
                nav_row = nav_row.child(crate::button("tour_finish", "Finish").primary().on_click(msg));
            }
        } else if let Some(msg) = b.on_next {
            nav_row = nav_row.child(crate::button("tour_next", "Next").primary().on_click(msg));
        }
        card = card.child(nav_row.build());
        card.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Next,
        Prev,
        Finish,
    }

    #[test]
    fn tour_produces_card() {
        let node: WidgetNode<Msg> = tour()
            .step(TourStep::new("Welcome"))
            .into();
        assert!(matches!(node, WidgetNode::Card(_)));
    }

    #[test]
    fn tour_shows_step_content() {
        let node: WidgetNode<Msg> = tour()
            .step(TourStep::new("Step 1").description("First step"))
            .step(TourStep::new("Step 2"))
            .current(0)
            .into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card");
        };
        // counter + title + description + nav row = 4
        assert_eq!(c.children.len(), 4);
    }

    #[test]
    fn tour_last_step_shows_finish() {
        let node: WidgetNode<Msg> = tour()
            .step(TourStep::new("Only"))
            .current(0)
            .on_finish(Msg::Finish)
            .into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card");
        };
        assert!(!c.children.is_empty());
    }
}
