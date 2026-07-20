//! ColorPicker component — color selection with preset swatches and preview.
//!
//! When open, renders a Column with a preview Card and a grid of color swatch
//! Cards. When closed, collapses to a single Card showing the selected color name.

use acme_core::WidgetKey;
use acme_widgets::*;

/// A single color swatch preset.
#[derive(Clone, Debug)]
pub struct ColorSwatch {
    pub name: String,
    pub hex: String,
}

/// Builder for a ColorPicker component.
pub struct ColorPickerBuilder<M> {
    pub id: WidgetKey,
    pub selected: Option<String>,
    pub swatches: Vec<ColorSwatch>,
    pub open: bool,
    pub on_change: Option<M>,
}

/// Create a new ColorPicker builder with default swatches.
pub fn color_picker<M: Clone + 'static>(id: impl Into<WidgetKey>) -> ColorPickerBuilder<M> {
    ColorPickerBuilder {
        id: id.into(),
        selected: None,
        swatches: default_swatches(),
        open: false,
        on_change: None,
    }
}

/// Create a color swatch preset.
pub fn color_swatch(name: impl Into<String>, hex: impl Into<String>) -> ColorSwatch {
    ColorSwatch {
        name: name.into(),
        hex: hex.into(),
    }
}

/// Default swatches covering the color spectrum.
fn default_swatches() -> Vec<ColorSwatch> {
    vec![
        color_swatch("Red", "#EF4444"),
        color_swatch("Orange", "#F97316"),
        color_swatch("Amber", "#F59E0B"),
        color_swatch("Yellow", "#EAB308"),
        color_swatch("Lime", "#84CC16"),
        color_swatch("Green", "#22C55E"),
        color_swatch("Emerald", "#10B981"),
        color_swatch("Teal", "#14B8A6"),
        color_swatch("Cyan", "#06B6D4"),
        color_swatch("Sky", "#0EA5E9"),
        color_swatch("Blue", "#3B82F6"),
        color_swatch("Indigo", "#6366F1"),
        color_swatch("Purple", "#A855F7"),
        color_swatch("Fuchsia", "#D946EF"),
        color_swatch("Pink", "#EC4899"),
        color_swatch("Gray", "#6B7280"),
    ]
}

impl<M: Clone + 'static> ColorPickerBuilder<M> {
    /// Add or replace all swatches.
    pub fn swatch(mut self, s: ColorSwatch) -> Self {
        self.swatches.push(s);
        self
    }

    /// Set the currently selected hex color.
    pub fn selected(mut self, value: Option<String>) -> Self {
        self.selected = value;
        self
    }

    /// Set whether the picker popup is open.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set the message dispatched when the selection changes.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<ColorPickerBuilder<M>> for WidgetNode<M> {
    fn from(b: ColorPickerBuilder<M>) -> Self {
        let display = b
            .selected
            .as_ref()
            .unwrap_or(&String::from("None"))
            .clone();

        // Closed: single Card showing the selected color.
        if !b.open {
            return card::<M>()
                .key(b.id)
                .variant(CardVariant::Outlined)
                .padding(8.0)
                .child(label::<M>(display))
                .build();
        }

        // Open: preview + grid of swatches
        let preview = card::<M>()
            .variant(CardVariant::Muted)
            .padding(8.0)
            .child(label::<M>(display));

        // Build rows of 8 swatches each
        let mut grid_rows: Vec<WidgetNode<M>> = Vec::new();
        let mut current_row = row::<M>().gap(4.0);
        let mut cells = 0usize;

        for (i, sw) in b.swatches.iter().enumerate() {
            let is_selected = b.selected.as_ref().is_some_and(|s| s == &sw.hex);
            let swatch_card = card::<M>()
                .key(format!("{}-s{}", b.id.as_str(), i).as_str())
                .padding(4.0)
                .variant(if is_selected {
                    CardVariant::Interactive
                } else {
                    CardVariant::Outlined
                })
                .child(label::<M>(sw.name.clone()));

            current_row = current_row.child(swatch_card);
            cells += 1;

            if cells == 8 {
                grid_rows.push(current_row.build());
                current_row = row::<M>().gap(4.0);
                cells = 0;
            }
        }
        if cells > 0 {
            grid_rows.push(current_row.build());
        }

        let mut grid = column::<M>().gap(2.0);
        for r in grid_rows {
            grid = grid.child(r);
        }

        column::<M>()
            .key(b.id)
            .gap(4.0)
            .child(preview)
            .child(grid)
            .build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn color_picker_builder_defaults() {
        let cp = color_picker::<TestMsg>("cp");
        assert!(cp.selected.is_none());
        assert!(!cp.open);
        assert_eq!(cp.swatches.len(), 16);
    }

    #[test]
    fn color_picker_closed_renders_card() {
        let node: WidgetNode<TestMsg> = color_picker("cp").open(false).into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant when closed");
        };
        assert_eq!(c.variant, CardVariant::Outlined);
    }

    #[test]
    fn color_picker_open_renders_column_with_preview_and_grid() {
        let node: WidgetNode<TestMsg> = color_picker("cp").open(true).into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant when open");
        };
        // preview + grid
        assert!(col.children.len() >= 2);
    }

    #[test]
    fn color_picker_selected_shows_in_preview() {
        let node: WidgetNode<TestMsg> = color_picker("cp")
            .open(true)
            .selected(Some("#EF4444".into()))
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        // First child is preview Card with label "#EF4444"
        let WidgetNode::Card(c) = &col.children[0] else {
            panic!("expected preview Card");
        };
        let WidgetNode::Label(l) = &c.children[0] else {
            panic!("expected Label");
        };
        assert_eq!(l.text, "#EF4444");
    }

    #[test]
    fn color_picker_swatch_grid_has_rows() {
        let node: WidgetNode<TestMsg> = color_picker("cp").open(true).into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Column(grid) = &col.children[1] else {
            panic!("expected grid Column");
        };
        // 16 swatches / 8 per row = 2 rows
        assert_eq!(grid.children.len(), 2);
    }
}
