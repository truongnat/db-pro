use super::agent_settings_view::AgentSettingsAction;
use super::agent_surface_view::{AgentPanelAction, AgentPanelContext};
use super::*;

impl DbProApp {
    pub(super) fn draw_agent_panel(&mut self, ctx: &egui::Context) {
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
        let (panel_width, actions) = panel.draw(ctx);
        self.workspace.set_agent_width(panel_width);
        self.apply_agent_panel_actions(actions, ctx, document_id.as_deref());
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
mod tests {
    use super::super::agent_settings_view::{AgentSettingsContext, AI_EGRESS_DISCLOSURE};
    use super::*;

    /// Every text run the frame actually painted.
    ///
    /// The assertions below read the rendered frame rather than the constant the panel is written
    /// from, so deleting the label while keeping the constant still fails (the same idiom the SSH
    /// qualification caveat is pinned with, `connection_view.rs`).
    fn rendered_settings_texts(app: &mut DbProApp) -> Vec<String> {
        fn collect(shape: &egui::Shape, texts: &mut Vec<String>) {
            match shape {
                egui::Shape::Text(text) => texts.push(text.galley.text().to_owned()),
                egui::Shape::Vec(shapes) => {
                    for shape in shapes {
                        collect(shape, texts);
                    }
                }
                _ => {}
            }
        }

        let ctx = egui::Context::default();
        DbProTheme::install_fonts(&ctx);
        let output = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut settings = AgentSettingsContext {
                    theme: app.theme,
                    provider_label: &app.agent.provider_label,
                    api_key_draft: &mut app.agent.api_key_draft,
                    api_key_show_password: &mut app.agent.api_key_show_password,
                    configure_request: app.agent.configure_request,
                    auto_run_read_only: &mut app.agent.auto_run_read_only,
                };
                let actions = settings.draw(ui);
                assert!(actions.is_empty());
            });
        });

        let mut texts = Vec::new();
        for clipped in &output.shapes {
            collect(&clipped.shape, &mut texts);
        }
        texts
    }

    #[test]
    fn agent_key_section_discloses_what_the_ai_path_sends() {
        let mut app = DbProApp::default();

        let texts = rendered_settings_texts(&mut app);

        assert!(
            texts.iter().any(|text| text == AI_EGRESS_DISCLOSURE),
            "the surface that enables the AI provider must state what leaves the machine (#242); painted texts: {texts:?}"
        );
        // The note must not replace the key-handling sentence the section already carried.
        assert!(
            texts
                .iter()
                .any(|text| text.contains("stored in DB Pro's secure secret store")),
            "the API-key handling caption is still painted; painted texts: {texts:?}"
        );
    }

    #[test]
    fn agent_egress_disclosure_names_the_provider_payload_and_limits() {
        // What is sent, and the code-derived bounds rather than the audit's stale 20x12.
        assert!(AI_EGRESS_DISCLOSURE.contains("prompts"));
        assert!(AI_EGRESS_DISCLOSURE.contains("SQL they reference"));
        assert!(AI_EGRESS_DISCLOSURE.contains("schema names and types"));
        assert!(AI_EGRESS_DISCLOSURE.contains("up to 20 sample result rows"));
        assert!(AI_EGRESS_DISCLOSURE.contains("50 columns"));
        assert!(AI_EGRESS_DISCLOSURE.contains("256 characters"));
        // And the only-egress claim, which is what the audit verified.
        assert!(AI_EGRESS_DISCLOSURE.contains("only other outbound connections"));
        // The audit's finding was silence, so the note states the data flow and nothing more: a
        // privacy claim would contradict LIM-019's "Actual behavior" field. This is a keyword guard,
        // not a proof of semantics -- it catches the obvious overclaim, and the painted-text test
        // above pins the wording itself.
        let lowered = AI_EGRESS_DISCLOSURE.to_lowercase();
        for overclaim in ["private", "anonymous", "never leaves", "encrypted", "secure"] {
            assert!(
                !lowered.contains(overclaim),
                "the disclosure must not claim {overclaim:?}: it records what is sent, not a privacy guarantee"
            );
        }
    }
}
