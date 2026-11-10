//! Deadlines shared by background work and transient frame retries.
use acme_render_wgpu::SurfaceAction;
use std::time::{Duration, Instant};

/// A skipped frame remains pending, but never requests an immediate busy loop.
#[derive(Default)]
pub(crate) struct FrameRetry {
    pub(crate) deadline: Option<Instant>,
}

impl FrameRetry {
    pub(crate) fn after_frame(&mut self, action: SurfaceAction, now: Instant) -> bool {
        self.deadline = match action {
            SurfaceAction::Skip | SurfaceAction::Reconfigure => {
                Some(now + Duration::from_millis(100))
            }
            _ => None,
        };
        self.deadline.is_some()
    }

    pub(crate) fn take_due(&mut self, now: Instant) -> bool {
        if self.deadline.is_some_and(|deadline| now >= deadline) {
            self.deadline = None;
            true
        } else {
            false
        }
    }
}

#[derive(Default)]
pub(crate) struct BackgroundPoll {
    interval: Option<Duration>,
    pub(crate) deadline: Option<Instant>,
}

impl BackgroundPoll {
    pub(crate) fn advance(&mut self, interval: Option<Duration>, now: Instant) -> bool {
        let Some(interval) = interval else {
            self.interval = None;
            self.deadline = None;
            return false;
        };
        // A public Application implementation owns this untrusted duration.
        // Bound it once here so neither zero nor Duration::MAX can break the loop.
        let interval = interval.clamp(Duration::from_millis(1), Duration::from_secs(86400));
        if self.interval.is_some_and(|previous| previous != interval) {
            self.deadline = Some(now + interval);
        }
        self.interval = Some(interval);
        if now >= self.deadline.unwrap_or(now) {
            self.deadline = Some(now + interval);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skipped_frames_retry_without_input_and_stop_after_success() {
        for action in [SurfaceAction::Skip, SurfaceAction::Reconfigure] {
            let now = Instant::now();
            let mut retry = FrameRetry::default();
            assert!(
                retry.after_frame(action, now),
                "unpresented frame stays dirty"
            );
            let deadline = retry.deadline.expect("retry without unrelated input");
            assert!(deadline > now);
            assert!(!retry.take_due(now));
            assert!(retry.take_due(deadline));
            assert!(!retry.take_due(deadline));
            assert!(!retry.after_frame(SurfaceAction::Rendered, deadline));
            assert_eq!(retry.deadline, None);
        }
    }

    #[test]
    fn terminal_actions_cancel_pending_retry() {
        for action in [SurfaceAction::Exit, SurfaceAction::DeviceLost] {
            let now = Instant::now();
            let mut retry = FrameRetry::default();
            retry.after_frame(SurfaceAction::Skip, now);
            assert!(!retry.after_frame(action, now));
            assert_eq!(retry.deadline, None);
        }
    }

    #[test]
    fn zero_background_interval_cannot_busy_loop() {
        let now = Instant::now();
        let mut poll = BackgroundPoll::default();
        assert!(poll.advance(Some(Duration::ZERO), now));
        assert!(poll.deadline.unwrap() > now);
        assert!(!poll.advance(Some(Duration::ZERO), now));
    }

    #[test]
    fn huge_background_interval_cannot_overflow() {
        let now = Instant::now();
        let mut poll = BackgroundPoll::default();
        assert!(poll.advance(Some(Duration::MAX), now));
        assert_eq!(poll.deadline, Some(now + Duration::from_secs(86400)));
    }

    #[test]
    fn changed_background_interval_reschedules_and_none_cancels() {
        let now = Instant::now();
        let mut poll = BackgroundPoll::default();
        assert!(poll.advance(Some(Duration::from_secs(60)), now));
        assert!(!poll.advance(Some(Duration::from_secs(1)), now));
        assert_eq!(poll.deadline, Some(now + Duration::from_secs(1)));
        assert!(poll.advance(Some(Duration::from_secs(1)), now + Duration::from_secs(1)));
        assert!(!poll.advance(None, now));
        assert_eq!(poll.deadline, None);
        assert!(poll.advance(Some(Duration::from_secs(1)), now));
    }
}
