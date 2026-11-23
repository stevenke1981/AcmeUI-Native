//! Pure Rust zero-dependency JSON serialization and deserialization for [`Theme`].

use std::fmt::{self, Write as _};

use crate::{
    AnimationTokens, ColorTokens, ControlHeights, RadiusTokens, ShadowDef, ShadowTokens,
    SpacingTokens, Theme, ThemeColor, ThemeMode, TypographyTokens, contrast_ratio,
};

const MAX_JSON_DEPTH: usize = 64;

/// Error returned when serializing or parsing a [`Theme`] JSON document fails.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeJsonError(String);

impl ThemeJsonError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    /// Human-readable error description.
    pub fn message(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ThemeJsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ThemeJsonError {}

impl Theme {
    /// Serialize all 8 sections of this [`Theme`] into a formatted JSON string.
    ///
    /// Preserves exact `f32` precision so [`Theme::from_json`] round-trips equal
    /// themes (`Theme::from_json(&theme.to_json()) == Ok(theme)`).
    pub fn to_json(&self) -> String {
        let mut out = String::with_capacity(4096);
        out.push_str("{\n");

        let mode_str = match self.mode {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        };
        let _ = writeln!(out, "  \"mode\": \"{mode_str}\",");

        // colors
        out.push_str("  \"colors\": {\n");
        let colors = color_fields(&self.colors);
        for (idx, (name, color)) in colors.iter().enumerate() {
            let comma = if idx + 1 < colors.len() { "," } else { "" };
            let _ = write!(out, "    \"{name}\": ");
            write_color(&mut out, *color);
            out.push_str(comma);
            out.push('\n');
        }
        out.push_str("  },\n");

        // spacing
        write_f32_section(&mut out, "spacing", &spacing_fields(&self.spacing), true);
        // radii
        write_f32_section(&mut out, "radii", &radii_fields(&self.radii), true);
        // typography
        write_f32_section(
            &mut out,
            "typography",
            &typography_fields(&self.typography),
            true,
        );

        // shadows
        out.push_str("  \"shadows\": {\n");
        let shadows = shadow_fields(&self.shadows);
        for (idx, (name, shadow)) in shadows.iter().enumerate() {
            let comma = if idx + 1 < shadows.len() { "," } else { "" };
            let _ = write!(out, "    \"{name}\": ");
            write_shadow(&mut out, *shadow);
            out.push_str(comma);
            out.push('\n');
        }
        out.push_str("  },\n");

        // animation
        out.push_str("  \"animation\": {\n");
        let animation = animation_fields(&self.animation);
        for (idx, (name, val)) in animation.iter().enumerate() {
            let comma = if idx + 1 < animation.len() { "," } else { "" };
            let _ = writeln!(out, "    \"{name}\": {val}{comma}");
        }
        out.push_str("  },\n");

        // control_heights
        write_f32_section(
            &mut out,
            "control_heights",
            &control_height_fields(&self.control_heights),
            false,
        );

        out.push('}');
        out
    }

    /// Parse a [`Theme`] from JSON, allowing partial overrides over the resolved
    /// [`Theme::light`] or [`Theme::dark`] defaults.
    /// When an accent color is overridden, an omitted `selection_foreground` inherits
    /// `accent_foreground`. An omitted `disabled_foreground` keeps its default
    /// when it meets 4.5:1 contrast on `disabled_bg`, or uses black or white.
    pub fn from_json(input: &str) -> Result<Theme, ThemeJsonError> {
        let root = parse_json(input)?;
        let entries = root
            .as_object()
            .ok_or_else(|| ThemeJsonError::new("theme root must be a JSON object"))?;

        let mode = match find_last(entries, "mode") {
            Some(val) => parse_mode(val)?,
            None => ThemeMode::Light,
        };

        let mut theme = match mode {
            ThemeMode::Light => Theme::light(),
            ThemeMode::Dark => Theme::dark(),
        };

        let mut accent_overridden = false;
        let mut selection_foreground_overridden = false;
        let mut disabled_foreground_overridden = false;
        for (key, value) in entries {
            match key.as_str() {
                "mode" => {} // Already resolved above.
                "colors" => {
                    let colors = value
                        .as_object()
                        .ok_or_else(|| ThemeJsonError::new("section 'colors' must be an object"))?;
                    for (name, color) in colors {
                        if let Some(target) = color_field_mut(&mut theme.colors, name) {
                            *target = parse_color(color, &format!("colors.{name}"))?;
                            match name.as_str() {
                                "accent" | "accent_foreground" => accent_overridden = true,
                                "selection_foreground" => selection_foreground_overridden = true,
                                "disabled_foreground" => disabled_foreground_overridden = true,
                                _ => {}
                            }
                        }
                    }
                }
                "spacing" => {
                    apply_f32_section(&mut theme.spacing, value, "spacing", spacing_field_mut)?
                }
                "radii" => apply_f32_section(&mut theme.radii, value, "radii", radii_field_mut)?,
                "typography" => apply_f32_section(
                    &mut theme.typography,
                    value,
                    "typography",
                    typography_field_mut,
                )?,
                "shadows" => apply_shadows(&mut theme.shadows, value)?,
                "animation" => apply_animation(&mut theme.animation, value)?,
                "control_heights" => apply_f32_section(
                    &mut theme.control_heights,
                    value,
                    "control_heights",
                    control_height_field_mut,
                )?,
                _ => {}
            }
        }

        let colors = &mut theme.colors;
        if accent_overridden && !selection_foreground_overridden {
            colors.selection_foreground = colors.accent_foreground;
        }
        let disabled_bg = colors.disabled_bg;
        if !disabled_foreground_overridden
            && contrast_ratio(colors.disabled_foreground, disabled_bg) < 4.5
        {
            let black = ThemeColor::rgb(0, 0, 0);
            let white = ThemeColor::rgb(255, 255, 255);
            colors.disabled_foreground =
                if contrast_ratio(black, disabled_bg) >= contrast_ratio(white, disabled_bg) {
                    black
                } else {
                    white
                };
        }

        theme
            .validate()
            .map_err(|err| ThemeJsonError::new(format!("theme validation failed: {err:?}")))?;

        for (name, shadow) in shadow_fields(&theme.shadows) {
            if !shadow.color.is_valid() {
                return Err(ThemeJsonError::new(format!(
                    "invalid shadow color in shadows.{name}"
                )));
            }
        }

        Ok(theme)
    }
}

// ── Serialization helpers ────────────────────────────────────────────────────

fn write_f32(out: &mut String, val: f32) {
    if !val.is_finite() {
        out.push('0');
        return;
    }
    let normalized = if val == 0.0 { 0.0 } else { val };
    let _ = write!(out, "{normalized}");
}

fn write_color(out: &mut String, color: ThemeColor) {
    out.push_str("{ \"red\": ");
    write_f32(out, color.red);
    out.push_str(", \"green\": ");
    write_f32(out, color.green);
    out.push_str(", \"blue\": ");
    write_f32(out, color.blue);
    out.push_str(", \"alpha\": ");
    write_f32(out, color.alpha);
    out.push_str(" }");
}

fn write_shadow(out: &mut String, shadow: ShadowDef) {
    out.push_str("{ \"offset_x\": ");
    write_f32(out, shadow.offset_x);
    out.push_str(", \"offset_y\": ");
    write_f32(out, shadow.offset_y);
    out.push_str(", \"blur\": ");
    write_f32(out, shadow.blur);
    out.push_str(", \"spread\": ");
    write_f32(out, shadow.spread);
    out.push_str(", \"color\": ");
    write_color(out, shadow.color);
    out.push_str(" }");
}

fn write_f32_section(
    out: &mut String,
    section: &str,
    fields: &[(&'static str, f32)],
    trailing_comma: bool,
) {
    let _ = writeln!(out, "  \"{section}\": {{");
    for (idx, (name, val)) in fields.iter().enumerate() {
        let comma = if idx + 1 < fields.len() { "," } else { "" };
        let _ = write!(out, "    \"{name}\": ");
        write_f32(out, *val);
        out.push_str(comma);
        out.push('\n');
    }
    if trailing_comma {
        out.push_str("  },\n");
    } else {
        out.push_str("  }\n");
    }
}

// ── Field tables ─────────────────────────────────────────────────────────────

fn color_fields(c: &ColorTokens) -> [(&'static str, ThemeColor); 36] {
    [
        ("background", c.background),
        ("foreground", c.foreground),
        ("surface", c.surface),
        ("surface_foreground", c.surface_foreground),
        ("primary", c.primary),
        ("primary_foreground", c.primary_foreground),
        ("secondary", c.secondary),
        ("secondary_foreground", c.secondary_foreground),
        ("accent", c.accent),
        ("accent_foreground", c.accent_foreground),
        ("muted", c.muted),
        ("muted_foreground", c.muted_foreground),
        ("border", c.border),
        ("input", c.input),
        ("ring", c.ring),
        ("success", c.success),
        ("warning", c.warning),
        ("danger", c.danger),
        ("info", c.info),
        ("success_soft", c.success_soft),
        ("warning_soft", c.warning_soft),
        ("danger_soft", c.danger_soft),
        ("info_soft", c.info_soft),
        ("surface_elevated", c.surface_elevated),
        ("surface_overlay", c.surface_overlay),
        ("surface_tooltip", c.surface_tooltip),
        ("primary_hover", c.primary_hover),
        ("primary_pressed", c.primary_pressed),
        ("danger_hover", c.danger_hover),
        ("danger_pressed", c.danger_pressed),
        ("ghost_hover", c.ghost_hover),
        ("ghost_pressed", c.ghost_pressed),
        ("disabled_bg", c.disabled_bg),
        ("disabled_text", c.disabled_text),
        ("disabled_foreground", c.disabled_foreground),
        ("selection_foreground", c.selection_foreground),
    ]
}

fn color_field_mut<'a>(c: &'a mut ColorTokens, key: &str) -> Option<&'a mut ThemeColor> {
    Some(match key {
        "background" => &mut c.background,
        "foreground" => &mut c.foreground,
        "surface" => &mut c.surface,
        "surface_foreground" => &mut c.surface_foreground,
        "primary" => &mut c.primary,
        "primary_foreground" => &mut c.primary_foreground,
        "secondary" => &mut c.secondary,
        "secondary_foreground" => &mut c.secondary_foreground,
        "accent" => &mut c.accent,
        "accent_foreground" => &mut c.accent_foreground,
        "muted" => &mut c.muted,
        "muted_foreground" => &mut c.muted_foreground,
        "border" => &mut c.border,
        "input" => &mut c.input,
        "ring" => &mut c.ring,
        "success" => &mut c.success,
        "warning" => &mut c.warning,
        "danger" => &mut c.danger,
        "info" => &mut c.info,
        "success_soft" => &mut c.success_soft,
        "warning_soft" => &mut c.warning_soft,
        "danger_soft" => &mut c.danger_soft,
        "info_soft" => &mut c.info_soft,
        "surface_elevated" => &mut c.surface_elevated,
        "surface_overlay" => &mut c.surface_overlay,
        "surface_tooltip" => &mut c.surface_tooltip,
        "primary_hover" => &mut c.primary_hover,
        "primary_pressed" => &mut c.primary_pressed,
        "danger_hover" => &mut c.danger_hover,
        "danger_pressed" => &mut c.danger_pressed,
        "ghost_hover" => &mut c.ghost_hover,
        "ghost_pressed" => &mut c.ghost_pressed,
        "disabled_bg" => &mut c.disabled_bg,
        "disabled_text" => &mut c.disabled_text,
        "disabled_foreground" => &mut c.disabled_foreground,
        "selection_foreground" => &mut c.selection_foreground,
        _ => return None,
    })
}

fn spacing_fields(s: &SpacingTokens) -> [(&'static str, f32); 10] {
    [
        ("half", s.half),
        ("px", s.px),
        ("px1", s.px1),
        ("px2", s.px2),
        ("px3", s.px3),
        ("px4", s.px4),
        ("px5", s.px5),
        ("px6", s.px6),
        ("px8", s.px8),
        ("px10", s.px10),
    ]
}

fn spacing_field_mut<'a>(s: &'a mut SpacingTokens, key: &str) -> Option<&'a mut f32> {
    Some(match key {
        "half" => &mut s.half,
        "px" => &mut s.px,
        "px1" => &mut s.px1,
        "px2" => &mut s.px2,
        "px3" => &mut s.px3,
        "px4" => &mut s.px4,
        "px5" => &mut s.px5,
        "px6" => &mut s.px6,
        "px8" => &mut s.px8,
        "px10" => &mut s.px10,
        _ => return None,
    })
}

fn radii_fields(r: &RadiusTokens) -> [(&'static str, f32); 6] {
    [
        ("none", r.none),
        ("sm", r.sm),
        ("md", r.md),
        ("lg", r.lg),
        ("xl", r.xl),
        ("full", r.full),
    ]
}

fn radii_field_mut<'a>(r: &'a mut RadiusTokens, key: &str) -> Option<&'a mut f32> {
    Some(match key {
        "none" => &mut r.none,
        "sm" => &mut r.sm,
        "md" => &mut r.md,
        "lg" => &mut r.lg,
        "xl" => &mut r.xl,
        "full" => &mut r.full,
        _ => return None,
    })
}

