//! Retained-tree cases.
//!
//! - `reconcile/...`: five keyed orderings (mount, reuse, remove) applied to a
//!   fresh `RetainedTree` on every sample.
//! - `retained/...`: a steady-state keyed tree of 1011 widgets (root column,
//!   10 rows, 1000 buttons) through `widget_to_views`, `reconcile_roots`,
//!   `bind_retained` and `to_layout_with_binding`.

use acme_core::{RetainedTree, ViewNode};
use acme_layout::WidgetLayoutContext;
use acme_widgets::retained_binding::bind_retained;
use acme_widgets::view_bridge::widget_to_views;
use acme_widgets::{WidgetNode, button, column, row};
use anyhow::{Result, ensure};

use crate::harness::{CaseResult, MetaValue, RunConfig, check_ok, measure};

const ROUND_KEYS: usize = 100;
const TREE_ROWS: usize = 10;
const BUTTONS_PER_ROW: usize = 100;

pub fn run(config: RunConfig) -> Result<Vec<CaseResult>> {
    let mut cases = vec![reconcile_rounds(config)?];
    cases.extend(retained_cases(config)?);
    Ok(cases)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct Totals {
    mounted: usize,
    reused: usize,
    removed: usize,
}

fn reconcile_rounds(config: RunConfig) -> Result<CaseResult> {
    let rounds = key_rounds(ROUND_KEYS);
    let totals = run_rounds(&mut RetainedTree::new(), &rounds)?;
    let samples = measure(config, || run_rounds(&mut RetainedTree::new(), &rounds))?;
    CaseResult::from_samples(
        format!("reconcile/{ROUND_KEYS}_keys_5_rounds_fresh_tree"),
        &samples,
        vec![
            ("mounted", MetaValue::Count(totals.mounted as u64)),
            ("reused", MetaValue::Count(totals.reused as u64)),
            ("removed", MetaValue::Count(totals.removed as u64)),
        ],
    )
}

fn key_rounds(count: usize) -> Vec<Vec<ViewNode>> {
    generate_orderings(count)
        .iter()
        .map(|ordering| {
            ordering
                .iter()
                .map(|i| ViewNode::new(format!("key-{i}").as_str(), "benchmark"))
                .collect()
        })
        .collect()
}

fn run_rounds(tree: &mut RetainedTree, rounds: &[Vec<ViewNode>]) -> Result<Totals> {
    let mut totals = Totals::default();
    for views in rounds {
        let report = check_ok(tree.reconcile_roots(views), "reconcile")?;
        totals.mounted += report.mounted.len();
        totals.reused += report.reused.len();
        totals.removed += report.removed.len();
    }
    Ok(totals)
}

/// Five key orderings that exercise the mount, reuse and remove paths.
fn generate_orderings(count: usize) -> Vec<Vec<usize>> {
    let all: Vec<usize> = (0..count).collect();
    let tail = count.saturating_sub(10);
    let far_tail = count.saturating_sub(20);

    vec![
        all.clone(),                     // 1. initial: all mounted
        all.into_iter().rev().collect(), // 2. reversed: all reused
        (0..tail).collect(),             // 3. remove last 10
        (tail..count) // 4. re-add the 10, rotate
            .chain(0..tail)
            .collect(),
        (0..far_tail) // 5. remove 20, add 10 new keys
            .chain(count..count + 10)
            .collect(),
    ]
}

fn retained_cases(config: RunConfig) -> Result<Vec<CaseResult>> {
    let roots = keyed_tree();
    let views = widget_to_views(&roots);
    let widgets = count_views(&views);
    let mut tree = RetainedTree::new();
    check_ok(tree.reconcile_roots(&views), "reconcile")?;
    let binding = bind_retained(&roots, &tree);
    ensure!(
        binding.mismatches() == 0,
        "bind_retained reported {} mismatches",
        binding.mismatches()
    );
    let context = layout_context();
    let suffix = format!("{widgets}_widgets");
    let meta = || vec![("widgets", MetaValue::Count(widgets as u64))];

    let to_views = measure(config, || Ok(widget_to_views(&roots)))?;
    let reconcile = measure(config, || {
        check_ok(tree.reconcile_roots(&views), "reconcile")
    })?;
    let bind = measure(config, || {
        let bound = bind_retained(&roots, &tree);
        ensure!(bound.mismatches() == 0, "bind_retained mismatch");
        Ok(bound.len())
    })?;
    let to_layout = measure(config, || {
        Ok(roots[0].to_layout_with_binding(&binding, &context))
    })?;

    Ok(vec![
        CaseResult::from_samples(
            format!("retained/widget_to_views_{suffix}"),
            &to_views,
            meta(),
        )?,
        CaseResult::from_samples(
            format!("retained/reconcile_steady_{suffix}"),
            &reconcile,
            meta(),
        )?,
        CaseResult::from_samples(format!("retained/bind_{suffix}"), &bind, meta())?,
        CaseResult::from_samples(
            format!("retained/to_layout_with_binding_{suffix}"),
            &to_layout,
            meta(),
        )?,
    ])
}

/// Root column, `TREE_ROWS` keyed rows, each with `BUTTONS_PER_ROW` keyed buttons.
fn keyed_tree() -> Vec<WidgetNode<()>> {
    let mut root = column::<()>().key("bench-root").gap(4.0);
    for r in 0..TREE_ROWS {
        let mut line = row::<()>().key(format!("row-{r}")).gap(2.0);
        for c in 0..BUTTONS_PER_ROW {
            line = line.child(button::<()>(format!("btn-{r}-{c}"), format!("B{c}")));
        }
        root = root.child(line.build());
    }
    vec![root.build()]
}

fn count_views(views: &[ViewNode]) -> usize {
    views
        .iter()
        .map(|view| 1 + count_views(&view.children))
        .sum()
}

fn layout_context() -> WidgetLayoutContext {
    WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 20.0,
        label_font_size: 14.0,
        control_height: 32.0,
        scale_factor: 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyed_tree_has_root_rows_and_buttons() {
        let roots = keyed_tree();
        let views = widget_to_views(&roots);
        assert_eq!(
            count_views(&views),
            1 + TREE_ROWS + TREE_ROWS * BUTTONS_PER_ROW
        );
    }

    #[test]
    fn orderings_cover_mount_reuse_and_remove() {
        let rounds = key_rounds(ROUND_KEYS);
        let totals = run_rounds(&mut RetainedTree::new(), &rounds).expect("reconcile");
        // 100 initial keys, 10 re-added in round 4, 10 brand-new keys in round 5.
        assert_eq!(totals.mounted, ROUND_KEYS + 10 + 10);
        assert!(totals.reused > 0);
        assert!(totals.removed > 0);
    }
}
