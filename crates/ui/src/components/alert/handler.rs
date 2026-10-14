//! Typed alert variant and dialog interaction decisions.

use egui::Color32;
use lucide_icons::Icon;

use super::{config, AlertDialogAction, AlertVariant};
use crate::DbProTheme;

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
            fill: theme.semantic.status.info.fill,
            border: theme.semantic.status.info.border,
            icon: theme.semantic.status.info.foreground,
            default_icon: Icon::Info,
        },
        AlertVariant::Success => AlertStyle {
            fill: theme.semantic.status.success.fill,
            border: theme.semantic.status.success.border,
            icon: theme.semantic.status.success.foreground,
            default_icon: Icon::CheckCircle2,
        },
        AlertVariant::Warning => AlertStyle {
            fill: theme.semantic.status.warning.fill,
            border: theme.semantic.status.warning.border,
            icon: theme.semantic.status.warning.foreground,
            default_icon: Icon::AlertTriangle,
        },
        AlertVariant::Destructive => AlertStyle {
            fill: theme.semantic.status.danger.fill,
            border: theme.semantic.status.danger.border,
            icon: theme.semantic.status.danger.foreground,
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
#[path = "tests/handler_tests.rs"]
mod tests;
