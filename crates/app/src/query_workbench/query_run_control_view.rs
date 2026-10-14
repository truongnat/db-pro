//! Query run/stop control rendering and intent collection.
use super::*;
use lucide_icons::Icon;

pub(super) struct QueryRunControlContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) connected: bool,
    pub(super) active_request_id: Option<RequestId>,
    /// Set while a query runs — rendered as a live elapsed timer.
    pub(super) execution_started_at: Option<std::time::Instant>,
    pub(super) cancel_supported: bool,
    pub(super) cancel_reason: Option<&'a str>,
    pub(super) modifier: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum QueryRunControlAction {
    Run,
    Cancel(RequestId),
    ReportUnsupportedCancel(String),
    ReportDisconnected,
}

pub(super) fn draw_run_control(
    context: &QueryRunControlContext<'_>,
    ui: &mut egui::Ui,
) -> Option<QueryRunControlAction> {
    let run_button = if context.active_request_id.is_some() {
        if context.cancel_supported {
            Button::new(context.theme)
                .text(t!("query.stop"))
                .icon(Icon::Square)
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .tooltip("Stop query (Esc)")
                .show(ui)
        } else {
            let tip = context
                .cancel_reason
                .unwrap_or("Query running (cancellation is unsupported by this provider)");
            Button::new(context.theme)
                .text(t!("query.running"))
                .icon(Icon::Loader)
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .tooltip(tip)
                .show(ui)
        }
    } else {
        let tip = if !context.connected {
            "Connect to a database before running".to_owned()
        } else {
            format!("Run query ({}↵)", context.modifier)
        };
        Button::new(context.theme)
            .text(t!("query.run"))
            .icon(Icon::Play)
            .variant(ButtonVariant::Default)
            .size(ButtonSize::Sm)
            .tooltip(tip)
            .show(ui)
    };
    // Live elapsed readout while a request is in flight — the running state
    // was previously indistinguishable from idle at a glance.
    if context.active_request_id.is_some() {
        if let Some(started) = context.execution_started_at {
            crate::components::feedback::Spinner::new(context.theme).show(ui);
            ui.label(
                RichText::new(format!("{:.1}s", started.elapsed().as_secs_f32()))
                    .small()
                    .color(context.theme.text_muted),
            );
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
    if !run_button.clicked() {
        return None;
    }

    match context.active_request_id {
        Some(request_id) if context.cancel_supported => Some(QueryRunControlAction::Cancel(request_id)),
        Some(_) => Some(QueryRunControlAction::ReportUnsupportedCancel(
            context
                .cancel_reason
                .unwrap_or("Query cancellation is not supported for this provider")
                .to_owned(),
        )),
        None if context.connected => Some(QueryRunControlAction::Run),
        None => Some(QueryRunControlAction::ReportDisconnected),
    }
}
