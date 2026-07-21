//! Headless benchmark application measuring layout, reconciliation, and frame
//! build performance for the AcmeUI retained‑mode UI framework.
#![forbid(unsafe_op_in_unsafe_fn)]

use std::time::Instant;

use acme_core::{NodeId, RetainedTree, ViewNode};
use acme_layout::{LayoutEngine, LayoutNode, LayoutStyle, Length};
use acme_platform::{Application, FrameContext, PlatformEvent, WindowConfig};
use acme_render_wgpu::{ClippedQuad, Frame, Quad, TextRun, scene_from_frame};
use acme_text::{FontSystem, GlyphAtlas, PreparedText, TextConstraints, TextStyle};
use acme_widgets::{WidgetNode, button, column, label, row};
use tracing::info;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    info!("AcmeUI Benchmarks — starting");

    acme_platform::run(BenchmarkApp::new())?;
    Ok(())
}

struct BenchmarkApp {
    done: bool,
    engine: LayoutEngine,
}

impl BenchmarkApp {
    fn new() -> Self {
        Self {
            done: false,
            engine: LayoutEngine::new(),
        }
    }
}

impl Application for BenchmarkApp {
    fn window_config(&self) -> WindowConfig {
        WindowConfig {
            title: "AcmeUI Benchmark".into(),
            width: 200.0,
            height: 200.0,
        }
    }

    fn event(&mut self, _event: PlatformEvent) -> bool {
        false
    }

    fn frame(&mut self, _context: FrameContext) -> acme_core::Scene {
        if !self.done {
            self.done = true;
            run_all_benchmarks(&mut self.engine);
            info!("Benchmarks complete — exiting");
            std::process::exit(0);
        }
        scene_from_frame(&Frame::default())
    }
}

fn run_all_benchmarks(engine: &mut LayoutEngine) {
    info!("=== AcmeUI Performance Benchmarks ===");

    // Layout benchmarks — tree sizes 100, 500, 1000
    for &count in &[100, 500, 1000] {
        layout_benchmark(engine, count);
    }

    // Reconciliation benchmark — 100 nodes, 5 orderings
    reconciliation_benchmark(100);

    // Frame build benchmark — quads, clipped quads, text runs
    frame_build_benchmark();

    // Widget tree build benchmark
    widget_tree_build_benchmark();

    // Text shaping throughput benchmark
    text_shaping_benchmark();
}

// ---------------------------------------------------------------------------
// 1. Layout benchmark
// ---------------------------------------------------------------------------

fn layout_benchmark(engine: &mut LayoutEngine, node_count: usize) {
    let root = build_layout_tree(node_count);
    let viewport = (800.0, 600.0);

    // Warm‑up run to ignore one‑time allocation costs
    let _ = engine.compute(&root, viewport);

    let start = Instant::now();
    let snapshot = engine
        .compute(&root, viewport)
        .expect("layout computation must succeed");
    let elapsed = start.elapsed();

    info!(
        "Layout {node_count} nodes → {} results in {elapsed:?}",
        snapshot.len(),
    );
}

/// Build a balanced tree with roughly `count` layout nodes.
fn build_layout_tree(count: usize) -> LayoutNode {
    let mut next = 0u64;
    build_subtree(&mut next, count, 4)
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
            let extra_count = if i < extra { 1 } else { 0 };
            build_subtree(next, base + extra_count, fanout)
        })
        .collect();

    LayoutNode::container(id, LayoutStyle::column(), children)
}

// ---------------------------------------------------------------------------
// 2. Reconciliation benchmark
// ---------------------------------------------------------------------------

