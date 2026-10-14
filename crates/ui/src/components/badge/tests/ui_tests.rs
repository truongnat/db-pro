//! Tests for `../ui.rs` (moved out of the source file).
use super::*;
use crate::DbProTheme;

#[test]
fn badge_uses_compact_workstation_metrics() {
    let theme = DbProTheme::light();
    let ctx = egui::Context::default();
    DbProTheme::install_fonts(&ctx);
    let mut height = 0.0;
    let mut width = 0.0;
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let response = Badge::new("Active", theme)
                .variant(BadgeVariant::Success)
                .dot(true)
                .show(ui);
            height = response.rect.height();
            width = response.rect.width();
        });
    });
    assert!(
        (super::super::config::BADGE_MIN_HEIGHT..=22.0).contains(&height),
        "badge height {height} should stay near 20px"
    );
    assert!(width > height, "badge should be wider than tall");
}

#[test]
fn semantic_badges_keep_a_visible_boundary() {
    let theme = DbProTheme::dark();
    for variant in [
        BadgeVariant::Default,
        BadgeVariant::Secondary,
        BadgeVariant::Outline,
        BadgeVariant::Destructive,
        BadgeVariant::Success,
        BadgeVariant::Warning,
        BadgeVariant::Info,
    ] {
        let palette = BadgePalette::from_variant(variant, &theme);
        assert_ne!(
            palette.border_stroke,
            Stroke::NONE,
            "{variant:?} lost its status boundary"
        );
    }
}
