//! Proposed API sketch. Adapt it; do not copy blindly without tests.

pub struct ThemeV2 {
    pub mode: ThemeMode,
    pub colors: ColorSystem,
    pub typography: TypographyScale,
    pub spacing: SpacingScale,
    pub radii: RadiusScale,
    pub controls: ControlSizes,
    pub density: Density,
    pub elevation: ElevationSystem,
    pub motion: MotionSystem,
    pub icons: IconSizes,
}

pub struct ColorSystem {
    pub surfaces: SurfaceColors,
    pub text: TextColors,
    pub borders: BorderColors,
    pub primary: InteractiveColor,
    pub success: InteractiveColor,
    pub warning: InteractiveColor,
    pub danger: InteractiveColor,
    pub info: InteractiveColor,
}

pub struct InteractiveColor {
    pub default: ThemeColor,
    pub hover: ThemeColor,
    pub pressed: ThemeColor,
    pub soft: ThemeColor,
    pub foreground: ThemeColor,
}
