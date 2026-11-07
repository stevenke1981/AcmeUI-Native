//! CPU-side construction cases: a 1200-quad / 100-clipped / 50-text `Frame`
//! and a 50-row widget tree. Neither touches a GPU or a window.

use acme_render_wgpu::{ClippedQuad, Frame, Quad, TextRun};
use acme_text::PreparedText;
use acme_widgets::{WidgetNode, button, column, label, row};
use anyhow::Result;

use crate::harness::{CaseResult, MetaValue, RunConfig, measure};

const ROW_COUNT: usize = 50;

pub fn run(config: RunConfig) -> Result<Vec<CaseResult>> {
    Ok(vec![frame_build_case(config)?, widget_tree_case(config)?])
}

fn frame_build_case(config: RunConfig) -> Result<CaseResult> {
    let samples = measure(config, || Ok(build_frame()))?;
    let frame = build_frame();
    CaseResult::from_samples(
        "frame_build/1200_quads_100_clipped_50_text",
        &samples,
        vec![
            ("quads", MetaValue::Count(frame.quads.len() as u64)),
            (
                "clipped_quads",
                MetaValue::Count(frame.clipped_quads.len() as u64),
            ),
            ("text_runs", MetaValue::Count(frame.text.len() as u64)),
        ],
    )
}

fn build_frame() -> Frame {
    let mut frame = Frame {
        clear: [0.08, 0.09, 0.12, 1.0],
        quads: Vec::with_capacity(1200),
        clipped_quads: Vec::with_capacity(100),
        text: Vec::with_capacity(50),
    };

    // 1000 solid quads
    for i in 0..1000 {
        let x = (i as f32 * 23.0) % 800.0;
        let y = (i as f32 * 17.0) % 600.0;
        frame
            .quads
            .push(Quad::solid([x, y, 12.0, 12.0], [0.2, 0.4, 0.8, 1.0]));
    }

    // 200 rounded quads
    for i in 0..200 {
        let x = (i as f32 * 31.0) % 800.0;
        let y = (i as f32 * 19.0) % 600.0;
        frame.quads.push(Quad {
            rect: [x, y, 24.0, 24.0],
            color: [0.8, 0.2, 0.3, 1.0],
            radius: 4.0,
            border_width: 1.0,
            border_color: [0.9, 0.9, 0.9, 1.0],
        });
    }

    // 100 clipped quads inside a clip region
    for i in 0..100 {
        let x = (i as f32 * 37.0) % 700.0 + 50.0;
        let y = (i as f32 * 29.0) % 500.0 + 50.0;
        frame.clipped_quads.push(ClippedQuad {
            quad: Quad::solid([x, y, 32.0, 32.0], [0.3, 0.7, 0.3, 1.0]),
            clip: [50.0, 50.0, 700.0, 500.0],
        });
    }

    // 50 text runs with empty glyph lists, so no font system is involved
    for i in 0..50 {
        let x = (i as f32 * 41.0) % 700.0 + 10.0;
        let y = (i as f32 * 23.0) % 500.0 + 10.0;
        frame.text.push(TextRun {
            prepared: PreparedText {
                atlas_generation: 0,
                glyphs: vec![],
                uploads: vec![],
            },
            origin: [x, y],
            color: [1.0, 1.0, 1.0, 1.0],
            clip: None,
        });
    }

    frame
}

fn widget_tree_case(config: RunConfig) -> Result<CaseResult> {
    let samples = measure(config, || Ok(build_widget_tree()))?;
    CaseResult::from_samples(
        format!("widget_tree/build_{ROW_COUNT}_rows"),
        &samples,
        vec![("rows", MetaValue::Count(ROW_COUNT as u64))],
    )
}

fn build_widget_tree() -> WidgetNode<()> {
    let mut col = column::<()>().key("tree-root").gap(4.0);
    for j in 0..ROW_COUNT {
        col = col.child(
            row::<()>()
                .key(format!("row-{j}"))
                .gap(8.0)
                .child(label::<()>(format!("Label {j}")))
                .child(button::<()>(format!("btn-{j}"), format!("Button {j}")))
                .build(),
        );
    }
    col.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_has_the_documented_element_counts() {
        let frame = build_frame();
        assert_eq!(frame.quads.len(), 1200);
        assert_eq!(frame.clipped_quads.len(), 100);
        assert_eq!(frame.text.len(), 50);
    }

    #[test]
    fn widget_tree_has_one_child_per_row() {
        let tree = build_widget_tree();
        assert_eq!(tree.children().len(), ROW_COUNT);
    }
}
