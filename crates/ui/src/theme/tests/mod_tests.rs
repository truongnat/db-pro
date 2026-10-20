//! Tests for `../mod.rs` (moved out of the source file).
use super::DbProTheme;

#[test]
fn light_and_dark_themes_keep_distinct_surface_tokens() {
    let light = DbProTheme::light();
    let dark = DbProTheme::dark();

    assert!(!light.dark_mode);
    assert!(dark.dark_mode);
    assert_ne!(light.surface_app, dark.surface_app);
    assert_ne!(light.text_primary, dark.text_primary);
}

#[test]
fn applying_theme_selects_matching_egui_style_across_frames() {
    let ctx = egui::Context::default();
    for theme in [DbProTheme::light(), DbProTheme::dark(), DbProTheme::light()] {
        theme.apply(&ctx);
        let expected = if theme.dark_mode {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        };
        assert_eq!(ctx.theme(), expected);
        for _ in 0..3 {
            let _ = crate::test_frame::frame(&ctx, Default::default(), |ui| {
                assert_eq!(ui.visuals().dark_mode, theme.dark_mode);
                assert_eq!(ui.visuals().panel_fill, theme.surface_panel);
                assert_eq!(ui.visuals().override_text_color, Some(theme.text_primary));
            });
        }
    }
}

#[test]
fn light_tokens_follow_codex_surface_contract() {
    let theme = DbProTheme::light();

    assert_eq!(theme.surface_app, egui::Color32::from_rgb(255, 255, 255));
    assert_eq!(theme.surface_panel, egui::Color32::from_rgb(249, 249, 249));
    assert_eq!(theme.surface_active, egui::Color32::from_rgb(238, 238, 238));
    assert_eq!(theme.text_primary, egui::Color32::from_rgb(26, 28, 31));
    assert_eq!(theme.accent, egui::Color32::from_rgb(0, 111, 204));
}

#[test]
fn dark_tokens_follow_codex_surface_contract() {
    let theme = DbProTheme::dark();

    assert_eq!(theme.surface_app, egui::Color32::from_rgb(33, 33, 33));
    assert_eq!(theme.surface_panel, egui::Color32::from_rgb(23, 23, 23));
    assert_eq!(theme.surface_active, egui::Color32::from_rgb(58, 58, 58));
    assert_ne!(theme.surface_hover, theme.surface_active);
    assert_eq!(theme.text_primary, egui::Color32::from_rgb(236, 236, 236));
    assert_eq!(theme.accent, egui::Color32::from_rgb(236, 236, 236));
    assert_eq!(theme.accent_foreground, egui::Color32::from_rgb(13, 13, 13));
}

#[test]
fn theme_text_and_accent_foregrounds_meet_normal_text_contrast() {
    for theme in [DbProTheme::light(), DbProTheme::dark()] {
        for foreground in [theme.text_primary, theme.text_secondary, theme.text_tertiary] {
            assert!(
                contrast_ratio(foreground, theme.surface_app) >= 4.5,
                "{foreground:?} on app {:?}",
                theme.surface_app
            );
            assert!(
                contrast_ratio(foreground, theme.surface_panel) >= 4.5,
                "{foreground:?} on panel {:?}",
                theme.surface_panel
            );
        }
        assert!(
            contrast_ratio(theme.accent_foreground, theme.accent) >= 4.5,
            "accent foreground {:?} on {:?}",
            theme.accent_foreground,
            theme.accent
        );
    }
}

#[test]
fn chart_series_stay_legible_on_the_canvas() {
    for theme in [DbProTheme::light(), DbProTheme::dark()] {
        assert_eq!(theme.chart_series, theme.semantic.chart.series);
        for color in theme.chart_series {
            assert!(
                contrast_ratio(color, theme.surface_app) >= 3.0,
                "series {color:?} on canvas {:?}",
                theme.surface_app
            );
        }
    }
}

fn contrast_ratio(left: egui::Color32, right: egui::Color32) -> f32 {
    let (lighter, darker) = if super::relative_luminance(left) >= super::relative_luminance(right) {
        (left, right)
    } else {
        (right, left)
    };
    (super::relative_luminance(lighter) + 0.05) / (super::relative_luminance(darker) + 0.05)
}

#[test]
fn flat_facade_fields_are_built_from_the_semantic_roles() {
    for theme in [DbProTheme::light(), DbProTheme::dark()] {
        assert_eq!(theme.surface_app, theme.semantic.background.canvas);
        assert_eq!(theme.surface_2, theme.semantic.background.subtle);
        assert_eq!(theme.surface_active, theme.semantic.background.selected);
        assert_eq!(theme.border_focus, theme.semantic.border.focus);
        assert_eq!(theme.text_muted, theme.semantic.foreground.muted);
        assert_eq!(theme.accent, theme.semantic.accent.solid);
        assert_eq!(theme.success, theme.semantic.status.success.solid);
        assert_eq!(theme.success_soft(), theme.semantic.status.success.subtle);
        assert_eq!(theme.overlay, theme.semantic.background.overlay);
        // Focus ring color stays the accent color in both shipped themes.
        assert_eq!(theme.border_focus, theme.accent);
    }
}

#[test]
fn soft_tint_matches_the_semantic_wash_recipe() {
    let light = DbProTheme::light();
    let dark = DbProTheme::dark();
    assert_eq!(light.soft_tint(light.danger), light.semantic.status.danger.subtle);
    assert_eq!(dark.soft_tint(dark.warning), dark.semantic.status.warning.subtle);
}

#[test]
fn install_fonts_registers_inter_medium_for_ui_labels() {
    let ctx = egui::Context::default();
    super::DbProTheme::install_fonts(&ctx);
    let _ = crate::test_frame::frame(&ctx, Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let galley = ui.painter().layout_no_wrap(
                "GPT-4o".to_owned(),
                super::DbProTheme::ui_medium_font(12.0),
                egui::Color32::WHITE,
            );
            assert!(galley.size().x > 8.0);
            assert!(galley.size().y > 8.0);

            let mono_vietnamese = ui.painter().layout_no_wrap(
                "SELECT * FROM bảng_dữ_liệu WHERE tên = 'tiếng Việt'".to_owned(),
                egui::FontId::monospace(13.0),
                egui::Color32::WHITE,
            );
            assert!(mono_vietnamese.size().x > 50.0);
        });
    });
}
