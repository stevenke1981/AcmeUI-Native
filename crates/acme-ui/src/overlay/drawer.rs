//! Drawer component — a side panel overlay using Stack positioning.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_widgets::painter_bridge::theme_color;
use acme_widgets::style::Styled;

/// Edge where the drawer slides out from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DrawerPlacement {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
}

/// Builder for a drawer (side panel).
pub struct DrawerBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub open: bool,
    pub content: Option<WidgetNode<M>>,
    pub placement: DrawerPlacement,
    pub backdrop: bool,
    pub on_close: Option<M>,
}

/// Create a drawer builder.
pub fn drawer<M: Clone + 'static>(id: impl Into<WidgetKey>) -> DrawerBuilder<M> {
    DrawerBuilder {
        id: id.into(),
        title: String::new(),
        open: false,
        content: None,
        placement: DrawerPlacement::Left,
        backdrop: false,
        on_close: None,
    }
}

impl<M: Clone + 'static> DrawerBuilder<M> {
    /// Set the drawer title.
    pub fn title(mut self, value: impl Into<String>) -> Self {
        self.title = value.into();
        self
    }

    /// Open or close the drawer.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set the drawer body content.
    pub fn content(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.content = Some(child.into());
        self
    }

    /// Set which edge the drawer is placed along.
    pub fn placement(mut self, placement: DrawerPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Toggle whether an overlay backdrop scrim renders behind the open drawer.
    pub fn backdrop(mut self, value: bool) -> Self {
        self.backdrop = value;
        self
    }

    /// Set the message dispatched when the close button or backdrop scrim is clicked.
    pub fn on_close(mut self, msg: M) -> Self {
        self.on_close = Some(msg);
        self
    }

    /// Build the widget node tree.
    ///
    /// When open: renders a `Stack` containing an optional clickable backdrop scrim
    /// and a `Card` with title, optional close button, and content.
    /// When closed: renders a minimal indicator `Card`.
    pub fn build(self) -> WidgetNode<M> {
        if !self.open {
            // Minimal closed indicator
            return crate::card::<M>()
                .key(self.id)
                .variant(crate::CardVariant::Muted)
                .child(crate::label::<M>("☰"))
                .padding(4.0)
                .build();
        }

        let id_str = self.id.as_str().to_string();
        let title_label = crate::label::<M>(self.title);
        let header: WidgetNode<M> = if let Some(ref msg) = self.on_close {
            crate::row::<M>()
                .gap(8.0)
                .child(title_label)
                .child(
                    crate::button::<M>(format!("{id_str}_close").as_str(), "✕")
                        .variant(crate::ButtonVariant::Ghost)
                        .on_click(msg.clone()),
                )
                .build()
        } else {
            title_label
        };

        let mut panel_col = crate::column::<M>().child(header).gap(8.0);

        if let Some(content) = self.content {
            panel_col = panel_col.child(content);
        }

        let panel_key = match self.placement {
            DrawerPlacement::Left => format!("{id_str}_panel_left"),
            DrawerPlacement::Right => format!("{id_str}_panel_right"),
            DrawerPlacement::Top => format!("{id_str}_panel_top"),
            DrawerPlacement::Bottom => format!("{id_str}_panel_bottom"),
        };
        let panel = crate::card::<M>()
            .key(panel_key.as_str())
            .variant(crate::CardVariant::Elevated)
            .child(panel_col.build())
            .padding(12.0)
            .build();

        let mut stack = crate::stack::<M>()
            .key(self.id)
            .w_full()
            .min_h(acme_layout::Length::Px(160.0));
        if self.backdrop {
            let mut scrim = crate::canvas::<M>()
                .key(format!("{id_str}_backdrop").as_str())
                .accessible_label("Close drawer")
                .layout(acme_layout::LayoutStyle {
                    width: acme_layout::Length::Percent(1.0),
                    height: acme_layout::Length::Percent(1.0),
                    ..Default::default()
                })
                .paint(|painter, rect, theme| {
                    let color = theme_color(theme.colors.surface_overlay);
                    painter.fill_rect(rect, color);
                });
            if let Some(msg) = self.on_close {
                scrim = scrim.on_click(msg);
            }
            stack = stack.child(scrim.build());
        }
        stack.child(panel).build()
    }
}

