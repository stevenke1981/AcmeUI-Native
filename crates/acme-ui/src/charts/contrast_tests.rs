//! ROADMAP 062: every non-text chart mark must reach 3:1 (WCAG 1.4.11) against
//! the colour it is painted on, for every theme pack and mode.

use super::*;
use crate::WidgetNode;
use acme_core::{Color, DrawCommand, Painter, Rect};
use acme_theme::packs::{available_themes, theme_by_name};
use acme_theme::{Theme, ThemeColor, ThemeMode, contrast_ratio};

#[derive(Clone, Debug, PartialEq)]
enum Msg {}

fn all_themes() -> Vec<(String, Theme)> {
    let mut out = vec![
        ("default-light".to_string(), Theme::light()),
        ("default-dark".to_string(), Theme::dark()),
    ];
    for name in available_themes() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = theme_by_name(name, mode).expect("registered pack resolves");
            out.push((format!("{name}/{mode:?}"), theme));
        }
    }
    out
}

fn tc(c: Color) -> ThemeColor {
    ThemeColor::rgba(c.r, c.g, c.b, 1.0)
}

fn same(a: ThemeColor, b: ThemeColor) -> bool {
    (a.red - b.red).abs() < 1e-4
        && (a.green - b.green).abs() < 1e-4
        && (a.blue - b.blue).abs() < 1e-4
}

fn quad_colors(node: WidgetNode<Msg>, size: (f32, f32), theme: &Theme) -> Vec<ThemeColor> {
    let WidgetNode::Canvas(c) = &node else {
        panic!("expected Canvas");
    };
    let mut painter = Painter::new();
    c.paint_into(&mut painter, Rect::new(0.0, 0.0, size.0, size.1), theme);
    painter
        .finish()
        .commands()
        .iter()
        .filter_map(|cmd| match cmd {
            DrawCommand::Quad(q) => Some(tc(q.color)),
            _ => None,
        })
        .collect()
}

/// Marks = quads that are not the chart's own structure (fill, border/grid,
/// muted gauge track). Each mark must reach 3:1 vs `on`, and none may be the
/// soft `accent` tint.
fn check(
    chart: &str,
    name: &str,
    theme: &Theme,
    quads: Vec<ThemeColor>,
    on: ThemeColor,
    fails: &mut Vec<String>,
) {
    let c = &theme.colors;
    let accent_is_distinct = ![
        c.primary,
        c.info,
        c.success,
        c.warning,
        c.ring,
        c.foreground,
    ]
    .iter()
    .any(|t| same(*t, c.accent));
    let mut marks = 0;
    for q in quads {
        if same(q, c.surface) || same(q, c.border) || same(q, c.muted) {
            continue;
        }
        marks += 1;
        let ratio = contrast_ratio(q, on);
        if ratio < 3.0 {
            fails.push(format!(
                "{chart} on {name}: mark ({:.2},{:.2},{:.2}) is {ratio:.2}:1 vs fill",
                q.red, q.green, q.blue
            ));
        }
        if accent_is_distinct && same(q, c.accent) {
            fails.push(format!("{chart} on {name}: paints a mark with accent"));
        }
    }
    assert!(marks > 0, "{chart} on {name}: no marks painted");
}

#[test]
fn bar_chart_marks_reach_3_to_1_on_every_pack() {
    let mut fails = Vec::new();
    for (name, theme) in all_themes() {
        let mut b = bar_chart::<Msg>("b").height(160.0).max_value(100.0);
        for i in 0..5 {
            b = b.bar(BarEntry::new(format!("b{i}"), 40.0 + i as f32 * 10.0));
        }
        let q = quad_colors(b.into(), (320.0, 160.0), &theme);
        check(
            "bar_chart",
            &name,
            &theme,
            q,
            theme.colors.surface,
            &mut fails,
        );
    }
    assert!(
        fails.is_empty(),
        "{}",
        fails.join(
            "
"
        )
    );
}

/// Palette order in `bar_chart`: primary, success, info, warning, danger.
/// Luminance contrast is blind to hue (green vs. yellow can be 1.0:1), so
/// separation is measured as Euclidean sRGB distance (0..sqrt(3)).
/// 0.10 is about 25/255 per channel: visibly different swatches, not identical.
const MIN_PALETTE_DISTANCE: f32 = 0.10;

fn rgb_distance(a: ThemeColor, b: ThemeColor) -> f32 {
    ((a.red - b.red).powi(2) + (a.green - b.green).powi(2) + (a.blue - b.blue).powi(2)).sqrt()
}

