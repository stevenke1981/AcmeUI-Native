//! Nord theme pack — arctic, north-bluish palette.

use crate::packs::{ThemePack, shadow_ladder};
use crate::{ColorTokens, Theme, ThemeColor, ThemeMode};

/// Nord design pack (arctic blue palette).
pub struct NordPack;

impl ThemePack for NordPack {
    fn name(&self) -> &'static str {
        "nord"
    }

    fn light(&self) -> Theme {
        // Derived from Nord "Snow Storm" polar light.
        let colors = ColorTokens {
            background: ThemeColor::rgb(236, 239, 244), // nord6
            foreground: ThemeColor::rgb(46, 52, 64),    // nord0
            surface: ThemeColor::rgb(229, 233, 240),    // nord5
            surface_foreground: ThemeColor::rgb(46, 52, 64),

            primary: ThemeColor::rgb(77, 109, 149), // nord10 #5E81AC darkened to #4D6D95 for AA text (062)
            primary_foreground: ThemeColor::rgb(236, 239, 244),
            secondary: ThemeColor::rgb(216, 222, 233), // nord4
            secondary_foreground: ThemeColor::rgb(46, 52, 64),
            accent: ThemeColor::rgb(216, 222, 233),
            accent_foreground: ThemeColor::rgb(76, 86, 106), // nord3 (polar night), AA on accent (062)
            selection_foreground: ThemeColor::rgb(76, 86, 106),
            muted: ThemeColor::rgb(229, 233, 240),
            muted_foreground: ThemeColor::rgb(76, 86, 106), // nord3

            border: ThemeColor::rgb(216, 222, 233),
            input: ThemeColor::rgb(216, 222, 233),
            ring: ThemeColor::rgb(94, 129, 172),

            success: ThemeColor::rgb(107, 140, 79), // nord14 darkened to #6B8C4F for 3:1 graphic (062)
            warning: ThemeColor::rgb(170, 123, 29), // nord13 darkened to #AA7B1D for 3:1 graphic (062)
            danger: ThemeColor::rgb(178, 72, 82), // nord11 darkened to #B24852, AA nord6 ink (062 review)
            info: ThemeColor::rgb(64, 141, 163),  // nord8 darkened to #408DA3 for 3:1 graphic (062)

            success_soft: ThemeColor::rgb(224, 233, 219),
            warning_soft: ThemeColor::rgb(246, 238, 214),
            danger_soft: ThemeColor::rgb(240, 222, 224),
            info_soft: ThemeColor::rgb(217, 235, 240),

            surface_elevated: ThemeColor::rgb(236, 239, 244),
            surface_overlay: ThemeColor::rgb(236, 239, 244),
            surface_tooltip: ThemeColor::rgb(59, 66, 82),

            primary_hover: ThemeColor::rgb(70, 98, 135), // darker under nord6 text (062 review)
            primary_pressed: ThemeColor::rgb(76, 86, 106),
            danger_hover: ThemeColor::rgb(160, 65, 74), // darker (062 review)
            danger_pressed: ThemeColor::rgb(145, 59, 67), // darker again (062 review)
            ghost_hover: ThemeColor::rgb(229, 233, 240),
            ghost_pressed: ThemeColor::rgb(216, 222, 233),

            disabled_bg: ThemeColor::rgb(229, 233, 240),
            disabled_text: ThemeColor::rgb(156, 163, 175),
            disabled_foreground: ThemeColor::rgb(98, 103, 110),
        };
        Theme::from_colors(
            ThemeMode::Light,
            colors,
            shadow_ladder(0.05, 0.07, 0.09, 0.12),
        )
    }

    fn dark(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(46, 52, 64), // nord0 (Polar Night)
            foreground: ThemeColor::rgb(236, 239, 244), // nord6
            surface: ThemeColor::rgb(59, 66, 82),    // nord1
            surface_foreground: ThemeColor::rgb(236, 239, 244),

            primary: ThemeColor::rgb(136, 192, 208), // nord8 (frost)
            primary_foreground: ThemeColor::rgb(46, 52, 64),
            secondary: ThemeColor::rgb(67, 76, 94), // nord2
            secondary_foreground: ThemeColor::rgb(229, 233, 240),
            accent: ThemeColor::rgb(67, 76, 94),
            accent_foreground: ThemeColor::rgb(147, 198, 212), // nord8 lightened to #93C6D4 for AA on accent (062)
            selection_foreground: ThemeColor::rgb(147, 198, 212),
            muted: ThemeColor::rgb(59, 66, 82),
            muted_foreground: ThemeColor::rgb(216, 222, 233), // nord4

            border: ThemeColor::rgb(76, 86, 106), // nord3
            input: ThemeColor::rgb(76, 86, 106),
            ring: ThemeColor::rgb(136, 192, 208),

            success: ThemeColor::rgb(163, 190, 140), // nord14
            warning: ThemeColor::rgb(235, 203, 139), // nord13
            danger: ThemeColor::rgb(208, 139, 145), // nord11 lightened to #D08B91, AA dark ink (062 review)
            info: ThemeColor::rgb(129, 161, 193),   // nord10

            success_soft: ThemeColor::rgb(40, 52, 45),
            warning_soft: ThemeColor::rgb(52, 47, 33),
            danger_soft: ThemeColor::rgb(52, 35, 37),
            info_soft: ThemeColor::rgb(35, 45, 58),

            surface_elevated: ThemeColor::rgb(67, 76, 94),
            surface_overlay: ThemeColor::rgb(76, 86, 106),
            surface_tooltip: ThemeColor::rgb(76, 86, 106),

            primary_hover: ThemeColor::rgb(162, 206, 218), // nord8 lightened, away from dark ink (062 review)
            primary_pressed: ThemeColor::rgb(186, 219, 228), // lighter again (062 review)
            danger_hover: ThemeColor::rgb(215, 156, 161),  // lighter (062 review)
            danger_pressed: ThemeColor::rgb(221, 172, 176), // lighter again (062 review)
            ghost_hover: ThemeColor::rgb(67, 76, 94),
            ghost_pressed: ThemeColor::rgb(76, 86, 106),

            disabled_bg: ThemeColor::rgb(67, 76, 94),
            disabled_text: ThemeColor::rgb(76, 86, 106),
            disabled_foreground: ThemeColor::rgb(187, 191, 198),
        };
        Theme::from_colors(
            ThemeMode::Dark,
            colors,
            shadow_ladder(0.30, 0.35, 0.45, 0.55),
        )
    }
}
