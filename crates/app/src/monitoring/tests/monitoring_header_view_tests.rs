use super::*;

#[test]
fn connected_status_does_not_share_a_row_with_auto_refresh_in_the_sidebar() {
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let mut poll = true;
    for width in [220.0, 260.0] {
        let mut header = MonitoringHeaderContext {
            theme: DbProTheme::dark(),
            connected: true,
            driver: "postgresql",
            connection_name: "Presentation fixture",
            poll: &mut poll,
        };
        let _ = crate::test_frame::frame(
            &ctx,
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, 400.0))),
                ..Default::default()
            },
            |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    let right = ui.max_rect().right();
                    let _ = header.draw(ui);
                    assert!(
                        ui.min_rect().right() <= right + 4.0,
                        "monitor header escapes {width}-point sidebar: {:?}",
                        ui.min_rect()
                    );
                });
            },
        );
    }
}
