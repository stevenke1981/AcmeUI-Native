//! Compiled checks for the code snippets in `docs/guides/MIGRATION.md`.
//!
//! One `m{NN}_` group per M-item. Each "after" snippet is copied from the guide
//! into a function here, so a snippet that stops compiling fails `cargo test`.
//! Identifiers that a snippet leaves free (for example `fonts` or `adapter`) are
//! declared by the surrounding function. Snippets labelled "before" describe the
//! removed API and are not compiled.
//!
//! Some M-items have no code snippet. Their tests pin the cited signature or
//! check the documented behaviour. M4 (benchmark CLI) and M10 (app-only changes)
//! have no library API and therefore no test here.

use acme_accessibility::{AccNodeId, AccessibilityAction, AccessibilityAdapter};
use acme_core::{DirtyFlags, NodeId, RetainedTree, Scene, ViewNode};
use acme_layout::{LayoutEngine, LayoutSnapshot, WidgetLayoutContext};
use acme_platform::{Application, FrameContext, PlatformError, PlatformEvent};
use acme_render_wgpu::batch::{RenderBatch, compile_scene};
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextLayout, TextStyle};
use acme_textinput::TextInputState;
use acme_ui::prelude::{WidgetNode, button, card, label};
use acme_widgets::retained_binding::bind_retained;
use acme_widgets::view_bridge::widget_to_views;

// ---------------------------------------------------------------------------
// M1. PlatformError and run() (breaking)
// ---------------------------------------------------------------------------