fn typography_fields(t: &TypographyTokens) -> [(&'static str, f32); 11] {
    [
        ("h1", t.h1),
        ("h2", t.h2),
        ("h3", t.h3),
        ("h4", t.h4),
        ("body", t.body),
        ("body_sm", t.body_sm),
        ("label", t.label),
        ("caption", t.caption),
        ("small", t.small),
        ("code", t.code),
        ("line_height", t.line_height),
    ]
}

fn typography_field_mut<'a>(t: &'a mut TypographyTokens, key: &str) -> Option<&'a mut f32> {
    Some(match key {
        "h1" => &mut t.h1,
        "h2" => &mut t.h2,
        "h3" => &mut t.h3,
        "h4" => &mut t.h4,
        "body" => &mut t.body,
        "body_sm" => &mut t.body_sm,
        "label" => &mut t.label,
        "caption" => &mut t.caption,
        "small" => &mut t.small,
        "code" => &mut t.code,
        "line_height" => &mut t.line_height,
        _ => return None,
    })
}

fn shadow_fields(s: &ShadowTokens) -> [(&'static str, ShadowDef); 4] {
    [("sm", s.sm), ("md", s.md), ("lg", s.lg), ("xl", s.xl)]
}

fn shadow_field_mut<'a>(s: &'a mut ShadowTokens, key: &str) -> Option<&'a mut ShadowDef> {
    Some(match key {
        "sm" => &mut s.sm,
        "md" => &mut s.md,
        "lg" => &mut s.lg,
        "xl" => &mut s.xl,
        _ => return None,
    })
}

