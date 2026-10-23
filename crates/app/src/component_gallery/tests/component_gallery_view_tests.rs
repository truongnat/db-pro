//! Tests for `../component_gallery_view.rs` (moved out of the source file).
use super::*;

/// Run one egui pass with `body` inside a central panel of the given size.
fn run_frame(width: f32, height: f32, mut body: impl FnMut(&mut Ui)) {
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let _ = crate::test_frame::frame(
        &ctx,
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, height))),
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| body(ui));
        },
    );
}

/// The detail column fills the space it has below the reading measure and caps
/// at it above, staying centred so the body keeps one measurable edge.
#[test]
fn detail_column_follows_the_local_width_and_caps_at_the_reading_measure() {
    let mut capped = 0.0;
    let mut filled = 0.0;

    run_frame(1920.0, 800.0, |ui| {
        let area = ui.available_rect_before_wrap();
        let mut column = egui::Rect::NOTHING;
        detail_column(ui, 0.0, |ui| {
            column = ui.max_rect();
            ui.label("specimen");
        });
        capped = column.width();
        assert!((capped - DETAIL_COLUMN_MAX_WIDTH).abs() < 0.5, "column {capped} != the measure");
        let left_gap = column.left() - area.left();
        let right_gap = area.right() - column.right();
        assert!((left_gap - right_gap).abs() < 0.5, "the column must stay centred");
    });

    run_frame(1000.0, 800.0, |ui| {
        let area = ui.available_rect_before_wrap();
        let mut column = egui::Rect::NOTHING;
        detail_column(ui, 0.0, |ui| {
            column = ui.max_rect();
            ui.label("specimen");
        });
        filled = column.width();
        let inner = area.width() - 2.0 * SPACE_LG;
        assert!((filled - inner).abs() < 0.5, "column {filled} != inner {inner}");
    });

    assert!(filled < capped, "a narrow detail area must not be padded out to the measure");
}

/// A specimen card takes the whole column instead of shrinking to its widest
/// row — the audit measured the card at 48 % of the column before this.
#[test]
fn specimen_cards_fill_the_detail_column() {
    let mut column = egui::Rect::NOTHING;
    let mut card = egui::Rect::NOTHING;
    run_frame(1920.0, 900.0, |ui| {
        detail_column(ui, 0.0, |ui| {
            column = ui.max_rect();
            Card::new(DbProTheme::dark())
                .fill_width()
                .show(ui, |ui| {
                    ui.label("specimen row");
                });
            card = ui.min_rect();
        });
    });
    let delta = (card.width() - column.width()).abs();
    assert!(delta < 0.5, "card {} vs column {}", card.width(), column.width());
}

/// The header renders inside the same column as the body, so its action
/// cluster ends on the body's right edge at every width — including below the
/// reading measure, where the panel frame's own inset would otherwise shift it.
#[test]
fn header_content_shares_the_body_column() {
    for width in [1000.0_f32, 1280.0, 1440.0, 1920.0] {
        let mut body_column = egui::Rect::NOTHING;
        let mut header_column = egui::Rect::NOTHING;
        let mut action = None;
        run_frame(width, 800.0, |ui| {
            let theme = DbProTheme::dark();
            detail_column(ui, 0.0, |ui| body_column = ui.max_rect());
            let frame = panel_frame(theme);
            let surface_inset = frame.inner_margin.left as f32;
            frame.show(ui, |ui| {
                ui.set_min_height(TOOLBAR_HEIGHT);
                detail_column(ui, surface_inset, |ui| {
                    header_column = ui.max_rect();
                    ui.horizontal(|ui| {
                        ui.label("Component Gallery");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let response = Button::new(theme).text("Reset").show(ui);
                            assert!(
                                (response.rect.right() - ui.max_rect().right()).abs() < 0.5,
                                "header actions must end on the column edge at {width}pt"
                            );
                        });
                    });
                });
            });
            action = gallery_header(ui, theme, true);
        });
        assert_eq!(action, None, "no header action fires without a pointer");
        assert!(
            (header_column.left() - body_column.left()).abs() < 0.5,
            "header starts at {} but the body column at {} ({width}pt)",
            header_column.left(),
            body_column.left()
        );
        assert!((header_column.right() - body_column.right()).abs() < 0.5);
    }
}
