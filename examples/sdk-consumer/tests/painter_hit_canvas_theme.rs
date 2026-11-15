#![forbid(unsafe_code)]

use acme_core::{
    ClickDetector, CursorHint, DrawCommand, HitRegionSpec, HitTestLayer, Logical, NodeId, Painter,
    Point, PointerOutcome, Rect, RectStyle,
};
use acme_layout::LayoutEngine;
use acme_theme::packs::theme_by_name;
use acme_theme::{Theme, ThemeMode, contrast_ratio};
use acme_widgets::hit_bridge::build_hit_layer;
use acme_widgets::painter_bridge::{
    layout_rect_to_logical, paint_canvas_with_theme, paint_widget_tree_overlays_or_primitives,
    theme_color,
};
use acme_widgets::{WidgetNode, canvas, column};

#[derive(Clone, Debug, PartialEq, Eq)]
enum CanvasAction {
    Inspect(u32),
}

#[test]
fn external_painter_with_clip_and_offset_emits_valid_scene() {
    let theme = Theme::light();
    let mut painter = Painter::with_clear(theme_color(theme.colors.background));

    let header = Rect::new(0.0, 0.0, 400.0, 40.0);
    painter.rounded_rect_bordered(
        header,
        theme.radii.md,
        theme_color(theme.colors.surface),
        1.0,
        theme_color(theme.colors.border),
    );

    let viewport = Rect::new(0.0, 48.0, 400.0, 120.0);
    painter.with_clip(viewport, |painter| {
        painter.with_offset(0.0, -30.0, |painter| {
            // Visible inside viewport after -30px scroll offset (screen y = 90..130).
            painter.rect(
                Rect::new(16.0, 120.0, 180.0, 40.0),
                RectStyle::solid(theme_color(theme.colors.primary)).radius(theme.radii.sm),
            );
            // Scrolled completely above viewport clip (screen y = -20..10).
            painter.fill_rect(
                Rect::new(16.0, 10.0, 180.0, 30.0),
                theme_color(theme.colors.danger),
            );
        });
    });

    let scene = painter
        .finish_checked()
        .expect("consumer Painter scene must validate");
    let cmds = scene.commands();
    assert!(matches!(cmds.first(), Some(DrawCommand::Quad(_))));
    assert!(
        cmds.iter()
            .filter(|c| matches!(c, DrawCommand::PushClip(_)))
            .count()
            == cmds
                .iter()
                .filter(|c| matches!(c, DrawCommand::PopClip))
                .count()
    );
    // Header quad + clipped visible scrolled quad (the off-screen danger quad is culled).
    let quad_count = cmds
        .iter()
        .filter(|c| matches!(c, DrawCommand::Quad(_)))
        .count();
    assert_eq!(quad_count, 2);
}

#[test]
fn external_hit_layer_and_click_detector_enforce_scroll_clip_and_drag_cancel() {
    let fixed_btn = NodeId::new(10);
    let scrolled_hidden = NodeId::new(11);
    let scrolled_visible = NodeId::new(12);

    let mut layer = HitTestLayer::new();
    layer.push_region(
        HitRegionSpec::rect(fixed_btn, Rect::new(16.0, 12.0, 120.0, 36.0))
            .cursor(CursorHint::Pointer)
            .tag(1),
    );

    let viewport = Rect::new(0.0, 60.0, 400.0, 160.0);
    layer.with_scroll_viewport(viewport, 0.0, 80.0, |layer| {
        // After -80 scroll, y = 70..106 becomes -10..26 (clipped above viewport y=60).
        layer.push_region(
            HitRegionSpec::rect(scrolled_hidden, Rect::new(16.0, 70.0, 140.0, 36.0))
                .cursor(CursorHint::Pointer)
                .tag(2),
        );
        // After -80 scroll, y = 180..220 becomes 100..140 (inside viewport 60..220).
        layer.push_region(
            HitRegionSpec::rect(scrolled_visible, Rect::new(16.0, 180.0, 140.0, 40.0))
                .cursor(CursorHint::Grab)
                .tag(3),
        );
    });

    assert_eq!(
        layer.hit_test(Point::new(50.0, 30.0)).map(|h| h.id),
        Some(fixed_btn)
    );
    assert_eq!(layer.hit_test(Point::new(50.0, 10.0)), None);
    assert_eq!(
        layer.hit_test_cursor(Point::new(50.0, 120.0)),
        CursorHint::Grab
    );

    let mut detector = ClickDetector::new();
    let visible_pt = Point::new(50.0, 120.0);
    let move_outcomes = detector.on_pointer_move(&layer, visible_pt);
    assert_eq!(
        move_outcomes,
        vec![PointerOutcome::HoverEnter {
            id: scrolled_visible,
            cursor: CursorHint::Grab,
        }]
    );

    let press = detector.on_pointer_button(&layer, 0, true, visible_pt, 0.10);
    assert!(matches!(
        press.as_slice(),
        [PointerOutcome::Pressed { id, button: 0, .. }] if *id == scrolled_visible
    ));
    let release = detector.on_pointer_button(&layer, 0, false, visible_pt, 0.15);
    assert!(release.iter().any(|o| matches!(
        o,
        PointerOutcome::Clicked {
            id,
            button: 0,
            count: 1,
            ..
        } if *id == scrolled_visible
    )));

    // Drag outside past threshold cancels click cleanly.
    let _ = detector.on_pointer_button(&layer, 0, true, Point::new(50.0, 30.0), 1.0);
    let drag_outcomes = detector.on_pointer_move(&layer, Point::new(300.0, 30.0));
    assert!(matches!(
        drag_outcomes.as_slice(),
        [PointerOutcome::DragStart { id, .. }] if *id == fixed_btn
    ));
    let drag_release = detector.on_pointer_button(&layer, 0, false, Point::new(300.0, 30.0), 1.1);
    assert!(
        !drag_release
            .iter()
            .any(|o| matches!(o, PointerOutcome::Clicked { .. }))
    );
    assert!(
        drag_release
            .iter()
            .any(|o| matches!(o, PointerOutcome::DragEnd { .. }))
    );
}

