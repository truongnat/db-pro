//! Tests for `../ui.rs` (moved out of the source file).
use std::cell::Cell;

use egui::{CentralPanel, Context, CursorIcon, Event, Pos2, RawInput, Rect, Vec2};

use super::{Checkbox, Radio};
use crate::DbProTheme;

fn hovered_cursor(enabled: bool, radio: bool) -> CursorIcon {
    let ctx = Context::default();
    DbProTheme::install_fonts(&ctx);
    let screen_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 200.0));
    let mut checked = false;
    let control_rect = Cell::new(Rect::NOTHING);
    let mut show = |ui: &mut egui::Ui| {
        CentralPanel::default().show(ui, |ui| {
            control_rect.set(if radio {
                Radio::new(false, "Option", DbProTheme::dark())
                    .enabled(enabled)
                    .show(ui)
                    .rect
            } else {
                Checkbox::new(&mut checked, "Option", DbProTheme::dark())
                    .enabled(enabled)
                    .show(ui)
                    .rect
            });
        });
    };
    let _ = crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(screen_rect),
            ..Default::default()
        },
        |ui| show(ui),
    );
    crate::test_frame::frame(
        &ctx,
        RawInput {
            screen_rect: Some(screen_rect),
            events: vec![Event::PointerMoved(control_rect.get().center())],
            ..Default::default()
        },
        |ui| show(ui),
    )
    .platform_output
    .cursor_icon
}

#[test]
fn enabled_selection_controls_use_pointing_hand() {
    assert_eq!(hovered_cursor(true, false), CursorIcon::PointingHand);
    assert_eq!(hovered_cursor(true, true), CursorIcon::PointingHand);
}

#[test]
fn disabled_selection_controls_do_not_use_pointing_hand() {
    assert_ne!(hovered_cursor(false, false), CursorIcon::PointingHand);
    assert_ne!(hovered_cursor(false, true), CursorIcon::PointingHand);
}
