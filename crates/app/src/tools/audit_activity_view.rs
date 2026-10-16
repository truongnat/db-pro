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

    /// Build the CSV, then route through the native save-file picker; the
    /// picked path comes back as `FilePicked { kind: "audit-export" }` and the
    /// write happens in `write_audit_export` (#406).
    fn export_selected_audit_events(&mut self) {
        match self.state.build_export_preview() {
            Ok((selected_count, export_warning)) => {
                self.feedback.runtime_message =
                    format!("Audit export · {selected_count} row(s) · {export_warning} — choose a destination file");
                let request_id = self.command_dispatcher.next_request_id();
                self.dispatch(UiCommand::PickAuditExportFile { request_id });
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

impl DbProApp {
    /// Write the staged audit CSV to the path the native save dialog returned
    /// (#406). Atomic so a failed write never leaves a truncated export that
    /// looks complete.
    pub(crate) fn write_audit_export(&mut self, path: &str) {
        let Some(csv) = self.management.audit.audit_export_preview.clone() else {
            self.feedback.runtime_message = "No audit export pending — select events and export again".to_owned();
            return;
        };
        match query_view::write_file_atomically(std::path::Path::new(path), csv.as_bytes()) {
            Ok(()) => {
                self.management.audit.audit_export_preview = None;
                self.feedback.runtime_message = format!("Audit export written to {path}");
            }
            Err(error) => {
                self.management.audit.audit_error = Some(format!("Audit export failed: {error}"));
            }
        }
    }
}
