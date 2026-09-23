use egui::{Response, Sense, Ui, UiBuilder, Vec2};

pub struct AspectRatio {
    ratio: f32, // width / height, e.g. 16.0 / 9.0
}

impl AspectRatio {
    pub fn new(ratio: f32) -> Self {
        Self {
            ratio: if ratio <= 0.001 { 1.0 } else { ratio },
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
        let height = available_w / self.ratio;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(available_w, height), Sense::hover());

        let mut child_ui = ui.new_child(UiBuilder::new().max_rect(rect));
        child_ui.set_clip_rect(rect);
        let ret = add_contents(&mut child_ui);

        (response, ret)
    }
}
