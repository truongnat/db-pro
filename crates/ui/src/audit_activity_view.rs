use super::audit_state::AuditState;
use super::command_dispatch::RuntimeCommandDispatcher;
use super::*;

#[path = "audit_surface_view.rs"]
mod audit_surface_view;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum AuditActivityEffect {
    OpenQuery(String),
}

pub(super) struct AuditActivityContext<'a, 'bridge> {
    pub(super) theme: DbProTheme,
    pub(super) state: &'a mut AuditState,
    pub(super) connection_id: Option<&'a str>,
    pub(super) command_dispatcher: &'a mut RuntimeCommandDispatcher<'bridge>,
    pub(super) feedback: &'a mut FeedbackState,
}

impl AuditActivityContext<'_, '_> {
    pub(super) fn draw(&mut self, ui: &mut egui::Ui) -> Option<AuditActivityEffect> {
        let actions = audit_surface_view::AuditSurfaceContext {
            theme: self.theme,
            state: self.state,
        }
        .draw(ui);
        self.apply_actions(actions)
    }

    fn apply_actions(
        &mut self,
        actions: Vec<audit_surface_view::AuditSurfaceAction>,
    ) -> Option<AuditActivityEffect> {
        let mut effect = None;
        for action in actions {
            match action {
                audit_surface_view::AuditSurfaceAction::Refresh => self.request_audit_page(),
                audit_surface_view::AuditSurfaceAction::ExportSelected => self.export_selected_audit_events(),
                audit_surface_view::AuditSurfaceAction::SetSelected { event_id, selected } => {
                    if selected {
                        self.state.audit_selected.insert(event_id);
                    } else {
                        self.state.audit_selected.remove(&event_id);
                    }
                }
                audit_surface_view::AuditSurfaceAction::ToggleBookmark(event_id) => {
                    if !self.state.audit_bookmarks.remove(&event_id) {
                        self.state.audit_bookmarks.insert(event_id);
                    }
                }
                audit_surface_view::AuditSurfaceAction::OpenQuery(query) => {
                    effect = Some(AuditActivityEffect::OpenQuery(query));
                }
            }
        }
        effect
    }

    fn request_audit_page(&mut self) {
        let Some(connection_id) = self.connection_id else {
            self.state.audit_error = Some("Connect a database first".into());
            return;
        };
        let request_id = self.command_dispatcher.next_request_id();
        self.dispatch(audit_events_load_command(
            self.state,
            request_id,
            connection_id.to_owned(),
        ));
    }

    fn export_selected_audit_events(&mut self) {
        match self.state.build_export_preview() {
            Ok((selected_count, export_warning)) => {
                self.feedback.runtime_message =
                    format!("Audit export preview · {selected_count} row(s) · {export_warning}");
            }
            Err(error) => self.state.audit_error = Some(error),
        }
    }

    fn dispatch(&mut self, command: UiCommand) -> bool {
        self.command_dispatcher.dispatch(command, self.feedback)
    }
}

fn audit_events_load_command(state: &AuditState, request_id: RequestId, connection_id: String) -> UiCommand {
    UiCommand::AuditEventsLoad {
        request_id,
        connection_id,
        filter: state.audit_filter(),
        limit: Some(100),
    }
}
