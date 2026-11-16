use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutEngine, WidgetLayoutContext};
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

fn days(node: &WidgetNode<()>) -> Vec<(u32, &acme_widgets::Card<()>)> {
    node.children()
        .iter()
        .flat_map(|week| week.children())
        .filter_map(|cell| {
            let WidgetNode::Card(card) = cell else {
                return None;
            };
            let WidgetNode::Label(label) = &card.children[0].children()[0] else {
                panic!("day label");
            };
            Some((label.text.parse().unwrap(), card))
        })
        .collect()
}

fn highlighted(node: &WidgetNode<()>) -> Vec<u32> {
    days(node)
        .into_iter()
        .filter_map(|(day, card)| {
            let WidgetNode::Row(content) = &card.children[0] else {
                panic!("day content");
            };
            (content.style.background == Some(ColorToken::Accent)).then_some(day)
        })
        .collect()
}

#[test]
fn ranges_are_inclusive_normalized_and_reject_invalid_month_days() {
    for (year, month, start, end, expected) in [
        (2026, 11, 5, 12, (5..=12).collect::<Vec<_>>()),
        (2026, 11, 12, 5, (5..=12).collect()),
        (2026, 11, 9, 9, vec![9]),
        (2024, 2, 28, 29, vec![28, 29]),
        (2023, 2, 28, 29, vec![]),
        (2026, 4, 1, 31, vec![]),
        (2026, 11, 0, 5, vec![]),
        (2026, 11, u32::MAX, 5, vec![]),
    ] {
        let node = calendar(year, month).selected_range(start, end).build();
        assert_eq!(highlighted(&node), expected);
        assert_eq!(days(&node).len(), days_in_month(year, month) as usize);
        assert!(
            days(&node)
                .iter()
                .all(|(_, c)| c.background_color.is_none())
        );
    }
    // Validate at build time against public year/month fields, not setter time.
    let mut builder = calendar::<()>(2024, 2).selected_range(28, 29);
    builder.year = 2023;
    assert!(highlighted(&builder.build()).is_empty());
}

#[test]
fn last_selection_setter_wins_and_today_remains_outlined_outside_the_range() {
    let node = calendar(2026, 11)
        .selected_day(2)
        .today_day(15)
        .selected_range(5, 12)
        .build();
    assert_eq!(highlighted(&node), (5..=12).collect::<Vec<_>>());
    let cells = days(&node);
    assert_eq!(cells[1].1.variant, crate::CardVariant::Plain);
    assert_eq!(cells[14].1.variant, crate::CardVariant::Outlined);
    let node = calendar(2026, 11)
        .selected_range(5, 12)
        .selected_day(15)
        .today_day(15)
        .build();
    assert!(highlighted(&node).is_empty());
    assert_eq!(days(&node)[14].1.variant, crate::CardVariant::Interactive);
    assert!(highlighted(&calendar(2026, 11).build()).is_empty());
}

fn context(scale: f32) -> WidgetLayoutContext {
    WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 20.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: scale,
    }
}

#[test]
fn calendar_weekday_and_day_slots_align_without_overlap_at_real_text_scales() {
    let mut fonts = acme_text::FontSystem::new();
    for scale in [1.0, 1.25, 2.0] {
        for (year, month) in [(2026, 11), (2024, 2), (2026, 8)] {
            let node = calendar::<()>(year, month).selected_range(5, 12).build();
            let layout = node.to_layout_with_context(NodeId::new(1), &context(scale));
            let snapshot = LayoutEngine::new()
                .compute_with_text(&layout, (240.0, 600.0), &mut fonts, scale)
                .unwrap();
            let weekdays = &layout.children[1];
            let mut bottom = 0.0;
            for week in &layout.children[1..] {
                let rect = snapshot.get(week.id).unwrap();
                assert!(rect.y >= bottom);
                bottom = rect.y + rect.height;
                assert_eq!(week.children.len(), 7);
                let mut right = rect.x;
                for (index, day) in week.children.iter().enumerate() {
                    let cell = snapshot.get(day.id).unwrap();
                    let header = snapshot.get(weekdays.children[index].id).unwrap();
                    assert_eq!(cell.x, header.x);
                    assert_eq!(cell.width, 28.0);
                    assert!(cell.x >= right && cell.x + cell.width <= 240.0);
                    assert!(cell.height > 0.0 && cell.height.is_finite());
                    right = cell.x + cell.width;
                }
            }
        }
    }
}

#[test]
fn range_and_optional_header_changes_preserve_retained_day_identity() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for (range, header) in [(None, true), (Some((5, 12)), false), (Some((9, 9)), true)] {
        let mut builder = calendar::<()>(2026, 11);
        if let Some((start, end)) = range {
            builder = builder.selected_range(start, end);
        }
        if !header {
            builder = builder.hide_header();
        }
        let roots = vec![builder.build()];
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let layout = roots[0].to_layout_with_binding(&binding, &context(1.0));
        let mut ids = Vec::new();
        for week in &layout.children[if header { 2 } else { 1 }..] {
            ids.push(week.id);
            for cell in &week.children {
                ids.push(cell.id);
            }
        }
        if let Some(previous) = &saved {
            assert_eq!(&ids, previous);
        }
        assert!(ids.iter().all(|id| !tree.get(*id).unwrap().focusable));
        saved = Some(ids);
    }
}
