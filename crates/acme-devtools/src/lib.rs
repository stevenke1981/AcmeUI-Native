//! Developer tooling for debugging and profiling AcmeUI applications.
//!
//! Provides per-frame metrics collection, widget-tree debug dumps,
//! layout inspection, and renderer diagnostics.
#![forbid(unsafe_op_in_unsafe_fn)]

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use acme_layout::{LayoutRect, LayoutSnapshot};
use acme_render_wgpu::{Frame, SurfaceStatus};
use acme_widgets::WidgetNode;

/// Rolling average window for FPS computation.
const FPS_WINDOW: usize = 60;

// ---------------------------------------------------------------------------
// FrameMetrics
// ---------------------------------------------------------------------------

/// Collects per-frame statistics including layout counters, GPU-resource counts,
/// and a rolling-average FPS over the last 60 frames.
#[derive(Clone, Debug)]
pub struct FrameMetrics {
    /// Number of quads in the last recorded frame.
    pub quad_count: usize,
    /// Number of clipped quads in the last recorded frame.
    pub clipped_quad_count: usize,
    /// Number of text runs in the last recorded frame.
    pub text_run_count: usize,
    /// Total glyph instances across all text runs in the last recorded frame.
    pub total_glyphs: usize,
    /// Number of atlas uploads in the last recorded frame.
    pub upload_count: usize,
    /// Number of nodes in the layout tree.
    pub layout_node_count: usize,
    /// Timestamp set by `begin_frame()` for frame timing.
    pub frame_start: Option<Instant>,
    /// Duration of the last completed frame, set by `end_frame()`.
    pub frame_duration: Option<Duration>,
    /// Rolling average FPS over the last `FPS_WINDOW` frames.
    pub fps: f64,

    /// Ring buffer of recent frame durations for rolling-average FPS.
    frame_times: VecDeque<Duration>,
}

impl Default for FrameMetrics {
    fn default() -> Self {
        Self {
            quad_count: 0,
            clipped_quad_count: 0,
            text_run_count: 0,
            total_glyphs: 0,
            upload_count: 0,
            layout_node_count: 0,
            frame_start: None,
            frame_duration: None,
            fps: 0.0,
            frame_times: VecDeque::with_capacity(FPS_WINDOW + 1),
        }
    }
}

impl FrameMetrics {
    /// Create a new `FrameMetrics` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark the start of a frame for timing purposes.
    ///
    /// The timestamp recorded here is used by `end_frame()` to compute the
    /// per-frame duration.
    pub fn begin_frame(&mut self) {
        self.frame_start = Some(Instant::now());
    }

    /// Record content statistics from a completed `Frame`.
    ///
    /// This captures quad counts, text-run counts, total glyph instances,
    /// and atlas upload counts. It does **not** update timing — call
    /// `begin_frame()` / `end_frame()` separately.
    pub fn record_frame(&mut self, frame: &Frame) {
        self.quad_count = frame.quads.len();
        self.clipped_quad_count = frame.clipped_quads.len();
        self.text_run_count = frame.text.len();
        self.total_glyphs = frame.text.iter().map(|run| run.prepared.glyphs.len()).sum();
        self.upload_count = frame
            .text
            .iter()
            .map(|run| run.prepared.uploads.len())
            .sum();
    }

    /// Record the node count from a `LayoutSnapshot`.
    pub fn record_layout(&mut self, snapshot: &LayoutSnapshot) {
        self.layout_node_count = snapshot.len();
    }

    /// Mark the end of a frame and update the FPS rolling average.
    ///
    /// The duration since the last `begin_frame()` call is pushed into a
    /// ring buffer of the last `FPS_WINDOW` frames. The `fps` field is
    /// recomputed as `count / total_seconds`.
    ///
    /// If `begin_frame()` was not called, this is a no-op.
    pub fn end_frame(&mut self) {
        let now = Instant::now();
        let start = match self.frame_start {
            Some(t) => t,
            None => return,
        };
        let duration = now - start;
        self.frame_duration = Some(duration);

        self.frame_times.push_back(duration);
        while self.frame_times.len() > FPS_WINDOW {
            self.frame_times.pop_front();
        }

        if self.frame_times.is_empty() {
            self.fps = 0.0;
            return;
        }

        let count = self.frame_times.len() as f64;
        let total_ns: u128 = self.frame_times.iter().map(|d| d.as_nanos()).sum();
        let avg_ns = total_ns as f64 / count;
        self.fps = if avg_ns > 0.0 {
            1_000_000_000.0 / avg_ns
        } else {
            0.0
        };
    }