fn reconciliation_benchmark(node_count: usize) {
    let mut tree = RetainedTree::new();
    let orderings = generate_orderings(node_count);

    let start = Instant::now();
    let mut total_mounted = 0usize;
    let mut total_reused = 0usize;
    let mut total_removed = 0usize;

    for ordering in &orderings {
        let views: Vec<ViewNode> = ordering
            .iter()
            .map(|i| ViewNode::new(format!("key-{i}").as_str(), "benchmark"))
            .collect();

        let report = tree
            .reconcile_roots(&views)
            .expect("reconciliation must succeed");

        total_mounted += report.mounted.len();
        total_reused += report.reused.len();
        total_removed += report.removed.len();
    }

    let elapsed = start.elapsed();

    info!("Reconciliation ({node_count} nodes, 5 rounds): {elapsed:?}");
    info!("  Mounted: {total_mounted}, Reused: {total_reused}, Removed: {total_removed}");
}

/// Five different key orderings that exercise mount, reuse, and remove paths.
fn generate_orderings(count: usize) -> Vec<Vec<usize>> {
    let all: Vec<usize> = (0..count).collect();
    let tail = count.saturating_sub(10);
    let far_tail = count.saturating_sub(20);

    vec![
        all.clone(),                     // 1. initial — all mounted
        all.into_iter().rev().collect(), // 2. reversed — all reused
        (0..tail).collect(),             // 3. remove last 10
        (tail..count) // 4. re‑add the 10, rotate
            .chain(0..tail)
            .collect(),
        (0..far_tail) // 5. remove 20, add 10 new keys
            .chain(count..count + 10)
            .collect(),
    ]
}

// ---------------------------------------------------------------------------
// 3. Frame build benchmark
// ---------------------------------------------------------------------------

fn frame_build_benchmark() {
    let start = Instant::now();

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

    // 50 text runs — empty prepared glyphs to avoid font‑system dependency
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

    let elapsed = start.elapsed();

    info!(
        "Frame built — {} quads, {} clipped quads, {} text runs in {elapsed:?}",
        frame.quads.len(),
        frame.clipped_quads.len(),
        frame.text.len(),
    );
}

// ---------------------------------------------------------------------------
// 4. Widget tree build benchmark
// ---------------------------------------------------------------------------

fn widget_tree_build_benchmark() {
    let start = Instant::now();
    let iterations = 500;

    for i in 0..iterations {
        let mut col = column().key(format!("root-{i}")).gap(4.0);
        for j in 0..50 {
            col = col.child(
                row()
                    .key(format!("row-{j}"))
                    .gap(8.0)
                    .child(label(format!("Label {j}")))
                    .child(button(format!("btn-{j}"), format!("Button {j}")))
                    .build(),
            );
        }
        let _tree: WidgetNode<()> = col.build();
    }

    let elapsed = start.elapsed();
    info!("Widget tree build — {iterations} trees × 50 rows in {elapsed:?}");
}

// ---------------------------------------------------------------------------
// 5. Text shaping throughput benchmark
// ---------------------------------------------------------------------------

fn text_shaping_benchmark() {
    let mut fonts = FontSystem::new();
    let mut atlas = GlyphAtlas::new(2048, 2048);
    let style = TextStyle {
        font_size: 14.0,
        ..TextStyle::default()
    };
    let constraints = TextConstraints::default();

    let samples = [
        "Hello, world! The quick brown fox jumps over the lazy dog.",
        "在一個寧靜的午後，古老的書架上擺滿了泛黃的書籍",
        "Rust is a systems programming language focused on safety.",
        "1234567890 !@#$%^&*() 特殊字符テスト",
    ];

    // Warm-up
    for text in &samples {
        let layout = fonts.shape(text, &style, constraints, 1.0);
        let _ = fonts.prepare(&layout, &mut atlas);
    }

    let iterations = 200;
    let start = Instant::now();

    for _ in 0..iterations {
        for text in &samples {
            let layout = fonts.shape(text, &style, constraints, 1.0);
            let _ = fonts.prepare(&layout, &mut atlas);
        }
    }

    let elapsed = start.elapsed();
    let total_shapes = iterations * samples.len();
    info!("Text shaping — {total_shapes} shapes in {elapsed:?}");
}
