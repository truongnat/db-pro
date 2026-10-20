use super::*;

#[test]
fn all_six_role_attributes_fit_narrow_sidebar_by_wrapping() {
    let context = egui::Context::default();
    DbProTheme::install_fonts(&context);
    for width in [260.0, 420.0, 1024.0] {
        let mut state = SecurityState::default();
        let view = SecurityRoleDetailsContext {
            theme: DbProTheme::dark(),
            state: &mut state,
            role: "analytics_reader",
        };
        let _ = crate::test_frame::frame(
            &context,
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, 400.0))),
                ..Default::default()
            },
            |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    let right = ui.max_rect().right();
                    assert!(view.draw_attributes(ui).is_empty());
                    assert!(
                        ui.min_rect().right() <= right + 1.0,
                        "attribute actions escape {width}-point sidebar"
                    );
                });
            },
        );
    }
}
