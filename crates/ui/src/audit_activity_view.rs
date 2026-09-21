use super::*;

#[path = "audit_surface_view.rs"]
mod audit_surface_view;

impl DbProApp {
    pub(super) fn draw_audit_activity(&mut self, ui: &mut egui::Ui) {
        let actions = audit_surface_view::AuditSurfaceContext {
            theme: self.theme,
            state: &mut self.management.audit,
        }
        .draw(ui);
        self.apply_audit_surface_actions(actions);
    }

    fn apply_audit_surface_actions(&mut self, actions: Vec<audit_surface_view::AuditSurfaceAction>) {
        for action in actions {
            match action {
                audit_surface_view::AuditSurfaceAction::Refresh => self.request_audit_page(),
                audit_surface_view::AuditSurfaceAction::ExportSelected => self.export_selected_audit_events(),
                audit_surface_view::AuditSurfaceAction::SetSelected { event_id, selected } => {
                    if selected {
                        self.management.audit.audit_selected.insert(event_id);
                    } else {
                        self.management.audit.audit_selected.remove(&event_id);
                    }
                }
                audit_surface_view::AuditSurfaceAction::ToggleBookmark(event_id) => {
                    if !self.management.audit.audit_bookmarks.remove(&event_id) {
                        self.management.audit.audit_bookmarks.insert(event_id);
                    }
                }
                audit_surface_view::AuditSurfaceAction::OpenQuery(query) => {
                    self.set_active_query_text(query);
                    self.workspace.active_tab = WorkspaceTab::Query;
                    self.workspace.activity = Activity::Explorer;
                }
            }
        }
    }

    fn request_audit_page(&mut self) {
        let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) else {
            self.management.audit.audit_error = Some("Connect a database first".into());
            return;
        };
        let request_id = self.task_bridge.next_request_id();
        self.dispatch_command(self.management.audit.events_load_command(request_id, connection_id));
    }

    fn export_selected_audit_events(&mut self) {
        match self.management.audit.build_export_preview() {
            Ok((selected_count, export_warning)) => {
                self.feedback.runtime_message =
                    format!("Audit export preview · {selected_count} row(s) · {export_warning}");
            }
            Err(error) => self.management.audit.audit_error = Some(error),
        }
    }
}
