//! Tailwind‑style spacing / sizing utility constants and helpers.
//!
//! Maps tailwind‑style token names to the AcmeUI design‑system 4 px grid.
//!
//! # Tailwind Mapping
//!
//! | Token | Value | Tailwind equivalent |
//! |-------|-------|---------------------|
//! | `0`   | 0 px  | `0`                 |
//! | `0.5` | 2 px  | `0.5`               |
//! | `1`   | 4 px  | `1`                 |
//! | `2`   | 8 px  | `2`                 |
//! | `3`   | 12 px | `3`                 |
//! | `4`   | 16 px | `4`                 |
//! | `5`   | 20 px | `5`                 |
//! | `6`   | 24 px | `6`                 |
//! | `8`   | 32 px | `8`                 |
//! | `10`  | 40 px | `10`                |
//! | `12`  | 48 px | `12`                |
//! | `16`  | 64 px | `16`                |
//! | `20`  | 80 px | `20`                |
//! | `24`  | 96 px | `24`                |

use acme_layout::Length;

/// Convert a Tailwind‑style spacing token to logical pixels.
///
/// Follows the AcmeUI 4 px grid: `spacing(n) = n × 4 px`, with special
/// cases for `0`, `0.5`, and the standard token set.
pub const fn spacing(token: u32) -> f32 {
    match token {
        0 => 0.0,
        1 => 4.0,
        2 => 8.0,
        3 => 12.0,
        4 => 16.0,
        5 => 20.0,
        6 => 24.0,
        8 => 32.0,
        10 => 40.0,
        12 => 48.0,
        16 => 64.0,
        20 => 80.0,
        24 => 96.0,
        _ => (token as f32) * 4.0,
    }
}

/// Create a `Length::Px` from the spacing token.
pub const fn spacing_length(token: u32) -> Length {
    Length::Px(spacing(token))
}

/// Tailwind‑style width tokens.
///
/// These produce `Length` values suitable for style methods.
pub mod w {
    use super::*;

    /// `w-0` — 0 px
    pub const fn _0() -> Length {
        Length::Px(0.0)
    }
    /// `w-px` — 1 px
    pub const fn px() -> Length {
        Length::Px(1.0)
    }
    /// `w-0.5` — 2 px
    pub const fn _05() -> Length {
        Length::Px(2.0)
    }
    /// `w-1` — 4 px
    pub const fn _1() -> Length {
        Length::Px(4.0)
    }
    /// `w-2` — 8 px
    pub const fn _2() -> Length {
        Length::Px(8.0)
    }
    /// `w-3` — 12 px
    pub const fn _3() -> Length {
        Length::Px(12.0)
    }
    /// `w-4` — 16 px
    pub const fn _4() -> Length {
        Length::Px(16.0)
    }
    /// `w-5` — 20 px
    pub const fn _5() -> Length {
        Length::Px(20.0)
    }
    /// `w-6` — 24 px
    pub const fn _6() -> Length {
        Length::Px(24.0)
    }
    /// `w-8` — 32 px
    pub const fn _8() -> Length {
        Length::Px(32.0)
    }
    /// `w-10` — 40 px
    pub const fn _10() -> Length {
        Length::Px(40.0)
    }
    /// `w-12` — 48 px
    pub const fn _12() -> Length {
        Length::Px(48.0)
    }
    /// `w-16` — 64 px
    pub const fn _16() -> Length {
        Length::Px(64.0)
    }
    /// `w-20` — 80 px
    pub const fn _20() -> Length {
        Length::Px(80.0)
    }
    /// `w-24` — 96 px
    pub const fn _24() -> Length {
        Length::Px(96.0)
    }
    /// `w-32` — 128 px
    pub const fn _32() -> Length {
        Length::Px(128.0)
    }
    /// `w-48` — 192 px
    pub const fn _48() -> Length {
        Length::Px(192.0)
    }
    /// `w-64` — 256 px
    pub const fn _64() -> Length {
        Length::Px(256.0)
    }
    /// `w-96` — 384 px
    pub const fn _96() -> Length {
        Length::Px(384.0)
    }
    /// `w-full` — 100%
    pub const fn full() -> Length {
        Length::Percent(100.0)
    }
    /// `w-screen` — 100vw
    pub const fn screen() -> Length {
        Length::Percent(100.0)
    }
    /// `w-auto`
    pub const fn auto() -> Length {
        Length::Auto
    }
}

