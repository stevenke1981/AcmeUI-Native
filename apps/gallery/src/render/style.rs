//! Style rendering — background, shadow, and resolved colour/size helpers.

use acme_core::{Color, DrawCommand, QuadPrimitive, Rect, Scene};
use acme_style::{ColorToken, ShadowDef, Style};
use acme_theme::{Theme, ThemeColor};

/// Push shadow quads for a given [`ShadowDef`] and base rect.
/// Draws a semi-transparent offset rectangle behind the content.
pub fn push_shadow(scene: &mut Scene, rect: [f32; 4], shadow: &ShadowDef, theme: &Theme) {
    let shadow_rect = [
        rect[0] + shadow.offset_x,
        rect[1] + shadow.offset_y,
        rect[2],
        rect[3],
    ];
    let shadow_color: ThemeColor = ColorToken::resolve(&shadow.color, theme);
    scene.push(DrawCommand::Quad(QuadPrimitive {
        rect: Rect::new(
            shadow_rect[0],
            shadow_rect[1],
            shadow_rect[2],
            shadow_rect[3],
        ),
        color: Color::rgba(
            shadow_color.red,
            shadow_color.green,
            shadow_color.blue,
            shadow_color.alpha,
        ),
        radius: shadow.blur,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    }));
}

/// Push quads for a widget's [`Style`] background and shadow, if set.
/// Returns `true` if any style-based quads were pushed.
pub fn push_widget_style(scene: &mut Scene, style: &Style, rect: [f32; 4], theme: &Theme) -> bool {
    let mut pushed = false;

    // Shadow
    if let Some(ref shadow) = style.shadow {
        push_shadow(scene, rect, shadow, theme);
        pushed = true;
    }

    // Background fill
    if let Some(ref token) = style.background {
        let color: ThemeColor = ColorToken::resolve(token, theme);
        scene.push(DrawCommand::Quad(QuadPrimitive {
            rect: Rect::new(rect[0], rect[1], rect[2], rect[3]),
            color: Color::rgba(color.red, color.green, color.blue, color.alpha),
            radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }));
        pushed = true;
    }

    pushed
}
