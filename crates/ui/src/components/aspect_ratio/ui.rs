use egui::{Response, Sense, Ui, UiBuilder};

use super::handler::{aspect_size, sanitize_ratio};

pub struct AspectRatio {
    ratio: f32, // width / height, e.g. 16.0 / 9.0
}

impl AspectRatio {
    pub fn new(ratio: f32) -> Self {
        Self {
            ratio: sanitize_ratio(ratio),
        }
    }

    pub fn sixteen_nine() -> Self {
        Self::new(16.0 / 9.0)
    }

    pub fn four_three() -> Self {
        Self::new(4.0 / 3.0)
    }

    pub fn square() -> Self {
        Self::new(1.0)
    }

    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui) -> R) -> (Response, R) {
        let available_w = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(aspect_size(available_w, self.ratio), Sense::hover());

        // The outer widget owns the reserved aspect-ratio rectangle, then the caller renders
        // arbitrary egui content into a clipped child UI so overflowing canvases or labels cannot
        // affect sibling layout. The handler supplies the pure sizing decision; this layer applies
        // egui allocation, clipping and content execution in that order.
        let mut child_ui = ui.new_child(UiBuilder::new().max_rect(rect));
        child_ui.set_clip_rect(rect);
        let ret = add_contents(&mut child_ui);

        (response, ret)
    }
}