/// Tailwind‑style height tokens.
pub mod h {
    use super::*;

    pub const fn _0() -> Length {
        Length::Px(0.0)
    }
    pub const fn px() -> Length {
        Length::Px(1.0)
    }
    pub const fn _05() -> Length {
        Length::Px(2.0)
    }
    pub const fn _1() -> Length {
        Length::Px(4.0)
    }
    pub const fn _2() -> Length {
        Length::Px(8.0)
    }
    pub const fn _3() -> Length {
        Length::Px(12.0)
    }
    pub const fn _4() -> Length {
        Length::Px(16.0)
    }
    pub const fn _5() -> Length {
        Length::Px(20.0)
    }
    pub const fn _6() -> Length {
        Length::Px(24.0)
    }
    pub const fn _8() -> Length {
        Length::Px(32.0)
    }
    pub const fn _10() -> Length {
        Length::Px(40.0)
    }
    pub const fn _12() -> Length {
        Length::Px(48.0)
    }
    pub const fn _16() -> Length {
        Length::Px(64.0)
    }
    pub const fn _20() -> Length {
        Length::Px(80.0)
    }
    pub const fn _24() -> Length {
        Length::Px(96.0)
    }
    pub const fn full() -> Length {
        Length::Percent(100.0)
    }
    pub const fn screen() -> Length {
        Length::Percent(100.0)
    }
    pub const fn auto() -> Length {
        Length::Auto
    }
}

/// Typography size tokens matching the design system.
pub mod text {
    /// 28 px — page title (`h1`).
    pub const fn h1() -> f32 {
        28.0
    }
    /// 22 px — section title (`h2`).
    pub const fn h2() -> f32 {
        22.0
    }
    /// 18 px — card title (`h3`).
    pub const fn h3() -> f32 {
        18.0
    }
    /// 16 px — subsection (`h4`).
    pub const fn h4() -> f32 {
        16.0
    }
    /// 14 px — default body text.
    pub const fn body() -> f32 {
        14.0
    }
    /// 13 px — compact body / label.
    pub const fn sm() -> f32 {
        13.0
    }
    /// 12 px — helper text, badges.
    pub const fn xs() -> f32 {
        12.0
    }
    /// 11 px — legal, timestamps.
    pub const fn _2xs() -> f32 {
        11.0
    }
}

/// Shadow definitions matching the design system elevation tokens.
///
/// These produce [`ShadowDef`](crate::ShadowDef) values.
pub mod shadow {
    use crate::{ColorToken, ShadowDef};

    /// Small shadow — `0 1px 2px` (subtle card).
    pub fn sm() -> ShadowDef {
        ShadowDef::new(0.0, 1.0, 2.0, ColorToken::rgba(0.0, 0.0, 0.0, 0.04))
    }
    /// Medium shadow — `0 4px 12px` (popover, menu).
    pub fn md() -> ShadowDef {
        ShadowDef::new(0.0, 4.0, 12.0, ColorToken::rgba(0.0, 0.0, 0.0, 0.06))
    }
    /// Large shadow — `0 8px 24px` (dialog, modal).
    pub fn lg() -> ShadowDef {
        ShadowDef::new(0.0, 8.0, 24.0, ColorToken::rgba(0.0, 0.0, 0.0, 0.08))
    }
    /// Extra‑large shadow — `0 16px 48px` (notification, tooltip).
    pub fn xl() -> ShadowDef {
        ShadowDef::new(0.0, 16.0, 48.0, ColorToken::rgba(0.0, 0.0, 0.0, 0.10))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacing_values_match_design_tokens() {
        assert_eq!(spacing(0), 0.0);
        assert_eq!(spacing(1), 4.0);
        assert_eq!(spacing(2), 8.0);
        assert_eq!(spacing(4), 16.0);
        assert_eq!(spacing(6), 24.0);
        assert_eq!(spacing(10), 40.0);
    }

    #[test]
    fn fallback_uses_4px_grid() {
        assert_eq!(spacing(7), 28.0);
        assert_eq!(spacing(15), 60.0);
    }

    #[test]
    fn w_full_is_percent() {
        assert_eq!(w::full(), Length::Percent(100.0));
    }
}
