//! Tests for horizontal and vertical layout bounds of selection controls.
use std::cell::Cell;

use egui::{CentralPanel, Context, Pos2, RawInput, Rect, Vec2};

use super::{Checkbox, Radio};
use crate::DbProTheme;

#[test]
fn horizontal_radios_do_not_consume_entire_row_width() {
    let ctx = Context::default();
    DbProTheme::install_fonts(&ctx);
    let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 200.0));
    let light_rect = Cell::new(Rect::NOTHING);
    let dark_rect = Cell::new(Rect::NOTHING);

    let _ = crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(screen_rect),
            ..Default::default()
        },
        |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let r1 = Radio::new(false, "Light", DbProTheme::dark()).show(ui);
                    light_rect.set(r1.rect);
                    let r2 = Radio::new(true, "Dark", DbProTheme::dark()).show(ui);
                    dark_rect.set(r2.rect);
                });
            });
        },
    );

    let r1 = light_rect.get();
    let r2 = dark_rect.get();

    // In horizontal layout, the first radio must NOT steal the entire 400px row.
    // It should allocate its content width (~50-80px), allowing the second radio
    // to sit adjacent to it well within the 400px panel.
    assert!(
        r1.width() < 120.0,
        "first horizontal radio width was {}, expected compact content width (< 120px)",
        r1.width()
    );
    assert!(
        r2.right() <= 250.0,
        "second horizontal radio right edge was {}, expected it to sit near first (< 250px)",
        r2.right()
    );
}

#[test]
fn compact_rows_honour_a_parent_wider_than_the_glyph_floor() {
    let ctx = Context::default();
    DbProTheme::install_fonts(&ctx);
    // Parent is narrower than the natural label width but wider than the glyph floor,
    // so the row must clamp to the parent instead of the measured content width.
    let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(70.0, 200.0));
    let rect = Cell::new(Rect::NOTHING);

    let _ = crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(screen_rect),
            ..Default::default()
        },
        |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                let available = ui.available_width();
                let response = Radio::new(false, "Light and Dark theme preference", DbProTheme::dark())
                    .compact(true)
                    .show(ui);
                rect.set(response.rect);
                assert!(
                    response.rect.width() <= available + 0.5,
                    "compact row width {} exceeded parent available {}",
                    response.rect.width(),
                    available
                );
            });
        },
    );

    assert!(rect.get().width() > 0.0);
}

#[test]
fn compact_rows_keep_the_glyph_floor_in_a_sliver_parent() {
    let ctx = Context::default();
    DbProTheme::install_fonts(&ctx);
    // A parent narrower than the control glyph + gap cannot contain the indicator at all.
    // The documented contract is that the glyph floor is still allocated.
    let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(10.0, 200.0));
    let rect = Cell::new(Rect::NOTHING);
    let floor = super::config::CONTROL_SIZE + super::config::CONTROL_SPACING;

    let _ = crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(screen_rect),
            ..Default::default()
        },
        |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                rect.set(
                    Radio::new(false, "Light", DbProTheme::dark())
                        .compact(true)
                        .show(ui)
                        .rect,
                );
            });
        },
    );

    assert!(
        (rect.get().width() - floor).abs() < 0.5,
        "sliver-parent row width was {}, expected the glyph floor {floor}",
        rect.get().width()
    );
}

#[test]
fn horizontal_checkboxes_do_not_consume_entire_row_width() {
    let ctx = Context::default();
    DbProTheme::install_fonts(&ctx);
    let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 200.0));
    let mut c1_val = false;
    let mut c2_val = true;
    let c1_rect = Cell::new(Rect::NOTHING);
    let c2_rect = Cell::new(Rect::NOTHING);

    let _ = crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(screen_rect),
            ..Default::default()
        },
        |ctx| {
            CentralPanel::default().show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let r1 = Checkbox::new(&mut c1_val, "Nullable", DbProTheme::dark()).show(ui);
                    c1_rect.set(r1.rect);
                    let r2 = Checkbox::new(&mut c2_val, "PK", DbProTheme::dark()).show(ui);
                    c2_rect.set(r2.rect);
                });
            });
        },
    );

    let r1 = c1_rect.get();
    let r2 = c2_rect.get();

    assert!(
        r1.width() < 140.0,
        "first horizontal checkbox width was {}, expected compact content width (< 140px)",
        r1.width()
    );
    assert!(
        r2.right() <= 250.0,
        "second horizontal checkbox right edge was {}, expected it to sit near first (< 250px)",
        r2.right()
    );
}
