//! Layout cases: `LayoutEngine::compute` on balanced trees of 100, 500 and
//! 1000 nodes at an 800x600 viewport. Tree construction is outside the timing.

use acme_core::NodeId;
use acme_layout::{LayoutEngine, LayoutNode, LayoutStyle, Length};
use anyhow::Result;

use crate::harness::{CaseResult, MetaValue, RunConfig, check_ok, measure};

const NODE_COUNTS: [usize; 3] = [100, 500, 1000];
const FANOUT: usize = 4;
const VIEWPORT: (f32, f32) = (800.0, 600.0);

pub fn run(config: RunConfig) -> Result<Vec<CaseResult>> {
    NODE_COUNTS
        .iter()
        .map(|&count| layout_case(config, count))
        .collect()
}

fn layout_case(config: RunConfig, count: usize) -> Result<CaseResult> {
    let root = build_layout_tree(count);
    let mut engine = LayoutEngine::new();
    let results = check_ok(engine.compute(&root, VIEWPORT), "layout")?.len();
    let samples = measure(config, || {
        check_ok(engine.compute(&root, VIEWPORT), "layout")
    })?;
    CaseResult::from_samples(
        format!("layout/{count}_nodes"),
        &samples,
        vec![("layout_results", MetaValue::Count(results as u64))],
    )
}

/// Build a balanced tree with roughly `count` layout nodes.
fn build_layout_tree(count: usize) -> LayoutNode {
    let mut next = 0u64;
    build_subtree(&mut next, count, FANOUT)
}

fn build_subtree(next: &mut u64, count: usize, fanout: usize) -> LayoutNode {
    let id = NodeId::new(*next);
    *next += 1;

    let remaining = count.saturating_sub(1);
    if remaining == 0 {
        return LayoutNode::leaf(
            id,
            LayoutStyle {
                width: Length::px(10.0),
                height: Length::px(10.0),
                ..LayoutStyle::default()
            },
        );
    }

    let n = remaining.min(fanout);
    let base = remaining / n;
    let extra = remaining % n;

    let children: Vec<_> = (0..n)
        .map(|i| {
            let extra_count = usize::from(i < extra);
            build_subtree(next, base + extra_count, fanout)
        })
        .collect();

    LayoutNode::container(id, LayoutStyle::column(), children)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_tree_has_the_requested_number_of_nodes() {
        for count in [1, 2, 100, 500, 1000] {
            let root = build_layout_tree(count);
            let mut engine = LayoutEngine::new();
            let snapshot = engine.compute(&root, VIEWPORT).expect("layout");
            assert_eq!(snapshot.len(), count);
        }
    }
}
