use super::*;
use acme_core::DrawCommand;

// Ignore upload transport: a later frame may reuse the same atlas glyphs.
fn header_paint(scene: &acme_core::Scene) -> Vec<DrawCommand> {
    let mut depth = 0;
    let mut paint = Vec::new();
    for command in scene.commands() {
        match command {
            DrawCommand::PushClip(_) => depth += 1,
            DrawCommand::PopClip => depth -= 1,
            DrawCommand::Text(text) if depth == 0 && text.origin.x.get() >= SIDEBAR_WIDTH => {
                let mut text = text.clone();
                text.uploads.clear();
                paint.push(DrawCommand::Text(text));
            }
            DrawCommand::Quad(quad) if depth == 0 && quad.rect.origin.x.get() >= SIDEBAR_WIDTH => {
                paint.push(command.clone());
            }
            _ => {}
        }
        assert!(depth >= 0);
    }
    assert_eq!(depth, 0);
    paint
}

#[test]
fn gallery_toolbar_paint_is_invariant_through_page_scroll_and_tour_transitions() {
    let mut g = Gallery::new();
    g.theme_pack = "default".into();
    for dark in [false, true] {
        g.dark = dark;
        for scale in [1.0, 1.25, 2.0] {
            for height in [760.0, 900.0] {
                let context = FrameContext {
                    window: WindowId(1),
                    logical_width: 1100.0,
                    logical_height: height,
                    scale_factor: scale,
                };
                // Start on the real initial page before switching to Overlay.
                g.selected_category = CATEGORIES.len() - 1;
                g.scroll = 0.0;
                let reference = header_paint(&g.frame(context.clone()));
                g.selected_category = 4;
                for stage in [0, 1, 2, 3, 0] {
                    g.tour.current = stage.min(2);
                    g.tour.finished = stage == 3;
                    for scroll in [0.0, 64.0, 128.0, f32::MAX] {
                        g.scroll = scroll;
                        let scene = g.frame(context.clone());
                        let paint = header_paint(&scene);
                        assert_eq!(
                            paint, reference,
                            "dark={dark}, scale={scale}, stage={stage}"
                        );
                        let (root, snapshot) = g.last_frame_layout.as_ref().unwrap();
                        let ids = extract_gallery_ids(root);
                        assert_eq!(snapshot.get(ids.toolbar).unwrap().height, TOOLBAR_HEIGHT);
                        for id in ids.toolbar_buttons {
                            let rect = snapshot.get(id).unwrap();
                            assert_eq!((rect.y, rect.height), (6.0, 28.0));
                        }
                        let text: Vec<_> = paint
                            .iter()
                            .filter_map(|command| match command {
                                DrawCommand::Text(text) => Some(text),
                                _ => None,
                            })
                            .collect();
                        assert_eq!(text.len(), 3);
                        for text in text {
                            assert!(!text.glyphs.is_empty());
                            for glyph in &text.glyphs {
                                let top = text.origin.y.get() * scale + glyph.y;
                                assert!(top >= 0.0 && top + glyph.height <= TOOLBAR_HEIGHT * scale);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn scrolled_gallery_toolbar_pointer_slots_remain_above_page_actions() {
    let mut g = Gallery::new();
    g.selected_category = 4;
    g.scroll = f32::MAX;
    g.frame(FrameContext {
        window: WindowId(1),
        logical_width: 1100.0,
        logical_height: 900.0,
        scale_factor: 1.0,
    });
    assert!(g.scroll > 0.0);
    for (x, message) in [
        (267.0, GalleryMessage::ToggleTheme),
        (385.0, GalleryMessage::CycleTheme),
        (503.0, GalleryMessage::DpiInfo),
    ] {
        g.event(PlatformEvent::PointerMoved {
            window: WindowId(1),
            x,
            y: 20.0,
        });
        let index = g.hit().expect("toolbar button must remain hittable");
        assert_eq!(g.hovered, Some(index));
        assert!(!g.button_info[index].scrolled);
        assert_eq!(g.button_info[index].message, message);
    }
}