impl<M: Clone + 'static> From<DrawerBuilder<M>> for WidgetNode<M> {
    fn from(b: DrawerBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WidgetNode, label};
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn drawer_has_non_zero_layout_rect() {
        let node: WidgetNode<Msg> = drawer::<Msg>("drawer")
            .title("Settings")
            .open(true)
            .content(label("Content here"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Stack with one child (Card)
        assert_eq!(layout.children.len(), 1);
    }

    #[test]
    fn drawer_closed_shows_minimal() {
        let node: WidgetNode<Msg> = drawer::<Msg>("d").title("Hidden").open(false).into();
        // Closed: Card with Muted variant
        assert!(matches!(node, WidgetNode::Card(_)));
    }

    #[test]
    fn drawer_open_shows_stack() {
        let node: WidgetNode<Msg> = drawer::<Msg>("d").title("Panel").open(true).build();
        assert!(matches!(node, WidgetNode::Stack(_)));
    }

    #[test]
    fn drawer_backdrop_and_close_wire_scrim_and_close_button() {
        #[derive(Clone, Debug, PartialEq)]
        enum DrawerMsg {
            Close,
        }

        let node: WidgetNode<DrawerMsg> = drawer("settings")
            .title("Settings")
            .placement(DrawerPlacement::Right)
            .backdrop(true)
            .on_close(DrawerMsg::Close)
            .open(true)
            .content(label("Body"))
            .into();

        let WidgetNode::Stack(stack) = &node else {
            panic!("expected Stack");
        };
        assert_eq!(stack.children.len(), 2);

        let WidgetNode::Canvas(scrim) = &stack.children[0] else {
            panic!("expected Canvas backdrop scrim");
        };
        assert_eq!(scrim.on_click, Some(DrawerMsg::Close));

        let WidgetNode::Card(panel) = &stack.children[1] else {
            panic!("expected Card panel");
        };
        assert_eq!(
            panel.key.as_ref().map(|k| k.as_str()),
            Some("settings_panel_right")
        );
        let WidgetNode::Column(col) = &panel.children[0] else {
            panic!("expected Column in panel");
        };
        let WidgetNode::Row(header) = &col.children[0] else {
            panic!("expected header Row");
        };
        let WidgetNode::Button(close_btn) = &header.children[1] else {
            panic!("expected close Button");
        };
        assert_eq!(close_btn.activate(), Some(&DrawerMsg::Close));
    }

    #[test]
    fn drawer_backdrop_scrim_fills_stack_and_dispatches_close_on_click() {
        #[derive(Clone, Debug, PartialEq)]
        enum DrawerMsg {
            Close,
        }

        let root: WidgetNode<DrawerMsg> = drawer("settings")
            .title("Settings")
            .backdrop(true)
            .on_close(DrawerMsg::Close)
            .open(true)
            .content(label("Body"))
            .into();

        let layout = root.to_layout(acme_core::NodeId::new(1));
        let snapshot = acme_layout::LayoutEngine::new()
            .compute(&layout, (360.0, 240.0))
            .unwrap();
        let stack_rect = snapshot.get(layout.id).expect("stack layout");
        let scrim_id = layout.children[0].id;
        let scrim_rect = snapshot.get(scrim_id).expect("scrim layout");
        assert!(scrim_rect.width > 0.0 && scrim_rect.height > 0.0);
        assert_eq!(scrim_rect.width, stack_rect.width);
        assert_eq!(scrim_rect.height, stack_rect.height);

        let viewport = acme_core::Rect::new(0.0, 0.0, 360.0, 240.0);
        let registry =
            acme_widgets::hit_bridge::build_hit_layer(&root, &layout, &snapshot, viewport);
        assert!(
            registry
                .layer
                .to_flat_entries()
                .iter()
                .any(|e| e.id == scrim_id && e.enabled),
            "scrim must register a hit region"
        );
        let pt = acme_core::Point::new(
            scrim_rect.x + scrim_rect.width - 2.0,
            scrim_rect.y + scrim_rect.height - 2.0,
        );
        let hit = registry.hit_test(pt).expect("scrim hit");
        assert_eq!(registry.message_for(hit.id), Some(&DrawerMsg::Close));
    }
}
