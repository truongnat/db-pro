use egui::{Id, Sense, Ui, Vec2};

use super::config::{DEFAULT_CONTENT_HEIGHT, MIN_CLIP_HEIGHT};
use super::geometry::clip_rect;

/// Reveals nested tree rows with an animated height clip.
pub fn reveal_children(ui: &mut Ui, id: Id, open: bool, add_contents: impl FnOnce(&mut Ui)) {
    let t = crate::components::animation::overlay_t(ui.ctx(), id, open);
    if t <= 0.01 {
        return;
    }
    let height_id = id.with("content_h");
    let last_h = ui
        .ctx()
        .data(|data| data.get_temp::<f32>(height_id))
        .unwrap_or(DEFAULT_CONTENT_HEIGHT);
    let clip_h = (last_h * t).max(MIN_CLIP_HEIGHT);
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, clip_h), Sense::hover());
    let mut used_h = last_h;
    ui.allocate_new_ui(
        egui::UiBuilder::new().max_rect(clip_rect(rect, width, last_h.max(clip_h))),
        |child_ui| {
            child_ui.set_clip_rect(rect);
            add_contents(child_ui);
            used_h = child_ui.min_rect().height().max(MIN_CLIP_HEIGHT);
        },
    );
    ui.ctx().data_mut(|data| data.insert_temp(height_id, used_h));
}
