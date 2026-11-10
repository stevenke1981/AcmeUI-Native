//! Representative P3 demo presence and real retained layout, not manual acceptance.
use super::*;
fn find<'a>(
    widget: &'a WidgetNode<GalleryMessage>,
    layout: &'a LayoutNode,
    key: &str,
) -> Vec<(&'a WidgetNode<GalleryMessage>, &'a LayoutNode)> {
    let mut found = Vec::new();
    if widget.key().is_some_and(|k| k.as_str() == key) {
        found.push((widget, layout));
    }
    for (w, l) in widget.children().iter().zip(&layout.children) {
        found.extend(find(w, l, key));
    }
    found
}
#[test]
fn all_fifteen_p3_builders_have_gallery_examples_with_stable_bound_roots() {
    use acme_layout::LayoutKind::*;
    let expected = [
        ("gallery_action_share", Row),
        ("gallery_avatar_md", Stack),
        ("gallery_banner_info", Row),
        ("mobile_md", Leaf),
        ("gallery_card_title", Column),
        ("gallery_chip_static", Row),
        ("gallery_list_wifi", Row),
        ("gallery_loader_indicator", Column),
        ("gallery_notification_active", Column),
        ("gallery_progress_quantity", Column),
        ("gallery_search", Row),
        ("gallery_mobile_segmented", Row),
        ("gallery_handle_default", Row),
        ("gallery_mobile_stepper", Row),
        ("gallery_mobile_toggle", Stack),
    ];
    let mut g = Gallery::new();
    let mut saved = None;
    for (pack, dark, scale) in [
        ("default", false, 1.0),
        ("apple", true, 1.25),
        ("default", false, 2.0),
    ] {
        g.theme_pack = pack.into();
        g.dark = dark;
        g.frame(FrameContext {
            window: WindowId(1),
            logical_width: 1100.0,
            logical_height: 1100.0,
            scale_factor: scale,
        });
        let widget = g.description();
        let (layout, snapshot) = g.last_frame_layout.as_ref().unwrap();
        let binding = bind_retained(std::slice::from_ref(&widget), &g.retained.tree);
        assert_eq!(binding.mismatches(), 0);
        let ids = expected.map(|(key, kind)| {
            let found = find(&widget, layout, key);
            assert_eq!(found.len(), 1, "{key}: one demo root");
            let (w, l) = found[0];
            assert_eq!(l.style.kind, kind, "{key}: kind");
            assert_eq!(
                binding.id_for(w),
                Some(l.id),
                "{key}: actual retained binding"
            );
            let r = snapshot.get(l.id).unwrap();
            assert!(r.width.is_finite() && r.height.is_finite());
            l.id
        });
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), 15);
        if let Some(previous) = saved {
            assert_eq!(ids, previous);
        }
        saved = Some(ids);
    }
}
