use super::*;

#[test]
fn long_context_chips_respect_the_database_context_width() {
    for width in [220.0, 280.0, 400.0] {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let response = ContextChip::new(
                    ContextChipKind::Connection,
                    "Analytics warehouse · regional customer reporting and operations",
                    DbProTheme::dark(),
                )
                .max_width(width)
                .show(ui);
                assert!(response.rect.width() <= width + 0.5);
            });
        });
    }
}
