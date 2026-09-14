use super::*;

impl DbProApp {
    pub(super) fn draw_agent_panel(&mut self, ctx: &egui::Context) {
        let mut submit = false;
        let mut copy_sql = None;
        let agent_width = self.agent_width;
        let response = egui::SidePanel::right("agent_panel")
            .resizable(true)
            .default_width(agent_width)
            .width_range(AGENT_MIN_WIDTH..=AGENT_MAX_WIDTH)
            .frame(sidebar_frame(self.theme))
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                self.draw_agent_header(ui, ctx);
                if self.agent_settings_open {
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
        self.agent_width = response.response.rect.width().clamp(AGENT_MIN_WIDTH, AGENT_MAX_WIDTH);
        if submit {
            self.submit_agent_prompt();
        }
        if let Some(sql) = copy_sql {
            ctx.output_mut(|output| output.copied_text = sql);
            self.copy_status = "Agent SQL copied".to_owned();
        }
    }

    fn draw_agent_header(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let typed_session_busy = self
            .query_documents
            .get(self.active_query_document)
            .and_then(|document| self.agent_sessions.get(&document.id))
            .is_some_and(|session| {
                session.active_run_id.is_some()
                    || session.request_id.is_some()
                    || session.pending_confirmation.is_some()
            });
        let can_clear_conversation = !typed_session_busy;
        ui.horizontal(|ui| {
            ui.label(icon_text(Icon::Sparkles, "Agent", self.theme.accent));
            if let Some(document_id) = self
                .query_documents
                .get(self.active_query_document)
                .map(|document| document.id.clone())
            {
                let session = self.agent_sessions.entry(document_id).or_default();
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
                    .query_documents
                    .get(self.active_query_document)
                    .and_then(|document| self.agent_sessions.get(&document.id))
                    .is_some_and(|session| !session.messages.is_empty());
                if can_clear_conversation
                    && (!self.agent_messages.is_empty() || typed_has_messages)
                    && compact_icon_button(ui, Icon::RotateCcw, self.theme)
                        .on_hover_text("Clear conversation")
                        .clicked()
                {
                    self.agent_messages.clear();
                    if let Some(document) = self.query_documents.get(self.active_query_document) {
                        if let Some(session) = self.agent_sessions.get_mut(&document.id) {
                            session.messages.clear();
                            session.activities.clear();
                            session.streaming_text.clear();
                            session.tool_results.clear();
                            session.state = db_pro_core::domain::agent::AgentSessionState::Idle;
                            session.active_run_id = None;
                        }
                    }
                }
                if compact_icon_button(ui, Icon::X, self.theme)
                    .on_hover_text("Close Agent")
                    .clicked()
                {
                    self.set_agent_open(false, ctx);
                }
                let settings_btn =
                    compact_icon_button(ui, Icon::Settings, self.theme).on_hover_text("Agent settings (API key)");
                if settings_btn.clicked() {
                    self.agent_settings_open = !self.agent_settings_open;
                    if self.agent_settings_open {
                        self.agent_api_key_draft.clear();
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
                    if compact_icon_button(ui, Icon::X, self.theme)
                        .on_hover_text("Cancel")
                        .clicked()
                    {
                        self.agent_settings_open = false;
                        self.agent_api_key_draft.clear();
                    }
                });
            });
            ui.add_space(4.0);
            ui.label(
                RichText::new("Enter a Groq or OpenAI API key to enable the AI provider.\nThe key is stored in the OS keychain and never written to disk in plain text.")
                    .font(font_caption())
                    .color(self.theme.text_secondary),
            );
            ui.add_space(8.0);
            let current_label = if self.agent_provider_label == "Offline draft" {
                "Not configured".to_owned()
            } else {
                format!("Active: {}", self.agent_provider_label)
            };
            ui.label(
                RichText::new(current_label)
                    .font(font_caption())
                    .color(if self.agent_provider_label == "Offline draft" {
                        self.theme.text_muted
                    } else {
                        self.theme.success
                    }),
            );
            ui.add_space(6.0);
        });
        ui.add_space(6.0);

        // API key input (password-style)
        let response = ui.add(
            TextEdit::singleline(&mut self.agent_api_key_draft)
                .hint_text("gsk_… or sk-…")
                .password(true)
                .font(egui::FontSelection::Default)
                .desired_width(f32::INFINITY),
        );
        // Allow Ctrl+Enter to save from the text field
        let save_shortcut = response.has_focus()
            && ui.input(|i| i.key_pressed(egui::Key::Enter) && DbProApp::primary_modifier_pressed(i));

        ui.add_space(6.0);

        ui.horizontal(|ui| {
            let key_non_empty = !self.agent_api_key_draft.trim().is_empty();
            let is_saving = self.agent_configure_request.is_some();
            let save_btn = compact_button_with_icon(
                ui,
                if is_saving { Icon::Loader } else { Icon::Check },
                if is_saving { "Saving…" } else { "Save key" },
                self.theme,
            );
            let save_clicked = (save_btn.clicked() || save_shortcut) && key_non_empty && !is_saving;
            if save_clicked {
                let request_id = self.task_bridge.next_request_id();
                self.agent_configure_request = Some(request_id);
                let api_key = self.agent_api_key_draft.trim().to_owned();
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
        });

        ui.add_space(8.0);
        ui.checkbox(
            &mut self.agent_auto_run_read_only,
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
                        if self.agent_provider_label == "Offline draft" {
                            badge(ui, "Preview", self.theme.surface_active, self.theme.text_secondary);
                        }
                        badge(
                            ui,
                            &self.agent_provider_label,
                            self.theme.accent_soft,
                            self.theme.accent,
                        );
                        if self.agent_auto_run_read_only {
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
                RichText::new(&self.agent_provider_detail)
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
                    && compact_button_with_icon(ui, Icon::ChartNoAxesCombined, "Explain query", self.theme).clicked()
                {
                    self.agent_input = "Explain the current SQL and its query plan".to_owned();
                    submit = true;
                }
                if !context.current_sql.trim().is_empty()
                    && compact_button_with_icon(ui, Icon::Gauge, "Optimize", self.theme).clicked()
                {
                    self.agent_input = "Optimize the current SQL and explain the trade-offs".to_owned();
                    submit = true;
                }
                if context.selected_table.is_some()
                    && compact_button_with_icon(ui, Icon::Table2, "Explain table", self.theme).clicked()
                {
                    self.agent_input = "Explain the selected table and suggest useful read-only queries".to_owned();
                    submit = true;
                }
                if context.last_error.is_some()
                    && compact_button_with_icon(ui, Icon::TriangleAlert, "Investigate error", self.theme).clicked()
                {
                    self.agent_input = "Investigate the current database error and propose a safe fix".to_owned();
                    submit = true;
                }
            });
        });
        ui.add_space(8.0);
        submit
    }

    fn draw_agent_thread(&mut self, ui: &mut egui::Ui, copy_sql: &mut Option<String>) -> bool {
        let typed_document_id = self
            .query_documents
            .get(self.active_query_document)
            .map(|document| document.id.clone());
        if typed_document_id.as_ref().is_some_and(|document_id| {
            self.agent_sessions.get(document_id).is_some_and(|session| {
                !session.messages.is_empty()
                    || !session.activities.is_empty()
                    || session.pending_confirmation.is_some()
                    || !session.streaming_text.is_empty()
            })
        }) {
            return self.draw_typed_agent_thread(ui);
        }
        let messages_height = (ui.available_height() - 86.0).max(160.0);
        let mut submit = false;
        egui::ScrollArea::vertical()
            .max_height(messages_height)
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if self.agent_messages.is_empty() {
                    ui.add_space(18.0);
                    ui.vertical_centered(|ui| {
                        ui.label(icon_text(Icon::Bot, "", self.theme.accent));
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new("Database copilot")
                                .strong()
                                .color(self.theme.text_primary),
                        );
                        ui.label(
                            RichText::new("Ask for an overview, a read-only query, or a query plan.")
                                .small()
                                .color(self.theme.text_secondary),
                        );
                    });
                    ui.add_space(18.0);
                    for suggestion in [
                        "Show me the schema overview",
                        "Count rows in customers",
                        "Explain customers query performance",
                    ] {
                        if ghost_button_with_icon(ui, Icon::WandSparkles, suggestion, self.theme).clicked() {
                            self.agent_input = suggestion.to_owned();
                            submit = true;
                        }
                    }
                } else {
                    let messages = self.agent_messages.clone();
                    for message in messages {
                        if message.role == AgentRole::User {
                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                agent_message_frame(self.theme, true).show(ui, |ui| {
                                    ui.label(RichText::new(message.content).color(self.theme.text_primary));
                                });
                            });
                        } else {
                            self.draw_agent_response(ui, message, copy_sql);
                        }
                        ui.add_space(8.0);
                    }
                }
                if self.agent_request.is_some() {
                    let mut expanded = true;
                    AgentThinking::new("Analyzing schema and generating SQL draft…", &mut expanded, self.theme)
                        .is_active(true)
                        .show(ui);
                    ui.add_space(SPACE_SM);
                }
            });
        ui.add_space(SPACE_XS);
        ui.separator();
        ui.add_space(SPACE_XS);
        submit
    }

    fn draw_typed_agent_thread(&mut self, ui: &mut egui::Ui) -> bool {
        let Some(document_id) = self
            .query_documents
            .get(self.active_query_document)
            .map(|document| document.id.clone())
        else {
            return false;
        };
        let Some(session) = self.agent_sessions.get(&document_id) else {
            return false;
        };
        let messages = session.messages.clone();
        let activities = session.activities.clone();
        let tool_results = session.tool_results.clone();
        let streaming_text = session.streaming_text.clone();
        let pending = session.pending_confirmation.clone();
        let session_state = session.state;
        let running = session.request_id.is_some() || session.active_run_id.is_some();

        let mut open_result_call_id = None;
        let mut retry = false;
        let messages_height = (ui.available_height() - 86.0).max(160.0);
        egui::ScrollArea::vertical()
            .max_height(messages_height)
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for message in messages {
                    let is_user = message.role == AgentRole::User;
                    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                        agent_message_frame(self.theme, is_user).show(ui, |ui| {
                            ui.label(RichText::new(message.content).color(self.theme.text_primary));
                        });
                    });
                    ui.add_space(SPACE_XS);
                }
                if !streaming_text.is_empty() {
                    agent_message_frame(self.theme, false).show(ui, |ui| {
                        ui.label(icon_text(Icon::Sparkles, "Agent", self.theme.accent));
                        ui.label(RichText::new(streaming_text).color(self.theme.text_primary));
                    });
                }
                for activity in activities {
                    let status_str = match activity.status {
                        super::agent_workflow_state::AgentUiActivityStatus::Running => "Running",
                        super::agent_workflow_state::AgentUiActivityStatus::AwaitingConfirmation => "Needs approval",
                        super::agent_workflow_state::AgentUiActivityStatus::Success => "Done",
                        super::agent_workflow_state::AgentUiActivityStatus::Failed => "Failed",
                        super::agent_workflow_state::AgentUiActivityStatus::Cancelled => "Cancelled",
                    };
                    let duration_str = activity.duration_ms.map(|d| format!(" · {d} ms")).unwrap_or_default();
                    ui.label(
                        RichText::new(format!("{status_str} · {}{duration_str}", activity.label))
                            .small()
                            .color(self.theme.text_secondary),
                    );
                    if let Some(call_id) = &activity.call_id {
                        if let Some(tool_result) = tool_results.get(call_id) {
                            if let db_pro_core::domain::agent::AgentToolOutput::QueryResult {
                                summary,
                                result_count,
                                ..
                            } = &tool_result.output
                            {
                                ui.horizontal(|ui| {
                                    let sample_len = summary.sample_rows.len();
                                    let total_rows = summary.row_count.unwrap_or(sample_len as u64);
                                    let is_sampled = total_rows > sample_len as u64;
                                    let rows_str = if is_sampled {
                                        format!("{sample_len} sampled of {total_rows} rows")
                                    } else {
                                        format!("{total_rows} rows")
                                    };
                                    let cols_str = format!("{} cols", summary.columns.len());
                                    let count_str = if *result_count > 1 {
                                        format!(" ({result_count} results)")
                                    } else {
                                        String::new()
                                    };
                                    ui.label(
                                        RichText::new(format!("↳ {rows_str}, {cols_str}{count_str}"))
                                            .small()
                                            .color(self.theme.text_muted),
                                    );
                                    let btn_label = if is_sampled {
                                        "Open sample in Results"
                                    } else {
                                        "Open in Results"
                                    };
                                    if compact_button_with_icon(ui, Icon::Table2, btn_label, self.theme).clicked() {
                                        open_result_call_id = Some(call_id.clone());
                                    }
                                });
                            }
                        }
                    }
                }
                if let Some(pending) = pending {
                    ui.add_space(SPACE_SM);
                    editor_frame(self.theme).show(ui, |ui| {
                        ui.label(
                            RichText::new(agent_confirmation_title(pending.kind))
                                .strong()
                                .color(self.theme.text_primary),
                        );
                        if pending.document_id != document_id {
                            ui.label(
                                RichText::new(format!("Target query: {}", pending.document_id))
                                    .small()
                                    .color(self.theme.accent),
                            );
                        }
                        if let Some(db_pro_core::domain::agent::AgentToolOutput::PatchPreview {
                            patch: _,
                            original,
                            proposed,
                            ..
                        }) = pending.preview.as_ref()
                        {
                            ui.label(
                                RichText::new("Proposed SQL change")
                                    .small()
                                    .color(self.theme.text_secondary),
                            );
                            egui::ScrollArea::vertical()
                                .max_height(120.0)
                                .id_salt("patch-diff-scroll")
                                .show(ui, |ui| {
                                    if !original.is_empty() {
                                        ui.label(
                                            RichText::new(format!("- {original}"))
                                                .monospace()
                                                .color(self.theme.danger),
                                        );
                                    }
                                    ui.label(
                                        RichText::new(format!("+ {proposed}"))
                                            .monospace()
                                            .color(self.theme.success),
                                    );
                                });
                        } else {
                            match pending.kind {
                                db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunMutation => {
                                    ui.label(
                                        RichText::new("Warning: This mutation will modify database data or schema.")
                                            .small()
                                            .color(self.theme.warning),
                                    );
                                }
                                db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunDestructive => {
                                    ui.label(
                                        RichText::new(
                                            "Caution: Destructive query may irreversibly drop or truncate data.",
                                        )
                                        .small()
                                        .color(self.theme.danger),
                                    );
                                }
                                db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunUnknown => {
                                    ui.label(
                                        RichText::new(
                                            "Classification unknown: Review the query carefully before running.",
                                        )
                                        .small()
                                        .color(self.theme.text_secondary),
                                    );
                                }
                                _ => {
                                    ui.label(
                                        RichText::new("Review the requested action before continuing.")
                                            .small()
                                            .color(self.theme.text_secondary),
                                    );
                                }
                            }
                        }
                        ui.horizontal(|ui| {
                            let approve_label = match pending.kind {
                                db_pro_core::domain::agent_workflow::AgentConfirmationKind::ApplyPatch => {
                                    "Apply Change"
                                }
                                db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunReadOnly => "Run Query",
                                db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunMutation => {
                                    "Run Mutation"
                                }
                                db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunDestructive => {
                                    "Execute Destructive Query"
                                }
                                db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunUnknown => {
                                    "Run Unclassified Query"
                                }
                            };
                            if primary_button_with_icon(ui, Icon::Check, approve_label, self.theme).clicked() {
                                self.agent_confirmation_action(true);
                            }
                            if secondary_button_with_icon(ui, Icon::X, "Reject", self.theme).clicked() {
                                self.agent_confirmation_action(false);
                            }
                        });
                    });
                }
                if session_state == db_pro_core::domain::agent::AgentSessionState::Failed {
                    ui.add_space(SPACE_SM);
                    ui.horizontal(|ui| {
                        if compact_button_with_icon(ui, Icon::RotateCcw, "Retry", self.theme).clicked() {
                            retry = true;
                        }
                    });
                }
                if running {
                    ui.add_space(SPACE_SM);
                    AgentThinking::new("Working in this query…", &mut true, self.theme)
                        .is_active(true)
                        .show(ui);
                }
            });
        if let Some(call_id) = open_result_call_id {
            self.open_agent_result_in_workspace(&call_id);
        }
        if retry {
            self.retry_agent_run();
        }
        ui.add_space(SPACE_XS);
        ui.separator();
        ui.add_space(SPACE_XS);
        false
    }

    fn draw_agent_response(&mut self, ui: &mut egui::Ui, message: AgentMessage, copy_sql: &mut Option<String>) {
        agent_message_frame(self.theme, false).show(ui, |ui| {
            ui.label(icon_text(Icon::Sparkles, "Agent", self.theme.accent));
            ui.add_space(SPACE_XS);
            ui.label(RichText::new(message.content).color(self.theme.text_primary));
            if message.requires_confirmation {
                ui.add_space(SPACE_SM);
                let sql_preview = message.sql.as_deref().unwrap_or("Pending database mutation");
                let approval = ExecutionApproval::new(
                    "Mutation Review Required",
                    "This query will modify data or schema. Review carefully before running.",
                    sql_preview,
                    RiskLevel::High,
                    self.theme,
                );
                let action = approval.show(ui);
                if let Some(ExecutionApprovalAction::Run) = action {
                    if let Some(ref sql) = message.sql {
                        self.insert_agent_sql(sql);
                    }
                }
            } else if let Some(sql) = message.sql {
                ui.add_space(SPACE_SM);
                editor_frame(self.theme).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        section_label(ui, "SQL draft", self.theme);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if compact_icon_button(ui, Icon::Copy, self.theme)
                                .on_hover_text("Copy SQL")
                                .clicked()
                            {
                                *copy_sql = Some(sql.clone());
                            }
                        });
                    });
                    ui.add_space(SPACE_XS);
                    ui.label(RichText::new(sql.as_str()).monospace().color(self.theme.code_keyword));
                });
                ui.add_space(SPACE_SM);
                if secondary_button_with_icon(ui, Icon::ArrowUp, "Insert into Query", self.theme).clicked() {
                    self.insert_agent_sql(&sql);
                }
                if self.connected
                    && self.active_connection_id.is_some()
                    && secondary_button_with_icon(ui, Icon::Play, "Run read-only", self.theme).clicked()
                {
                    self.run_agent_read_only(&sql);
                }
            }
        });
    }

    fn draw_agent_composer(&mut self, ui: &mut egui::Ui, submit: &mut bool) {
        let active_mode = self
            .query_documents
            .get(self.active_query_document)
            .and_then(|document| self.agent_sessions.get(&document.id))
            .map(|session| session.mode);
        let composer_mode = match active_mode {
            Some(db_pro_core::domain::agent::AgentMode::Ask) => AgentMode::Chat,
            Some(db_pro_core::domain::agent::AgentMode::Edit) => AgentMode::Plan,
            Some(db_pro_core::domain::agent::AgentMode::Agent) => AgentMode::Code,
            None => AgentMode::Code,
        };
        let is_generating = self
            .query_documents
            .get(self.active_query_document)
            .and_then(|document| self.agent_sessions.get(&document.id))
            .is_some_and(|session| session.active_run_id.is_some() || session.request_id.is_some());
        let action = AgentComposer::new(
            &mut self.agent_input,
            &self.agent_provider_label,
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

fn agent_confirmation_title(kind: db_pro_core::domain::agent_workflow::AgentConfirmationKind) -> &'static str {
    match kind {
        db_pro_core::domain::agent_workflow::AgentConfirmationKind::ApplyPatch => "Apply Agent change?",
        db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunReadOnly => "Run read-only query?",
        db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunMutation => "Run mutation?",
        db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunDestructive => "Execute destructive query?",
        db_pro_core::domain::agent_workflow::AgentConfirmationKind::RunUnknown => "Run unclassified query?",
    }
}
