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
        ui.horizontal(|ui| {
            ui.label(icon_text(Icon::Sparkles, "Agent", self.theme.accent));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if !self.agent_messages.is_empty()
                    && compact_icon_button(ui, Icon::RotateCcw, self.theme)
                        .on_hover_text("Clear conversation")
                        .clicked()
                {
                    self.agent_messages.clear();
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
        let is_generating = self.agent_request.is_some();
        let action = AgentComposer::new(
            &mut self.agent_input,
            &self.agent_provider_label,
            AgentMode::Code,
            self.theme,
        )
        .is_generating(is_generating)
        .show(ui);

        if let Some(AgentComposerAction::Submit) = action {
            *submit = true;
        }
    }
}
