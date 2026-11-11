use super::*;
use std::time::{Duration, Instant};

pub(super) struct SpinnerDemo {
    pub elapsed: Duration,
    pub reduced_motion: bool,
    pub visible: bool,
    epoch: Instant,
}

impl SpinnerDemo {
    pub fn new() -> Self {
        Self {
            elapsed: Duration::ZERO,
            reduced_motion: false,
            visible: false,
            epoch: Instant::now(),
        }
    }

    pub fn poll_interval(&self) -> Option<Duration> {
        (self.visible && !self.reduced_motion).then_some(Duration::from_millis(33))
    }

    pub fn tick(&mut self, now: Instant) -> bool {
        if self.poll_interval().is_none() {
            return false;
        }
        let elapsed = now.saturating_duration_since(self.epoch);
        let changed = elapsed != self.elapsed;
        self.elapsed = elapsed;
        changed
    }
}

pub(super) fn visible(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &acme_layout::LayoutSnapshot,
    scroll: f32,
    height: f32,
) -> bool {
    if widget
        .key()
        .is_some_and(|key| key.as_str().starts_with("gallery_loader_"))
    {
        return layout
            .children
            .first()
            .and_then(|node| snapshot.get(node.id))
            .is_some_and(|r| r.y + r.height > scroll + TOOLBAR_HEIGHT && r.y < scroll + height);
    }
    widget
        .children()
        .iter()
        .zip(&layout.children)
        .any(|(w, l)| visible(w, l, snapshot, scroll, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_polling_stops_when_hidden_or_reduced_and_resumes_with_current_time() {
        let mut demo = SpinnerDemo::new();
        let start = demo.epoch;
        assert_eq!(demo.poll_interval(), None);
        assert!(!demo.tick(start + Duration::from_millis(50)));
        demo.visible = true;
        assert_eq!(demo.poll_interval(), Some(Duration::from_millis(33)));
        assert!(demo.tick(start + Duration::from_millis(250)));
        assert_eq!(demo.elapsed, Duration::from_millis(250));
        demo.reduced_motion = true;
        assert_eq!(demo.poll_interval(), None);
        assert!(!demo.tick(start + Duration::from_millis(500)));
        demo.reduced_motion = false;
        assert!(demo.tick(start + Duration::from_millis(750)));
        assert_eq!(demo.elapsed, Duration::from_millis(750));
        demo.visible = false;
        assert!(!demo.tick(start + Duration::from_secs(1)));
    }
}
