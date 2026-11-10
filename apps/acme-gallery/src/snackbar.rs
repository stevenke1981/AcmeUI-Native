//! Gallery host: bottom placement and deadlines stay outside the toast builder.
use super::*;
use acme_ui::overlay::toast;
use std::time::{Duration, Instant};

pub(super) struct SnackbarDemo {
    pub visible: bool,
    pub with_action: bool,
    pub deadline: Option<Instant>,
    pub status: &'static str,
    pub bounds: Option<[f32; 4]>,
    retained: RetainedLayoutState,
}
impl SnackbarDemo {
    pub fn new() -> Self {
        Self {
            visible: false,
            with_action: false,
            deadline: None,
            status: "hidden",
            bounds: None,
            retained: RetainedLayoutState::new(),
        }
    }
    pub fn show(&mut self, now: Instant, with_action: bool) {
        self.visible = true;
        self.with_action = with_action;
        let duration = toast::<GalleryMessage>("").duration_ms(5000).duration_ms;
        self.deadline = Some(now + Duration::from_millis(duration));
        self.status = "shown for 5 seconds";
    }
    pub fn close(&mut self, status: &'static str) {
        self.visible = false;
        self.deadline = None;
        self.bounds = None;
        self.status = status;
    }
    pub fn expire(&mut self, now: Instant) -> bool {
        if self.deadline.is_none_or(|deadline| now < deadline) {
            return false;
        }
        self.close("automatically dismissed");
        true
    }
    pub fn poll_interval(&self) -> Option<Duration> {
        self.deadline.map(|_| Duration::from_millis(100))
    }
    pub fn covers(&self, x: f32, y: f32) -> bool {
        self.visible
            && self
                .bounds
                .is_some_and(|r| x >= r[0] && x < r[0] + r[2] && y >= r[1] && y < r[1] + r[3])
    }
    fn widget(&self) -> WidgetNode<GalleryMessage> {
        let mut builder = toast("Item archived · 項目已封存。You can undo this change.")
            .key("gallery_snackbar_active")
            .tone(acme_ui::Tone::Success)
            .on_dismiss(GalleryMessage::SnackbarDismiss);
        if self.with_action {
            builder = builder.action("Undo · 復原", GalleryMessage::SnackbarAction);
        }
        builder.into()
    }
}
pub(super) fn is_overlay_action(message: GalleryMessage) -> bool {
    matches!(
        message,
        GalleryMessage::SnackbarAction | GalleryMessage::SnackbarDismiss
    )
}
impl Gallery {
    pub(super) fn snackbar_examples(&self) -> WidgetNode<GalleryMessage> {
        column()
            .key("gallery_snackbar_examples")
            .gap(8.0)
            .child(
                row()
                    .gap(8.0)
                    .child(
                        button("snackbar_show_action", "Show + Undo")
                            .on_click(GalleryMessage::SnackbarShow(true)),
                    )
                    .child(
                        button("snackbar_show_plain", "Show message")
                            .on_click(GalleryMessage::SnackbarShow(false)),
                    ),
            )
            .child(label(format!("Snackbar: {}", self.snackbar.status)))
            .child(label(
                "Appears at the viewport bottom for 5 seconds. Show again to restart.",
            ))
            .build()
    }
    pub(super) fn paint_snackbar(
        &mut self,
        scene: &mut acme_core::Scene,
        context: &FrameContext,
        theme: &Theme,
    ) {
        self.snackbar.bounds = None;
        if !self.snackbar.visible {
            return;
        }
        let width = (context.logical_width - SIDEBAR_WIDTH - 32.0).clamp(0.0, 600.0);
        let widget = column().width(width).child(self.snackbar.widget()).build();
        let layout_context = WidgetLayoutContext {
            body_font_size: theme.typography.body,
            body_line_height: theme.typography.body * theme.typography.line_height,
            label_font_size: theme.typography.label,
            control_height: 36.0,
            scale_factor: context.scale_factor,
        };
        let layout = retained_layout_root(&widget, &mut self.snackbar.retained, &layout_context);
        let snapshot = self
            .layout
            .compute_with_text(
                &layout,
                (width, context.logical_height),
                &mut self.fonts,
                context.scale_factor,
            )
            .expect("finite snackbar viewport");
        let height = snapshot.get(layout.id).unwrap().height;
        let x = SIDEBAR_WIDTH + (context.logical_width - SIDEBAR_WIDTH - width).max(0.0) / 2.0;
        let y = (context.logical_height - height - 16.0).max(TOOLBAR_HEIGHT);
        let mut shifted = acme_layout::LayoutSnapshot::default();
        for (id, rect) in snapshot.iter() {
            shifted.insert(
                id,
                acme_layout::LayoutRect {
                    x: rect.x + x,
                    y: rect.y + y,
                    ..*rect
                },
            );
        }
        let clip = [
            SIDEBAR_WIDTH,
            TOOLBAR_HEIGHT,
            (context.logical_width - SIDEBAR_WIDTH).max(0.0),
            (context.logical_height - TOOLBAR_HEIGHT).max(0.0),
        ];
        self.snackbar.bounds = Some(intersect_clip([x, y, width, height], clip));
        let mut index = self.button_info.len();
        collect_hit_regions(&widget, &layout, &shifted, false, &mut self.button_info);
        let mut frame = Frame::default();
        render_content(
            &mut frame,
            &widget,
            &layout,
            &shifted,
            theme,
            context.scale_factor,
            0.0,
            clip,
            &mut index,
            self.hovered,
            self.pressed,
            &mut self.fonts,
            &mut self.atlas,
        );
        // Append a separate frame after page text, preserving overlay painter order.
        for command in scene_from_frame(&frame).commands() {
            scene.push(command.clone());
        }
    }
}
