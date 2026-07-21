//! ToggleGroup — group of toggle buttons with single or multiple selection.
//! Aligns with shadcn/ui Toggle Group.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Selection mode for the toggle group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleGroupMode {
    #[default]
    Single,
    Multiple,
}

/// A single toggle item.
#[derive(Clone, Debug)]
pub struct ToggleItem {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl ToggleItem {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
}

/// Builder for a toggle group.
pub struct ToggleGroupBuilder<M> {
    pub id: WidgetKey,
    pub items: Vec<ToggleItem>,
    pub selected: Vec<String>,
    pub mode: ToggleGroupMode,
    pub on_select: Option<fn(&str) -> M>,
}

/// Create a toggle group builder.
pub fn toggle_group<M: Clone + 'static>() -> ToggleGroupBuilder<M> {
    ToggleGroupBuilder {
        id: WidgetKey::from("toggle_group"),
        items: Vec::new(),
        selected: Vec::new(),
        mode: ToggleGroupMode::Single,
        on_select: None,
    }
}

impl<M: Clone + 'static> ToggleGroupBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn item(mut self, item: ToggleItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn selected(mut self, values: Vec<impl Into<String>>) -> Self {
        self.selected = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn mode(mut self, value: ToggleGroupMode) -> Self {
        self.mode = value;
        self
    }

    pub fn on_select(mut self, f: fn(&str) -> M) -> Self {
        self.on_select = Some(f);
        self
    }
}

impl<M: Clone + 'static> From<ToggleGroupBuilder<M>> for WidgetNode<M> {
    fn from(b: ToggleGroupBuilder<M>) -> Self {
        let mut row = crate::row::<M>().key(b.id).gap(4.0).padding(4.0);
        for item in &b.items {
            let is_selected = b.selected.contains(&item.value);
            let prefix = if is_selected { "▣ " } else { "▢ " };
            let text = format!("{}{}", prefix, item.label);
            row = row.child(crate::label(text));
        }
        row.build()
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
    fn toggle_group_produces_row() {
        let node: WidgetNode<Msg> = toggle_group()
            .item(ToggleItem::new("b", "Bold"))
            .item(ToggleItem::new("i", "Italic"))
            .into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn toggle_group_child_count() {
        let node: WidgetNode<Msg> = toggle_group()
            .item(ToggleItem::new("a", "A"))
            .item(ToggleItem::new("b", "B"))
            .item(ToggleItem::new("c", "C"))
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 3);
    }

    #[test]
    fn toggle_group_mode_default_single() {
        let b = toggle_group::<Msg>();
        assert_eq!(b.mode, ToggleGroupMode::Single);
    }
}
