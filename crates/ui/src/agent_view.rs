use super::*;

impl DbProApp {
    pub(super) fn draw_agent_panel(&mut self, ctx: &egui::Context) {
        let mut submit = false;
        let mut copy_sql = None;
        egui::SidePanel::right("agent_panel")
            .default_width(360.0)
            .min_width(300.0)
            .max_width(380.0)
            .frame(card_frame(self.theme))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(icon_text(Icon::Sparkles, "Agent", self.theme.accent));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if !self.agent_messages.is_empty()
                            && compact_icon_button(ui, Icon::RotateCcw, self.theme).clicked()
                        {
                            self.agent_messages.clear();
                        }
                        if compact_icon_button(ui, Icon::X, self.theme).clicked() {
                            self.set_agent_open(false, ctx);
                        }
                    });
                });
                ui.add_space(6.0);
                let context = self.agent_context();
                toolbar_frame(self.theme).show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        badge(ui, "Preview", self.theme.surface_active, self.theme.text_secondary);
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
                    });
                    ui.label(
                        RichText::new(&self.agent_provider_detail)
                            .small()
                            .color(self.theme.text_muted),
                    );
                });
                ui.add_space(8.0);
                let messages_height = (ui.available_height() - 86.0).max(160.0);
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
                                let is_user = message.role == AgentRole::User;
                                if is_user {
                                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                        card_frame(self.theme).show(ui, |ui| {
                                            ui.label(RichText::new(message.content).color(self.theme.text_primary));
                                        });
                                    });
                                } else {
                                    card_frame(self.theme).show(ui, |ui| {
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
                                                        if compact_icon_button(ui, Icon::Copy, self.theme).clicked() {
                                                            copy_sql = Some(sql.clone());
                                                        }
                                                    });
                                                });
                                                ui.add_space(4.0);
                                                ui.label(
                                                    RichText::new(sql.as_str())
                                                        .monospace()
                                                        .color(self.theme.code_keyword),
                                                );
                                            });
                                            ui.add_space(6.0);
                                            if secondary_button_with_icon(
                                                ui,
                                                Icon::ArrowUp,
                                                "Insert into Query",
                                                self.theme,
                                            )
                                            .clicked()
                                            {
                                                self.insert_agent_sql(&sql);
                                            }
                                            if !message.requires_confirmation
                                                && self.connected
                                                && self.active_connection_id.is_some()
                                                && secondary_button_with_icon(
                                                    ui,
                                                    Icon::Play,
                                                    "Run read-only",
                                                    self.theme,
                                                )
                                                .clicked()
                                            {
                                                self.run_agent_read_only(&sql);
                                            }
                                        }
                                    });
                                }
                                ui.add_space(8.0);
                            }
                        }
                        if self.agent_request.is_some() {
                            card_frame(self.theme).show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    if self.reduce_motion {
                                        ui.label(icon_text(Icon::LoaderCircle, "", self.theme.accent));
                                    } else {
                                        ui.spinner();
                                    }
                                    ui.label(RichText::new("Thinking with Codex…").color(self.theme.text_secondary));
                                });
                            });
                            ui.add_space(8.0);
                        }
                    });
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);
                toolbar_frame(self.theme).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let input_width = (ui.available_width() - 34.0).max(120.0);
                        let response = input(
                            ui,
                            &mut self.agent_input,
                            "Ask about schema or draft SQL…",
                            input_width,
                            self.theme,
                        );
                        let send =
                            compact_icon_button_enabled(ui, Icon::Send, self.agent_request.is_none(), self.theme);
                        if send.clicked()
                            || (self.agent_request.is_none()
                                && response.has_focus()
                                && ui.input(|input| input.key_pressed(egui::Key::Enter) && input.modifiers.command))
                        {
                            submit = true;
                        }
                    });
                    ui.label(
                        RichText::new(format!(
                            "{} · Cmd/Ctrl+Enter to send · writes stay unexecuted",
                            self.agent_provider_label
                        ))
                        .small()
                        .color(self.theme.text_muted),
                    );
                });
            });
        if submit {
            self.submit_agent_prompt();
        }
        if let Some(sql) = copy_sql {
            ctx.output_mut(|output| output.copied_text = sql);
            self.copy_status = "Agent SQL copied".to_owned();
        }
    }
}
