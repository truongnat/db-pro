use super::*;

#[test]
fn search_keeps_results_visible_in_a_tall_container() {
    for theme in [DbProTheme::light(), DbProTheme::dark()] {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        theme.apply(&ctx);
        let mut query = String::new();
        let mut search = egui::Rect::NOTHING;
        let mut result = egui::Rect::NOTHING;
        for _ in 0..2 {
            let _ = crate::test_frame::frame(
                &ctx,
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(580.0, 800.0))),
                    ..Default::default()
                },
                |ui| {
                    egui::CentralPanel::default().show(ui, |ui| {
                        search = SearchInput::new(&mut query, "Type a command or search…", theme)
                            .show(ui)
                            .rect;
                        result = ui.label("Schema workbench").rect;
                    });
                },
            );
        }
        assert!(search.height() < 48.0, "single-line search grew to {search:?}");
        assert!(
            result.top() < 80.0,
            "search pushed the first result out of view: {result:?}"
        );
    }
}
