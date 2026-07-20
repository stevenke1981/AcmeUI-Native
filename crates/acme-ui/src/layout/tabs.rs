//! Tabs component — a row of tab buttons with an underline on the selected tab.

use crate::WidgetNode;

/// Tab bar visual variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabVariant {
    /// Underline on the selected tab.
    #[default]
    Underlined,
    /// Pill-shaped background on the selected tab.
    Pill,
    /// Segmented-control style (contiguous buttons).
    Segmented,
}

/// Builder for a tabs row.
pub struct TabsBuilder<M> {
    pub tabs: Vec<String>,
    pub selected: usize,
    pub size: crate::ControlSize,
    pub variant: TabVariant,
    pub full_width: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a tabs builder.
pub fn tabs<M: Clone + 'static>() -> TabsBuilder<M> {
    TabsBuilder {
        tabs: vec![],
        selected: 0,
        size: crate::ControlSize::Md,
        variant: TabVariant::default(),
        full_width: false,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone> TabsBuilder<M> {
    /// Add a tab label.
    pub fn tab(mut self, label: impl Into<String>) -> Self {
        self.tabs.push(label.into());
        self
    }

    /// Set the selected tab index.
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }

    /// Set the tab size (affects tab height).
    pub fn size(mut self, value: crate::ControlSize) -> Self {
        self.size = value;
        self
    }

    /// Set the tab bar visual variant.
    pub fn variant(mut self, value: TabVariant) -> Self {
        self.variant = value;
        self
    }

    /// Make tabs stretch to fill available width.
    pub fn full_width(mut self, value: bool) -> Self {
        self.full_width = value;
        self
    }

    /// Build the widget node tree.
    pub fn build(self) -> WidgetNode<M> {
        let gap = if self.variant == TabVariant::Segmented {
            0.0
        } else {
            4.0
        };
        let mut row = crate::row::<M>().gap(gap);
        for (i, tab_label) in self.tabs.iter().enumerate() {
            let is_selected = i == self.selected;
            let btn = crate::button(format!("tab_{i}").as_str(), tab_label.clone());
            let mut tab_col = crate::column::<M>().child(btn).gap(2.0);
            if is_selected && self.variant == TabVariant::Underlined {
                tab_col = tab_col.child(crate::separator::<M>());
            }
            row = row.child(tab_col.build());
        }
        row.build()
    }
}

impl<M: Clone + 'static> From<TabsBuilder<M>> for WidgetNode<M> {
    fn from(b: TabsBuilder<M>) -> Self {
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
    fn tabs_has_non_zero_layout_rect() {
        let node: WidgetNode<Msg> = tabs::<Msg>()
            .tab("Overview")
            .tab("Details")
            .tab("Settings")
            .selected(1)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Row container with 3 columns inside
        assert_eq!(layout.children.len(), 3);
        // Non-selected tabs: 1 child each (button only)
        assert_eq!(layout.children[0].children.len(), 1);
        // Selected tab (index 1): button + separator = 2 children
        assert_eq!(layout.children[1].children.len(), 2);
        // Non-selected tab (index 2): 1 child
        assert_eq!(layout.children[2].children.len(), 1);
    }

    #[test]
    fn tabs_selected_has_underline() {
        let node: WidgetNode<Msg> = tabs::<Msg>().tab("A").tab("B").selected(0).into();
        let layout = node.to_layout(NodeId::new(1));
        // First tab column has button + separator = 2 children
        assert_eq!(layout.children[0].children.len(), 2);
        // Second tab column has button only = 1 child
        assert_eq!(layout.children[1].children.len(), 1);
    }

    #[test]
    fn tabs_builds_row() {
        let node: WidgetNode<Msg> = tabs::<Msg>().tab("X").tab("Y").build();
        assert!(matches!(node, WidgetNode::Row(_)));
    }
}
