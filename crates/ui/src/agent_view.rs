use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};

/// What the AI features send off the machine, stated where the user enables them (#242).
///
/// The #122 trust-boundary audit (`docs/release/audit-security-boundaries.md` §5, finding T-1) found
/// the AI path to be the app's only egress and the product silent about it; the registry entry that
/// records the same facts is `docs/release/known-limitations.md` LIM-019. The numbers here track the
/// code rather than the audit text, which said 20×12: the agent tool result carries at most
/// `MAX_AGENT_SAMPLE_ROWS` (20) rows and `MAX_AGENT_RESULT_COLUMNS` (50) columns, each cell
/// truncated to `MAX_AGENT_CELL_CHARS` (256) characters
/// (`crates/core/src/domain/agent.rs:10-12`, applied in `agent_context.rs`). Egress requires a
/// configured key: with no provider the runtime answers "AI provider is not configured" and sends
/// nothing (`crates/runtime/src/worker.rs:1118`).
const AI_EGRESS_DISCLOSURE: &str = "With a key configured, the AI features send data to that provider: your prompts, the SQL they reference and the schema names and types around them. When the agent runs a query, up to 20 sample result rows (50 columns, 256 characters per cell) are sent too.\nYour database and SSH connections are the app's only other outbound connections.";

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
                    self.draw_agent_settings(ui);
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
        let typed_session_busy = self
            .query
            .session
            .documents
            .get(self.query.session.active_document_index)
            .and_then(|document| self.agent.sessions.get(&document.id))
            .is_some_and(|session| {
                session.active_run_id.is_some()
                    || session.request_id.is_some()
                    || session.pending_confirmation.is_some()
            });
        let can_clear_conversation = !typed_session_busy;
        ui.horizontal(|ui| {
            ui.label(icon_text(Icon::Sparkles, "Agent", self.theme.accent));
            if let Some(document_id) = self
                .query
                .session
                .documents
                .get(self.query.session.active_document_index)
                .map(|document| document.id.clone())
            {
                let session = self.agent.sessions.entry(document_id).or_default();
                let is_disabled = session.active_run_id.is_some()
                    || session.request_id.is_some()
                    || session.pending_confirmation.is_some();
                ui.add_enabled_ui(!is_disabled, |ui| {
                    egui::ComboBox::from_id_salt("agent-workflow-mode")
                        .selected_text(match session.mode {
                            db_pro_core::domain::agent::AgentMode::Ask => "Ask",
                            db_pro_core::domain::agent::AgentMode::Edit => "Edit",
                            db_pro_core::domain::agent::AgentMode::Agent => "Agent",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut session.mode, db_pro_core::domain::agent::AgentMode::Ask, "Ask");
                            ui.selectable_value(&mut session.mode, db_pro_core::domain::agent::AgentMode::Edit, "Edit");
                            ui.selectable_value(
                                &mut session.mode,
                                db_pro_core::domain::agent::AgentMode::Agent,
                                "Agent",
                            );
                        });
                });
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let typed_has_messages = self
                    .query
                    .session
                    .documents
                    .get(self.query.session.active_document_index)
                    .and_then(|document| self.agent.sessions.get(&document.id))
                    .is_some_and(|session| !session.messages.is_empty());
                if can_clear_conversation
                    && (!self.agent.messages.is_empty() || typed_has_messages)
                    && Button::new(self.theme)
                        .icon(Icon::RotateCcw)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Clear conversation")
                        .show(ui)
                        .clicked()
                {
                    self.agent.messages.clear();
                    if let Some(document) = self
                        .query
                        .session
                        .documents
                        .get(self.query.session.active_document_index)
                    {
                        if let Some(session) = self.agent.sessions.get_mut(&document.id) {
                            session.messages.clear();
                            session.activities.clear();
                            session.streaming_text.clear();
                            session.tool_results.clear();
                            session.state = db_pro_core::domain::agent::AgentSessionState::Idle;
                            session.active_run_id = None;
                        }
                    }
                }
                if Button::new(self.theme)
                    .icon(Icon::X)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Close Agent")
                    .show(ui)
                    .clicked()
                {
                    self.set_agent_open(false, ctx);
                }
                if Button::new(self.theme)
                    .icon(Icon::Settings)
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::IconSm)
                    .tooltip("Agent settings (API key)")
                    .show(ui)
                    .clicked()
                {
                    self.agent.settings_open = !self.agent.settings_open;
                    if self.agent.settings_open {
                        self.agent.api_key_draft.clear();
                        self.agent.api_key_show_password = false;
                    }
                }
            });
        });
    }

    fn draw_agent_settings(&mut self, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(icon_text(Icon::KeyRound, "API Key", self.theme.text_primary));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if Button::new(self.theme)
                        .icon(Icon::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip("Cancel")
                        .show(ui)
                        .clicked()
                    {
                        self.agent.settings_open = false;
                        self.agent.api_key_draft.clear();
                        self.agent.api_key_show_password = false;
                    }
                });
            });
            ui.add_space(4.0);
            ui.label(
                RichText::new("Enter a Groq or OpenAI API key to enable the AI provider.\nThe key is stored in DB Pro's secure secret store and never written to disk in plain text.")
                    .font(font_caption())
                    .color(self.theme.text_secondary),
            );
            ui.add_space(6.0);
            ui.label(
                RichText::new(AI_EGRESS_DISCLOSURE)
                    .font(font_caption())
                    .color(self.theme.text_muted),
            );
            ui.add_space(8.0);
            let current_label = if self.agent.provider_label == "Offline draft" {
                "Not configured".to_owned()
            } else {
                format!("Active: {}", self.agent.provider_label)
            };
            ui.label(
                RichText::new(current_label)
                    .font(font_caption())
                    .color(if self.agent.provider_label == "Offline draft" {
                        self.theme.text_muted
                    } else {
                        self.theme.success
                    }),
            );
            ui.add_space(6.0);
        });
        ui.add_space(6.0);

        let response = PasswordInput::new(
            &mut self.agent.api_key_draft,
            "gsk_… or sk-…",
            &mut self.agent.api_key_show_password,
            self.theme,
        )
        .id_salt("agent.api_key")
        .width(ui.available_width())
        .show(ui);
        // Allow Ctrl+Enter to save from the text field
        let save_shortcut = response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) && DbProApp::primary_modifier_pressed(i));

        ui.add_space(6.0);

        ui.horizontal(|ui| {
            let key_non_empty = !self.agent.api_key_draft.trim().is_empty();
            let is_saving = self.agent.configure_request.is_some();
            let save_btn = Button::new(self.theme)
                .icon(if is_saving { Icon::Loader } else { Icon::Check })
                .text(if is_saving { "Saving…" } else { "Save key" })
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .enabled(!is_saving && key_non_empty)
                .loading(is_saving)
                .show(ui);
            let save_clicked = (save_btn.clicked() || save_shortcut) && key_non_empty && !is_saving;
            if save_clicked {
                let request_id = self.task_bridge.next_request_id();
                self.agent.configure_request = Some(request_id);
                let api_key = self.agent.api_key_draft.trim().to_owned();
                let _ = self
                    .task_bridge
                    .send(UiCommand::SaveAgentApiKey { request_id, api_key });
            }
            if !key_non_empty {
                ui.label(
                    RichText::new("Paste an API key above")
                        .font(font_caption())
                        .color(self.theme.text_muted),
                );
            }
            let can_forget = self.agent.provider_label != "Offline draft" && !is_saving && !key_non_empty;
            if can_forget {
                let forget_button = Button::new(self.theme)
                    .icon(Icon::Trash2)
                    .text("Forget key")
                    .variant(ButtonVariant::Destructive)
                    .size(ButtonSize::Sm)
                    .show(ui);
                if forget_button.clicked() {
                    let request_id = self.task_bridge.next_request_id();
                    self.agent.configure_request = Some(request_id);
                    self.dispatch_command(UiCommand::ForgetAgentApiKey { request_id });
                }
            }
        });

        ui.add_space(8.0);
        ui.checkbox(
            &mut self.agent.auto_run_read_only,
            "Auto-run read-only queries in Agent mode",
        );
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);
        ui.label(
            RichText::new("Supported providers:\n• Groq  — gsk_… key, model openai/gpt-oss-120b\n• OpenAI — sk-… key, model gpt-5.6\n\nThe provider is detected automatically from the key prefix.")
                .font(font_caption())
                .color(self.theme.text_muted),
        );
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
        if context.selected_table.is_none() && context.current_sql.trim().is_empty() && context.last_error.is_none() {
            return false;
        }

        let mut submit = false;
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if !context.current_sql.trim().is_empty()
                    && Button::new(self.theme)
                        .icon(Icon::ChartNoAxesCombined)
                        .text("Explain query")
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                {
                    self.agent.input = "Explain the current SQL and its query plan".to_owned();
                    submit = true;
                }
                if !context.current_sql.trim().is_empty()
                    && Button::new(self.theme)
                        .icon(Icon::Gauge)
                        .text("Optimize")
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                {
                    self.agent.input = "Optimize the current SQL and explain the trade-offs".to_owned();
                    submit = true;
                }
                if context.selected_table.is_some()
                    && Button::new(self.theme)
                        .icon(Icon::Table2)
                        .text("Explain table")
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                {
                    self.agent.input = "Explain the selected table and suggest useful read-only queries".to_owned();
                    submit = true;
                }
                if context.last_error.is_some()
                    && Button::new(self.theme)
                        .icon(Icon::TriangleAlert)
                        .text("Investigate error")
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                {
                    self.agent.input = "Investigate the current database error and propose a safe fix".to_owned();
                    submit = true;
                }
            });
        });
        ui.add_space(8.0);
        submit
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
                app.draw_agent_settings(ui);
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
