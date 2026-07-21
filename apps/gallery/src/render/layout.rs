//! Layout ID extractors and style application for the fixed gallery frame.

use acme_core::NodeId;
use acme_layout::{LayoutKind, LayoutNode, LayoutStyle, Length};

use crate::types::{SIDEBAR_WIDTH, TOOLBAR_HEIGHT};

/// Stable layout node IDs for the fixed gallery frame.
pub struct GalleryNodeIds {
    #[allow(dead_code)]
    pub _root: NodeId,
    pub sidebar: NodeId,
    pub sidebar_label: NodeId,
    #[allow(dead_code)]
    pub _sidebar_separator: NodeId,
    pub sidebar_buttons: [NodeId; 8],
    #[allow(dead_code)]
    pub content_area: NodeId,
    pub toolbar: NodeId,
    pub toolbar_buttons: [NodeId; 3],
    pub scroll_view: NodeId,
}

/// Extract structural node IDs from the root layout tree.
pub fn extract_gallery_ids(root: &LayoutNode) -> GalleryNodeIds {
    let sb = &root.children[0];
    let ca = &root.children[1];
    let tb = &ca.children[0];
    GalleryNodeIds {
        _root: root.id,
        sidebar: sb.id,
        sidebar_label: sb.children[0].id,
        _sidebar_separator: sb.children[1].id,
        sidebar_buttons: [
            sb.children[2].id,
            sb.children[3].id,
            sb.children[4].id,
            sb.children[5].id,
            sb.children[6].id,
            sb.children[7].id,
            sb.children[8].id,
            sb.children[9].id,
        ],
        content_area: ca.id,
        toolbar: tb.id,
        toolbar_buttons: [tb.children[0].id, tb.children[1].id, tb.children[2].id],
        scroll_view: ca.children[1].id,
    }
}

/// Apply sizes, gaps, and scroll flags to the layout tree based on window size.
pub fn apply_gallery_styles(root: &mut LayoutNode, width: f32, height: f32) {
    // Root: Row fills window
    root.style = LayoutStyle {
        kind: LayoutKind::Row,
        width: Length::px(width),
        height: Length::px(height),
        gap: 0.0,
        ..Default::default()
    };

    // Sidebar: fixed width, full height
    let sb = &mut root.children[0];
    sb.style = LayoutStyle {
        kind: LayoutKind::Column,
        width: Length::px(SIDEBAR_WIDTH),
        height: Length::px(height),
        gap: 4.0,
        padding: acme_layout::Edges {
            left: 12.0,
            right: 12.0,
            top: 16.0,
            bottom: 16.0,
        },
        ..Default::default()
    };
    // Category buttons
    for i in 2..=9 {
        sb.children[i].style.width = Length::px(SIDEBAR_WIDTH - 24.0);
        sb.children[i].style.height = Length::px(40.0);
    }

    // Content area: fills remaining width
    let cw = (width - SIDEBAR_WIDTH).max(400.0);
    let ca = &mut root.children[1];
    ca.style = LayoutStyle {
        kind: LayoutKind::Column,
        width: Length::px(cw),
        height: Length::px(height),
        gap: 0.0,
        ..Default::default()
    };

    // Toolbar
    let tb = &mut ca.children[0];
    tb.style = LayoutStyle {
        kind: LayoutKind::Row,
        width: Length::px(cw),
        height: Length::px(TOOLBAR_HEIGHT),
        gap: 8.0,
        padding: acme_layout::Edges {
            left: 16.0,
            right: 16.0,
            top: 8.0,
            bottom: 8.0,
        },
        ..Default::default()
    };
    for btn in &mut tb.children {
        btn.style.width = Length::px(130.0);
        btn.style.height = Length::px(32.0);
    }

    // Scroll view
    let sh = (height - TOOLBAR_HEIGHT).max(100.0);
    let sv = &mut ca.children[1];
    sv.style = LayoutStyle {
        kind: LayoutKind::Column,
        width: Length::px(cw),
        height: Length::px(sh),
        overflow: acme_layout::Overflow::Scroll,
        flex_grow: 1.0,
        ..Default::default()
    };

    // Page-content first child — full width
    if let Some(content) = sv.children.first_mut() {
        content.style.width = Length::px(cw);
    }
}
