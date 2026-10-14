//! Tests for `../handler.rs` (moved out of the source file).
use super::*;

#[test]
fn each_variant_maps_to_its_semantic_theme_and_icon() {
    let theme = DbProTheme::light();
    assert_eq!(
        char::from(style(theme, AlertVariant::Default).default_icon),
        char::from(Icon::Terminal)
    );
    assert_eq!(style(theme, AlertVariant::Info).icon, theme.info);
    assert_eq!(
        char::from(style(theme, AlertVariant::Success).default_icon),
        char::from(Icon::CheckCircle2)
    );
    assert_eq!(style(theme, AlertVariant::Warning).icon, theme.warning);
    assert_eq!(
        char::from(style(theme, AlertVariant::Destructive).default_icon),
        char::from(Icon::AlertCircle)
    );
}

#[test]
fn backdrop_only_dismisses_non_destructive_dialogs() {
    assert!(should_close_from_backdrop(false, true));
    assert!(!should_close_from_backdrop(true, true));
    assert!(!should_close_from_backdrop(false, false));
}

#[test]
fn dialog_width_respects_narrow_viewport_and_outer_maximum() {
    assert_eq!(dialog_outer_width(280.0), 248.0);
    assert_eq!(dialog_content_width(280.0), 208.0);
    assert_eq!(dialog_outer_width(1280.0), 440.0);
    assert_eq!(dialog_outer_width(20.0), 0.0);
}

#[test]
fn close_action_matches_confirmation() {
    assert_eq!(action_for_close(true), AlertDialogAction::Confirm);
    assert_eq!(action_for_close(false), AlertDialogAction::Cancel);
}
