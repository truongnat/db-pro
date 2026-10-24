use super::*;

#[test]
fn active_tab_scroll_is_requested_only_when_target_changes() {
    let ctx = egui::Context::default();
    let first_target = egui::Id::new("query-one");
    let second_target = egui::Id::new("query-two");
    let mut should_scroll = false;

    for (target, expected) in [
        (first_target, true),
        (first_target, false),
        (second_target, true),
        (second_target, false),
        (first_target, true),
    ] {
        crate::test_frame::frame(&ctx, Default::default(), |ui| {
            should_scroll = active_tab_target_changed(ui.ctx(), target);
        });
        assert_eq!(should_scroll, expected);
    }
}

#[test]
fn active_tab_at_end_of_overflowing_strip_is_scrolled_into_view() {
    let ctx = egui::Context::default();
    let mut offset = egui::Vec2::ZERO;
    for _ in 0..30 {
        let _ = crate::test_frame::frame(
            &ctx,
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(260.0, 100.0),
                )),
                predicted_dt: 1.0 / 60.0,
                ..Default::default()
            },
            |ui| {
                let output = crate::components::ScrollArea::new(DbProTheme::dark())
                    .horizontal(true)
                    .vertical(false)
                    .id_salt("active-tab-scroll-test")
                    .show_with_output(ui, |ui| {
                        ui.horizontal(|ui| {
                            for index in 0..20 {
                                ui.allocate_exact_size(egui::vec2(90.0, 32.0), egui::Sense::hover());
                                if index == 19
                                    && active_tab_target_changed(
                                        ui.ctx(),
                                        egui::Id::new("last-query-document"),
                                    )
                                {
                                    ui.scroll_to_cursor(None);
                                }
                            }
                        });
                    });
                offset = output.state.offset;
            },
        );
    }

    assert!(offset.x > 0.0, "the selected tab at the strip end must become visible");
}