#[test]
fn bar_chart_palette_is_distinct_including_wrap() {
    let mut fails = Vec::new();
    for (name, theme) in all_themes() {
        // 6 bars: palette slots 0..=4 then wrap back to slot 0.
        let mut b = bar_chart::<Msg>("b").height(160.0).max_value(100.0);
        for i in 0..6 {
            b = b.bar(BarEntry::new(format!("b{i}"), 50.0));
        }
        let q = quad_colors(b.into(), (320.0, 160.0), &theme);
        let m = &q[q.len() - 6..];
        if !same(m[5], m[0]) {
            fails.push(format!("{name}: palette does not wrap with period 5"));
        }
        // Hard: every visually adjacent pair (incl. wrap 4 -> 0), and the
        // fifth slot against every other slot.
        for i in 0..5 {
            for j in (i + 1)..5 {
                let adjacent = j == i + 1 || (i == 0 && j == 4);
                if !(adjacent || j == 4) {
                    continue;
                }
                let d = rgb_distance(m[i], m[j]);
                if d < MIN_PALETTE_DISTANCE {
                    fails.push(format!(
                        "{name}: palette slots {i}/{j} distance {d:.3} (contrast {:.2}:1)",
                        contrast_ratio(m[i], m[j])
                    ));
                }
            }
        }
        // Informational: non-adjacent pairs that are theme-level collisions.
        for (i, j) in [(0usize, 2usize), (0, 3), (1, 3), (1, 4)] {
            let d = rgb_distance(m[i], m[j]);
            if d < MIN_PALETTE_DISTANCE {
                eprintln!("CLOSE-PAIR {name}: slots {i}/{j} distance {d:.3}");
            }
        }
    }
    assert!(
        fails.is_empty(),
        "{}",
        fails.join(
            "
"
        )
    );
}

fn only_primary_marks(
    chart: &str,
    name: &str,
    theme: &Theme,
    quads: Vec<ThemeColor>,
    allow_foreground: bool,
    fails: &mut Vec<String>,
) {
    let c = &theme.colors;
    for q in quads {
        if same(q, c.surface) || same(q, c.border) || same(q, c.muted) {
            continue;
        }
        if same(q, c.primary) || (allow_foreground && same(q, c.foreground)) {
            continue;
        }
        fails.push(format!(
            "{chart} on {name}: mark ({:.2},{:.2},{:.2}) is neither primary nor foreground",
            q.red, q.green, q.blue
        ));
    }
}

#[test]
fn line_sparkline_gauge_marks_use_primary() {
    let mut fails = Vec::new();
    for (name, theme) in all_themes() {
        let mut l = line_chart::<Msg>("l").width(320.0).height(160.0);
        for i in 0..5 {
            l = l.point(ChartPoint::new(format!("p{i}"), 10.0 + i as f32 * 7.0));
        }
        let q = quad_colors(l.into(), (320.0, 160.0), &theme);
        only_primary_marks("line_chart", &name, &theme, q, false, &mut fails);

        let mut s = sparkline::<Msg>("s").width(120.0).height(24.0);
        for v in [1.0, 4.0, 2.0, 6.0, 3.0] {
            s = s.value(v);
        }
        let q = quad_colors(s.into(), (120.0, 24.0), &theme);
        only_primary_marks("sparkline", &name, &theme, q, false, &mut fails);

        let g = gauge::<Msg>("g").size(120.0).max(100.0).value(72.0);
        let q = quad_colors(g.into(), (120.0, 120.0), &theme);
        only_primary_marks("gauge", &name, &theme, q, true, &mut fails);
    }
    assert!(
        fails.is_empty(),
        "{}",
        fails.join(
            "
"
        )
    );
}

#[test]
fn line_chart_marks_reach_3_to_1_on_every_pack() {
    let mut fails = Vec::new();
    for (name, theme) in all_themes() {
        let mut l = line_chart::<Msg>("l").width(320.0).height(160.0);
        for i in 0..5 {
            l = l.point(ChartPoint::new(format!("p{i}"), 10.0 + i as f32 * 7.0));
        }
        let q = quad_colors(l.into(), (320.0, 160.0), &theme);
        check(
            "line_chart",
            &name,
            &theme,
            q,
            theme.colors.surface,
            &mut fails,
        );
    }
    assert!(
        fails.is_empty(),
        "{}",
        fails.join(
            "
"
        )
    );
}

#[test]
fn gauge_marks_reach_3_to_1_on_every_pack() {
    let mut fails = Vec::new();
    for (name, theme) in all_themes() {
        let g = gauge::<Msg>("g").size(120.0).max(100.0).value(72.0);
        let q = quad_colors(g.into(), (120.0, 120.0), &theme);
        check(
            "gauge",
            &name,
            &theme,
            q,
            theme.colors.background,
            &mut fails,
        );
    }
    assert!(
        fails.is_empty(),
        "{}",
        fails.join(
            "
"
        )
    );
}

#[test]
fn sparkline_marks_reach_3_to_1_on_every_pack() {
    let mut fails = Vec::new();
    for (name, theme) in all_themes() {
        let mut s = sparkline::<Msg>("s").width(120.0).height(24.0);
        for v in [1.0, 4.0, 2.0, 6.0, 3.0] {
            s = s.value(v);
        }
        let q = quad_colors(s.into(), (120.0, 24.0), &theme);
        check(
            "sparkline",
            &name,
            &theme,
            q,
            theme.colors.background,
            &mut fails,
        );
    }
    assert!(
        fails.is_empty(),
        "{}",
        fails.join(
            "
"
        )
    );
}
