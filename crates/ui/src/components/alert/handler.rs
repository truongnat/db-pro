//! Typed alert variant and dialog interaction decisions.

use crate::DbProTheme;
use egui::Color32;
use lucide_icons::Icon;

use super::{config, AlertDialogAction, AlertVariant};

#[derive(Debug, Clone, Copy)]
pub(crate) struct AlertStyle {
    pub(crate) fill: Color32,
    pub(crate) border: Color32,
    pub(crate) icon: Color32,
    pub(crate) default_icon: Icon,
}

pub(crate) fn style(theme: DbProTheme, variant: AlertVariant) -> AlertStyle {
    match variant {
        AlertVariant::Default => AlertStyle {
            fill: theme.surface_panel,
            border: theme.border_default,
            icon: theme.text_primary,
            default_icon: Icon::Terminal,
        },
        AlertVariant::Info => AlertStyle {
            fill: theme.info.linear_multiply(config::VARIANT_FILL_OPACITY),
            border: theme.info.linear_multiply(config::VARIANT_BORDER_OPACITY),
            icon: theme.info,
            default_icon: Icon::Info,
        },
        AlertVariant::Success => AlertStyle {
            fill: theme.success.linear_multiply(config::VARIANT_FILL_OPACITY),
            border: theme.success.linear_multiply(config::VARIANT_BORDER_OPACITY),
            icon: theme.success,
            default_icon: Icon::CheckCircle2,
        },
        AlertVariant::Warning => AlertStyle {
            fill: theme.warning.linear_multiply(config::VARIANT_FILL_OPACITY),
            border: theme.warning.linear_multiply(config::VARIANT_BORDER_OPACITY),
            icon: theme.warning,
            default_icon: Icon::AlertTriangle,
        },
        AlertVariant::Destructive => AlertStyle {
            fill: theme.danger.linear_multiply(config::VARIANT_FILL_OPACITY),
            border: theme.danger.linear_multiply(config::VARIANT_BORDER_OPACITY),
            icon: theme.danger,
            default_icon: Icon::AlertCircle,
        },
    }
}

pub(crate) fn action_for_close(confirmed: bool) -> AlertDialogAction {
    if confirmed {
        AlertDialogAction::Confirm
    } else {
        AlertDialogAction::Cancel
    }
}

pub(crate) fn should_close_from_backdrop(destructive: bool, clicked: bool) -> bool {
    clicked && !destructive
}

pub(crate) fn dialog_outer_width(viewport_width: f32) -> f32 {
    viewport_width
        .min(config::DIALOG_MAX_WIDTH)
        .min((viewport_width - config::DIALOG_VIEWPORT_INSET * 2.0).max(0.0))
}

pub(crate) fn dialog_content_width(viewport_width: f32) -> f32 {
    (dialog_outer_width(viewport_width) - config::DIALOG_PADDING * 2.0).max(0.0)
}

#[cfg(test)]
mod tests {
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
}
