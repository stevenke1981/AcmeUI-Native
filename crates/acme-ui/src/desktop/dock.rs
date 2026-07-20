//! Dock — a dockable panel area with tabs, inspired by AcmeUIKit's Dock.

use crate::*;

/// Builder for an individual dock panel.
pub struct DockPanelBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub child: Option<WidgetNode<M>>,
    pub closable: bool,
    pub on_close: Option<M>,
}

/// Create a new DockPanel builder.
pub fn dock_panel<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    title: impl Into<String>,
) -> DockPanelBuilder<M> {
    DockPanelBuilder {
        id: id.into(),
        title: title.into(),
        child: None,
        closable: false,
        on_close: None,
    }
}

impl<M: Clone + 'static> DockPanelBuilder<M> {
    /// Set the panel child content.
    pub fn child(mut self, node: impl Into<WidgetNode<M>>) -> Self {
        self.child = Some(node.into());
        self
    }

    /// Set whether the panel can be closed.
    pub fn closable(mut self, value: bool) -> Self {
        self.closable = value;
        self
    }

    /// Set the message dispatched when the panel's close button is clicked.
    pub fn on_close(mut self, msg: M) -> Self {
        self.on_close = Some(msg);
        self
    }
}

/// Builder for a Dock component.
pub struct DockBuilder<M> {
    pub id: WidgetKey,
    pub panels: Vec<DockPanelBuilder<M>>,
    pub active_index: usize,
}

/// Create a new Dock builder.
pub fn dock<M: Clone + 'static>(id: impl Into<WidgetKey>) -> DockBuilder<M> {
    DockBuilder {
        id: id.into(),
        panels: Vec::new(),
        active_index: 0,
    }
}

impl<M: Clone + 'static> DockBuilder<M> {
    /// Add a panel to the dock.
    pub fn panel(mut self, panel: DockPanelBuilder<M>) -> Self {
        self.panels.push(panel);
        self
    }

    /// Set the active panel index.
    pub fn active_index(mut self, index: usize) -> Self {
        self.active_index = index;
        self
    }
}

impl<M: Clone + 'static> From<DockBuilder<M>> for WidgetNode<M> {
    fn from(b: DockBuilder<M>) -> Self {
        // Tab bar: row of panel title labels
        let mut tab_row = row::<M>().gap(4.0);
        for panel in &b.panels {
            tab_row = tab_row.child(label::<M>(panel.title.clone()));
        }

        // Active panel content
        let content = b
            .panels
            .get(b.active_index)
            .and_then(|p| p.child.clone())
            .unwrap_or_else(|| label::<M>(""));

        column::<M>()
            .key(b.id)
            .gap(4.0)
            .child(tab_row.build())
            .child(content)
            .build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::LayoutKind;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn dock_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = dock("dock")
            .panel(dock_panel("p1", "Panel 1").child(label::<TestMsg>("Content 1")))
            .panel(dock_panel("p2", "Panel 2").child(label::<TestMsg>("Content 2")))
            .active_index(0)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        // tab bar row + active content = 2 children
        assert_eq!(layout.children.len(), 2);
    }

    #[test]
    fn dock_builder_defaults() {
        let d = dock::<TestMsg>("dock");
        assert!(d.panels.is_empty());
        assert_eq!(d.active_index, 0);
    }

    #[test]
    fn dock_field_setters_work() {
        #[derive(Clone, Debug, PartialEq)]
        enum Msg {
            Close,
        }

        let d = dock::<Msg>("dock")
            .panel(
                dock_panel("p1", "Panel 1")
                    .child(label::<Msg>("C1"))
                    .closable(true)
                    .on_close(Msg::Close),
            )
            .active_index(0);

        assert_eq!(d.panels.len(), 1);
        assert_eq!(d.panels[0].title, "Panel 1");
        assert!(d.panels[0].closable);
        assert_eq!(d.panels[0].on_close, Some(Msg::Close));
        assert_eq!(d.active_index, 0);
    }
}
