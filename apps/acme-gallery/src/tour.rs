//! A host-controlled inline tour; target spotlight placement is separate.
use super::*;
use acme_ui::overlay::{TourStep, tour};

#[derive(Default)]
pub(super) struct TourDemo {
    pub current: usize,
    pub finished: bool,
}

impl TourDemo {
    pub fn apply(&mut self, message: GalleryMessage) -> bool {
        match message {
            GalleryMessage::TourPrevious
                if !self.finished && self.current > 0 && self.current < 3 =>
            {
                self.current -= 1
            }
            GalleryMessage::TourNext if !self.finished && self.current < 2 => self.current += 1,
            GalleryMessage::TourFinish if !self.finished && self.current == 2 => {
                self.finished = true
            }
            GalleryMessage::TourRestart if self.finished => {
                self.current = 0;
                self.finished = false;
            }
            _ => return false,
        }
        true
    }
}

impl Gallery {
    pub(super) fn tour_examples(&self) -> WidgetNode<GalleryMessage> {
        let mut examples = column().key("gallery_tour").gap(8.0);
        if self.tour.finished {
            examples = examples.child(label("Tour finished")).child(
                button("gallery_tour_restart", "Restart tour")
                    .on_click(GalleryMessage::TourRestart),
            );
        } else {
            examples = examples
                .child(label("Follow the inline steps; the host owns progress."))
                .child(
                    tour()
                        .key("gallery_tour_control")
                        .step(TourStep::new("Welcome").description("Start this guided example."))
                        .step(
                            TourStep::new("Actions")
                                .description("Previous and Next are separate messages."),
                        )
                        .step(
                            TourStep::new("Ready")
                                .description("Finish, then restart whenever needed."),
                        )
                        .current(self.tour.current)
                        .on_prev(GalleryMessage::TourPrevious)
                        .on_next(GalleryMessage::TourNext)
                        .on_finish(GalleryMessage::TourFinish),
                );
        }
        examples.build()
    }
}
