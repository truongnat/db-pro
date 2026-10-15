//! Tests for `../table_workspace_surface_view.rs` (moved out of the source file).
    use super::*;

    #[test]
    fn dependency_filter_tabs_and_total_share_the_header_baseline() {
        let context = egui::Context::default();
        DbProTheme::install_fonts(&context);
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1024.0, 160.0));
        let mut search = String::new();
        let mut tab_rects = Vec::new();
        let mut total_rect = None;

        let _ = crate::test_frame::frame(&context, egui::RawInput {
            screen_rect: Some(screen),
            ..Default::default()
        }, |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                draw_metadata_filter_header_with_trailing(
                    ui,
                    DbProTheme::dark(),
                    "Dependencies",
                    &mut search,
                    "Filter dependencies…",
                    |ui, _rtl| {
                        for (label, selected) in [
                            ("All", false),
                            ("Depends On (Outgoing)", true),
                            ("Depended By (Incoming)", false),
                        ] {
                            tab_rects.push(tab_button(ui, DbProTheme::dark(), None, label, selected).rect);
                        }
                        total_rect = Some(ui.label("3 of 3").rect);
                    },
                );
            });
        });

        let total_rect = total_rect.expect("result count is rendered");
        assert!(!tab_rects.is_empty(), "filter tabs are rendered");
        assert!(tab_rects.iter().all(|tab_rect| {
            (tab_rect.center().y - total_rect.center().y).abs() < 1.0
        }), "filter tabs and result count should stay on one action row: {tab_rects:?} vs {total_rect:?}");
    }