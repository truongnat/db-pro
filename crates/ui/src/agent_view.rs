use super::agent_context_actions_view::{AgentContextAction, AgentContextActionsContext};
use super::agent_header_view::{AgentHeaderAction, AgentHeaderContext};
use super::agent_settings_view::{AgentSettingsAction, AgentSettingsContext};
use super::*;

impl DbProApp {
    pub(super) fn draw_agent_panel(&mut self, ctx: &egui::Context) {
        let mut submit = false;
        let mut copy_sql = None;
        let agent_width = self.workspace.agent_width;
        let response = egui::SidePanel::right("agent_panel")
            .resizable(true)
            .default_width(agent_width)
            .width_range(AGENT_MIN_WIDTH..=AGENT_MAX_WIDTH)
            .frame(sidebar_frame(self.theme))
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                self.draw_agent_header(ui, ctx);
                if self.agent.settings_open {
                    let mut settings = AgentSettingsContext {
                        theme: self.theme,
                        provider_label: &self.agent.provider_label,
                        api_key_draft: &mut self.agent.api_key_draft,
                        api_key_show_password: &mut self.agent.api_key_show_password,
                        configure_request: self.agent.configure_request,
                        auto_run_read_only: &mut self.agent.auto_run_read_only,
                    };
                    let actions = settings.draw(ui);
                    self.apply_agent_settings_actions(actions);
                } else {
                    ui.add_space(6.0);
                    let context = self.agent_context();
                    self.draw_agent_context(ui, &context);
                    ui.add_space(8.0);
                    submit |= self.draw_agent_context_actions(ui, &context);
                    submit |= self.draw_agent_thread(ui, &mut copy_sql);
                    self.draw_agent_composer(ui, &mut submit);
                }
            });
        self.workspace.set_agent_width(response.response.rect.width());
        if submit {
            self.submit_agent_prompt();
        }
        if let Some(sql) = copy_sql {
            ctx.output_mut(|output| output.copied_text = sql);
            self.feedback.copy_status = "Agent SQL copied".to_owned();
        }
    }

    fn draw_agent_header(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let Some(document_id) = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .map(|document| document.id.clone())
        else {
            return;
        };
        let (mode, mode_disabled, has_messages) = {
            let session = self.agent.sessions.entry(document_id.clone()).or_default();
            let disabled = session.active_run_id.is_some()
                || session.request_id.is_some()
                || session.pending_confirmation.is_some();
            (&mut session.mode, disabled, !session.messages.is_empty())
        };
        let actions = {
            let mut header = AgentHeaderContext {
                theme: self.theme,
                mode: Some(mode),
                mode_disabled,
                can_clear_conversation: !mode_disabled,
                has_messages,
            };
            header.draw(ui)
        };
        for action in actions {
            match action {
                AgentHeaderAction::ClearConversation => self.agent.clear_session(&document_id),
                AgentHeaderAction::Close => self.set_agent_open(false, ctx),
                AgentHeaderAction::ToggleSettings => self.agent.toggle_settings(),
            }
        }
    }

    fn apply_agent_settings_actions(&mut self, actions: Vec<AgentSettingsAction>) {
        for action in actions {
            match action {
                AgentSettingsAction::Close => {
                    self.agent.close_settings();
                }
                AgentSettingsAction::SaveKey(api_key) => {
                    let request_id = self.task_bridge.next_request_id();
                    self.agent.configure_request = Some(request_id);
                    if self
                        .task_bridge
                        .send(UiCommand::SaveAgentApiKey { request_id, api_key })
                        .is_err()
                    {
                        self.agent.configure_request = None;
                        self.feedback.runtime_message = "Agent runtime unavailable".to_owned();
                    }
                }
                AgentSettingsAction::ForgetKey => {
                    let request_id = self.task_bridge.next_request_id();
                    self.agent.configure_request = Some(request_id);
                    self.dispatch_command(UiCommand::ForgetAgentApiKey { request_id });
                }
            }
        }
    }

    fn draw_agent_context(&self, ui: &mut egui::Ui, context: &AgentContext) {
        toolbar_frame(self.theme).show(ui, |ui| {
            egui::ScrollArea::horizontal()
                .id_salt("agent-context-chips")
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if self.agent.provider_label == "Offline draft" {
                            badge(ui, "Preview", self.theme.surface_active, self.theme.text_secondary);
                        }
                        badge(
                            ui,
                            &self.agent.provider_label,
                            self.theme.accent_soft,
                            self.theme.accent,
                        );
                        if self.agent.auto_run_read_only {
                            badge(ui, "Auto-run Read-only", self.theme.accent_soft, self.theme.accent);
                        }
                        ContextChip::new(
                            ContextChipKind::Connection,
                            context.connection_name.as_deref().unwrap_or("No connection"),
                            self.theme,
                        )
                        .show(ui);
                        ContextChip::new(ContextChipKind::Database, &context.driver, self.theme).show(ui);
                        if let Some(schema) = context.schema.as_deref() {
                            ContextChip::new(ContextChipKind::Schema, schema, self.theme).show(ui);
                        }
                        if let Some(table) = context.selected_table.as_deref() {
                            ContextChip::new(ContextChipKind::Table, table, self.theme).show(ui);
                        }
                        if context.explain_plan.is_some() {
                            ContextChip::new(ContextChipKind::Editor, "EXPLAIN PLAN", self.theme).show(ui);
                        }
                    });
                });
            ui.label(
                RichText::new(&self.agent.provider_detail)
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );
        });
    }

    fn draw_agent_context_actions(&mut self, ui: &mut egui::Ui, context: &AgentContext) -> bool {
        let actions = AgentContextActionsContext {
            theme: self.theme,
            context,
        }
        .draw(ui);
        let Some(AgentContextAction::Submit(prompt)) = actions.into_iter().next() else {
            return false;
        };
        self.agent.input = prompt.to_owned();
        true
    }

    fn draw_agent_composer(&mut self, ui: &mut egui::Ui, submit: &mut bool) {
        let active_mode = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .and_then(|document| self.agent.sessions.get(&document.id))
            .map(|session| session.mode);
        let composer_mode = match active_mode {
            Some(db_pro_core::domain::agent::AgentMode::Ask) => AgentMode::Chat,
            Some(db_pro_core::domain::agent::AgentMode::Edit) => AgentMode::Plan,
            Some(db_pro_core::domain::agent::AgentMode::Agent) => AgentMode::Code,
            None => AgentMode::Code,
        };
        let is_generating = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .and_then(|document| self.agent.sessions.get(&document.id))
            .is_some_and(|session| session.active_run_id.is_some() || session.request_id.is_some());
        let action = AgentComposer::new(
            &mut self.agent.input,
            &self.agent.provider_label,
            composer_mode,
            self.theme,
        )
        .is_generating(is_generating)
        .show(ui);

        match action {
            Some(AgentComposerAction::Submit) => *submit = true,
            Some(AgentComposerAction::Stop) => self.cancel_active_agent_run(),
            Some(AgentComposerAction::Clear) | None => {}
        }
    }
}

pub(super) fn agent_confirmation_title(
    kind: db_pro_core::domain::agent_workflow::AgentConfirmationKind,
) -> &'static str {
    match kind {
        db_pro_core::domain::agent_workflow::AgentConfirmationKind::ApplyPatch => "Apply Agent change?",
        db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunReadOnly => "Run read-only query?",
        db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunMutation => "Run mutation?",
        db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunDestructive => "Execute destructive query?",
        db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunUnknown => "Run unclassified query?",
    }
}

#[cfg(test)]
mod tests {
    use super::super::agent_settings_view::AI_EGRESS_DISCLOSURE;
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
