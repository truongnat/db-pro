//! Tests for `../ui.rs` (moved out of the source file).
use super::*;
use crate::DbProTheme;

#[test]
fn empty_diff_renders_a_labeled_state() {
    let theme = DbProTheme::light();
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);

    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let response = DiffViewer::new("migration.sql", &[], theme).show(ui);
            assert!(response.rect.width() > 0.0);
            assert!(response.rect.height() > 0.0);
        });
    });
}

#[test]
fn diff_viewer_renders_in_egui_context() {
    let theme = DbProTheme::light();
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);

    let lines = vec![
        DiffLine::context(1, 1, "SELECT * FROM users"),
        DiffLine::removed(2, "- WHERE active = false"),
        DiffLine::added(2, "+ WHERE active = true"),
    ];

    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let resp = DiffViewer::new("migration.sql", &lines, theme).show(ui);
            assert!(resp.rect.width() > 0.0);
            assert!(resp.rect.height() > 0.0);
        });
    });
}
