use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutEngine, LayoutKind, WidgetLayoutContext};
use acme_widgets::retained_binding::bind_retained;
use acme_widgets::view_bridge::widget_to_views;

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Open,
}

#[test]
fn defaults_preserve_the_existing_builder_contract() {
    let builder = mobile_card::<Msg>("Title");
    assert_eq!(builder.id.as_str(), "mobile_card");
    assert_eq!(builder.title, "Title");
    assert!(builder.subtitle.is_none());
    assert!(builder.media.is_none());
    assert!(builder.elevated);
}

#[test]
fn optional_slots_keep_content_order_and_semantic_variants() {
    for elevated in [false, true] {
        for subtitle in [None, Some("副標題")] {
            for media in [false, true] {
                let mut builder = mobile_card("標題").key("article").elevated(elevated);
                if let Some(subtitle) = subtitle {
                    builder = builder.subtitle(subtitle);
                }
                if media {
                    builder =
                        builder.media(crate::button("photo", "Open media").on_click(Msg::Open));
                }
                let WidgetNode::Card(card) = WidgetNode::from(builder) else {
                    panic!("mobile card must remain a Card");
                };
                assert_eq!(card.key.as_ref().unwrap().as_str(), "article");
                assert_eq!(
                    card.variant,
                    if elevated {
                        CardVariant::Elevated
                    } else {
                        CardVariant::Outlined
                    }
                );
                assert!(
                    card.background_color.is_none(),
                    "theme supplies the surface"
                );
                assert_eq!(card.children.len(), 1 + usize::from(media));
                if media {
                    let WidgetNode::Button(button) = &card.children[0] else {
                        panic!("media must precede the text and remain intact");
                    };
                    assert_eq!(button.activate(), Some(&Msg::Open));
                }
                let text = card.children.last().unwrap();
                assert!(matches!(text, WidgetNode::Column(_)));
                assert_eq!(text.children().len(), 1 + usize::from(subtitle.is_some()));
                for (node, expected) in text
                    .children()
                    .iter()
                    .zip(std::iter::once("標題").chain(subtitle))
                {
                    let WidgetNode::Label(label) = node else {
                        panic!("expected label");
                    };
                    assert_eq!(label.text, expected);
                    assert!(label.color.is_none(), "theme supplies the foreground");
                }
            }
        }
    }
}

#[test]
fn media_and_text_have_padded_non_overlapping_layout() {
    let context = WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 20.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: 1.0,
    };
    for width in [240.0, 400.0] {
        let node: WidgetNode<Msg> = mobile_card("Title")
            .subtitle("Subtitle")
            .media(crate::row().key("photo").height(80.0).build())
            .into();
        let layout = node.to_layout_with_context(NodeId::new(1), &context);
        assert_eq!(layout.style.kind, LayoutKind::Column);
        let snapshot = LayoutEngine::new()
            .compute(&layout, (width, 300.0))
            .unwrap();
        let card = snapshot.get(layout.id).unwrap();
        let media = snapshot.get(layout.children[0].id).unwrap();
        let text_layout = &layout.children[1];
        let text = snapshot.get(text_layout.id).unwrap();
        let title = snapshot.get(text_layout.children[0].id).unwrap();
        let subtitle = snapshot.get(text_layout.children[1].id).unwrap();
        assert_eq!(media.height, 80.0);
        assert_eq!(media.x, card.x + 16.0);
        assert_eq!(media.y, card.y + 16.0);
        assert_eq!(media.width, card.width - 32.0);
        assert_eq!(text.y, media.y + media.height + 8.0);
        assert_eq!(text.x, media.x);
        assert_eq!(text.width, media.width);
        assert_eq!(subtitle.y, title.y + title.height + 4.0);
        assert_eq!(card.y + card.height, subtitle.y + subtitle.height + 16.0);
    }
}

#[test]
fn adding_and_removing_media_preserves_the_text_subtree_identity() {
    let mut tree = RetainedTree::default();
    let mut previous = None;
    for media in [false, true, false] {
        let mut builder = mobile_card("Title").key("article").subtitle("Subtitle");
        if media {
            builder = builder.media(crate::row().key("photo").height(80.0).build());
        }
        let node: WidgetNode<Msg> = builder.into();
        tree.reconcile_roots(&widget_to_views(std::slice::from_ref(&node)))
            .unwrap();
        let binding = bind_retained(std::slice::from_ref(&node), &tree);
        assert_eq!(binding.mismatches(), 0);
        let text = node.children().last().unwrap();
        let ids: Vec<_> = std::iter::once(&node)
            .chain(std::iter::once(text))
            .chain(text.children())
            .map(|widget| binding.id_for(widget).unwrap())
            .collect();
        if let Some(previous) = &previous {
            assert_eq!(
                &ids, previous,
                "media toggling must preserve card, text, title and subtitle IDs"
            );
        }
        previous = Some(ids);
    }
}
