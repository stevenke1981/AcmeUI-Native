//! Style rendering — background, shadow, and resolved colour/size helpers.

use acme_core::{Color, Painter, Rect, RectStyle};
use acme_style::{ColorToken, ShadowDef, Style};
use acme_theme::{Theme, ThemeColor};

/// Push shadow quads for a given [`ShadowDef`] and base rect.
/// Draws a semi-transparent offset rectangle behind the content.
pub fn push_shadow(painter: &mut Painter<'_>, rect: [f32; 4], shadow: &ShadowDef, theme: &Theme) {
    let shadow_color: ThemeColor = ColorToken::resolve(&shadow.color, theme);
    painter.rect(
        Rect::new(
            rect[0] + shadow.offset_x,
            rect[1] + shadow.offset_y,
            rect[2],
            rect[3],
        ),
        RectStyle::solid(Color::rgba(
            shadow_color.red,
            shadow_color.green,
            shadow_color.blue,
            shadow_color.alpha,
        ))
        .radius(shadow.blur),
    );
}

/// Push quads for a widget's [`Style`] background and shadow, if set.
/// Returns `true` if any style-based quads were pushed.
pub fn push_widget_style(
    painter: &mut Painter<'_>,
    style: &Style,
    rect: [f32; 4],
    theme: &Theme,
) -> bool {
    let mut pushed = false;

    // Shadow
    if let Some(ref shadow) = style.shadow {
        push_shadow(painter, rect, shadow, theme);
        pushed = true;
    }

    // Background fill
    if let Some(ref token) = style.background {
        let color: ThemeColor = ColorToken::resolve(token, theme);
        painter.rect(
            Rect::new(rect[0], rect[1], rect[2], rect[3]),
            RectStyle::solid(Color::rgba(color.red, color.green, color.blue, color.alpha)),
        );
        pushed = true;
    }

    pushed
}
