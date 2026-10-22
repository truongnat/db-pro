// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
//! Debug-capability status block of the routine workbench (#252).
//!
//! The block is honest by construction: it renders what the catalog probe measured and, when
//! the server cannot host a PL/pgSQL debugger, names the blocker plus the manual DBA path.
//! Nothing here pretends a debug session exists — DB Pro has no `pldbgapi` protocol client in
//! this build, and it never installs server extensions.

use super::*;
use crate::components::alert::{Alert, AlertVariant};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use db_pro_core::domain::routine_debug::{RoutineDebugProbe, RoutineDebugState};
use egui::RichText;

/// One rendered line group of the status block, computed without touching egui so it can be
/// asserted in a unit test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RoutineDebugGuidance {
    /// Badge text naming the measured server state.
    pub(super) status: String,
    /// Why the state matters, in the user's language.
    pub(super) reason: String,
    /// Manual install guidance; empty when the server needs no change.
    pub(super) install_guide: Option<String>,
    /// Standing limitation: interactive stepping is not implemented in this build.
    pub(super) stepping_note: String,
    /// Extra warning for connections the user marked as production.
    pub(super) production_warning: Option<String>,
}

/// Builds the guidance for the current probe result.
///
/// `environment_label` is `Some(...)` only for a production connection, keeping the warning
/// tied to the same [`db_pro_core::domain::connection::ConnectionEnvironment`] the safety
/// guards use instead of a re-derived string.
pub(super) fn build_debug_guidance(
    probe: Option<&RoutineDebugProbe>,
    probing: bool,
    environment_label: Option<&str>,
) -> RoutineDebugGuidance {
    let stepping_note = t!("routine.debug_stepping_note").into_owned();
    let production_warning = environment_label
        .map(|environment| t!("routine.debug_production_warning", environment = environment).into_owned());
    let Some(probe) = probe else {
        let (status, reason) = if probing {
            (
                t!("routine.debug_checking").into_owned(),
                t!("routine.debug_checking").into_owned(),
            )
        } else {
            (
                t!("routine.debug_state_unprobed").into_owned(),
                t!("routine.debug_reason_unprobed").into_owned(),
            )
        };
        return RoutineDebugGuidance {
            status,
            reason,
            install_guide: None,
            stepping_note,
            production_warning,
        };
    };
    let version = if probe.server_version.is_empty() {
        "an unknown release"
    } else {
        &probe.server_version
    };
    let (status, reason, install_guide) = match probe.state {
        RoutineDebugState::UnsupportedDriver => (
            t!("routine.debug_state_unsupported").into_owned(),
            t!("routine.debug_reason_unsupported").into_owned(),
            None,
        ),
        RoutineDebugState::ProbeFailed => (
            t!("routine.debug_state_probe_failed").into_owned(),
            t!(
                "routine.debug_reason_probe_failed",
                error = probe.probe_error.as_deref().unwrap_or("the catalog read failed")
            )
            .into_owned(),
            None,
        ),
        RoutineDebugState::ExtensionMissing => (
            t!("routine.debug_state_missing").into_owned(),
            t!("routine.debug_reason_missing", version = version).into_owned(),
            Some(t!("routine.debug_install_guide").into_owned()),
        ),
        RoutineDebugState::ExtensionNotPreloaded => (
            t!("routine.debug_state_not_preloaded").into_owned(),
            t!("routine.debug_reason_not_preloaded").into_owned(),
            Some(t!("routine.debug_install_guide").into_owned()),
        ),
        // cc-scan:allow LINE_TOO_LONG — literal must not wrap
        RoutineDebugState::ServerReady => (
            t!("routine.debug_state_server_ready").into_owned(),
            t!("routine.debug_reason_server_ready").into_owned(),
            None,
        ),
    };
    RoutineDebugGuidance {
        status,
        reason,
        install_guide,
        stepping_note,
        production_warning,
    }
}

pub(super) struct RoutineDebugStatusContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) guidance: RoutineDebugGuidance,
    /// Measured server state, or `None` while nothing has been probed for this connection.
    pub(super) state: Option<RoutineDebugState>,
    /// Whether the block is showing a pending probe (button must not spam requests).
    pub(super) probing: bool,
    /// Whether a re-check is possible at all (needs a live connection).
    pub(super) can_recheck: bool,
    pub(super) recheck_slot: &'a mut bool,
}

/// Draws the status block; returns `true` when the user asked for a fresh probe.
pub(super) fn draw_debug_status(context: &mut RoutineDebugStatusContext<'_>, ui: &mut egui::Ui) -> bool {
    let mut requested = false;
    card_frame(context.theme).show(ui, |ui| {
        section_label(ui, t!("routine.debug_title").into_owned(), context.theme);
        ui.add_space(SPACE_SM);
        Badge::new(context.guidance.status.as_str(), context.theme)
            .variant(status_badge_variant(context.state))
            .compact(true)
            .show(ui);
        ui.add_space(SPACE_SM);
        Alert::new(
            t!("routine.debug_title").into_owned(),
            context.guidance.reason.clone(),
            context.theme,
        )
        .variant(alert_variant(context.state))
        .show(ui);
        if let Some(guide) = &context.guidance.install_guide {
            ui.add_space(SPACE_SM);
            ui.label(RichText::new(guide).small().color(context.theme.text_secondary));
        }
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new(context.guidance.stepping_note.clone())
                .small()
                .color(context.theme.text_muted),
        );
        if let Some(warning) = &context.guidance.production_warning {
            ui.add_space(SPACE_SM);
            ui.label(RichText::new(warning.clone()).small().color(context.theme.warning));
        }
        ui.add_space(SPACE_SM);
        ui.add_enabled_ui(!context.probing && context.can_recheck, |ui| {
            if Button::new(context.theme)
                .icon(lucide_icons::Icon::RefreshCw)
                .text(t!("routine.debug_recheck").into_owned())
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                *context.recheck_slot = true;
                requested = true;
            }
        });
    });
    requested
}

fn alert_variant(state: Option<RoutineDebugState>) -> AlertVariant {
    match state {
        None => AlertVariant::Default,
        Some(RoutineDebugState::ServerReady) => AlertVariant::Info,
        Some(RoutineDebugState::ExtensionNotPreloaded | RoutineDebugState::ProbeFailed) => AlertVariant::Warning,
        Some(RoutineDebugState::ExtensionMissing | RoutineDebugState::UnsupportedDriver) => AlertVariant::Destructive,
    }
}

fn status_badge_variant(state: Option<RoutineDebugState>) -> BadgeVariant {
    match state {
        None => BadgeVariant::Secondary,
        Some(RoutineDebugState::ServerReady) => BadgeVariant::Info,
        Some(RoutineDebugState::ExtensionNotPreloaded | RoutineDebugState::ProbeFailed) => BadgeVariant::Warning,
        Some(RoutineDebugState::ExtensionMissing | RoutineDebugState::UnsupportedDriver) => BadgeVariant::Secondary,
    }
}

#[cfg(test)]
#[path = "tests/routine_debug_status_view_tests.rs"]
mod tests;
