use egui::Ui;

use super::config;

/// Converts the builder's independent axis flags into egui's axis tuple.
/// Keeping this decision here prevents the presentation layer from duplicating
/// the ordering convention required by `egui::ScrollArea::new`.
pub fn scroll_axes(horizontal: bool, vertical: bool) -> [bool; 2] {
    [horizontal, vertical]
}

pub fn apply_scrollbar_visuals(ui: &mut Ui) -> egui::Visuals {
    let previous = ui.style().visuals.clone();
    let mut visuals = previous.clone();
    visuals.clip_rect_margin = config::SCROLL_CLIP_RECT_MARGIN;
    ui.style_mut().visuals = visuals;
    previous
}

pub fn restore_visuals(ui: &mut Ui, previous: egui::Visuals) {
    ui.style_mut().visuals = previous;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_order_matches_egui_scroll_area() {
        assert_eq!(scroll_axes(false, true), [false, true]);
        assert_eq!(scroll_axes(true, false), [true, false]);
        assert_eq!(scroll_axes(true, true), [true, true]);
    }

    #[test]
    fn applying_visuals_disables_clip_margin_only() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let previous = ui.style().visuals.clone();
                let saved = apply_scrollbar_visuals(ui);
                assert_eq!(saved, previous);
                assert_eq!(ui.style().visuals.clip_rect_margin, config::SCROLL_CLIP_RECT_MARGIN);
                restore_visuals(ui, saved);
                assert_eq!(ui.style().visuals, previous);
            });
        });
    }
}
