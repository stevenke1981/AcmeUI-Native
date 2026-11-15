//! Mobile slide-over navigation drawer with themed backdrop scrim and close trigger.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_widgets::painter_bridge::theme_color;
use acme_widgets::style::Styled;
use acme_widgets::{ButtonVariant, CardVariant, button, canvas, card, column, label, row, stack};

/// Edge where the mobile drawer opens from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MobileDrawerSide {
    #[default]
    Left,
    Right,
    Bottom,
}

/// Builder for a mobile navigation drawer.
pub struct MobileDrawerBuilder<M> {
    pub id: WidgetKey,
    pub title: Option<String>,
    pub open: bool,
    pub side: MobileDrawerSide,
    pub child: Option<WidgetNode<M>>,
    pub on_close: Option<M>,
}

/// Create a mobile drawer builder.
pub fn mobile_drawer<M: Clone + 'static>(id: impl Into<WidgetKey>) -> MobileDrawerBuilder<M> {
    MobileDrawerBuilder {
        id: id.into(),
        title: None,
        open: false,
        side: MobileDrawerSide::Left,
        child: None,
        on_close: None,
    }
}

impl<M: Clone + 'static> MobileDrawerBuilder<M> {
    /// Set an optional title shown in the drawer header.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set whether the drawer is open.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Set which side the drawer slides in from.
    pub fn side(mut self, side: MobileDrawerSide) -> Self {
        self.side = side;
        self
    }

    /// Set the body content rendered inside the drawer.
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.child = Some(child.into());
        self
    }

    /// Set the message dispatched when the close button or backdrop scrim is clicked.
    pub fn on_close(mut self, msg: M) -> Self {
        self.on_close = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<MobileDrawerBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileDrawerBuilder<M>) -> Self {
        let id_str = b.id.as_str().to_string();
        if !b.open {
            let closed_handle = card::<M>()
                .key(format!("{id_str}_handle").as_str())
                .variant(CardVariant::Muted)
                .padding(4.0)
                .child(label::<M>("☰"))
                .build();
            return stack::<M>().key(b.id).child(closed_handle).build();
        }

        let mut scrim = canvas::<M>()
            .key(format!("{id_str}_scrim").as_str())
            .accessible_label("Close navigation drawer")
            .layout(acme_layout::LayoutStyle {
                width: acme_layout::Length::Percent(1.0),
                height: acme_layout::Length::Percent(1.0),
                ..Default::default()
            })
            .paint(|painter, rect, theme| {
                let color = theme_color(theme.colors.surface_overlay);
                painter.fill_rect(rect, color);
            });
        if let Some(ref msg) = b.on_close {
            scrim = scrim.on_click(msg.clone());
        }

        let mut panel_col = column::<M>().gap(8.0);
        if b.title.is_some() || b.on_close.is_some() {
            let mut header = row::<M>().gap(8.0);
            if let Some(title) = b.title {
                header = header.child(label::<M>(title));
            }
            if let Some(msg) = b.on_close {
                header = header.child(
                    button::<M>(format!("{id_str}_close").as_str(), "✕")
                        .variant(ButtonVariant::Ghost)
                        .on_click(msg),
                );
            }
            panel_col = panel_col.child(header.build());
        }
        if let Some(child) = b.child {
            panel_col = panel_col.child(child);
        }

        let side_tag = match b.side {
            MobileDrawerSide::Left => "left",
            MobileDrawerSide::Right => "right",
            MobileDrawerSide::Bottom => "bottom",
        };
        let panel = card::<M>()
            .key(format!("{id_str}_panel_{side_tag}").as_str())
            .variant(CardVariant::Elevated)
            .padding(12.0)
            .child(panel_col.build())
            .build();

        stack::<M>()
            .key(b.id)
            .w_full()
            .min_h(acme_layout::Length::Px(160.0))
            .child(scrim.build())
            .child(panel)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Close,
    }

    #[test]
    fn mobile_drawer_closed_has_single_handle_in_stack() {
        let node: WidgetNode<Msg> = mobile_drawer("nav").into();
        let WidgetNode::Stack(s) = &node else {
            panic!("expected Stack");
        };
        assert_eq!(s.children.len(), 1);
        assert!(matches!(&s.children[0], WidgetNode::Card(_)));
    }

    #[test]
    fn mobile_drawer_open_wires_scrim_and_close_button() {
        let node: WidgetNode<Msg> = mobile_drawer("nav")
            .title("Menu")
            .open(true)
            .side(MobileDrawerSide::Right)
            .on_close(Msg::Close)
            .child(label("Item 1"))
            .into();

        let WidgetNode::Stack(s) = &node else {
            panic!("expected Stack");
        };
        assert_eq!(s.children.len(), 2);

        let WidgetNode::Canvas(scrim) = &s.children[0] else {
            panic!("expected Canvas scrim");
        };
        assert_eq!(scrim.on_click, Some(Msg::Close));

        let WidgetNode::Card(panel) = &s.children[1] else {
            panic!("expected Card panel");
        };
        assert_eq!(
            panel.key.as_ref().map(|k| k.as_str()),
            Some("nav_panel_right")
        );
        let WidgetNode::Column(col) = &panel.children[0] else {
            panic!("expected panel Column");
        };
        let WidgetNode::Row(header) = &col.children[0] else {
            panic!("expected header Row");
        };
        let WidgetNode::Button(close_btn) = &header.children[1] else {
            panic!("expected close Button");
        };
        assert_eq!(close_btn.activate(), Some(&Msg::Close));
    }

    #[test]
    fn mobile_drawer_scrim_fills_stack_and_dispatches_close_on_click() {
        let root: WidgetNode<Msg> = mobile_drawer("nav")
            .title("Menu")
            .open(true)
            .on_close(Msg::Close)
            .child(label("Item 1"))
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
        assert_eq!(registry.message_for(hit.id), Some(&Msg::Close));
    }
}
