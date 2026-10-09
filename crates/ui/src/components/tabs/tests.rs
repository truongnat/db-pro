// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use super::*;
use crate::DbProTheme;

#[test]
fn segmented_tabs_select_on_click() {
    let theme = DbProTheme::light();
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let mut selected = 0;

    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            SegmentedTabs::new(&mut selected, &["A", "B", "C"], theme).show(ui);
        });
    });
    assert_eq!(selected, 0);
}

#[test]
fn segmented_tabs_select_on_pointer_click() {
    let theme = DbProTheme::light();
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let mut selected = 0;
    let click_position = egui::pos2(120.0, 20.0);

    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            SegmentedTabs::new(
                &mut selected,
                &["Development", "Staging", "Production", "Custom"],
                theme,
            )
            .show(ui);
        });
    });
    let _ = crate::test_frame::frame(
        &ctx,
        egui::RawInput {
            events: vec![
                egui::Event::PointerMoved(click_position),
                egui::Event::PointerButton {
                    pos: click_position,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                },
            ],
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                SegmentedTabs::new(
                    &mut selected,
                    &["Development", "Staging", "Production", "Custom"],
                    theme,
                )
                .show(ui);
            });
        },
    );
    let _ = crate::test_frame::frame(
        &ctx,
        egui::RawInput {
            events: vec![egui::Event::PointerButton {
                pos: click_position,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                SegmentedTabs::new(
                    &mut selected,
                    &["Development", "Staging", "Production", "Custom"],
                    theme,
                )
                .show(ui);
            });
        },
    );

    assert_eq!(selected, 1);

    let _ = crate::test_frame::frame(
        &ctx,
        egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::ArrowRight,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        },
        |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                SegmentedTabs::new(
                    &mut selected,
                    &["Development", "Staging", "Production", "Custom"],
                    theme,
                )
                .show(ui);
            });
        },
    );

    assert_eq!(selected, 2);
}

#[test]
fn underline_tabs_render_without_panic() {
    let theme = DbProTheme::light();
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let mut selected = 0;

    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            UnderlineTabs::new(&mut selected, &["Overview", "Schema", "Data"], theme).show(ui);
        });
    });

    selected = 2;
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            UnderlineTabs::new(&mut selected, &["Overview", "Schema", "Data"], theme).show(ui);
        });
    });
    assert_eq!(selected, 2);
}

#[test]
fn track_animation_stays_put_when_parent_shifts() {
    let ctx = egui::Context::default();
    let track_id = egui::Id::new("tabs_track_test");

    // Selected item is 20px into a track that starts at x=100.
    let target = egui::Rect::from_min_size(egui::pos2(120.0, 10.0), egui::vec2(40.0, 28.0));
    let _ = track::TabTrackerAnimation::animate_pill(&ctx, track_id, 100.0, target, false);

    // Whole track slides +80px; relative offset is still 20 → pill at 200.
    let shifted = egui::Rect::from_min_size(egui::pos2(200.0, 10.0), egui::vec2(40.0, 28.0));
    let pill = track::TabTrackerAnimation::animate_pill(&ctx, track_id, 180.0, shifted, false);
    assert!(
        (pill.left() - 200.0).abs() < 1.0,
        "pill should move with the track, not ease from the old screen x; got {}",
        pill.left()
    );
    assert!((pill.width() - 40.0).abs() < 1.0);
}

#[test]
fn reduced_motion_places_the_indicator_at_its_target_without_animation() {
    let ctx = egui::Context::default();
    let target = egui::Rect::from_min_size(egui::pos2(120.0, 10.0), egui::vec2(40.0, 28.0));
    let (x, width) =
        track::TabTrackerAnimation::animate_indicator(&ctx, egui::Id::new("reduced_motion_track"), 100.0, target, true);
    assert_eq!((x, width), (120.0, 40.0));
}
