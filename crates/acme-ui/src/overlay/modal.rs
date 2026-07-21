//! Modal component — a centered dialog overlay that blocks interaction
//! with the underlying content.
//!
//! Renders as a dialog container with header (title + close), body,
//! and optional footer actions.
//!
//! Distinct from ConfirmDialog: Modal is a general-purpose dialog container
//! with more flexible content layout.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a modal dialog.
pub struct ModalBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub children: Vec<WidgetNode<M>>,
    pub footer_actions: Vec<WidgetNode<M>>,
    pub width: f32,
    pub closable: bool,
    pub backdrop: bool,
    pub on_close: Option<M>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a modal dialog builder.
pub fn modal<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    title: impl Into<String>,
) -> ModalBuilder<M> {
    ModalBuilder {
        id: id.into(),
        title: title.into(),
        children: vec![],
        footer_actions: vec![],
        width: 480.0,
        closable: true,
        backdrop: true,
        on_close: None,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> ModalBuilder<M> {
    /// Add body content to the modal.
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Add a footer action button.
    pub fn action(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.footer_actions.push(child.into());
        self
    }

    /// Set the modal width in pixels.
    pub fn width(mut self, value: f32) -> Self {
        self.width = value;
        self
    }

    /// Show/hide the close button in the header.
    pub fn closable(mut self, value: bool) -> Self {
        self.closable = value;
        self
    }

    /// Show/hide the semi-transparent backdrop.
    pub fn backdrop(mut self, value: bool) -> Self {
        self.backdrop = value;
        self
    }

    /// Set the message dispatched when the modal is dismissed.
    pub fn on_close(mut self, msg: M) -> Self {
        self.on_close = Some(msg);
        self
    }

    /// Build the modal widget.
    pub fn build(self) -> WidgetNode<M> {
        let mut body = crate::column::<M>().gap(16.0);

        // Header row: title + optional close button
        let mut header =
            crate::row::<M>().child(crate::label_builder(&self.title).font_size(20.0).build());

        if self.closable {
            header = header.child(crate::row::<M>().child(crate::label::<M>("✕")).build());
        }

        body = body.child(header.build());

        // Body content
        for child in self.children {
            body = body.child(child);
        }

        // Footer actions
        if !self.footer_actions.is_empty() {
            let mut footer = crate::row::<M>().gap(8.0);
            for action in self.footer_actions {
                footer = footer.child(action);
            }
            body = body.child(footer.build());
        }

        let content = body.build();

        crate::dialog::<M>(self.id.clone(), content)
            .title(self.title)
            .width(self.width)
            .build()
    }
}

impl<M: Clone + 'static> From<ModalBuilder<M>> for WidgetNode<M> {
    fn from(b: ModalBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutEngine, WidgetLayoutContext};

    fn test_context() -> WidgetLayoutContext {
        WidgetLayoutContext {
            body_font_size: 16.0,
            body_line_height: 22.0,
            label_font_size: 14.0,
            control_height: 32.0,
            scale_factor: 1.0,
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn modal_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = modal("m", "Dialog Title").build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let snapshot = LayoutEngine::new()
            .compute(&layout, (800.0, 600.0))
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(
            rect.width > 0.0,
            "modal width should be > 0 (dialog default 480px)"
        );
    }

    #[test]
    fn modal_with_content_and_actions() {
        let node: WidgetNode<TestMsg> = modal("m", "Confirm")
            .child(crate::label::<TestMsg>("Are you sure?"))
            .action(crate::label::<TestMsg>("OK"))
            .action(crate::label::<TestMsg>("Cancel"))
            .build();
        let WidgetNode::Dialog(d) = &node else {
            panic!("expected Dialog")
        };
        // Dialog content is a column inside
        assert_eq!(d.title, "Confirm");
    }

    #[test]
    fn modal_builder_defaults() {
        let m = modal::<TestMsg>("m", "Defaults");
        assert_eq!(m.title, "Defaults");
        assert!((m.width - 480.0).abs() < f32::EPSILON);
        assert!(m.closable);
        assert!(m.backdrop);
        assert!(m.children.is_empty());
    }

    #[test]
    fn modal_on_close_default_none() {
        let m = modal::<TestMsg>("m", "Test");
        assert!(m.on_close.is_none());
    }

    #[test]
    fn modal_width_configurable() {
        let m = modal::<TestMsg>("m", "Wide").width(640.0);
        assert!((m.width - 640.0).abs() < f32::EPSILON);
    }
}