fn animation_fields(a: &AnimationTokens) -> [(&'static str, u32); 4] {
    [
        ("fast_ms", a.fast_ms),
        ("normal_ms", a.normal_ms),
        ("slow_ms", a.slow_ms),
        ("slide_ms", a.slide_ms),
    ]
}

fn animation_field_mut<'a>(a: &'a mut AnimationTokens, key: &str) -> Option<&'a mut u32> {
    Some(match key {
        "fast_ms" => &mut a.fast_ms,
        "normal_ms" => &mut a.normal_ms,
        "slow_ms" => &mut a.slow_ms,
        "slide_ms" => &mut a.slide_ms,
        _ => return None,
    })
}

fn control_height_fields(h: &ControlHeights) -> [(&'static str, f32); 5] {
    [
        ("xs", h.xs),
        ("sm", h.sm),
        ("md", h.md),
        ("lg", h.lg),
        ("xl", h.xl),
    ]
}

fn control_height_field_mut<'a>(h: &'a mut ControlHeights, key: &str) -> Option<&'a mut f32> {
    Some(match key {
        "xs" => &mut h.xs,
        "sm" => &mut h.sm,
        "md" => &mut h.md,
        "lg" => &mut h.lg,
        "xl" => &mut h.xl,
        _ => return None,
    })
}

