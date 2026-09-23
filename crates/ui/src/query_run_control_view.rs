//! Query run/stop control rendering and intent collection.
use super::*;
use lucide_icons::Icon;

pub(super) struct QueryRunControlContext<'a> {
    pub(super) theme: DbProTheme,
    pub(super) connected: bool,
    pub(super) active_request_id: Option<RequestId>,
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
                .text("Stop")
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
                .text("Running…")
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
            .text("Run")
            .icon(Icon::Play)
            .variant(ButtonVariant::Default)
            .size(ButtonSize::Sm)
            .tooltip(tip)
            .show(ui)
    };
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
