// cc-scan:allow-file HUGE_FUNCTION,LONG_FUNCTION,HARD_COMPLEXITY,COMPLEXITY,DEEP_NESTING
// cc-scan:allow-file HARD_PARAMS,TOO_MANY_PARAMS,DUPLICATE_BLOCK
// egui painter/view file: fns are linear layout code; branches are per-state paint variants.
use super::agent_settings_view::AgentSettingsAction;
use super::agent_surface_view::{AgentPanelAction, AgentPanelContext};
use super::*;

impl DbProApp {
    pub(super) fn draw_agent_panel(&mut self, ui: &mut egui::Ui) {
        let document_id = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .map(|document| document.id.clone());
        let (composer_mode, is_generating) = document_id
            .as_deref()
            .map(|id| self.agent.sessions.entry(id.to_owned()).or_default())
            .map(|session| {
                let composer_mode = match session.mode {
                    db_pro_core::domain::agent::AgentMode::Ask => AgentMode::Chat,
                    db_pro_core::domain::agent::AgentMode::Edit => AgentMode::Plan,
                    db_pro_core::domain::agent::AgentMode::Agent => AgentMode::Code,
                };
                (
                    composer_mode,
                    session.active_run_id.is_some() || session.request_id.is_some(),
                )
            })
            .unwrap_or((AgentMode::Code, false));
        let context = self.agent_context();
        let session = document_id
            .as_deref()
            .map(|id| self.agent.sessions.entry(id.to_owned()).or_default());

        let panel = AgentPanelContext {
            theme: self.theme,
            default_width: self.workspace.agent_width,
            document_id: document_id.as_deref(),
            settings_open: self.agent.settings_open,
            provider_label: &self.agent.provider_label,
            provider_detail: &self.agent.provider_detail,
            api_key_draft: &mut self.agent.api_key_draft,
            api_key_show_password: &mut self.agent.api_key_show_password,
            configure_request: self.agent.configure_request,
            auto_run_read_only: &mut self.agent.auto_run_read_only,
            context: &context,
            session,
            input: &mut self.agent.input,
            composer_mode,
            is_generating,
        };
        let (panel_width, actions) = panel.draw(ui);
        self.workspace.set_agent_width(panel_width);
        self.apply_agent_panel_actions(actions, ui.ctx(), document_id.as_deref());
    }

    fn apply_agent_panel_actions(
        &mut self,
        actions: Vec<AgentPanelAction>,
        ctx: &egui::Context,
        document_id: Option<&str>,
    ) {
        let mut submit = false;
        for action in actions {
            match action {
                AgentPanelAction::Header(action) => match action {
                    agent_header_view::AgentHeaderAction::ClearConversation => {
                        if let Some(document_id) = document_id {
                            self.agent.clear_session(document_id);
                        }
                    }
                    agent_header_view::AgentHeaderAction::Close => self.set_agent_open(false, ctx),
                    agent_header_view::AgentHeaderAction::ToggleSettings => self.agent.toggle_settings(),
                },
                AgentPanelAction::Settings(action) => self.apply_agent_settings_action(action),
                AgentPanelAction::Context(agent_context_actions_view::AgentContextAction::Submit(prompt)) => {
                    self.agent.input = prompt.to_owned();
                    submit = true;
                }
                AgentPanelAction::Thread(action) => match action {
                    agent_thread_surface_view::AgentThreadAction::Submit(prompt) => {
                        self.agent.input = prompt;
                        submit = true;
                    }
                    agent_thread_surface_view::AgentThreadAction::OpenResult(call_id) => {
                        self.open_agent_result_in_workspace(&call_id);
                    }
                    agent_thread_surface_view::AgentThreadAction::Retry => self.retry_agent_run(),
                    agent_thread_surface_view::AgentThreadAction::Confirm(approved) => {
                        self.agent_confirmation_action(approved);
                    }
                    agent_thread_surface_view::AgentThreadAction::LoadEarlier => {
                        if let Some(session) = document_id.and_then(|id| self.agent.sessions.get_mut(id)) {
                            session.visible_message_limit += 100;
                        }
                    }
                },
                AgentPanelAction::Composer(action) => match action {
                    AgentComposerAction::Submit => submit = true,
                    AgentComposerAction::Stop => self.cancel_active_agent_run(),
                    AgentComposerAction::Clear => {}
                },
            }
        }
        if submit {
            self.submit_agent_prompt();
        }
    }

    fn apply_agent_settings_action(&mut self, action: AgentSettingsAction) {
        match action {
            AgentSettingsAction::Close => {
                self.agent.close_settings();
            }
            AgentSettingsAction::SaveKey(api_key) => {
                let request_id = self.next_request_id();
                self.agent.configure_request = Some(request_id);
                if !self.dispatch_command(UiCommand::SaveAgentApiKey { request_id, api_key }) {
                    self.agent.configure_request = None;
                }
            }
            AgentSettingsAction::ForgetKey => {
                let request_id = self.next_request_id();
                if self.dispatch_command(UiCommand::ForgetAgentApiKey { request_id }) {
                    self.agent.configure_request = Some(request_id);
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/agent_view_tests.rs"]
mod tests;

