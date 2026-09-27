use egui::Ui;

pub fn apply_scrollbar_visuals(ui: &mut Ui) -> egui::Visuals {
    let previous = ui.style().visuals.clone();
    let mut visuals = previous.clone();
    visuals.clip_rect_margin = 0.0;
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
    fn applying_visuals_disables_clip_margin_only() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let previous = apply_scrollbar_visuals(ui);
                assert_eq!(ui.style().visuals.clip_rect_margin, 0.0);
                restore_visuals(ui, previous);
            });
        });
    }
}
