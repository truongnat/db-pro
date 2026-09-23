use crate::components::animation::faded_overlay;
use egui::{Id, Pos2, Rect, Rounding, Ui};

pub struct OverlayPaint {
    pub screen: Rect,
    pub overlay: egui::Color32,
    pub progress: f32,
}

pub fn paint_dim(ui: &mut Ui, paint: OverlayPaint) {
    ui.painter().rect_filled(
        paint.screen,
        Rounding::ZERO,
        faded_overlay(paint.overlay, paint.progress),
    );
}

pub fn overlay_widget_id(ui: &mut Ui, salt: Option<Id>, kind: &'static str) -> Id {
    if let Some(salt) = salt {
        return salt;
    }
    let id = ui.auto_id_with(kind);
    ui.skip_ahead_auto_ids(1);
    id
}

pub fn screen_rect_fallback(ctx: &egui::Context) -> Rect {
    let s = ctx.screen_rect();
    if s.width() > 1.0 && s.height() > 1.0 {
        s
    } else {
        Rect::from_min_size(Pos2::ZERO, egui::vec2(1280.0, 800.0))
    }
}