    /// Return a human-readable summary of the current metrics.
    pub fn summary(&self) -> String {
        let frame_time = self
            .frame_duration
            .map_or_else(|| "—".into(), |d| format!("{:?}", d));
        format!(
            "Frame: {} quads, {} clipped, {} text runs, {} glyphs, {} uploads \
             | Layout: {} nodes \
             | Frame time: {} \
             | FPS: {:.1}",
            self.quad_count,
            self.clipped_quad_count,
            self.text_run_count,
            self.total_glyphs,
            self.upload_count,
            self.layout_node_count,
            frame_time,
            self.fps,
        )
    }
}

// ---------------------------------------------------------------------------
// WidgetTreeDump
// ---------------------------------------------------------------------------

/// Debug string representation of the widget tree with layout information.
pub struct WidgetTreeDump;

impl WidgetTreeDump {
    /// Produce an indented debug dump of a widget tree.
    ///
    /// Each line shows a sequential node id, the widget kind, an optional key,
    /// the layout rectangle (if available in `snapshot`), and extra information
    /// such as label text.
    ///
    /// Node ids are assigned sequentially by a pre-order traversal, matching
    /// the convention used by `WidgetNode::to_layout()`.
    ///
    /// # Example output
    ///
    /// ```text
    /// [1] Row key="gallery" @ (0,0 1080x720)
    ///   [2] Column key="gallery" @ (28,28 1024x664)
    ///     [3] Label @ (28,28 500x30) text="AcmeUI Native Gallery"
    /// ```
    pub fn dump_widget_tree<M>(
        root: &WidgetNode<M>,
        snapshot: &LayoutSnapshot,
        indent: usize,
    ) -> String {
        let mut out = String::new();
        let mut counter = 1u64;
        Self::write_node(root, snapshot, indent, &mut counter, &mut out);
        out
    }

    fn write_node<M>(
        node: &WidgetNode<M>,
        snapshot: &LayoutSnapshot,
        indent: usize,
        counter: &mut u64,
        out: &mut String,
    ) {
        let id = *counter;
        *counter += 1;
        let prefix = " ".repeat(indent);

        let kind = Self::node_kind(node);
        let key_str = Self::key_string(node);
        let extra = Self::extra_info(node);

        let rect_str = snapshot
            .get(id)
            .map(|r| format!(" @ ({},{}) {}x{}", r.x, r.y, r.width, r.height))
            .unwrap_or_default();

        out.push_str(&format!(
            "{prefix}[{id}] {kind}{key_str}{rect_str}{extra}\n"
        ));

        for child in node.children() {
            Self::write_node(child, snapshot, indent + 2, counter, out);
        }
    }