// ── Deserialization helpers ──────────────────────────────────────────────────

fn find_last<'a>(entries: &'a [(String, JsonValue)], key: &str) -> Option<&'a JsonValue> {
    entries
        .iter()
        .rev()
        .find_map(|(k, v)| (k == key).then_some(v))
}

fn parse_mode(value: &JsonValue) -> Result<ThemeMode, ThemeJsonError> {
    let s = value
        .as_str()
        .ok_or_else(|| ThemeJsonError::new("field 'mode' must be a string"))?;
    match s {
        "light" => Ok(ThemeMode::Light),
        "dark" => Ok(ThemeMode::Dark),
        _ => Err(ThemeJsonError::new(format!(
            "invalid theme mode '{s}', expected 'light' or 'dark'"
        ))),
    }
}

fn parse_f32(value: &JsonValue, context: &str) -> Result<f32, ThemeJsonError> {
    let raw = value
        .as_number_lexeme()
        .ok_or_else(|| ThemeJsonError::new(format!("field '{context}' must be a number")))?;
    let parsed: f32 = raw
        .parse()
        .map_err(|_| ThemeJsonError::new(format!("invalid number '{raw}' in '{context}'")))?;
    if !parsed.is_finite() {
        return Err(ThemeJsonError::new(format!(
            "non-finite number '{raw}' in '{context}'"
        )));
    }
    Ok(parsed)
}

fn parse_u32(value: &JsonValue, context: &str) -> Result<u32, ThemeJsonError> {
    let raw = value
        .as_number_lexeme()
        .ok_or_else(|| ThemeJsonError::new(format!("field '{context}' must be an integer")))?;
    raw.parse::<u32>().map_err(|_| {
        ThemeJsonError::new(format!(
            "field '{context}' must be an unsigned integer, got '{raw}'"
        ))
    })
}

fn parse_color(value: &JsonValue, context: &str) -> Result<ThemeColor, ThemeJsonError> {
    let entries = value
        .as_object()
        .ok_or_else(|| ThemeJsonError::new(format!("field '{context}' must be a color object")))?;
    let mut red: Option<f32> = None;
    let mut green: Option<f32> = None;
    let mut blue: Option<f32> = None;
    let mut alpha: Option<f32> = None;

    for (key, val) in entries {
        match key.as_str() {
            "red" => red = Some(parse_f32(val, &format!("{context}.red"))?),
            "green" => green = Some(parse_f32(val, &format!("{context}.green"))?),
            "blue" => blue = Some(parse_f32(val, &format!("{context}.blue"))?),
            "alpha" => alpha = Some(parse_f32(val, &format!("{context}.alpha"))?),
            _ => {}
        }
    }

    let red = red.ok_or_else(|| ThemeJsonError::new(format!("missing '{context}.red'")))?;
    let green = green.ok_or_else(|| ThemeJsonError::new(format!("missing '{context}.green'")))?;
    let blue = blue.ok_or_else(|| ThemeJsonError::new(format!("missing '{context}.blue'")))?;
    let alpha = alpha.unwrap_or(1.0);

    Ok(ThemeColor::rgba(red, green, blue, alpha))
}

