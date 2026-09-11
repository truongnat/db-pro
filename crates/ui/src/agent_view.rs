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
                self.draw_agent_header(ui, ctx);
                ui.add_space(6.0);
                let context = self.agent_context();
                self.draw_agent_context(ui, &context);
                ui.add_space(8.0);
                submit |= self.draw_agent_context_actions(ui, &context);
                submit |= self.draw_agent_thread(ui, &mut copy_sql);
                self.draw_agent_composer(ui, &mut submit);
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
            });
        });
    }

    fn draw_agent_context(&self, ui: &mut egui::Ui, context: &AgentContext) {
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                if self.agent_provider_label == "Offline draft" {
                    badge(ui, "Preview", self.theme.surface_active, self.theme.text_secondary);
                }
                badge(
                    ui,
                    &self.agent_provider_label,
                    self.theme.accent_soft,
                    self.theme.accent,
                );
                badge(
                    ui,
                    context.connection_name.as_deref().unwrap_or("No connection"),
                    self.theme.accent_soft,
                    self.theme.accent,
                );
                badge(
                    ui,
                    &context.driver,
                    self.theme.surface_active,
                    self.theme.text_secondary,
                );
                badge(
                    ui,
                    &format!("{} tables", context.tables.len()),
                    self.theme.surface_active,
                    self.theme.text_secondary,
                );
                if let Some(schema) = context.schema.as_deref() {
                    badge(ui, schema, self.theme.surface_active, self.theme.text_secondary);
                }
                if let Some(table) = context.selected_table.as_deref() {
                    badge(ui, table, self.theme.accent_soft, self.theme.accent);
                }
                if context.explain_plan.is_some() {
                    badge(ui, "EXPLAIN", self.theme.accent_soft, self.theme.accent);
                }
            });
            ui.label(
                RichText::new(&self.agent_provider_detail)
                    .small()
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
                    agent_message_frame(self.theme, false).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if self.reduce_motion {
                                ui.label(icon_text(Icon::LoaderCircle, "", self.theme.accent));
                            } else {
                                ui.spinner();
                            }
                            ui.label(RichText::new("Thinking with Agent…").color(self.theme.text_secondary));
                        });
                    });
                    ui.add_space(8.0);
                }
            });
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);
        submit
    }

    fn draw_agent_response(&mut self, ui: &mut egui::Ui, message: AgentMessage, copy_sql: &mut Option<String>) {
        agent_message_frame(self.theme, false).show(ui, |ui| {
            ui.label(icon_text(Icon::Sparkles, "Agent", self.theme.accent));
            ui.add_space(4.0);
            ui.label(RichText::new(message.content).color(self.theme.text_primary));
            if message.requires_confirmation {
                ui.add_space(8.0);
                ui.label(icon_text(
                    Icon::TriangleAlert,
                    "Review carefully before running",
                    self.theme.warning,
                ));
            }
            if let Some(sql) = message.sql {
                ui.add_space(8.0);
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
                    ui.add_space(4.0);
                    ui.label(RichText::new(sql.as_str()).monospace().color(self.theme.code_keyword));
                });
                ui.add_space(6.0);
                if secondary_button_with_icon(ui, Icon::ArrowUp, "Insert into Query", self.theme).clicked() {
                    self.insert_agent_sql(&sql);
                }
                if !message.requires_confirmation
                    && self.connected
                    && self.active_connection_id.is_some()
                    && secondary_button_with_icon(ui, Icon::Play, "Run read-only", self.theme).clicked()
                {
                    self.run_agent_read_only(&sql);
                }
            }
        });
    }

    fn draw_agent_composer(&mut self, ui: &mut egui::Ui, submit: &mut bool) {
        toolbar_frame(self.theme).show(ui, |ui| {
            ui.horizontal(|ui| {
                let input_width = (ui.available_width() - 52.0).max(120.0);
                let ready = self.agent_request.is_none();
                let response = ui
                    .add_enabled_ui(ready, |ui| {
                        input(
                            ui,
                            &mut self.agent_input,
                            if ready {
                                "Ask about schema or draft SQL…"
                            } else {
                                "Waiting for Agent response…"
                            },
                            input_width,
                            self.theme,
                        )
                    })
                    .inner;
                let send = compact_icon_button_enabled(ui, Icon::Send, ready, self.theme);
                if send.clicked()
                    || (ready
                        && response.has_focus()
                        && ui.input(|input| {
                            input.key_pressed(egui::Key::Enter) && Self::primary_modifier_pressed(input)
                        }))
                {
                    *submit = true;
                }
            });
            let composer_hint = if self.agent_request.is_some() {
                "Waiting for the current response"
            } else {
                "Cmd/Ctrl+Enter to send · writes stay unexecuted"
            };
            ui.label(
                RichText::new(format!("{} · {composer_hint}", self.agent_provider_label))
                    .small()
                    .color(self.theme.text_muted),
            );
        });
    }
}
