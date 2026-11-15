//! QRCode — QR code display placeholder.
//! Aligns with Ant Design QRCode component.

use crate::WidgetNode;
use acme_core::{Rect, WidgetKey};
use acme_widgets::painter_bridge::theme_color;
use acme_widgets::{canvas, stack};

const QR_MODULES: usize = 21;

fn is_finder_dark(r: usize, c: usize) -> Option<bool> {
    let corners = [(0usize, 0usize), (0, QR_MODULES - 7), (QR_MODULES - 7, 0)];
    for (top, left) in corners {
        // 7x7 finder plus 1-module separator ring
        let r_end = (top + 7).min(QR_MODULES);
        let c_end = (left + 7).min(QR_MODULES);
        let sep_r_start = top.saturating_sub(1);
        let sep_c_start = left.saturating_sub(1);
        let sep_r_end = (top + 8).min(QR_MODULES);
        let sep_c_end = (left + 8).min(QR_MODULES);

        if r >= top && r < r_end && c >= left && c < c_end {
            let lr = r - top;
            let lc = c - left;
            let border = lr == 0 || lr == 6 || lc == 0 || lc == 6;
            let center = (2..=4).contains(&lr) && (2..=4).contains(&lc);
            return Some(border || center);
        }
        if r >= sep_r_start && r < sep_r_end && c >= sep_c_start && c < sep_c_end {
            return Some(false);
        }
    }
    None
}

fn qr_matrix(payload: &str) -> [[bool; QR_MODULES]; QR_MODULES] {
    let mut seed: u64 = 0xcbf29ce484222325;
    for byte in payload.as_bytes() {
        seed ^= u64::from(*byte);
        seed = seed.wrapping_mul(0x100000001b3);
    }

    let mut grid = [[false; QR_MODULES]; QR_MODULES];
    for (r, row) in grid.iter_mut().enumerate() {
        for (c, cell) in row.iter_mut().enumerate() {
            if let Some(dark) = is_finder_dark(r, c) {
                *cell = dark;
            } else if r == 6 || c == 6 {
                *cell = (r + c).is_multiple_of(2);
            } else if r == 13 && c == 8 {
                *cell = true;
            } else {
                let coord = ((r as u64) << 8) ^ (c as u64);
                let mut h = seed ^ coord.wrapping_mul(0x9e3779b97f4a7c15);
                h ^= h >> 30;
                h = h.wrapping_mul(0xbf58476d1ce4e5b9);
                h ^= h >> 27;
                *cell = (h & 1) == 1;
            }
        }
    }
    grid
}

/// Builder for a QR code display.
pub struct QRCodeBuilder<M> {
    pub id: WidgetKey,
    pub value: String,
    pub size: f32,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a QR code builder.
pub fn qr_code<M: Clone + 'static>(value: impl Into<String>) -> QRCodeBuilder<M> {
    QRCodeBuilder {
        id: WidgetKey::from("qr_code"),
        value: value.into(),
        size: 128.0,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> QRCodeBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn size(mut self, value: f32) -> Self {
        self.size = value;
        self
    }
}

impl<M: Clone + 'static> From<QRCodeBuilder<M>> for WidgetNode<M> {
    fn from(b: QRCodeBuilder<M>) -> Self {
        let size = b.size.max(21.0);
        let grid = qr_matrix(&b.value);
        let accessible = format!("QR code: {}", b.value);
        let canvas_node = canvas::<M>()
            .key(format!("{}_canvas", b.id.as_str()).as_str())
            .size(size, size)
            .accessible_label(accessible)
            .paint(move |painter, rect, theme| {
                let bg = theme_color(theme.colors.surface_elevated);
                let fg = theme_color(theme.colors.foreground);
                let border = theme_color(theme.colors.border);

                painter.rounded_rect_bordered(rect, 6.0, bg, 1.0, border);

                let side = rect.size.width.get().min(rect.size.height.get());
                let quiet = (side * 0.08).max(4.0);
                let inner_side = (side - quiet * 2.0).max(1.0);
                let origin_x = rect.origin.x.get() + (rect.size.width.get() - inner_side) * 0.5;
                let origin_y = rect.origin.y.get() + (rect.size.height.get() - inner_side) * 0.5;
                let cell = inner_side / QR_MODULES as f32;

                for (r, row) in grid.iter().enumerate() {
                    for (c, &dark) in row.iter().enumerate() {
                        if dark {
                            painter.fill_rect(
                                Rect::new(
                                    origin_x + c as f32 * cell,
                                    origin_y + r as f32 * cell,
                                    cell,
                                    cell,
                                ),
                                fg,
                            );
                        }
                    }
                }
            })
            .build();

        stack::<M>()
            .key(b.id)
            .size(b.size, b.size)
            .child(canvas_node)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::{DrawCommand, Painter};
    use acme_theme::Theme;
    use acme_widgets::painter_bridge::paint_canvas_with_theme;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn qr_code_produces_stack() {
        let node: WidgetNode<Msg> = qr_code("https://example.com").into();
        assert!(matches!(node, WidgetNode::Stack(_)));
    }

    #[test]
    fn qr_code_default_size() {
        let b = qr_code::<Msg>("test");
        assert_eq!(b.size, 128.0);
    }

    #[test]
    fn qr_code_custom_size() {
        let b = qr_code::<Msg>("x").size(256.0);
        assert_eq!(b.size, 256.0);
    }

    #[test]
    fn qr_code_canvas_paints_deterministic_modules_using_theme() {
        let node: WidgetNode<Msg> = qr_code("https://acme.ui").size(128.0).into();
        let WidgetNode::Stack(stack) = &node else {
            panic!("expected Stack");
        };
        let WidgetNode::Canvas(c) = &stack.children[0] else {
            panic!("expected Canvas child");
        };

        let mut light_painter = Painter::new();
        paint_canvas_with_theme(
            c,
            Rect::new(0.0, 0.0, 128.0, 128.0),
            &Theme::light(),
            &mut light_painter,
        );
        let light_scene = light_painter.finish();

        let mut dark_painter = Painter::new();
        paint_canvas_with_theme(
            c,
            Rect::new(0.0, 0.0, 128.0, 128.0),
            &Theme::dark(),
            &mut dark_painter,
        );
        let dark_scene = dark_painter.finish();

        let light_quads: Vec<_> = light_scene
            .commands()
            .iter()
            .filter_map(|cmd| match cmd {
                DrawCommand::Quad(q) => Some(q),
                _ => None,
            })
            .collect();
        let dark_quads: Vec<_> = dark_scene
            .commands()
            .iter()
            .filter_map(|cmd| match cmd {
                DrawCommand::Quad(q) => Some(q),
                _ => None,
            })
            .collect();

        assert!(light_quads.len() > 80);
        assert_eq!(light_quads.len(), dark_quads.len());
        assert_ne!(light_quads[1].color, dark_quads[1].color);
    }
}
