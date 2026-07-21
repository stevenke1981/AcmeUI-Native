//! Free-standing page-section helper builders for the Gallery.

use acme_widgets::{
    ButtonVariant, WidgetNode, button, column, label, label_with_size, row,
};

use crate::types::*;

pub fn standard_component_sections() -> Vec<(&'static str, WidgetNode<GalleryMessage>)> {
    vec![
        ("Anatomy", anatomy_diagram()),
        ("Variants", variants_demo()),
        ("Sizes", sizes_demo()),
        ("Light / Dark", light_dark_demo()),
        ("Density", density_demo()),
        ("Keyboard Behavior", keyboard_behavior()),
        ("Accessibility Properties", accessibility_props()),
        ("Long Traditional Chinese Text", long_text_section()),
        ("Screenshot Mode", screenshot_info()),
    ]
}

fn anatomy_diagram() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)
        .child(label("── Component Anatomy ──"))
        .child(label("┌─────────────────────┐"))
        .child(label("│  Container / Root   │"))
        .child(label("│  ├─ Label / Content  │"))
        .child(label("│  └─ (children...)    │"))
        .child(label("└─────────────────────┘"))
        .build()
}

fn variants_demo() -> WidgetNode<GalleryMessage> {
    row()
        .gap(8.0)
        .child(
            button("v_primary", "Primary")
                .primary()
                .on_click(GalleryMessage::DpiInfo),
        )
        .child(button("v_secondary", "Secondary").on_click(GalleryMessage::DpiInfo))
        .child(
            button("v_ghost", "Ghost")
                .variant(ButtonVariant::Ghost)
                .on_click(GalleryMessage::DpiInfo),
        )
        .child(
            button("v_danger", "Danger")
                .variant(ButtonVariant::Danger)
                .on_click(GalleryMessage::DpiInfo),
        )
        .build()
}

fn sizes_demo() -> WidgetNode<GalleryMessage> {
    column()
        .gap(10.0)
        .child(label("XS  ·  S  ·  M  ·  L  ·  XL"))
        .child(
            row()
                .gap(8.0)
                .child(label("[XS btn]"))
                .child(label("[S btn]"))
                .child(label("[M btn — default]"))
                .child(label("[L btn]"))
                .child(label("[XL btn]"))
                .build(),
        )
        .build()
}

fn light_dark_demo() -> WidgetNode<GalleryMessage> {
    row()
        .gap(16.0)
        .child(
            column()
                .gap(8.0)
                .child(label_with_size("☀ Light", 16.0))
                .child(label("Component in light theme"))
                .build(),
        )
        .child(
            column()
                .gap(8.0)
                .child(label_with_size("🌙 Dark", 16.0))
                .child(label("Component in dark theme"))
                .build(),
        )
        .build()
}

pub fn density_demo() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)
        .child(label("Compact (0.75×) vs Comfortable (1.0×)"))
        .child(label("Toggle via toolbar button above."))
        .build()
}

fn keyboard_behavior() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)
        .child(label("Space  ·  Activate focused widget"))
        .child(label("Enter  ·  Submit / Confirm"))
        .child(label("Tab    ·  Move focus forward"))
        .child(label("⇧+Tab ·  Move focus backward"))
        .child(label("Esc    ·  Dismiss overlay / cancel"))
        .build()
}

fn accessibility_props() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)
        .child(label(
            "role=\"button\"  ·  aria-label=\"...\"  ·  tabindex=\"0\"",
        ))
        .child(label("aria-disabled  ·  aria-expanded  ·  aria-controls"))
        .child(label(
            "Focus ring visible  ·  Screen‑reader labels via AccessKit",
        ))
        .build()
}

pub fn long_text_section() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)
        .child(label_with_size("Long Traditional Chinese string:", 14.0))
        .child(label_with_size(LONG_CHINESE_TEXT, 14.0))
        .build()
}

fn screenshot_info() -> WidgetNode<GalleryMessage> {
    column()
        .gap(8.0)
        .child(label("Screenshot sizes: 1280×800 · 1024×700 · 800×600"))
        .child(label("Toggle theme & density via toolbar before capture."))
        .build()
}

/// Apply density scale factor to a base spacing value.
pub fn spacing(density: Density, base: f32) -> f32 {
    base * density.spacing_scale()
}

/// Static-ish table cell text for original row `i`, column `col`.
pub fn table_cell_text(row: usize, col: usize) -> String {
    const OWNERS: &[&str] = &["Ada", "Lin", "Sam", "Mei", "Kai", "Zoe"];
    const STATUSES: &[&str] = &["Active", "Draft", "Review", "Done", "Blocked"];
    match col {
        0 => format!("Project {row:02}"),
        1 => STATUSES[row % STATUSES.len()].to_string(),
        2 => OWNERS[row % OWNERS.len()].to_string(),
        _ => format!("2026-0{}-{:02}", (row % 9) + 1, (row % 28) + 1),
    }
}

pub fn table_row_cells(row: usize) -> [String; 4] {
    [
        table_cell_text(row, 0),
        table_cell_text(row, 1),
        table_cell_text(row, 2),
        table_cell_text(row, 3),
    ]
}

/// Display order of original row indices under the current sort.
pub fn table_display_order(sort_col: Option<usize>, sort_asc: bool) -> Vec<usize> {
    let mut order: Vec<usize> = (0..TABLE_ROW_COUNT).collect();
    if let Some(col) = sort_col {
        order.sort_by(|&a, &b| {
            let cmp = table_cell_text(a, col).cmp(&table_cell_text(b, col));
            if sort_asc { cmp } else { cmp.reverse() }
        });
    }
    order
}