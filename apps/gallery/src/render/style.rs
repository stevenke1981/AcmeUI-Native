//! Style rendering — background, shadow, and resolved colour/size helpers.

use acme_render_wgpu::{Frame, Quad};
use acme_style::{ColorToken, ShadowDef, Style};
use acme_theme::{Theme, ThemeColor};

use crate::render::geometry::rgba;

/// Push shadow quads for a given [`ShadowDef`] and base rect.
/// Draws a semi-transparent offset rectangle behind the content.
pub fn push_shadow(frame: &mut Frame, rect: [f32; 4], shadow: &ShadowDef, theme: &Theme) {
    let shadow_rect = [
        rect[0] + shadow.offset_x,
        rect[1] + shadow.offset_y,
        rect[2],
        rect[3],
    ];
    let shadow_color: ThemeColor = ColorToken::resolve(&shadow.color, theme);
    frame.quads.push(Quad {
        rect: shadow_rect,
        color: rgba(shadow_color),
        radius: shadow.blur,
        border_width: 0.0,
        border_color: [0.0; 4],
    });
}

/// Push quads for a widget's [`Style`] background and shadow, if set.
/// Returns `true` if any style-based quads were pushed.
pub fn push_widget_style(frame: &mut Frame, style: &Style, rect: [f32; 4], theme: &Theme) -> bool {
    let mut pushed = false;

    // Shadow
    if let Some(ref shadow) = style.shadow {
        push_shadow(frame, rect, shadow, theme);
        pushed = true;
    }

    // Background fill
    if let Some(ref token) = style.background {
        let color: ThemeColor = ColorToken::resolve(token, theme);
        frame.quads.push(Quad {
            rect,
            color: rgba(color),
            radius: 0.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
        pushed = true;
    }

    pushed
}


