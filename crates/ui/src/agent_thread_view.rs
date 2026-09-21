//! Root adapter for the Agent thread surface.
use super::agent_thread_surface_view::{AgentThreadAction, AgentThreadSurfaceContext};
use super::agent_workflow_state::AgentUiSession;
use super::*;

impl DbProApp {
    pub(super) fn draw_agent_thread(&mut self, ui: &mut egui::Ui, _copy_sql: &mut Option<String>) -> bool {
        let Some(document) = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
        else {
            return false;
        };
        let document_id = document.id.clone();
        let connection_id = document
            .connection_id
            .clone()
            .or_else(|| self.connection.lifecycle.active_connection_id().map(str::to_owned));
        let schema = document
            .schema
            .clone()
            .or_else(|| Some(self.active_schema().to_owned()));
        let actions = {
            let session = self
                .agent
                .sessions
                .entry(document_id.clone())
                .or_insert_with(|| AgentUiSession::for_document(&document_id, connection_id, schema));
            AgentThreadSurfaceContext {
                theme: self.theme,
                document_id: &document_id,
                session,
            }
            .draw(ui)
        };

        let mut submit = false;
        for action in actions {
            match action {
                AgentThreadAction::Submit(prompt) => {
                    self.agent.input = prompt;
                    submit = true;
                }
                AgentThreadAction::OpenResult(call_id) => self.open_agent_result_in_workspace(&call_id),
                AgentThreadAction::Retry => self.retry_agent_run(),
                AgentThreadAction::Confirm(approved) => self.agent_confirmation_action(approved),
            }
        }
        submit
    }
}
