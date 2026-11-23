//! Gruvbox theme pack — retro groove warm palette with aqua/orange accents.

use crate::packs::{ThemePack, shadow_ladder};
use crate::{ColorTokens, Theme, ThemeColor, ThemeMode};

/// Gruvbox design pack.
pub struct GruvboxPack;

impl ThemePack for GruvboxPack {
    fn name(&self) -> &'static str {
        "gruvbox"
    }

    fn light(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(251, 241, 199), // bg
            foreground: ThemeColor::rgb(60, 56, 54),    // fg
            surface: ThemeColor::rgb(235, 219, 178),    // bg1
            surface_foreground: ThemeColor::rgb(60, 56, 54),

            primary: ThemeColor::rgb(61, 117, 120), // aqua/blue #458588 darkened to #3D7578 for AA text (062)
            primary_foreground: ThemeColor::rgb(251, 241, 199),
            secondary: ThemeColor::rgb(235, 219, 178),
            secondary_foreground: ThemeColor::rgb(60, 56, 54),
            accent: ThemeColor::rgb(175, 58, 3), // faded orange #AF3A03 (AA, 060)
            accent_foreground: ThemeColor::rgb(251, 241, 199),
            selection_foreground: ThemeColor::rgb(251, 241, 199),
            muted: ThemeColor::rgb(235, 219, 178),
            muted_foreground: ThemeColor::rgb(102, 92, 84), // fg3 #665C54, AA on bg1 (062)

            border: ThemeColor::rgb(235, 219, 178),
            input: ThemeColor::rgb(213, 196, 161),
            ring: ThemeColor::rgb(69, 133, 136),

            success: ThemeColor::rgb(121, 116, 14), // faded green #79740E, 3:1 graphic (062)
            warning: ThemeColor::rgb(161, 114, 25), // yellow darkened to #A17219 for 3:1 graphic (062)
            danger: ThemeColor::rgb(204, 36, 29),   // red
            info: ThemeColor::rgb(69, 133, 136),    // blue

            success_soft: ThemeColor::rgb(235, 238, 200),
            warning_soft: ThemeColor::rgb(245, 232, 195),
            danger_soft: ThemeColor::rgb(248, 215, 205),
            info_soft: ThemeColor::rgb(215, 232, 232),

            surface_elevated: ThemeColor::rgb(251, 241, 199),
            surface_overlay: ThemeColor::rgb(251, 241, 199),
            surface_tooltip: ThemeColor::rgb(80, 73, 69),

            primary_hover: ThemeColor::rgb(55, 105, 108), // darker under light text: old pressed #37696C (062 review)
            primary_pressed: ThemeColor::rgb(50, 95, 98), // darker again (062 review)
            danger_hover: ThemeColor::rgb(184, 33, 26),   // darker under light text (062 review)
            danger_pressed: ThemeColor::rgb(170, 30, 24),
            ghost_hover: ThemeColor::rgb(235, 219, 178),
            ghost_pressed: ThemeColor::rgb(213, 196, 161),

            disabled_bg: ThemeColor::rgb(235, 219, 178),
            disabled_text: ThemeColor::rgb(124, 111, 100),
            disabled_foreground: ThemeColor::rgb(104, 93, 84),
        };
        Theme::from_colors(
            ThemeMode::Light,
            colors,
            shadow_ladder(0.05, 0.07, 0.09, 0.12),
        )
    }

    fn dark(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(40, 40, 40),    // bg
            foreground: ThemeColor::rgb(235, 219, 178), // fg
            surface: ThemeColor::rgb(60, 56, 54),       // bg1
            surface_foreground: ThemeColor::rgb(235, 219, 178),

            primary: ThemeColor::rgb(81, 156, 159), // aqua/blue #458588 lightened to #519C9F for AA dark text (062)
            primary_foreground: ThemeColor::rgb(40, 40, 40), // bg #282828 on primary, AA (062)
            secondary: ThemeColor::rgb(60, 56, 54),
            secondary_foreground: ThemeColor::rgb(235, 219, 178),
            // Bright orange #FE8019 with dark text: >= 3:1 vs background and
            // surface for accent graphics, AA for selected text (060).
            accent: ThemeColor::rgb(254, 128, 25),
            accent_foreground: ThemeColor::rgb(40, 40, 40),
            selection_foreground: ThemeColor::rgb(40, 40, 40),
            muted: ThemeColor::rgb(60, 56, 54),
            muted_foreground: ThemeColor::rgb(189, 174, 147), // fg3 #BDAE93, AA on bg1 (062)

            border: ThemeColor::rgb(60, 56, 54),
            input: ThemeColor::rgb(80, 73, 69),
            ring: ThemeColor::rgb(81, 156, 159), // = primary #519C9F (062 review)

            success: ThemeColor::rgb(152, 151, 26), // green
            warning: ThemeColor::rgb(215, 153, 33), // yellow
            danger: ThemeColor::rgb(251, 88, 69), // bright red lightened to #FB5845, AA dark ink (062 review)
            info: ThemeColor::rgb(81, 156, 159),  // = primary #519C9F, 3:1 graphic (062 review)

            success_soft: ThemeColor::rgb(45, 50, 25),
            warning_soft: ThemeColor::rgb(55, 45, 20),
            danger_soft: ThemeColor::rgb(55, 25, 22),
            info_soft: ThemeColor::rgb(28, 45, 48),

            surface_elevated: ThemeColor::rgb(60, 56, 54),
            surface_overlay: ThemeColor::rgb(80, 73, 69),
            surface_tooltip: ThemeColor::rgb(80, 73, 69),

            primary_hover: ThemeColor::rgb(93, 169, 172), // lighter step from primary (062 review)
            primary_pressed: ThemeColor::rgb(113, 180, 182), // lighter again (062 review)
            danger_hover: ThemeColor::rgb(252, 114, 97), // lighter, away from dark ink (062 review)
            danger_pressed: ThemeColor::rgb(252, 135, 121), // lighter again (062 review)
            ghost_hover: ThemeColor::rgb(60, 56, 54),
            ghost_pressed: ThemeColor::rgb(80, 73, 69),

            disabled_bg: ThemeColor::rgb(60, 56, 54),
            disabled_text: ThemeColor::rgb(168, 153, 132),
            disabled_foreground: ThemeColor::rgb(176, 162, 143),
        };
        Theme::from_colors(
            ThemeMode::Dark,
            colors,
            shadow_ladder(0.30, 0.35, 0.45, 0.55),
        )
    }
}
