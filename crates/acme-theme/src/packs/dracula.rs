//! Dracula theme pack — vibrant dark palette with purple accent.

use crate::packs::{ThemePack, shadow_ladder};
use crate::{ColorTokens, Theme, ThemeColor, ThemeMode};

/// Dracula design pack.
pub struct DraculaPack;

impl ThemePack for DraculaPack {
    fn name(&self) -> &'static str {
        "dracula"
    }

    fn light(&self) -> Theme {
        // Derived "Dracula light" (inverted background).
        let colors = ColorTokens {
            background: ThemeColor::rgb(248, 248, 242), // foreground as bg
            foreground: ThemeColor::rgb(40, 42, 54),    // background as fg
            surface: ThemeColor::rgb(255, 255, 255),
            surface_foreground: ThemeColor::rgb(40, 42, 54),

            primary: ThemeColor::rgb(168, 112, 247), // purple #BD93F9 darkened to #A870F7 for 3:1 graphic (062)
            primary_foreground: ThemeColor::rgb(35, 36, 47), // #23242F, Dracula bg darkened for AA on primary (062)
            secondary: ThemeColor::rgb(235, 235, 230),
            secondary_foreground: ThemeColor::rgb(40, 42, 54),
            accent: ThemeColor::rgb(235, 228, 248),
            accent_foreground: ThemeColor::rgb(118, 78, 199), // #764EC7, AA on accent (062)
            muted: ThemeColor::rgb(235, 235, 230),
            muted_foreground: ThemeColor::rgb(88, 103, 152), // comment #6272A4 darkened to #586798 for AA (062)

            border: ThemeColor::rgb(220, 220, 214),
            input: ThemeColor::rgb(220, 220, 214),
            ring: ThemeColor::rgb(168, 112, 247), // = primary, 3:1 vs bg and surface (062)

            success: ThemeColor::rgb(5, 164, 45), // green darkened to #05A42D for 3:1 graphic (062)
            warning: ThemeColor::rgb(137, 148, 6), // yellow darkened to #899406 for 3:1 graphic (062)
            danger: ThemeColor::rgb(255, 74, 74),  // red darkened to #FF4A4A for 3:1 graphic (062)
            info: ThemeColor::rgb(3, 154, 186),    // cyan darkened to #039ABA for 3:1 graphic (062)

            success_soft: ThemeColor::rgb(224, 247, 229),
            warning_soft: ThemeColor::rgb(250, 248, 214),
            danger_soft: ThemeColor::rgb(255, 228, 228),
            info_soft: ThemeColor::rgb(222, 245, 250),

            surface_elevated: ThemeColor::rgb(255, 255, 255),
            surface_overlay: ThemeColor::rgb(255, 255, 255),
            surface_tooltip: ThemeColor::rgb(68, 71, 90),

            primary_hover: ThemeColor::rgb(200, 165, 250),
            primary_pressed: ThemeColor::rgb(210, 182, 251), // lighter than hover, away from dark ink (062 review)
            danger_hover: ThemeColor::rgb(255, 110, 110),
            danger_pressed: ThemeColor::rgb(255, 132, 132), // lighter than hover, away from dark ink (062 review)
            ghost_hover: ThemeColor::rgb(235, 235, 230),
            ghost_pressed: ThemeColor::rgb(220, 220, 214),

            disabled_bg: ThemeColor::rgb(235, 235, 230),
            disabled_text: ThemeColor::rgb(170, 170, 165),
        };
        Theme::from_colors(
            ThemeMode::Light,
            colors,
            shadow_ladder(0.05, 0.07, 0.09, 0.12),
        )
    }

    fn dark(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(40, 42, 54),    // Dracula bg
            foreground: ThemeColor::rgb(248, 248, 242), // foreground
            surface: ThemeColor::rgb(68, 71, 90),       // current line
            surface_foreground: ThemeColor::rgb(248, 248, 242),

            primary: ThemeColor::rgb(189, 147, 249), // purple
            primary_foreground: ThemeColor::rgb(40, 42, 54),
            secondary: ThemeColor::rgb(68, 71, 90),
            secondary_foreground: ThemeColor::rgb(248, 248, 242),
            accent: ThemeColor::rgb(68, 71, 90),
            accent_foreground: ThemeColor::rgb(255, 151, 211), // pink #FF79C6 lightened to #FF97D3 for AA (062)
            muted: ThemeColor::rgb(68, 71, 90),
            muted_foreground: ThemeColor::rgb(175, 184, 209), // comment #6272A4 lightened to #AFB8D1 for AA (062)

            border: ThemeColor::rgb(98, 114, 164),
            input: ThemeColor::rgb(98, 114, 164),
            ring: ThemeColor::rgb(189, 147, 249),

            success: ThemeColor::rgb(80, 250, 123),  // green
            warning: ThemeColor::rgb(241, 250, 140), // yellow
            danger: ThemeColor::rgb(255, 97, 97), // red lightened to #FF6161 for 3:1 on surface (062)
            info: ThemeColor::rgb(139, 233, 253), // cyan

            success_soft: ThemeColor::rgb(30, 50, 38),
            warning_soft: ThemeColor::rgb(50, 50, 30),
            danger_soft: ThemeColor::rgb(50, 25, 30),
            info_soft: ThemeColor::rgb(28, 45, 55),

            surface_elevated: ThemeColor::rgb(68, 71, 90),
            surface_overlay: ThemeColor::rgb(80, 83, 105),
            surface_tooltip: ThemeColor::rgb(98, 114, 164),

            primary_hover: ThemeColor::rgb(200, 165, 250),
            primary_pressed: ThemeColor::rgb(210, 182, 251), // lighter than hover, away from dark ink (062 review)
            danger_hover: ThemeColor::rgb(255, 123, 123),    // visible lighter step (062 review)
            danger_pressed: ThemeColor::rgb(255, 144, 144),  // lighter again (062 review)
            ghost_hover: ThemeColor::rgb(68, 71, 90),
            ghost_pressed: ThemeColor::rgb(80, 83, 105),

            disabled_bg: ThemeColor::rgb(68, 71, 90),
            disabled_text: ThemeColor::rgb(98, 114, 164),
        };
        Theme::from_colors(
            ThemeMode::Dark,
            colors,
            shadow_ladder(0.30, 0.35, 0.45, 0.55),
        )
    }
}