/// Snippet "After" from M1, wrapped in a function. Not called: `run` opens a window.
#[allow(dead_code)]
fn m01_run_and_report(app: impl Application) {
    match acme_platform::run(app) {
        Ok(()) => {}
        Err(acme_platform::PlatformError::NoWindows { reasons }) => {
            eprintln!("no window could be opened: {}", reasons.join("; "));
            std::process::exit(1);
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[test]
fn m01_platform_error_messages_match_the_guide() {
    let no_windows = PlatformError::NoWindows {
        reasons: vec!["first".into(), "second".into()],
    };
    assert_eq!(
        no_windows.to_string(),
        "no window could be created: first; second"
    );
    let renderer = PlatformError::Renderer("device lost".into());
    assert_eq!(
        renderer.to_string(),
        "unrecoverable renderer failure: device lost"
    );
}

// ---------------------------------------------------------------------------
// M2. Renderer selection through ACME_RENDER_BACKEND
// ---------------------------------------------------------------------------

#[test]
fn m02_render_backend_env_value_snippet() {
    use acme_platform::RenderBackendPreference;

    assert_eq!(
        RenderBackendPreference::from_env_value(Some(" GPU ")),
        Ok(RenderBackendPreference::Gpu)
    );
    // Other rules from the M2 parsing table.
    assert_eq!(
        RenderBackendPreference::from_env_value(None),
        Ok(RenderBackendPreference::Auto)
    );
    assert_eq!(
        RenderBackendPreference::from_env_value(Some("   ")),
        Ok(RenderBackendPreference::Auto)
    );
    assert_eq!(
        RenderBackendPreference::from_env_value(Some("CPU")),
        Ok(RenderBackendPreference::Cpu)
    );
    // from_env_value rejects unknown text with an error. The M2 table says the
    // result is Auto with a warning; that is true of from_env, not from_env_value.
    assert!(RenderBackendPreference::from_env_value(Some("cpuu")).is_err());
}

// ---------------------------------------------------------------------------
// M3. GlyphAtlas eviction moved to begin_frame (behaviour)
// ---------------------------------------------------------------------------

/// Stand-in for the per-frame text state that the M3 snippet runs in.
struct TextFrame {
    atlas: GlyphAtlas,
}

impl TextFrame {
    /// Snippet from M3, with the surrounding function signature.
    #[allow(unused_variables, clippy::let_and_return)]
    fn draw_text(&mut self, fonts: &mut FontSystem, layout: TextLayout) -> usize {
        self.atlas.begin_frame();
        let prepared = fonts.prepare(&layout, &mut self.atlas);

        // Counts glyphs that were dropped during this prepare call.
        let dropped = fonts.diagnostics().atlas_full_glyphs;
        dropped
    }
}

#[test]
fn m03_glyph_atlas_begin_frame_snippet_drops_nothing_in_a_roomy_atlas() {
    let mut frame = TextFrame {
        atlas: GlyphAtlas::new(2048, 2048),
    };
    let mut fonts = FontSystem::new();
    let style = TextStyle::default();
    let layout = fonts.shape("Hello", &style, TextConstraints::default(), 1.0);

    let dropped = frame.draw_text(&mut fonts, layout);

    assert_eq!(dropped, 0, "a short line must fit a 2048x2048 atlas");
}

// ---------------------------------------------------------------------------
// M4. apps/benchmark CLI: no library API, no test here.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// M5. Retained binding API (additive)
// ---------------------------------------------------------------------------

/// Snippet "Pattern" from M5, wrapped in a function that declares its locals.
/// Returns the accessibility adapter and the retained root ID.
fn m05_bind_and_update(
    description: &WidgetNode<()>,
) -> Result<(AccessibilityAdapter, NodeId), Box<dyn std::error::Error>> {
    let mut tree = RetainedTree::new();
    let mut layout_engine = LayoutEngine::new();
    let mut fonts = FontSystem::new();
    let layout_context = WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 20.0,
        label_font_size: 12.0,
        control_height: 36.0,
        scale_factor: 1.0,
    };
    let viewport = (640.0_f32, 420.0_f32);
    let scale_factor = 1.0_f32;
    let mut accessibility = AccessibilityAdapter::new(1);

    // ---- snippet from M5 ----
    let views = widget_to_views(std::slice::from_ref(description));
    if tree.reconcile_roots(&views).is_ok() {
        let binding = bind_retained(std::slice::from_ref(description), &tree);
        if binding.mismatches() == 0 {
            let root = description.to_layout_with_binding(&binding, &layout_context);
            let snapshot =
                layout_engine.compute_with_text(&root, viewport, &mut fonts, scale_factor)?;
            accessibility.update_with_binding(description, &snapshot, &binding);
        }
    }
    // ---- end snippet ----

    let root_id = tree.roots()[0];
    Ok((accessibility, root_id))
}

#[test]
fn m05_retained_binding_snippet_uses_the_retained_root_id() {
    let description = card::<()>().child(label("Retained")).build();

    let (accessibility, root_id) =
        m05_bind_and_update(&description).expect("the binding path should lay out");

    let update = accessibility
        .build_tree_update()
        .expect("update_with_binding should store a tree");
    let tree_root = update.tree.as_ref().expect("tree info").root;
    assert_eq!(
        tree_root,
        AccNodeId::from(root_id.get()),
        "the root AccessKit ID should be the retained root ID"
    );
}

// ---------------------------------------------------------------------------
// M6. AccessibilityAdapter role/state changes; snippet for clear_widget_focus
// ---------------------------------------------------------------------------

/// Sequential layout and accessibility update for a description (no binding).
fn sequential_adapter(description: &WidgetNode<()>) -> AccessibilityAdapter {
    let root = description.to_layout(NodeId::new(1));
    let snapshot: LayoutSnapshot = LayoutEngine::new()
        .compute_with_text(&root, (640.0, 420.0), &mut FontSystem::new(), 1.0)
        .expect("sequential layout should succeed");
    let mut adapter = AccessibilityAdapter::new(1);
    adapter.update(description, &snapshot);
    adapter
}

fn focused_id(adapter: &AccessibilityAdapter) -> AccNodeId {
    adapter
        .build_tree_update()
        .expect("tree should be stored")
        .focus
}

fn root_id_of(adapter: &AccessibilityAdapter) -> AccNodeId {
    adapter
        .build_tree_update()
        .expect("tree should be stored")
        .tree
        .expect("tree info")
        .root
}

#[test]
fn m06_clear_widget_focus_snippet_moves_focus_to_the_root() {
    let description = card::<()>().child(button::<()>("ok", "OK")).build();
    let mut adapter = sequential_adapter(&description);
    // A focus target with no accessibility node, for example a synthetic table row.
    adapter.focus_widget(NodeId::new(9_999));
    assert_ne!(focused_id(&adapter), root_id_of(&adapter));

    // ---- snippet from M6 ----
    // Clear focus when keyboard focus moves to a row that has no accessibility node.
    adapter.clear_widget_focus();
    // ---- end snippet ----

    assert_eq!(focused_id(&adapter), root_id_of(&adapter));
}

// ---------------------------------------------------------------------------
// M7. AccessibilityAdapter::clear_widget_focus (additive)
// ---------------------------------------------------------------------------

#[test]
fn m07_clear_widget_focus_signature_and_idempotence() {
    let _signature: fn(&mut AccessibilityAdapter) = AccessibilityAdapter::clear_widget_focus;
    let description = card::<()>().child(button::<()>("ok", "OK")).build();
    let mut adapter = sequential_adapter(&description);

    adapter.clear_widget_focus();
    adapter.clear_widget_focus();

    assert_eq!(focused_id(&adapter), root_id_of(&adapter));
}

// ---------------------------------------------------------------------------
// M8. Text input (acme-textinput)
// ---------------------------------------------------------------------------

#[test]
fn m08_scroll_caret_into_view_snippet_and_readonly_undo() {
    let _signature: fn(&mut TextInputState, &mut FontSystem, &TextStyle, f32, f32) =
        acme_textinput::scroll_caret_into_view;

    // ---- snippet from M8 ----
    let mut state = TextInputState::new();
    let mut fonts = FontSystem::new();
    let style = TextStyle::default();
    let scale = 1.0;
    let content_width = 240.0;
    acme_textinput::scroll_caret_into_view(&mut state, &mut fonts, &style, scale, content_width);
    // ---- end snippet ----

    // Control: an editable field can undo its insert.
    let mut editable = TextInputState::new();
    editable.insert_char('a');
    assert!(editable.undo(), "control: an editable field can undo");

    // M8: undo on a read-only field returns false and leaves the text alone.
    let mut locked = TextInputState::new();
    locked.insert_char('a');
    locked.readonly = true;
    assert!(!locked.undo(), "undo must be refused on a read-only field");
    assert_eq!(locked.display_text(), "a");
}

// ---------------------------------------------------------------------------
// M9. Core invalidation (acme-core)
// ---------------------------------------------------------------------------

#[test]
fn m09_child_change_dirties_parent_and_unchanged_frame_stays_clean() {
    let mut tree = RetainedTree::new();
    let frame_one = ViewNode::new("root", "Column").child(ViewNode::new("a", "Label"));
    tree.reconcile_roots(std::slice::from_ref(&frame_one))
        .expect("first frame");
    let root = tree.roots()[0];
    tree.clear_dirty(root);

    let frame_two = frame_one.clone().child(ViewNode::new("b", "Label"));
    tree.reconcile_roots(std::slice::from_ref(&frame_two))
        .expect("second frame");
    let dirty = tree.get(root).expect("root node").dirty;
    assert!(dirty.contains(DirtyFlags::CHILDREN), "child list changed");
    assert!(dirty.contains(DirtyFlags::LAYOUT), "layout can change");

    tree.clear_dirty(root);
    tree.reconcile_roots(std::slice::from_ref(&frame_two))
        .expect("unchanged frame");
    assert_eq!(tree.get(root).expect("root node").dirty, DirtyFlags::NONE);
}

// ---------------------------------------------------------------------------
// M10. Gallery and Playground changes: apps only, no library API, no test here.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// M11. RenderBatch.clip type (breaking)
// ---------------------------------------------------------------------------

#[test]
fn m11_render_batch_clip_is_an_f32_rect() {
    // Compile-time pin: this stops compiling if the field goes back to Option<[u32; 4]>.
    fn clip_of(batch: &RenderBatch) -> Option<[f32; 4]> {
        batch.clip
    }

    let batches = compile_scene(&Scene::new());
    assert!(batches.is_empty(), "an empty scene has no batches");
    assert!(batches.iter().map(clip_of).all(|clip| clip.is_none()));
}

// ---------------------------------------------------------------------------
// M12. Application default methods (additive)
// ---------------------------------------------------------------------------

/// An implementer that keeps the defaults for the M12 methods.
struct KeepsDefaults;

impl Application for KeepsDefaults {
    fn frame(&mut self, _context: FrameContext) -> Scene {
        Scene::new()
    }
}

#[test]
fn m12_background_defaults_are_inert() {
    let mut app = KeepsDefaults;
    assert!(app.background_poll_interval().is_none());
    assert!(!app.background_tick());
}

// ---------------------------------------------------------------------------
// M13. route_action returns Vec<PlatformEvent> (documentation error)
// ---------------------------------------------------------------------------

#[test]
fn m13_route_action_returns_a_vec_of_events() {
    let _signature: fn(&AccessibilityAdapter, &AccessibilityAction) -> Vec<PlatformEvent> =
        AccessibilityAdapter::route_action;
    let adapter = AccessibilityAdapter::new(1);

    let events = adapter.route_action(&AccessibilityAction::Focus(NodeId::new(7)));

    assert!(
        matches!(
            events.as_slice(),
            [PlatformEvent::FocusChanged {
                gained: true,
                node_id: 7,
                ..
            }]
        ),
        "focus action should route to one FocusChanged event, got {events:?}"
    );
}
