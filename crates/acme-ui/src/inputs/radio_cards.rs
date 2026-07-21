//! RadioCards — card-style radio selection group.
//! Aligns with Radix Themes Radio Cards.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// A single radio card option.
#[derive(Clone, Debug)]
pub struct RadioCardOption {
    pub value: String,
    pub title: String,
    pub description: Option<String>,
}

impl RadioCardOption {
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

/// Builder for radio cards.
pub struct RadioCardsBuilder<M> {
    pub id: WidgetKey,
    pub options: Vec<RadioCardOption>,
    pub selected: Option<String>,
    pub on_select: Option<fn(&str) -> M>,
}

/// Create a radio cards builder.
pub fn radio_cards<M: Clone + 'static>() -> RadioCardsBuilder<M> {
    RadioCardsBuilder {
        id: WidgetKey::from("radio_cards"),
        options: Vec::new(),
        selected: None,
        on_select: None,
    }
}

impl<M: Clone + 'static> RadioCardsBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn option(mut self, option: RadioCardOption) -> Self {
        self.options.push(option);
        self
    }

    pub fn selected(mut self, value: impl Into<String>) -> Self {
        self.selected = Some(value.into());
        self
    }

    pub fn on_select(mut self, f: fn(&str) -> M) -> Self {
        self.on_select = Some(f);
        self
    }
}

impl<M: Clone + 'static> From<RadioCardsBuilder<M>> for WidgetNode<M> {
    fn from(b: RadioCardsBuilder<M>) -> Self {
        let mut col = crate::column::<M>().key(b.id).gap(8.0).padding(4.0);
        for option in &b.options {
            let is_selected = b.selected.as_deref() == Some(&option.value);
            let marker = if is_selected { "◉" } else { "○" };
            let mut card = crate::card::<M>()
                .variant(acme_widgets::CardVariant::Outlined)
                .padding(12.0)
                .gap(4.0)
                .child(crate::label(format!("{} {}", marker, option.title)));
            if let Some(desc) = &option.description {
                card = card.child(crate::label(desc.clone()));
            }
            col = col.child(card.build());
        }
        col.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Selected(String),
    }

    fn select_msg(v: &str) -> Msg {
        Msg::Selected(v.to_string())
    }

    #[test]
    fn radio_cards_produces_column() {
        let node: WidgetNode<Msg> = radio_cards()
            .option(RadioCardOption::new("a", "Option A"))
            .into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn radio_cards_child_count() {
        let node: WidgetNode<Msg> = radio_cards()
            .option(RadioCardOption::new("a", "A"))
            .option(RadioCardOption::new("b", "B"))
            .option(RadioCardOption::new("c", "C"))
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        assert_eq!(c.children.len(), 3);
    }

    #[test]
    fn radio_cards_selected_default_none() {
        let b = radio_cards::<Msg>();
        assert!(b.selected.is_none());
    }
}
