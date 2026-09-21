use super::*;
impl DbProApp {
    pub(super) fn draw_topbar(&mut self, ctx: &egui::Context) {
        let connection_name = self.active_connection_name().to_owned();
        let driver = self.active_driver().to_owned();
        let has_connection = self.connection.lifecycle.active_connection_id().is_some();
        let (connection_icon, connection_color) = self
            .active_connection()
            .map(|connection| self.connection_indicator(connection))
            .unwrap_or((Icon::Circle, self.theme.warning));
        let actions = shell_topbar_view::ShellTopbarContext {
            theme: self.theme,
            sidebar_open: self.workspace.sidebar_open,
            active_document_index: self.query.session.active_document_index,
            document_count: self.query.session.documents.len(),
            has_connection,
            connection_name: &connection_name,
            connection_icon,
            connection_color,
            driver: &driver,
            agent_open: self.workspace.agent_open,
            dark_mode: self.preferences.dark_mode,
        }
        .draw(ctx);
        for action in actions {
            match action {
                shell_topbar_view::ShellTopbarAction::ToggleSidebar => {
                    self.workspace.sidebar_open = !self.workspace.sidebar_open;
                }
                shell_topbar_view::ShellTopbarAction::PreviousDocument => {
                    self.switch_query_document(self.query.session.active_document_index.saturating_sub(1));
                }
                shell_topbar_view::ShellTopbarAction::NextDocument => {
                    self.switch_query_document(self.query.session.active_document_index + 1);
                }
                shell_topbar_view::ShellTopbarAction::OpenCommandPalette => {
                    self.palette.open(PaletteMode::Commands);
                }
                shell_topbar_view::ShellTopbarAction::OpenComponentGallery => {
                    self.workspace.active_tab = WorkspaceTab::ComponentGallery;
                }
                shell_topbar_view::ShellTopbarAction::ToggleAgent => {
                    self.set_agent_open(!self.workspace.agent_open, ctx);
                }
                shell_topbar_view::ShellTopbarAction::ToggleTheme => {
                    self.preferences.dark_mode = !self.preferences.dark_mode;
                }
                shell_topbar_view::ShellTopbarAction::OpenQuickOpen => {
                    self.palette.open(PaletteMode::QuickOpen);
                }
            }
        }
    }

    pub(super) fn draw_statusbar(&mut self, ctx: &egui::Context) {
        let (icon, color, label) = self.statusbar_state();
        let runtime_status = self.runtime_status();
        TopBottomPanel::bottom("statusbar")
            .exact_height(28.0)
            .frame(egui::Frame {
                fill: self.theme.surface_panel,
                inner_margin: egui::Margin::symmetric(SPACE_MD, SPACE_XXS),
                stroke: egui::Stroke::new(STROKE_THIN, self.theme.border_subtle),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                ui.horizontal_centered(|ui| {
                    ui.add_space(SPACE_SM);
                    ui.label(icon_text(icon, "", color));
                    ui.label(
                        RichText::new(label)
                            .font(font_caption())
                            .color(self.theme.text_secondary),
                    );
                    ui.separator();
                    if self.connection.lifecycle.is_connected() {
                        ui.label(
                            RichText::new(self.active_connection_name())
                                .font(font_caption())
                                .color(self.theme.text_secondary),
                        );
                    }
                    ui.label(
                        RichText::new(self.active_driver())
                            .font(font_caption())
                            .color(self.theme.text_muted),
                    );
                    if let Some(connection) = self.active_connection() {
                        ui.label(
                            RichText::new(&connection.database)
                                .font(font_caption())
                                .color(self.theme.text_muted),
                        );
                        ui.label(
                            RichText::new(self.active_schema())
                                .font(font_caption())
                                .color(self.theme.text_muted),
                        );
                    }
                    if let Some(result) = self
                        .query
                        .session
                        .active_result()
                        .or(self.table.data_query.result.as_ref())
                    {
                        ui.label(
                            RichText::new(format!("{} ms", result.duration_ms))
                                .font(font_mono_sm())
                                .color(self.theme.text_muted),
                        );
                    }
                    if let Some((message, message_color)) = runtime_status {
                        ui.separator();
                        ui.add_sized(
                            [260.0, 18.0],
                            egui::Label::new(RichText::new(message).font(font_caption()).color(message_color)),
                        );
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if Button::new(self.theme)
                            .icon(Icon::PanelBottom)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip("Toggle output panel")
                            .show(ui)
                            .clicked()
                        {
                            self.workspace.bottom_panel_open = !self.workspace.bottom_panel_open;
                        }
                        if self.shows_editor_status() {
                            ui.label(RichText::new("UTF-8").font(font_mono_sm()).color(self.theme.text_muted));
                            ui.label(
                                RichText::new(format!(
                                    "Ln {}, Col {}",
                                    self.query.editor.query_cursor_line, self.query.editor.query_cursor_column
                                ))
                                .font(font_mono_sm())
                                .color(self.theme.text_muted),
                            );
                        } else {
                            ui.label(
                                RichText::new(self.statusbar_context_label())
                                    .font(font_caption())
                                    .color(self.theme.text_muted),
                            );
                        }
                    });
                });
            });
    }

    pub(super) fn draw_output_panel(&mut self, ctx: &egui::Context) {
        let active_result = self
            .query
            .session
            .active_result()
            .or(self.table.data_query.result.as_ref());
        shell_output_panel_view::ShellOutputPanelContext {
            theme: self.theme,
            workspace: &mut self.workspace,
            output: &mut self.query.output,
            session: &self.query.session,
            editor: &self.query.editor,
            active_result,
        }
        .draw(ctx);
    }
}
