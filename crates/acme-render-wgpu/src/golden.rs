//! Screenshot golden-test pipeline scaffold.
//!
//! This module provides the infrastructure for visual regression testing.
//! Actual screenshot capture requires a GPU context and is gated behind
//! `#[ignore]` tests that run only in CI with a display adapter.
//!
//! ## Usage
//!
//! ```ignore
//! let config = GoldenConfig::new("button_primary", 200, 60);
//! let frame = render_widget_to_frame(&widget, &config);
//! assert_golden(&frame, &config);
//! ```

/// Configuration for a golden screenshot test.
#[derive(Clone, Debug)]
pub struct GoldenConfig {
    /// Test name — used as the golden file stem.
    pub name: &'static str,
    /// Render width in logical pixels.
    pub width: f32,
    /// Render height in logical pixels.
    pub height: f32,
    /// Theme variant to render with.
    pub dark: bool,
    /// Maximum allowed per-pixel channel difference (0–255).
    pub tolerance: u8,
}

impl GoldenConfig {
    /// Create a new golden config with default tolerance of 2.
    pub fn new(name: &'static str, width: f32, height: f32) -> Self {
        Self {
            name,
            width,
            height,
            dark: false,
            tolerance: 2,
        }
    }

    /// Set the theme to dark mode.
    pub fn dark(mut self) -> Self {
        self.dark = true;
        self
    }

    /// Set the per-channel tolerance.
    pub fn tolerance(mut self, value: u8) -> Self {
        self.tolerance = value;
        self
    }

    /// Path to the golden reference file.
    pub fn golden_path(&self) -> String {
        format!("tests/golden/{}.png", self.name)
    }

    /// Path to write the actual output for comparison.
    pub fn actual_path(&self) -> String {
        format!("tests/golden/{}.actual.png", self.name)
    }
}

/// Compare two RGBA pixel buffers and return the maximum per-channel difference.
pub fn max_pixel_diff(a: &[u8], b: &[u8]) -> u8 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| x.abs_diff(*y))
        .max()
        .unwrap_or(0)
}

/// Check whether two pixel buffers match within the given tolerance.
pub fn pixels_match(a: &[u8], b: &[u8], tolerance: u8) -> bool {
    a.len() == b.len() && max_pixel_diff(a, b) <= tolerance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn golden_config_paths() {
        let config = GoldenConfig::new("test_button", 200.0, 60.0);
        assert_eq!(config.golden_path(), "tests/golden/test_button.png");
        assert_eq!(config.actual_path(), "tests/golden/test_button.actual.png");
        assert!(!config.dark);
        assert_eq!(config.tolerance, 2);
    }

    #[test]
    fn pixel_diff_identical() {
        let a = vec![128u8; 16];
        assert_eq!(max_pixel_diff(&a, &a), 0);
        assert!(pixels_match(&a, &a, 0));
    }

    #[test]
    fn pixel_diff_within_tolerance() {
        let a = vec![100u8; 4];
        let b = vec![102u8; 4];
        assert_eq!(max_pixel_diff(&a, &b), 2);
        assert!(pixels_match(&a, &b, 2));
        assert!(!pixels_match(&a, &b, 1));
    }

    #[test]
    fn pixel_diff_length_mismatch() {
        let a = vec![0u8; 4];
        let b = vec![0u8; 8];
        assert!(!pixels_match(&a, &b, 255));
    }
}