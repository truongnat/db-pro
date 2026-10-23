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

    // The light ladder is a paper stack: pure white is the top plane (cards,
    // popovers, editor) and the app/chrome planes step down from it.
    assert_eq!(theme.surface_elevated, egui::Color32::from_rgb(255, 255, 255));
    assert_eq!(theme.surface_app, egui::Color32::from_rgb(248, 248, 248));
    assert_eq!(theme.surface_field, egui::Color32::from_rgb(246, 246, 246));
    assert_eq!(theme.surface_panel, egui::Color32::from_rgb(243, 243, 243));
    assert_eq!(theme.surface_2, egui::Color32::from_rgb(240, 240, 240));
    assert_eq!(theme.surface_hover, egui::Color32::from_rgb(237, 237, 237));
    assert_eq!(theme.surface_active, egui::Color32::from_rgb(231, 231, 231));
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
fn light_surface_ladder_keeps_hover_and_selected_distinct_from_chrome() {
    let theme = DbProTheme::light();

    // A hovered row must not look like the base fill it sits on, and a selected row
    // must be stronger than a hover — otherwise the states are indistinguishable.
    assert_ne!(theme.surface_hover, theme.surface_panel);
    assert_ne!(theme.surface_hover, theme.surface_app);
    assert_ne!(theme.surface_active, theme.surface_hover);
    assert_ne!(theme.surface_active, theme.surface_panel);

    // Elevation ladder: a raised card may not reuse the fill of the plane it is drawn
    // on (canvas on the settings page, panel in the workspace body), and a resting
    // field may not reuse the card fill it sits on.
    assert_ne!(theme.surface_elevated, theme.surface_app);
    assert_ne!(theme.surface_elevated, theme.surface_panel);
    assert_ne!(theme.surface_field, theme.surface_elevated);
    assert_ne!(theme.surface_field, theme.surface_2);

    let luminance = |color: egui::Color32| {
        let linear = |channel: u8| {
            let value = f32::from(channel) / 255.0;
            if value <= 0.039_28 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b())
    };
    // Darker in this ladder means a stronger state; brighter means a raised plane.
    assert!(luminance(theme.surface_panel) < luminance(theme.surface_app));
    assert!(luminance(theme.surface_hover) < luminance(theme.surface_panel));
    assert!(luminance(theme.surface_active) < luminance(theme.surface_hover));
    assert!(luminance(theme.surface_elevated) > luminance(theme.surface_app));
    assert!(luminance(theme.surface_elevated) > luminance(theme.surface_panel));
    assert!(luminance(theme.surface_field) < luminance(theme.surface_elevated));

    // Quantitative floors. A 1.00–1.08 step is what made cards read as a hairline-only
    // edge, so the ladder is held to a visible step rather than mere inequality.
    assert!(
        contrast_ratio(theme.surface_elevated, theme.surface_app) >= 1.05,
        "card on canvas {:?}",
        contrast_ratio(theme.surface_elevated, theme.surface_app)
    );
    assert!(
        contrast_ratio(theme.surface_elevated, theme.surface_panel) >= 1.10,
        "card on panel {:?}",
        contrast_ratio(theme.surface_elevated, theme.surface_panel)
    );
    assert!(
        contrast_ratio(theme.surface_panel, theme.surface_hover) >= 1.05,
        "hover on panel {:?}",
        contrast_ratio(theme.surface_panel, theme.surface_hover)
    );
    assert!(
        contrast_ratio(theme.surface_hover, theme.surface_active) >= 1.05,
        "selected on hover {:?}",
        contrast_ratio(theme.surface_hover, theme.surface_active)
    );
    assert!(
        contrast_ratio(theme.surface_field, theme.surface_elevated) >= 1.05,
        "field on card {:?}",
        contrast_ratio(theme.surface_field, theme.surface_elevated)
    );
    // The resting field stroke has to read against the field fill it outlines.
    assert!(
        contrast_ratio(theme.border_default, theme.surface_field) >= 1.15,
        "rest stroke on field {:?}",
        contrast_ratio(theme.border_default, theme.surface_field)
    );
}

#[test]
fn raised_surfaces_are_brighter_than_the_plane_they_sit_on_in_both_themes() {
    for theme in [DbProTheme::light(), DbProTheme::dark()] {
        assert_ne!(theme.surface_elevated, theme.surface_app);
        assert_ne!(theme.surface_elevated, theme.surface_panel);
        assert_ne!(theme.surface_field, theme.surface_elevated);
        assert!(
            super::relative_luminance(theme.surface_elevated) > super::relative_luminance(theme.surface_app),
            "card {:?} must be raised above canvas {:?}",
            theme.surface_elevated,
            theme.surface_app
        );
        assert!(
            super::relative_luminance(theme.surface_elevated) > super::relative_luminance(theme.surface_panel),
            "card {:?} must be raised above panel {:?}",
            theme.surface_elevated,
            theme.surface_panel
        );
        // A resting field is a control, never the same plane as the card behind it.
        assert_ne!(theme.border_default, theme.surface_field);
    }
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
        assert_eq!(theme.surface_field, theme.semantic.background.field);
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
