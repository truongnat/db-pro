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

/// The grid shape follows the local content width, not the screen: wide
/// workspaces gain a column instead of keeping the old fixed 620 pt island.
#[test]
fn quick_action_grid_spends_wide_workspaces_instead_of_a_fixed_island() {
    // 1920-wide shell → ~1612 pt workspace: all four actions on one row.
    let columns = grid_columns(1612.0);
    assert_eq!(columns, 4);
    let grid = grid_width(1612.0, columns);
    // The grid spends the width but keeps side air: it is capped, not full-bleed,
    // so four max-width cards do not run edge to edge under a centred hero.
    assert!(grid >= 1132.0, "the grid must spend the workspace width, got {grid}");
    assert!(grid <= GRID_MAX_WIDTH, "grid {grid} exceeded its cap");
    let card = (grid - 3.0 * SPACE_MD) / 4.0;
    assert!(card <= CARD_MAX_WIDTH, "card {card} exceeded the reading measure");

    // 1440-wide shell → ~1132 pt: two columns, each at the reading measure.
    let columns = grid_columns(1132.0);
    assert_eq!(columns, 2);
    let grid = grid_width(1132.0, columns);
    let two_column_measure = 2.0 * CARD_MAX_WIDTH + SPACE_MD;
    assert!((grid - two_column_measure).abs() < 0.5, "grid {grid} != {two_column_measure}");
    assert!(grid / 1132.0 >= 0.75, "grid used {grid} of 1132pt");
    assert!(grid <= GRID_MAX_WIDTH, "grid {grid} exceeded its cap");

    // 1280-wide shell → ~972 pt keeps the two-column composition, widened.
    assert_eq!(grid_columns(972.0), 2);
    assert!((grid_width(972.0, 2) - two_column_measure).abs() < 0.5);

    // Narrow surfaces keep the single stacked column, capped at the measure.
    assert_eq!(grid_columns(480.0), 1);
    assert!((grid_width(480.0, 1) - CARD_MAX_WIDTH).abs() < 0.5);
    assert!((grid_width(240.0, 1) - 240.0).abs() < 0.5);
}

/// Rendered geometry: at 1440/1920 the drawn grid covers most of the surface
/// and never overflows it.
#[test]
fn welcome_grid_fills_wide_surfaces() {
    for width in [1132.0_f32, 1612.0] {
        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let mut drawn = 0.0;
        let _ = crate::test_frame::frame(
            &ctx,
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, 900.0))),
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let available = ui.available_rect_before_wrap();
                    assert!(draw_quick_actions(DbProTheme::dark(), ui).is_none());
                    assert!(
                        ui.min_rect().right() <= available.right() + 0.5,
                        "the grid overflows the {width}pt surface"
                    );
                    drawn = ui.min_rect().width();
                });
            },
        );
        assert!(drawn >= width * 0.7, "grid used {drawn} of the {width}pt surface");
    }
}
