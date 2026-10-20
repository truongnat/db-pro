use super::*;

#[test]
fn welcome_cards_fit_both_sides_of_the_grid_breakpoint() {
    for width in [240.0, 280.0, 571.0, 572.0, 600.0, 619.0, 620.0] {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let _ = crate::test_frame::frame(
            &ctx,
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, 500.0))),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let available = ui.available_rect_before_wrap();
                    assert!(draw_quick_actions(DbProTheme::dark(), ui).is_none());
                    assert!(
                        ui.min_rect().right() <= available.right() + 0.5,
                        "action cards overflow the remaining {width}pt workspace"
                    );
                });
            },
        );
    }
}
