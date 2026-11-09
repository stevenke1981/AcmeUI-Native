use super::*;

fn render(
    state: &mut TextInputState,
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
    theme: &Theme,
    scale: f32,
    width: f32,
) -> Scene {
    let mut scene = Scene::default();
    render_text_input(
        &mut scene,
        state,
        fonts,
        atlas,
        [20.0, 30.0, width, 44.0],
        theme,
        scale,
        state.focused,
        None,
    );
    scene
}

#[test]
fn committed_placeholder_and_composing_ink_stay_inside_the_content_height() {
    let mut fonts = FontSystem::new();
    let mut atlas = GlyphAtlas::default();
    let theme = Theme::light();
    for scale in [1.0, 1.25, 2.0] {
        for mode in [0, 1, 2] {
            let mut state = TextInputState::new();
            state.focused = true;
            match mode {
                0 => {
                    handle_text(&mut state, "Hgj");
                }
                1 => {
                    state.placeholder = "Hgj".into();
                }
                _ => {
                    handle_text(&mut state, "HH");
                    state.cursor_home();
                    state.set_preedit("Hgj", None);
                }
            }
            let scene = render(&mut state, &mut fonts, &mut atlas, &theme, scale, 240.0);
            let mut count = 0;
            for command in scene.commands() {
                if let DrawCommand::Text(text) = command {
                    for glyph in &text.glyphs {
                        count += 1;
                        let y = text.origin.y.get() + glyph.y / scale;
                        assert!(
                            y >= 30.0 + theme.spacing.px2 - 1.0,
                            "ink above content: scale={scale}, mode={mode}, y={y}"
                        );
                        assert!(
                            y + glyph.height / scale <= 74.0 - theme.spacing.px2 + 1.0,
                            "ink below content"
                        );
                    }
                }
            }
            assert!(count > 0);
        }
    }
}

#[test]
fn cached_input_matches_fresh_shaping_after_dpi_width_and_typography_changes() {
    let mut fonts = FontSystem::new();
    let mut atlas = GlyphAtlas::default();
    let mut cached = TextInputState::new();
    handle_text(&mut cached, "Query");
    let mut theme = Theme::light();
    render(&mut cached, &mut fonts, &mut atlas, &theme, 1.0, 240.0);
    for (scale, width, font_size) in [(2.0, 120.0, 14.0), (1.25, 320.0, 18.0), (1.0, 180.0, 14.0)] {
        theme.typography.body = font_size;
        let actual = render(&mut cached, &mut fonts, &mut atlas, &theme, scale, width);
        let mut fresh = TextInputState::new();
        handle_text(&mut fresh, "Query");
        let expected = render(&mut fresh, &mut fonts, &mut atlas, &theme, scale, width);
        let geometry = |scene: &Scene| {
            scene
                .commands()
                .iter()
                .filter_map(|c| {
                    if let DrawCommand::Text(t) = c {
                        Some((
                            t.origin,
                            t.glyphs
                                .iter()
                                .map(|g| (g.x, g.y, g.width, g.height))
                                .collect::<Vec<_>>(),
                        ))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            geometry(&actual),
            geometry(&expected),
            "cache reused stale shaping"
        );
    }
}
