//! Taskbar component — a desktop-style taskbar with start button,
//! window buttons, and system tray area.
//!
//! Renders as a horizontal Row inside a muted Card bar across the bottom.
//! Suitable for desktop application shells.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// A taskbar item representing a window or application.
#[derive(Clone, Debug)]
pub struct TaskbarItem {
    pub label: String,
    pub active: bool,
}

impl TaskbarItem {
    /// Create a new taskbar window item.
    pub fn new(label: impl Into<String>) -> Self {
        Self { label: label.into(), active: false }
    }

    /// Mark this item as the active window.
    pub fn active(mut self, value: bool) -> Self {
        self.active = value;
        self
    }
}

/// Builder for a desktop taskbar.
pub struct TaskbarBuilder<M> {
    pub id: WidgetKey,
    pub items: Vec<TaskbarItem>,
    pub start_label: String,
    pub tray_icons: Vec<crate::WidgetNode<M>>,
    pub height: f32,
    pub show_start: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a taskbar builder.
pub fn taskbar<M: Clone + 'static>(id: impl Into<WidgetKey>) -> TaskbarBuilder<M> {
    TaskbarBuilder {
        id: id.into(),
        items: vec![],
        start_label: "Start".to_string(),
        tray_icons: vec![],
        height: 40.0,
        show_start: true,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> TaskbarBuilder<M> {
    /// Set the Start button label.
    pub fn start_label(mut self, value: impl Into<String>) -> Self {
        self.start_label = value.into();
        self
    }

    /// Add a taskbar window item.
    pub fn item(mut self, entry: TaskbarItem) -> Self {
        self.items.push(entry);
        self
    }

    /// Add a tray icon widget.
    pub fn tray_icon(mut self, icon: impl Into<crate::WidgetNode<M>>) -> Self {
        self.tray_icons.push(icon.into());
        self
    }

    /// Set the taskbar height.
    pub fn height(mut self, value: f32) -> Self {
        self.height = value;
        self
    }

    /// Show/hide the Start button.
    pub fn show_start(mut self, value: bool) -> Self {
        self.show_start = value;
        self
    }

    /// Build the taskbar widget.
    pub fn build(self) -> WidgetNode<M> {
        let mut row = crate::row::<M>()
            .key(self.id.clone())
            .gap(4.0);

        // Start button
        if self.show_start {
            row = row.child(
                crate::button("taskbar_start", &self.start_label)
                    .primary(),
            );
        }

        // Window items
        for (i, item) in self.items.iter().enumerate() {
            let btn_key = format!("taskbar_item_{}", i);
            let btn = crate::button(btn_key.as_str(), &item.label);
            let btn = if item.active { btn.primary() } else { btn };
            row = row.child(btn);
        }

        // Spacer before tray
        row = row.child(crate::row::<M>().build());

        // Tray icons
        for icon in self.tray_icons {
            row = row.child(icon);
        }

        // Wrap in a Card with the inner row
        crate::card::<M>()
            .variant(crate::CardVariant::Muted)
            .border_radius(0.0)
            .padding(4.0)
            .child(row)
            .build()
    }
}

impl<M: Clone + 'static> From<TaskbarBuilder<M>> for WidgetNode<M> {
    fn from(b: TaskbarBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutEngine, WidgetLayoutContext};

    fn test_context() -> WidgetLayoutContext {
        WidgetLayoutContext { body_font_size: 16.0, body_line_height: 22.0, label_font_size: 14.0, control_height: 32.0, scale_factor: 1.0 }
    }

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn taskbar_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = taskbar("tb").build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let snapshot = LayoutEngine::new().compute(&layout, (800.0, 600.0)).unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0);
        assert!(rect.height > 0.0);
    }

    #[test]
    fn taskbar_with_items() {
        let node: WidgetNode<TestMsg> = taskbar("tb")
            .item(TaskbarItem::new("Terminal"))
            .item(TaskbarItem::new("Browser").active(true))
            .tray_icon(crate::label::<TestMsg>("🔊"))
            .build();
        let WidgetNode::Card(c) = &node else { panic!("expected Card") };
        let WidgetNode::Row(r) = &c.children[0] else { panic!("expected Row") };
        // start button + 2 items + spacer + 1 tray icon = 5 children
        assert_eq!(r.children.len(), 5);
    }

    #[test]
    fn taskbar_no_start_button() {
        let node: WidgetNode<TestMsg> = taskbar("tb").show_start(false).build();
        let WidgetNode::Card(c) = &node else { panic!("expected Card") };
        let WidgetNode::Row(r) = &c.children[0] else { panic!("expected Row") };
        // spacer only (no start, no items, no tray)
        assert_eq!(r.children.len(), 1);
    }

    #[test]
    fn taskbar_builder_defaults() {
        let t = taskbar::<TestMsg>("tb");
        assert_eq!(t.start_label, "Start");
        assert!(t.show_start);
        assert!(t.items.is_empty());
        assert!(t.tray_icons.is_empty());
        assert!((t.height - 40.0).abs() < f32::EPSILON);
    }

    #[test]
    fn taskbar_stores_items() {
        let t = taskbar::<TestMsg>("tb")
            .item(TaskbarItem::new("App1"))
            .item(TaskbarItem::new("App2").active(true));
        assert_eq!(t.items.len(), 2);
        assert!(t.items[1].active);
    }

    #[test]
    fn taskbar_tray_icons() {
        let t = taskbar::<TestMsg>("tb")
            .tray_icon(crate::label::<TestMsg>("🔊"))
            .tray_icon(crate::label::<TestMsg>("📶"));
        assert_eq!(t.tray_icons.len(), 2);
    }
}