#[test]
fn external_canvas_widget_paints_via_bridge_and_dispatches_clicks() {
    let theme = Theme::dark();
    let tree = column::<CanvasAction>()
        .key("canvas-container")
        .padding(12.0)
        .child(
            canvas()
                .key("chart-canvas")
                .size(200.0, 64.0)
                .cursor(CursorHint::Pointer)
                .on_click(CanvasAction::Inspect(42))
                .paint(|painter, rect, theme| {
                    painter.rounded_rect_bordered(
                        rect,
                        theme.radii.md,
                        theme_color(theme.colors.surface),
                        1.0,
                        theme_color(theme.colors.border),
                    );
                    painter.fill_rect(
                        Rect::new(
                            rect.origin.x.get() + 8.0,
                            rect.origin.y.get() + 8.0,
                            40.0,
                            rect.size.height.get() - 16.0,
                        ),
                        theme_color(theme.colors.accent),
                    );
                })
                .build(),
        )
        .build();

    let layout = tree.to_layout(NodeId::new(1));
    let mut engine = LayoutEngine::new();
    let snapshot = engine
        .compute(&layout, (320.0, 120.0))
        .expect("canvas layout must succeed");
    let canvas_layout = &layout.children[0];
    let canvas_rect = layout_rect_to_logical(*snapshot.get(canvas_layout.id).unwrap());

    let mut painter = Painter::with_clear(theme_color(theme.colors.background));
    paint_widget_tree_overlays_or_primitives(&tree, &layout, &snapshot, &theme, &mut painter);
    if let WidgetNode::Column(col) = &tree
        && let WidgetNode::Canvas(c) = &col.children[0]
    {
        paint_canvas_with_theme(c, canvas_rect, &theme, &mut painter);
    }
    let scene = painter.finish_checked().expect("canvas scene valid");
    assert!(
        scene
            .commands()
            .iter()
            .any(|cmd| matches!(cmd, DrawCommand::Quad(_)))
    );

    let viewport = Rect::<Logical>::new(0.0, 0.0, 320.0, 120.0);
    let registry = build_hit_layer(&tree, &layout, &snapshot, viewport);
    let mut detector = ClickDetector::new();
    let hit_pt = Point::new(
        canvas_rect.origin.x.get() + canvas_rect.size.width.get() * 0.5,
        canvas_rect.origin.y.get() + canvas_rect.size.height.get() * 0.5,
    );

    let hover = registry.handle_pointer_move(&mut detector, hit_pt);
    assert_eq!(hover.cursor, CursorHint::Pointer);

    let _ = registry.handle_pointer_button(&mut detector, 0, true, hit_pt, 0.1);
    let release = registry.handle_pointer_button(&mut detector, 0, false, hit_pt, 0.2);
    assert_eq!(release.messages, vec![CanvasAction::Inspect(42)]);
}

#[test]
fn external_theme_json_round_trip_and_high_contrast_pack() {
    for mode in [ThemeMode::Light, ThemeMode::Dark] {
        let base = match mode {
            ThemeMode::Light => Theme::light(),
            ThemeMode::Dark => Theme::dark(),
        };
        let json = base.to_json();
        let parsed = Theme::from_json(&json).expect("Theme::from_json must parse Theme::to_json");
        assert_eq!(parsed, base);

        let hc = theme_by_name("high-contrast", mode).expect("high-contrast pack available");
        let hc_roundtrip =
            Theme::from_json(&hc.to_json()).expect("high-contrast JSON must round-trip");
        assert_eq!(hc_roundtrip, hc);
        assert!(contrast_ratio(hc.colors.foreground, hc.colors.background) >= 7.0);
    }
}
