use super::*;
use crate::DbProTheme;

#[test]
fn segmented_tabs_select_on_click() {
    let theme = DbProTheme::light();
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let mut selected = 0;

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            SegmentedTabs::new(&mut selected, &["A", "B", "C"], theme).show(ui);
        });
    });
    assert_eq!(selected, 0);
}

#[test]
fn underline_tabs_render_without_panic() {
    let theme = DbProTheme::light();
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let mut selected = 0;

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            UnderlineTabs::new(&mut selected, &["Overview", "Schema", "Data"], theme).show(ui);
        });
    });

    selected = 2;
    let _ = ctx.run(Default::default(), |ctx| {
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
    let _ = track::TabTrackerAnimation::animate_pill(&ctx, track_id, 100.0, target);

    // Whole track slides +80px; relative offset is still 20 → pill at 200.
    let shifted = egui::Rect::from_min_size(egui::pos2(200.0, 10.0), egui::vec2(40.0, 28.0));
    let pill = track::TabTrackerAnimation::animate_pill(&ctx, track_id, 180.0, shifted);
    assert!(
        (pill.left() - 200.0).abs() < 1.0,
        "pill should move with the track, not ease from the old screen x; got {}",
        pill.left()
    );
    assert!((pill.width() - 40.0).abs() < 1.0);
}