    fn node_kind<M>(node: &WidgetNode<M>) -> &'static str {
        match node {
            WidgetNode::Row(_) => "Row",
            WidgetNode::Column(_) => "Column",
            WidgetNode::Stack(_) => "Stack",
            WidgetNode::Label(_) => "Label",
            WidgetNode::Button(_) => "Button",
            WidgetNode::Card(_) => "Card",
            WidgetNode::Separator(_) => "Separator",
            WidgetNode::ScrollView(_) => "ScrollView",
            WidgetNode::VirtualList(_) => "VirtualList",
            WidgetNode::Tooltip(_) => "Tooltip",
            WidgetNode::Popover(_) => "Popover",
            WidgetNode::Menu(_) => "Menu",
            WidgetNode::Dialog(_) => "Dialog",
            WidgetNode::Tree(_) => "Tree",
            WidgetNode::Table(_) => "Table",
            WidgetNode::DataGrid(_) => "DataGrid",
        }
    }

    fn key_string<M>(node: &WidgetNode<M>) -> String {
        match node {
            WidgetNode::Row(v)
            | WidgetNode::Column(v)
            | WidgetNode::Stack(v)
            | WidgetNode::Card(v) => v
                .key
                .as_ref()
                .map_or_else(String::new, |k| format!(" key=\"{}\"", k.as_str())),
            WidgetNode::Button(v) => format!(" key=\"{}\"", v.key.as_str()),
            WidgetNode::ScrollView(v) => format!(" key=\"{}\"", v.key.as_str()),
            WidgetNode::VirtualList(v) => format!(" key=\"{}\"", v.key.as_str()),
            WidgetNode::Tooltip(v) => format!(" key=\"{}\"", v.key.as_str()),
            WidgetNode::Popover(v) => format!(" key=\"{}\"", v.key.as_str()),
            WidgetNode::Menu(v) => format!(" key=\"{}\"", v.key.as_str()),
            WidgetNode::Dialog(v) => format!(" key=\"{}\"", v.key.as_str()),
            WidgetNode::Tree(v) => format!(" key=\"{}\"", v.key.as_str()),
            WidgetNode::Table(v) => format!(" key=\"{}\"", v.key.as_str()),
            WidgetNode::DataGrid(v) => format!(" key=\"{}\"", v.key.as_str()),
            WidgetNode::Label(_) | WidgetNode::Separator(_) => String::new(),
        }
    }

    fn extra_info<M>(node: &WidgetNode<M>) -> String {
        match node {
            WidgetNode::Label(v) => format!(" text=\"{}\"", v.text),
            WidgetNode::Button(v) => format!(" label=\"{}\"", v.label),
            WidgetNode::Separator(v) => format!(" thickness={}", v.thickness),
            WidgetNode::Tooltip(v) => format!(" text=\"{}\"", v.text),
            WidgetNode::VirtualList(v) => format!(" items={}", v.item_count),
            WidgetNode::Popover(v) => format!(" placement={:?}", v.placement),
            WidgetNode::Menu(v) => format!(" items={}", v.items.len()),
            WidgetNode::Dialog(v) => {
                format!(" title=\"{}\" w={:?} h={:?}", v.title, v.width, v.height)
            }
            WidgetNode::Tree(v) => format!(" items={}", v.items.len()),
            WidgetNode::Table(v) => format!(" cols={} rows={}", v.columns.len(), v.rows.len()),
            WidgetNode::DataGrid(v) => {
                format!(" cols={} rows={}", v.columns.len(), v.rows.len())
            }
            _ => String::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// LayoutInspector
// ---------------------------------------------------------------------------

/// Helper to query layout state from a `LayoutSnapshot`.
pub struct LayoutInspector;

impl LayoutInspector {
    /// Find the node with the **smallest area** whose layout rect contains
    /// the given point `(x, y)`.
    ///
    /// Preferring the smallest area is a proxy for the deepest (most specific)
    /// node in the tree. Returns the node id and a reference to its rect.
    pub fn find_node(snapshot: &LayoutSnapshot, x: f32, y: f32) -> Option<(u64, &LayoutRect)> {
        let mut result: Option<(u64, &LayoutRect)> = None;
        let mut best_area: Option<f32> = None;
        for (id, rect) in snapshot.iter() {
            if x >= rect.x && y >= rect.y && x <= rect.x + rect.width && y <= rect.y + rect.height {
                let area = rect.width * rect.height;
                if best_area.is_none_or(|best| area < best) {
                    result = Some((id, rect));
                    best_area = Some(area);
                }
            }
        }
        result
    }

    /// Return the parent chain for a given node id.
    ///
    /// **Note:** `LayoutSnapshot` does not store parent/child relationships,
    /// so this function returns the node's own entry as a single-element vec
    /// when the id exists, or an empty vec when the id is unknown.
    pub fn node_path(snapshot: &LayoutSnapshot, id: u64) -> Vec<(u64, LayoutRect)> {
        snapshot
            .iter()
            .find(|(nid, _)| *nid == id)
            .map(|(nid, rect)| vec![(nid, *rect)])
            .unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// RenderDiagnostics
// ---------------------------------------------------------------------------

/// Diagnostics about the renderer state.
pub struct RenderDiagnostics;

impl RenderDiagnostics {
    /// Return a human-readable static string describing a `SurfaceStatus`.
    pub fn surface_status_summary(status: SurfaceStatus) -> &'static str {
        match status {
            SurfaceStatus::Ready => "surface is ready",
            SurfaceStatus::Suspended => "surface is suspended (window hidden or zero-sized)",
            SurfaceStatus::Recovering => "surface is recovering from an error",
        }
    }

    /// Return a formatted string describing the frame's contents.
    pub fn frame_size(frame: &Frame) -> String {
        let glyphs: usize = frame.text.iter().map(|r| r.prepared.glyphs.len()).sum();
        let uploads: usize = frame.text.iter().map(|r| r.prepared.uploads.len()).sum();
        format!(
            "Frame: {} quads, {} clipped quads, {} text runs, {} glyphs, {} atlas uploads",
            frame.quads.len(),
            frame.clipped_quads.len(),
            frame.text.len(),
            glyphs,
            uploads,
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use acme_layout::{LayoutEngine, LayoutNode, LayoutStyle, Length};
    use acme_render_wgpu::Quad;

    // ── FrameMetrics ────────────────────────────────────────────────

    #[test]
    fn frame_metrics_defaults() {
        let m = FrameMetrics::new();
        assert_eq!(m.quad_count, 0);
        assert_eq!(m.clipped_quad_count, 0);
        assert_eq!(m.text_run_count, 0);
        assert_eq!(m.total_glyphs, 0);
        assert_eq!(m.upload_count, 0);
        assert_eq!(m.layout_node_count, 0);
        assert!(m.frame_start.is_none());
        assert!(m.frame_duration.is_none());
        assert_eq!(m.fps, 0.0);
    }

    #[test]
    fn frame_metrics_begin_end_frame_records_timing() {
        let mut m = FrameMetrics::new();

        m.begin_frame();
        // Simulate a tiny amount of work.
        std::hint::spin_loop();
        m.end_frame();

        assert!(m.frame_duration.is_some());
        assert!(m.fps > 0.0, "FPS should be positive, got {}", m.fps);
    }

    #[test]
    fn frame_metrics_end_frame_without_begin_is_noop() {
        let mut m = FrameMetrics::new();
        m.end_frame();
        assert!(m.frame_duration.is_none());
    }

    #[test]
    fn frame_metrics_record_frame_captures_stats() {
        let mut m = FrameMetrics::new();
        let frame = Frame {
            quads: vec![Quad::solid([0.0, 0.0, 10.0, 10.0], [1.0, 0.0, 0.0, 1.0])],
            clipped_quads: vec![],
            text: vec![],
            ..Frame::default()
        };
        m.record_frame(&frame);

        assert_eq!(m.quad_count, 1);
        assert_eq!(m.clipped_quad_count, 0);
        assert_eq!(m.text_run_count, 0);
        assert_eq!(m.total_glyphs, 0);
        assert_eq!(m.upload_count, 0);
    }

    #[test]
    fn frame_metrics_record_layout_snapshot() {
        let mut m = FrameMetrics::new();
        let node = LayoutNode::leaf(
            1,
            LayoutStyle {
                width: Length::px(100.0),
                height: Length::px(100.0),
                ..Default::default()
            },
        );
        let snapshot = LayoutEngine::new().compute(&node, (200.0, 200.0)).unwrap();
        m.record_layout(&snapshot);
        assert_eq!(m.layout_node_count, 1);
    }

    #[test]
    fn frame_metrics_summary_contains_key_info() {
        let mut m = FrameMetrics::new();
        m.quad_count = 42;
        m.fps = 60.0;
        m.frame_duration = Some(Duration::from_millis(16));
        let summary = m.summary();
        assert!(summary.contains("42 quads"));
        assert!(summary.contains("60.0"));
        assert!(summary.contains("16"));
    }

    #[test]
    fn frame_metrics_fps_rolling_average() {
        let mut m = FrameMetrics::new();

        // Record 3 frames with a tiny delay each.
        for _ in 0..3 {
            m.begin_frame();
            std::hint::spin_loop();
            m.end_frame();
        }

        assert!(
            m.fps > 0.0,
            "Rolling average FPS should be positive, got {}",
            m.fps
        );
        assert!(m.frame_duration.is_some());
    }

    // ── WidgetTreeDump ──────────────────────────────────────────────

    #[test]
    fn dump_empty_trivial_tree() {
        // A column wrapping a single label.
        let tree = acme_widgets::column::<()>()
            .child(acme_widgets::label::<()>("Hello"))
            .build();
        let mut id_counter = 1;
        let layout = tree.to_layout(&mut id_counter);
        let snapshot = LayoutEngine::new()
            .compute(&layout, (800.0, 600.0))
            .unwrap();

        let dump = WidgetTreeDump::dump_widget_tree(&tree, &snapshot, 0);

        // Column is ID 1, Label is ID 2
        assert!(
            dump.contains("[2] Label"),
            "dump should contain Label:\n{dump}"
        );
        assert!(
            dump.contains("Hello"),
            "dump should contain label text:\n{dump}"
        );
    }

    #[test]
    fn dump_shows_key_and_kind() {
        #[derive(Debug, Clone, PartialEq)]
        enum Msg {}

        let tree = acme_widgets::row::<Msg>()
            .key("root")
            .child(acme_widgets::button::<Msg>("btn", "Click"))
            .build();
        let mut id_counter = 1;
        let layout = tree.to_layout(&mut id_counter);
        let snapshot = LayoutEngine::new()
            .compute(&layout, (800.0, 600.0))
            .unwrap();

        let dump = WidgetTreeDump::dump_widget_tree(&tree, &snapshot, 0);
        assert!(dump.contains("[1] Row"), "expected [1] Row, got:\n{dump}");
        assert!(
            dump.contains("key=\"root\""),
            "expected key=\"root\", got:\n{dump}"
        );
        assert!(
            dump.contains("[2] Button"),
            "expected [2] Button, got:\n{dump}"
        );
    }

    #[test]
    fn dump_has_rects_for_layout_nodes() {
        let tree = acme_widgets::label::<()>("Hello");
        let mut id_counter = 1;
        let layout = tree.to_layout(&mut id_counter);
        let snapshot = LayoutEngine::new()
            .compute(&layout, (800.0, 600.0))
            .unwrap();

        let dump = WidgetTreeDump::dump_widget_tree(&tree, &snapshot, 0);
        assert!(
            dump.contains("@"),
            "dump should contain layout rects:\n{dump}"
        );
    }

    // ── LayoutInspector ─────────────────────────────────────────────

    #[test]
    fn find_node_empty_snapshot() {
        let snapshot = LayoutSnapshot::default();
        let found = LayoutInspector::find_node(&snapshot, 10.0, 10.0);
        assert!(found.is_none());
    }

    #[test]
    fn find_node_hit_test_finds_correct_node() {
        let node = LayoutNode::container(
            1,
            LayoutStyle {
                width: Length::px(200.0),
                height: Length::px(100.0),
                ..LayoutStyle::row()
            },
            vec![LayoutNode::leaf(
                2,
                LayoutStyle {
                    width: Length::px(100.0),
                    height: Length::px(100.0),
                    ..Default::default()
                },
            )],
        );
        let snapshot = LayoutEngine::new().compute(&node, (200.0, 100.0)).unwrap();

        // Both nodes contain (50, 50). We expect the last in iteration order
        // (child, id=2) to be returned.
        let found = LayoutInspector::find_node(&snapshot, 50.0, 50.0);
        assert!(found.is_some(), "should find a node at (50, 50)");
        let (id, _) = found.unwrap();
        assert_eq!(id, 2, "expected child node id=2 at (50,50)");

        // Outside all nodes.
        let outside = LayoutInspector::find_node(&snapshot, 999.0, 999.0);
        assert!(outside.is_none());
    }

    #[test]
    fn node_path_returns_known_node() {
        let node = LayoutNode::leaf(
            42,
            LayoutStyle {
                width: Length::px(50.0),
                height: Length::px(50.0),
                ..Default::default()
            },
        );
        let snapshot = LayoutEngine::new().compute(&node, (100.0, 100.0)).unwrap();
        let path = LayoutInspector::node_path(&snapshot, 42);
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].0, 42);
    }

    #[test]
    fn node_path_returns_empty_for_unknown() {
        let snapshot = LayoutSnapshot::default();
        let path = LayoutInspector::node_path(&snapshot, 999);
        assert!(path.is_empty());
    }

    // ── RenderDiagnostics ───────────────────────────────────────────

    #[test]
    fn surface_status_summary_ready() {
        let s = RenderDiagnostics::surface_status_summary(SurfaceStatus::Ready);
        assert!(s.contains("ready"));
    }

    #[test]
    fn surface_status_summary_suspended() {
        let s = RenderDiagnostics::surface_status_summary(SurfaceStatus::Suspended);
        assert!(s.contains("suspended"));
    }

    #[test]
    fn surface_status_summary_recovering() {
        let s = RenderDiagnostics::surface_status_summary(SurfaceStatus::Recovering);
        assert!(s.contains("recovering"));
    }

    #[test]
    fn frame_size_empty() {
        let frame = Frame::default();
        let s = RenderDiagnostics::frame_size(&frame);
        assert!(s.contains("0 quads"));
    }
}
