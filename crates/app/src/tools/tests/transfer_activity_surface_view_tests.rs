//! Tests for `../transfer_activity_surface_view.rs` (moved out of the source file).
use super::*;

#[test]
fn transfer_choice_groups_and_masking_actions_reflow_within_sidebar() {
    let context = egui::Context::default();
    DbProTheme::install_fonts(&context);
    for width in [260.0, 420.0, 900.0] {
        let mut management = DatabaseManagementState::default();
        let mut view = TransferActivityContext {
            theme: DbProTheme::dark(),
            management: &mut management,
            table_details: &[],
            is_production: false,
        };
        let _ = crate::test_frame::frame(
            &context,
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, 900.0))),
                ..Default::default()
            },
            |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    let right = ui.max_rect().right();
                    let mut action = None;
                    view.draw_masking_controls(ui, &mut action);
                    assert!(
                        ui.min_rect().right() <= right + 1.0,
                        "masking controls must fit at {width}"
                    );
                    view.draw_copy_choices(ui);
                    assert!(
                        ui.min_rect().right() <= right + 1.0,
                        "controls escape {width}-point region: {:?}",
                        ui.min_rect()
                    );
                    assert!(action.is_none());
                });
            },
        );
    }
}
