//! Tests for `../ui.rs` (moved out of the source file).
use super::*;

fn run_ui(mut on_ui: impl FnMut(&mut egui::Ui)) {
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| on_ui(ui));
    });
}

#[test]
fn terminal_block_renders_without_panicking() {
    let theme = DbProTheme::light();
    run_ui(|ui| {
        let resp = TerminalBlock::new("SELECT 1;", theme).title("Query Output").show(ui);
        assert!(resp.rect.is_finite());
        assert!(resp.rect.width() > 0.0);
    });
}

#[test]
fn progress_ring_renders_without_panicking() {
    let theme = DbProTheme::light();
    run_ui(|ui| {
        let resp = ProgressRing::new(0.65, 18.0, theme).show(ui);
        assert!(resp.rect.is_finite());
        assert_eq!(resp.rect.width(), 36.0);
        assert_eq!(resp.rect.height(), 36.0);
    });
}

#[test]
fn invalid_progress_ring_geometry_remains_finite() {
    let theme = DbProTheme::light();
    run_ui(|ui| {
        let resp = ProgressRing::new(f32::NAN, f32::NEG_INFINITY, theme).show(ui);
        assert!(resp.rect.is_finite());
        assert_eq!(resp.rect.width(), 36.0);
        assert_eq!(resp.rect.height(), 36.0);
    });
}
