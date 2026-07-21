//! CheckboxCards — card-style checkbox selection group.
//! Aligns with Radix Themes Checkbox Cards.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// A single checkbox card option.
#[derive(Clone, Debug)]
pub struct CheckboxCardOption {
    pub value: String,
    pub title: String,
    pub description: Option<String>,
}

impl CheckboxCardOption {
    pub fn new(value: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            title: title.into(),
            description: None,
        }
    }

    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
}

/// Builder for checkbox cards.
pub struct CheckboxCardsBuilder<M> {
    pub id: WidgetKey,
    pub options: Vec<CheckboxCardOption>,
    pub selected: Vec<String>,
    pub columns: u32,
    pub on_toggle: Option<fn(&str) -> M>,
}

/// Create a checkbox cards builder.
pub fn checkbox_cards<M: Clone + 'static>() -> CheckboxCardsBuilder<M> {
    CheckboxCardsBuilder {
        id: WidgetKey::from("checkbox_cards"),
        options: Vec::new(),
        selected: Vec::new(),
        columns: 2,
        on_toggle: None,
    }
}

impl<M: Clone + 'static> CheckboxCardsBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn option(mut self, option: CheckboxCardOption) -> Self {
        self.options.push(option);
        self
    }

    pub fn selected(mut self, values: Vec<impl Into<String>>) -> Self {
        self.selected = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn columns(mut self, value: u32) -> Self {
        self.columns = value.max(1);
        self
    }

    pub fn on_toggle(mut self, f: fn(&str) -> M) -> Self {
        self.on_toggle = Some(f);
        self
    }
}

impl<M: Clone + 'static> From<CheckboxCardsBuilder<M>> for WidgetNode<M> {
    fn from(b: CheckboxCardsBuilder<M>) -> Self {
        let mut grid = crate::column::<M>().key(b.id).gap(8.0).padding(4.0);
        for option in &b.options {
            let is_checked = b.selected.contains(&option.value);
            let marker = if is_checked { "☑" } else { "☐" };
            let mut card = crate::card::<M>()
                .variant(acme_widgets::CardVariant::Outlined)
                .padding(12.0)
                .gap(4.0)
                .child(crate::label(format!("{} {}", marker, option.title)));
            if let Some(desc) = &option.description {
                card = card.child(crate::label(desc.clone()));
            }
            grid = grid.child(card.build());
        }
        grid.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Toggled(String),
    }

    fn toggle_msg(v: &str) -> Msg {
        Msg::Toggled(v.to_string())
    }

    #[test]
    fn checkbox_cards_produces_column() {
        let node: WidgetNode<Msg> = checkbox_cards()
            .option(CheckboxCardOption::new("a", "Option A"))
            .into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn checkbox_cards_child_count() {
        let node: WidgetNode<Msg> = checkbox_cards()
            .option(CheckboxCardOption::new("a", "A"))
            .option(CheckboxCardOption::new("b", "B"))
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        assert_eq!(c.children.len(), 2);
    }

    #[test]
    fn checkbox_cards_selected_default_empty() {
        let b = checkbox_cards::<Msg>();
        assert!(b.selected.is_empty());
    }
}