fn apply_f32_section<T>(
    target: &mut T,
    value: &JsonValue,
    section: &str,
    lookup: for<'a> fn(&'a mut T, &str) -> Option<&'a mut f32>,
) -> Result<(), ThemeJsonError> {
    let entries = value
        .as_object()
        .ok_or_else(|| ThemeJsonError::new(format!("section '{section}' must be an object")))?;
    for (key, val) in entries {
        if let Some(slot) = lookup(target, key) {
            *slot = parse_f32(val, &format!("{section}.{key}"))?;
        }
    }
    Ok(())
}

fn apply_shadows(shadows: &mut ShadowTokens, value: &JsonValue) -> Result<(), ThemeJsonError> {
    let entries = value
        .as_object()
        .ok_or_else(|| ThemeJsonError::new("section 'shadows' must be an object"))?;
    for (key, val) in entries {
        if let Some(target) = shadow_field_mut(shadows, key) {
            let shadow_entries = val.as_object().ok_or_else(|| {
                ThemeJsonError::new(format!("field 'shadows.{key}' must be an object"))
            })?;
            for (field, field_val) in shadow_entries {
                match field.as_str() {
                    "offset_x" => {
                        target.offset_x = parse_f32(field_val, &format!("shadows.{key}.offset_x"))?;
                    }
                    "offset_y" => {
                        target.offset_y = parse_f32(field_val, &format!("shadows.{key}.offset_y"))?;
                    }
                    "blur" => {
                        target.blur = parse_f32(field_val, &format!("shadows.{key}.blur"))?;
                    }
                    "spread" => {
                        target.spread = parse_f32(field_val, &format!("shadows.{key}.spread"))?;
                    }
                    "color" => {
                        target.color = parse_color(field_val, &format!("shadows.{key}.color"))?;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn apply_animation(
    animation: &mut AnimationTokens,
    value: &JsonValue,
) -> Result<(), ThemeJsonError> {
    let entries = value
        .as_object()
        .ok_or_else(|| ThemeJsonError::new("section 'animation' must be an object"))?;
    for (key, val) in entries {
        if let Some(target) = animation_field_mut(animation, key) {
            *target = parse_u32(val, &format!("animation.{key}"))?;
        }
    }
    Ok(())
}

// ── Minimal RFC 8259 JSON parser ─────────────────────────────────────────────

#[derive(Debug)]
enum JsonValue {
    Null,
    Bool,
    Number(String),
    String(String),
    Array,
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    fn as_object(&self) -> Option<&[(String, JsonValue)]> {
        match self {
            Self::Object(entries) => Some(entries),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    fn as_number_lexeme(&self) -> Option<&str> {
        match self {
            Self::Number(s) => Some(s),
            _ => None,
        }
    }
}

fn parse_json(input: &str) -> Result<JsonValue, ThemeJsonError> {
    let mut parser = JsonParser {
        src: input,
        bytes: input.as_bytes(),
        pos: 0,
    };
    parser.skip_ws();
    let value = parser.parse_value(0)?;
    parser.skip_ws();
    if parser.pos < parser.bytes.len() {
        return Err(ThemeJsonError::new(format!(
            "unexpected trailing token at byte {}",
            parser.pos
        )));
    }
    Ok(value)
}

struct JsonParser<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn skip_ws(&mut self) {
        while let Some(&b) = self.bytes.get(self.pos) {
            if matches!(b, b' ' | b'\t' | b'\n' | b'\r') {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn parse_value(&mut self, depth: usize) -> Result<JsonValue, ThemeJsonError> {
        if depth > MAX_JSON_DEPTH {
            return Err(ThemeJsonError::new("JSON nesting depth limit exceeded"));
        }
        self.skip_ws();
        let Some(b) = self.peek() else {
            return Err(ThemeJsonError::new("unexpected end of JSON input"));
        };
        match b {
            b'{' => self.parse_object(depth + 1),
            b'[' => self.parse_array(depth + 1),
            b'"' => self.parse_string().map(JsonValue::String),
            b't' => self.parse_literal("true", JsonValue::Bool),
            b'f' => self.parse_literal("false", JsonValue::Bool),
            b'n' => self.parse_literal("null", JsonValue::Null),
            b'-' | b'0'..=b'9' => self.parse_number().map(JsonValue::Number),
            other => Err(ThemeJsonError::new(format!(
                "unexpected byte 0x{other:02x} at byte {}",
                self.pos
            ))),
        }
    }

    fn parse_literal(
        &mut self,
        expected: &str,
        value: JsonValue,
    ) -> Result<JsonValue, ThemeJsonError> {
        if self.src[self.pos..].starts_with(expected) {
            self.pos += expected.len();
            Ok(value)
        } else {
            Err(ThemeJsonError::new(format!(
                "invalid literal near byte {}",
                self.pos
            )))
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<JsonValue, ThemeJsonError> {
        self.pos += 1; // '{'
        self.skip_ws();
        let mut entries = Vec::new();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(entries));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err(ThemeJsonError::new(format!(
                    "expected object key string at byte {}",
                    self.pos
                )));
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.peek() != Some(b':') {
                return Err(ThemeJsonError::new(format!(
                    "expected ':' after key '{key}' at byte {}",
                    self.pos
                )));
            }
            self.pos += 1;
            let value = self.parse_value(depth)?;
            entries.push((key, value));

            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(JsonValue::Object(entries));
                }
                _ => {
                    return Err(ThemeJsonError::new(format!(
                        "expected ',' or '}}' in object at byte {}",
                        self.pos
                    )));
                }
            }
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<JsonValue, ThemeJsonError> {
        self.pos += 1; // '['
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Array);
        }
        loop {
            let _ = self.parse_value(depth)?;
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(JsonValue::Array);
                }
                _ => {
                    return Err(ThemeJsonError::new(format!(
                        "expected ',' or ']' in array at byte {}",
                        self.pos
                    )));
                }
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, ThemeJsonError> {
        self.pos += 1; // opening '"'
        let mut out = String::new();
        loop {
            let start = self.pos;
            while let Some(&b) = self.bytes.get(self.pos) {
                if b == b'"' || b == b'\\' || b < 0x20 {
                    break;
                }
                self.pos += 1;
            }
            if self.pos > start {
                out.push_str(&self.src[start..self.pos]);
            }
            let Some(b) = self.peek() else {
                return Err(ThemeJsonError::new("unterminated JSON string"));
            };
            if b == b'"' {
                self.pos += 1;
                return Ok(out);
            }
            if b < 0x20 {
                return Err(ThemeJsonError::new(format!(
                    "unescaped control character in JSON string at byte {}",
                    self.pos
                )));
            }
            // Escape sequence after '\'
            self.pos += 1;
            let Some(esc) = self.peek() else {
                return Err(ThemeJsonError::new("unterminated string escape"));
            };
            self.pos += 1;
            match esc {
                b'"' => out.push('"'),
                b'\\' => out.push('\\'),
                b'/' => out.push('/'),
                b'b' => out.push('\u{0008}'),
                b'f' => out.push('\u{000C}'),
                b'n' => out.push('\n'),
                b'r' => out.push('\r'),
                b't' => out.push('\t'),
                b'u' => {
                    let ch = self.parse_unicode_escape()?;
                    out.push(ch);
                }
                other => {
                    return Err(ThemeJsonError::new(format!(
                        "invalid string escape '\\{}'",
                        other as char
                    )));
                }
            }
        }
    }

    fn parse_hex4(&mut self) -> Result<u16, ThemeJsonError> {
        let Some(end) = self.pos.checked_add(4) else {
            return Err(ThemeJsonError::new("incomplete \\uXXXX escape"));
        };
        let Some(raw) = self.bytes.get(self.pos..end) else {
            return Err(ThemeJsonError::new("incomplete \\uXXXX escape"));
        };
        let Ok(hex) = std::str::from_utf8(raw) else {
            return Err(ThemeJsonError::new("invalid non-ASCII \\uXXXX escape"));
        };
        self.pos = end;
        u16::from_str_radix(hex, 16)
            .map_err(|_| ThemeJsonError::new(format!("invalid hex escape '\\u{hex}'")))
    }

    fn parse_unicode_escape(&mut self) -> Result<char, ThemeJsonError> {
        let first = self.parse_hex4()?;
        if (0xD800..=0xDBFF).contains(&first) {
            if self.bytes.get(self.pos) != Some(&b'\\')
                || self.bytes.get(self.pos + 1) != Some(&b'u')
            {
                return Err(ThemeJsonError::new("unpaired high UTF-16 surrogate"));
            }
            self.pos += 2;
            let second = self.parse_hex4()?;
            if !(0xDC00..=0xDFFF).contains(&second) {
                return Err(ThemeJsonError::new("invalid low UTF-16 surrogate"));
            }
            let code_point =
                0x10000 + (((u32::from(first) - 0xD800) << 10) | (u32::from(second) - 0xDC00));
            char::from_u32(code_point)
                .ok_or_else(|| ThemeJsonError::new("invalid surrogate code point"))
        } else if (0xDC00..=0xDFFF).contains(&first) {
            Err(ThemeJsonError::new("unexpected low UTF-16 surrogate"))
        } else {
            char::from_u32(u32::from(first))
                .ok_or_else(|| ThemeJsonError::new("invalid unicode scalar value"))
        }
    }

    fn parse_number(&mut self) -> Result<String, ThemeJsonError> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        match self.peek() {
            Some(b'0') => {
                self.pos += 1;
            }
            Some(b'1'..=b'9') => {
                self.pos += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => {
                return Err(ThemeJsonError::new(format!(
                    "invalid number at byte {}",
                    self.pos
                )));
            }
        }
        if self.peek() == Some(b'.') {
            self.pos += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(ThemeJsonError::new(format!(
                    "expected fractional digit at byte {}",
                    self.pos
                )));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(ThemeJsonError::new(format!(
                    "expected exponent digit at byte {}",
                    self.pos
                )));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        Ok(self.src[start..self.pos].to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs::ThemePack;
    use crate::packs::high_contrast::HighContrastPack;

    #[test]
    fn round_trip_builtin_light_preserves_exact_fields() {
        let original = Theme::light();
        let json = original.to_json();
        let parsed = Theme::from_json(&json).expect("light theme JSON should parse");
        assert_eq!(parsed, original);
    }

    #[test]
    fn round_trip_builtin_dark_preserves_exact_fields() {
        let original = Theme::dark();
        let json = original.to_json();
        let parsed = Theme::from_json(&json).expect("dark theme JSON should parse");
        assert_eq!(parsed, original);
    }

    #[test]
    fn round_trip_high_contrast_light_and_dark_preserves_exact_fields() {
        for original in [HighContrastPack.light(), HighContrastPack.dark()] {
            let json = original.to_json();
            let parsed = Theme::from_json(&json).expect("high-contrast theme JSON should parse");
            assert_eq!(parsed, original);
        }
    }

    #[test]
    fn round_trip_all_registered_theme_packs_preserves_exact_fields() {
        for name in crate::packs::available_themes() {
            for mode in [ThemeMode::Light, ThemeMode::Dark] {
                let original = crate::packs::theme_by_name(name, mode).unwrap();
                let json = original.to_json();
                let parsed = Theme::from_json(&json)
                    .unwrap_or_else(|e| panic!("{name} ({mode:?}) failed JSON round-trip: {e}"));
                assert_eq!(parsed, original, "{name} ({mode:?}) round-trip mismatch");
            }
        }
    }

    #[test]
    fn partial_json_defaults_from_mode_and_applies_overrides() {
        let empty = Theme::from_json("{}").expect("empty object defaults to light");
        assert_eq!(empty, Theme::light());

        let dark_override = Theme::from_json(
            r#"{
                "mode": "dark",
                "colors": {
                    "primary": { "red": 1, "green": 0.5, "blue": 0 }
                },
                "radii": { "md": 10 },
                "animation": { "fast_ms": 50 },
                "control_heights": { "md": 36 }
            }"#,
        )
        .expect("partial dark override should parse");

        let mut expected = Theme::dark();
        expected.colors.primary = ThemeColor::rgba(1.0, 0.5, 0.0, 1.0);
        expected.radii.md = 10.0;
        expected.animation.fast_ms = 50;
        expected.control_heights.md = 36.0;
        assert_eq!(dark_override, expected);
    }

    #[test]
    fn omitted_selection_foreground_follows_overridden_accent_colors() {
        let theme = Theme::from_json(
            r#"{"colors": {
                "accent": { "red": 0.02, "green": 0.12, "blue": 0.3 },
                "accent_foreground": { "red": 1, "green": 1, "blue": 1 }
            }}"#,
        )
        .unwrap();
        let (bg, fg) = theme.selected_item_colors();
        assert_eq!(bg, ThemeColor::rgba(0.02, 0.12, 0.3, 1.0));
        assert_eq!(fg, theme.colors.accent_foreground);
        assert!(theme.contrast(fg, bg) >= 4.5);
    }

    #[test]
    fn explicit_selection_foreground_is_preserved_and_validated() {
        let input = r#"{"colors": {
            "accent": { "red": 0, "green": 0, "blue": 0 },
            "accent_foreground": { "red": 1, "green": 1, "blue": 1 },
            "selection_foreground": { "red": 0.8, "green": 0.8, "blue": 0.8 }
        }}"#;
        let theme = Theme::from_json(input).unwrap();
        assert_eq!(
            theme.selected_item_colors().1,
            ThemeColor::rgba(0.8, 0.8, 0.8, 1.0)
        );
        assert!(Theme::from_json(&input.replace("0.8", "0.02")).is_err());
    }

    #[test]
    fn omitted_disabled_foreground_stays_readable_on_custom_backgrounds() {
        for (mode, shade) in [("light", 0.05), ("dark", 0.95)] {
            let theme = Theme::from_json(&format!(
                r#"{{"mode": "{mode}", "colors": {{
                    "disabled_bg": {{ "red": {shade}, "green": {shade}, "blue": {shade} }}
                }}}}"#,
            ))
            .unwrap();
            assert_eq!(
                theme.colors.disabled_bg,
                ThemeColor::rgba(shade, shade, shade, 1.0)
            );
            assert!(
                theme.contrast(theme.colors.disabled_foreground, theme.colors.disabled_bg) >= 4.5
            );
        }
    }

    #[test]
    fn partial_shadow_and_typography_and_spacing_overrides_work() {
        let parsed = Theme::from_json(
            r#"{
                "spacing": { "px4": 20 },
                "typography": { "body": 15, "line_height": 1.6 },
                "shadows": { "sm": { "blur": 6, "color": { "red": 0, "green": 0, "blue": 0, "alpha": 0.25 } } }
            }"#,
        )
        .expect("partial spacing/typography/shadow override should parse");

        let mut expected = Theme::light();
        expected.spacing.px4 = 20.0;
        expected.typography.body = 15.0;
        expected.typography.line_height = 1.6;
        expected.shadows.sm.blur = 6.0;
        expected.shadows.sm.color = ThemeColor::rgba(0.0, 0.0, 0.0, 0.25);
        assert_eq!(parsed, expected);
    }

    #[test]
    fn rejects_malformed_json_syntax() {
        let invalid_inputs = [
            "",
            "   ",
            "{",
            r#"{"mode": "light",}"#,
            r#"{mode: "light"}"#,
            r#"{"mode": "light"} trailing"#,
            r#"{"spacing": {"px": 01}}"#,
            r#"{"spacing": {"px": .5}}"#,
            r#"{"spacing": {"px": 1.}}"#,
            r#"{"mode": "unterminated}"#,
        ];
        for input in invalid_inputs {
            let err = Theme::from_json(input).expect_err("malformed JSON must be rejected");
            assert!(!err.to_string().is_empty());
        }
    }

    #[test]
    fn rejects_wrong_field_types_and_invalid_mode() {
        let bad_types = [
            "[]",
            "42",
            r#"{"mode": "sepia"}"#,
            r#"{"mode": 123}"#,
            r#"{"colors": "blue"}"#,
            r##"{"colors": {"primary": "#ffffff"}}"##,
            r#"{"colors": {"primary": {"red": "1", "green": 0, "blue": 0, "alpha": 1}}}"#,
            r#"{"spacing": {"px": "4"}}"#,
            r#"{"animation": {"fast_ms": 100.5}}"#,
            r#"{"animation": {"fast_ms": -1}}"#,
            r#"{"shadows": {"sm": []}}"#,
        ];
        for input in bad_types {
            assert!(
                Theme::from_json(input).is_err(),
                "expected type error for: {input}"
            );
        }
    }

    #[test]
    fn rejects_non_finite_numbers_and_invalid_colors() {
        // Number exceeding f32::MAX overflows to infinity -> rejected as non-finite.
        assert!(Theme::from_json(r#"{"spacing": {"px": 1e40}}"#).is_err());
        assert!(Theme::from_json(r#"{"spacing": {"px": -1e40}}"#).is_err());

        // Color channel outside [0.0, 1.0] -> rejected by theme.validate().
        assert!(
            Theme::from_json(
                r#"{"colors": {"primary": {"red": 1.2, "green": 0, "blue": 0, "alpha": 1}}}"#
            )
            .is_err()
        );
        assert!(
            Theme::from_json(
                r#"{"colors": {"background": {"red": 0, "green": -0.1, "blue": 0, "alpha": 1}}}"#
            )
            .is_err()
        );
        assert!(
            Theme::from_json(
                r#"{"shadows": {"sm": {"color": {"red": 0, "green": 0, "blue": 0, "alpha": 1.5}}}}"#
            )
            .is_err()
        );
    }

    #[test]
    fn error_implements_std_error_and_display() {
        let err = Theme::from_json("not json").unwrap_err();
        let dyn_err: &dyn std::error::Error = &err;
        assert_eq!(dyn_err.to_string(), err.message());
    }

    #[test]
    fn rejects_non_ascii_unicode_escape_without_panicking() {
        let inputs = [
            r#"{"mode": "\u中文"}"#,
            r#"{"mode": "\u12中"}"#,
            r#"{"mode": "\uαβ"}"#,
        ];
        for input in inputs {
            assert!(
                Theme::from_json(input).is_err(),
                "expected clean error for {input}"
            );
        }
    }
}
