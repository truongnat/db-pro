use super::*;
use egui::Sense;

impl DbProApp {
    pub(super) fn draw_topbar(&mut self, ctx: &egui::Context) {
        let connection_name = self.active_connection_name().to_owned();
        let driver = self.active_driver().to_owned();
        let has_connection = self.connection.lifecycle.active_connection_id().is_some();
        let (connection_icon, connection_color) = self
            .active_connection()
            .map(|connection| self.connection_indicator(connection))
            .unwrap_or((Icon::Circle, self.theme.warning));
        let modifier = Self::primary_modifier_label();
        TopBottomPanel::top("topbar")
            .exact_height(38.0)
            .frame(egui::Frame {
                fill: self.theme.surface_app,
                inner_margin: egui::Margin::symmetric(SPACE_MD, 4.0),
                stroke: egui::Stroke::new(STROKE_THIN, self.theme.border_subtle),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                ui.horizontal_centered(|ui| {
                    let toggle_tooltip = if self.workspace.sidebar_open {
                        format!("Collapse Sidebar ({}B)", modifier)
                    } else {
                        format!("Expand Sidebar ({}B)", modifier)
                    };
                    let toggle_icon = if self.workspace.sidebar_open {
                        Icon::PanelLeftClose
                    } else {
                        Icon::PanelLeft
                    };
                    if Button::new(self.theme)
                        .icon(toggle_icon)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .tooltip(toggle_tooltip)
                        .show(ui)
                        .clicked()
                    {
                        self.workspace.sidebar_open = !self.workspace.sidebar_open;
                    }
                    ui.add_space(2.0);

                    let can_go_back = self.query.session.active_document_index > 0;
                    let can_go_forward =
                        self.query.session.active_document_index + 1 < self.query.session.documents.len();
                    if Button::new(self.theme)
                        .icon(Icon::ArrowLeft)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .enabled(can_go_back)
                        .tooltip("Previous Document")
                        .show(ui)
                        .clicked()
                        && self.query.session.active_document_index > 0
                    {
                        self.switch_query_document(self.query.session.active_document_index - 1);
                    }
                    if Button::new(self.theme)
                        .icon(Icon::ArrowRight)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .enabled(can_go_forward)
                        .tooltip("Next Document")
                        .show(ui)
                        .clicked()
                        && self.query.session.active_document_index + 1 < self.query.session.documents.len()
                    {
                        self.switch_query_document(self.query.session.active_document_index + 1);
                    }

                    ui.add_space(SPACE_SM);
                    ui.label(RichText::new("│").font(font_caption()).color(self.theme.border_subtle));
                    ui.add_space(SPACE_SM);

                    if has_connection {
                        ui.horizontal(|ui| {
                            ui.label(icon_text(connection_icon, "", connection_color));
                            let display_name = crate::components::truncate_ellipsis(&connection_name, 22);
                            let resp = ui.label(
                                RichText::new(display_name)
                                    .font(font_caption())
                                    .strong()
                                    .color(self.theme.text_primary),
                            );
                            if connection_name.len() > 22 {
                                resp.on_hover_text(&connection_name);
                            }
                            let driver_tag = if driver.to_ascii_lowercase().contains("sqlite") {
                                "SQLite"
                            } else {
                                "PostgreSQL"
                            };
                            Badge::new(driver_tag, self.theme)
                                .variant(BadgeVariant::Secondary)
                                .compact(true)
                                .show(ui);
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label(icon_text(Icon::Database, "", self.theme.accent));
                            ui.label(
                                RichText::new("DB PRO")
                                    .font(font_caption())
                                    .strong()
                                    .color(self.theme.text_primary),
                            );
                        });
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if Button::new(self.theme)
                            .icon(Icon::Command)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip(format!("Command Palette ({}⇧P)", modifier))
                            .show(ui)
                            .clicked()
                        {
                            self.palette.open(PaletteMode::Commands);
                        }
                        if Button::new(self.theme)
                            .icon(Icon::Palette)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip("Component Gallery (UI Design System)")
                            .show(ui)
                            .clicked()
                        {
                            self.workspace.active_tab = WorkspaceTab::ComponentGallery;
                        }
                        let agent_tooltip = if self.workspace.agent_open {
                            "Close Copilot Panel"
                        } else {
                            "Open Copilot Assistant"
                        };
                        if Button::new(self.theme)
                            .icon(Icon::Bot)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip(agent_tooltip)
                            .show(ui)
                            .clicked()
                        {
                            self.set_agent_open(!self.workspace.agent_open, ctx);
                        }
                        let theme_icon = if self.preferences.dark_mode {
                            Icon::Sun
                        } else {
                            Icon::Moon
                        };
                        let theme_tooltip = if self.preferences.dark_mode {
                            "Switch to Light Theme"
                        } else {
                            "Switch to Dark Theme"
                        };
                        if Button::new(self.theme)
                            .icon(theme_icon)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip(theme_tooltip)
                            .show(ui)
                            .clicked()
                        {
                            self.preferences.dark_mode = !self.preferences.dark_mode;
                        }

                        ui.add_space(SPACE_SM);
                        let search_width = (ui.available_width() - 32.0).clamp(180.0, 360.0);
                        let (rect, resp) = ui.allocate_exact_size(egui::vec2(search_width, 26.0), Sense::click());
                        let hovered = resp.hovered();
                        let bg = if hovered {
                            self.theme.surface_hover
                        } else {
                            self.theme.surface_panel
                        };
                        let border = if hovered {
                            self.theme.border_strong
                        } else {
                            self.theme.border_subtle
                        };
                        ui.painter().rect(
                            rect,
                            egui::Rounding::same(RADIUS_MD),
                            bg,
                            egui::Stroke::new(1.0, border),
                        );
                        let search_icon_pos = egui::pos2(rect.left() + 8.0, rect.center().y);
                        ui.painter().text(
                            search_icon_pos,
                            egui::Align2::LEFT_CENTER,
                            char::from(Icon::Search).to_string(),
                            egui::FontId::new(12.0, egui::FontFamily::Name("lucide".into())),
                            self.theme.text_muted,
                        );
                        ui.painter().text(
                            egui::pos2(rect.left() + 26.0, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            "Search commands, tables, schemas...",
                            egui::FontId::proportional(11.5),
                            self.theme.text_muted,
                        );
                        ui.painter().text(
                            egui::pos2(rect.right() - 8.0, rect.center().y),
                            egui::Align2::RIGHT_CENTER,
                            format!("{modifier}P"),
                            egui::FontId::monospace(10.0),
                            self.theme.text_muted,
                        );
                        if resp.clicked() {
                            self.palette.open(PaletteMode::QuickOpen);
                        }
                    });
                });
            });
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
