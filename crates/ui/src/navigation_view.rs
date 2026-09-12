use super::*;
use egui::{vec2, Align2, Color32, FontFamily, FontId, Margin, Pos2, Rect, Rounding, Sense, Stroke};

impl DbProApp {
    pub(super) fn draw_topbar(&mut self, ctx: &egui::Context) {
        let connection_name = self.active_connection_name().to_owned();
        let driver = self.active_driver().to_owned();
        let has_connection = self.active_connection_id.is_some();
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
                    // 0. Toggle Sidebar Button (Codex style)
                    let toggle_tooltip = if self.sidebar_open {
                        format!("Collapse Sidebar ({}B)", modifier)
                    } else {
                        format!("Expand Sidebar ({}B)", modifier)
                    };
                    let toggle_icon = if self.sidebar_open {
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
                        self.sidebar_open = !self.sidebar_open;
                    }
                    ui.add_space(2.0);

                    // 1. History Navigation (Back / Forward)
                    let can_go_back = self.active_query_document > 0;
                    let can_go_forward = self.active_query_document + 1 < self.query_documents.len();
                    if Button::new(self.theme)
                        .icon(Icon::ArrowLeft)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .enabled(can_go_back)
                        .tooltip("Previous Document")
                        .show(ui)
                        .clicked()
                        && self.active_query_document > 0
                    {
                        self.switch_query_document(self.active_query_document - 1);
                    }
                    if Button::new(self.theme)
                        .icon(Icon::ArrowRight)
                        .variant(ButtonVariant::Ghost)
                        .size(ButtonSize::IconSm)
                        .enabled(can_go_forward)
                        .tooltip("Next Document")
                        .show(ui)
                        .clicked()
                        && self.active_query_document + 1 < self.query_documents.len()
                    {
                        self.switch_query_document(self.active_query_document + 1);
                    }

                    ui.add_space(SPACE_SM);
                    ui.label(RichText::new("│").font(font_caption()).color(self.theme.border_subtle));
                    ui.add_space(SPACE_SM);

                    // 2. Active Context Breadcrumb
                    if has_connection {
                        ui.horizontal(|ui| {
                            ui.label(icon_text(connection_icon, "", connection_color));
                            ui.label(
                                RichText::new(&connection_name)
                                    .font(font_caption())
                                    .strong()
                                    .color(self.theme.text_primary),
                            );
                            let driver_tag = if driver.to_ascii_lowercase().contains("sqlite") {
                                "SQLite"
                            } else {
                                "PostgreSQL"
                            };
                            egui::Frame {
                                fill: self.theme.surface_panel,
                                inner_margin: egui::Margin::symmetric(5.0, 1.0),
                                rounding: egui::Rounding::same(3.0),
                                stroke: egui::Stroke::new(1.0, self.theme.border_subtle),
                                ..Default::default()
                            }
                            .show(ui, |ui| {
                                ui.label(RichText::new(driver_tag).size(9.5).color(self.theme.text_secondary));
                            });
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

                    // 3. Right actions + Center Search Box
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if Button::new(self.theme)
                            .icon(Icon::Command)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip(format!("Command Palette ({modifier}⇧P)"))
                            .show(ui)
                            .clicked()
                        {
                            self.open_palette(PaletteMode::Commands);
                        }
                        if Button::new(self.theme)
                            .icon(Icon::Palette)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip("Component Gallery (UI Design System)")
                            .show(ui)
                            .clicked()
                        {
                            self.active_tab = WorkspaceTab::ComponentGallery;
                        }
                        let agent_tooltip = if self.agent_open {
                            "Close Copilot Panel (⌘I)"
                        } else {
                            "Open Copilot Assistant (⌘I)"
                        };
                        if Button::new(self.theme)
                            .icon(Icon::Bot)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip(agent_tooltip)
                            .show(ui)
                            .clicked()
                        {
                            self.set_agent_open(!self.agent_open, ctx);
                        }
                        let theme_icon = if self.dark_mode { Icon::Sun } else { Icon::Moon };
                        let theme_tooltip = if self.dark_mode {
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
                            self.dark_mode = !self.dark_mode;
                        }

                        ui.add_space(SPACE_SM);

                        // 4. Center Command Search Box
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

                        let text_pos = egui::pos2(rect.left() + 26.0, rect.center().y);
                        ui.painter().text(
                            text_pos,
                            egui::Align2::LEFT_CENTER,
                            "Search commands, tables, schemas...",
                            egui::FontId::proportional(11.5),
                            self.theme.text_muted,
                        );

                        let kbd_pos = egui::pos2(rect.right() - 8.0, rect.center().y);
                        let kbd_text = format!("{modifier}P");
                        ui.painter().text(
                            kbd_pos,
                            egui::Align2::RIGHT_CENTER,
                            &kbd_text,
                            egui::FontId::monospace(10.0),
                            self.theme.text_muted,
                        );

                        if resp.clicked() {
                            self.open_palette(PaletteMode::QuickOpen);
                        }
                    });
                });
            });
    }

    pub(super) fn draw_statusbar(&mut self, ctx: &egui::Context) {
        let (icon, color, label) = self.statusbar_state();
        let show_runtime_message = self.has_runtime_error();
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
                    if self.connected {
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
                    if let Some(result) = self.query_result.as_ref().or(self.table_data_result.as_ref()) {
                        ui.label(
                            RichText::new(format!("{} ms", result.duration_ms))
                                .font(font_mono_sm())
                                .color(self.theme.text_muted),
                        );
                    }
                    if show_runtime_message {
                        ui.separator();
                        ui.add_sized(
                            [260.0, 18.0],
                            egui::Label::new(
                                RichText::new(self.runtime_message.as_str())
                                    .font(font_caption())
                                    .color(self.theme.danger),
                            ),
                        );
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if compact_icon_button(ui, Icon::PanelBottom, self.theme)
                            .on_hover_text("Toggle output panel")
                            .clicked()
                        {
                            self.bottom_panel_open = !self.bottom_panel_open;
                        }
                        if self.shows_editor_status() {
                            ui.label(RichText::new("UTF-8").font(font_mono_sm()).color(self.theme.text_muted));
                            ui.label(
                                RichText::new(format!(
                                    "Ln {}, Col {}",
                                    self.query_cursor_line, self.query_cursor_column
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
        if !self.bottom_panel_open {
            return;
        }
        let height = self.bottom_panel_height;
        let response = TopBottomPanel::bottom("output_panel")
            .resizable(true)
            .default_height(height)
            .height_range(OUTPUT_MIN_HEIGHT..=OUTPUT_MAX_HEIGHT)
            .frame(panel_frame(self.theme))
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                ui.horizontal(|ui| {
                    section_label(ui, "OUTPUT", self.theme);
                    for (tab, label) in [
                        (OutputTab::Results, "Results"),
                        (OutputTab::Messages, "Messages"),
                        (OutputTab::Explain, "Explain"),
                        (OutputTab::History, "History"),
                    ] {
                        if tab_frame(self.theme, self.output_tab == tab)
                            .show(ui, |ui| ui.selectable_label(self.output_tab == tab, label))
                            .inner
                            .clicked()
                        {
                            self.output_tab = tab;
                        }
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if compact_icon_button(ui, Icon::X, self.theme)
                            .on_hover_text("Close output")
                            .clicked()
                        {
                            self.bottom_panel_open = false;
                        }
                    });
                });
                ui.separator();
                match self.output_tab {
                    OutputTab::Results => {
                        let result = self.query_result.as_ref().or(self.table_data_result.as_ref());
                        ui.label(
                            RichText::new(
                                result
                                    .map(|value| format!("{} rows · {} ms", value.row_count, value.duration_ms))
                                    .unwrap_or_else(|| "No result".to_owned()),
                            )
                            .small()
                            .color(self.theme.text_secondary),
                        );
                    }
                    OutputTab::Messages => {
                        for message in self.query_messages.iter().rev().take(8) {
                            ui.label(RichText::new(message).small().color(self.theme.text_secondary));
                        }
                    }
                    OutputTab::Explain => {
                        if let Some(plan) = self.explain_plan.as_deref() {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.label(RichText::new(plan).monospace().small().color(self.theme.text_secondary));
                            });
                        } else {
                            ui.label(
                                RichText::new("Run Explain to inspect the query plan")
                                    .small()
                                    .color(self.theme.text_muted),
                            );
                        }
                    }
                    OutputTab::History => {
                        for query in self.query_history.iter().rev().take(8) {
                            ui.label(
                                RichText::new(query)
                                    .monospace()
                                    .small()
                                    .color(self.theme.text_secondary),
                            );
                        }
                    }
                }
            });
        self.bottom_panel_height = response
            .response
            .rect
            .height()
            .clamp(OUTPUT_MIN_HEIGHT, OUTPUT_MAX_HEIGHT);
    }

    pub(super) fn draw_activity_bar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("activity_bar")
            .resizable(false)
            .exact_width(48.0)
            .frame(activity_bar_frame(self.theme))
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                ui.vertical_centered(|ui| {
                    ui.add_space(SPACE_SM);
                    for (activity, icon, hint) in [
                        (Some(Activity::Explorer), Icon::Database, "Explorer"),
                        (Some(Activity::Queries), Icon::FileCode2, "Queries"),
                        (Some(Activity::History), Icon::History, "History"),
                        (Some(Activity::Transfers), Icon::Upload, "Transfers"),
                        (Some(Activity::Monitor), Icon::Gauge, "Monitor"),
                        (Some(Activity::Diagram), Icon::ArrowRightLeft, "ER diagram"),
                        (None, Icon::Bot, "Agent (Copilot)"),
                    ] {
                        let active = activity.is_some_and(|value| self.activity == value)
                            || (hint == "Queries" && self.active_tab == WorkspaceTab::Query)
                            || (hint == "Agent (Copilot)" && self.agent_open);
                        let response = icon_button(ui, icon, active, self.theme);
                        if response.on_hover_text(hint).clicked() {
                            match (activity, hint) {
                                (Some(value), _) => {
                                    self.activity = value;
                                    self.sidebar_open = true;
                                    if value == Activity::Queries {
                                        self.active_tab = WorkspaceTab::Query;
                                    } else if value == Activity::Diagram {
                                        self.active_tab = WorkspaceTab::Diagram;
                                    }
                                }
                                (None, "Agent (Copilot)") => self.set_agent_open(!self.agent_open, ctx),
                                _ => {}
                            }
                        }
                        ui.add_space(SPACE_XS);
                    }
                    ui.add_space((ui.available_height() - 44.0).max(0.0));
                    let settings = icon_button(ui, Icon::Settings2, self.activity == Activity::Settings, self.theme);
                    if settings.on_hover_text("Settings").clicked() {
                        self.activity = Activity::Settings;
                        self.sidebar_open = true;
                    }
                    ui.add_space(SPACE_SM);
                });
            });
    }

    pub(super) fn draw_sidebar(&mut self, ctx: &egui::Context) {
        let sidebar_width = self.sidebar_width;
        let response = egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(sidebar_width.max(248.0))
            .width_range(232.0..=SIDEBAR_MAX_WIDTH)
            .frame(sidebar_frame(self.theme))
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());

                // ── 1. Codex-style Header Row: Workspace Selector + Action Icons ──
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let active_name = if self.active_connection_id.is_some() {
                        self.active_connection_name()
                    } else {
                        "DB Pro"
                    };

                    // Workspace / Connection Dropdown Selector (e.g. "Codex ⌵")
                    let selector_resp = egui::Frame {
                        fill: Color32::TRANSPARENT,
                        rounding: Rounding::same(6.0),
                        inner_margin: Margin::symmetric(4.0, 3.0),
                        ..Default::default()
                    }
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
                            ui.label(
                                RichText::new(active_name)
                                    .font(FontId::proportional(13.5))
                                    .strong()
                                    .color(self.theme.text_primary),
                            );
                            ui.label(
                                RichText::new(char::from(Icon::ChevronDown).to_string())
                                    .font(FontId::new(11.0, FontFamily::Name("lucide".into())))
                                    .color(self.theme.text_secondary),
                            );
                        });
                    });
                    let selector_interact = ui.interact(
                        selector_resp.response.rect,
                        ui.id().with("workspace_selector"),
                        Sense::click(),
                    );
                    if selector_interact.hovered() {
                        ui.painter().rect_filled(
                            selector_resp.response.rect,
                            Rounding::same(6.0),
                            self.theme.surface_hover,
                        );
                    }
                    if selector_interact.clicked() {
                        self.open_palette(PaletteMode::Commands);
                    }
                    selector_interact.on_hover_text("Switch connection / workspace");

                    // Right header action buttons: Search & New Connection
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if Button::new(self.theme)
                            .icon(Icon::Plus)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip("New Connection")
                            .show(ui)
                            .clicked()
                        {
                            self.open_new_connection();
                        }
                        if Button::new(self.theme)
                            .icon(Icon::Search)
                            .variant(ButtonVariant::Ghost)
                            .size(ButtonSize::IconSm)
                            .tooltip(format!("Search / Command Palette ({}⇧P)", Self::primary_modifier_label()))
                            .show(ui)
                            .clicked()
                        {
                            self.open_palette(PaletteMode::Commands);
                        }
                    });
                });
                ui.add_space(4.0);

                // ── 2. Codex-style Primary Action: "+ New query" Button ──────
                let new_query_rect = ui.available_rect_before_wrap();
                let new_query_h = 30.0;
                let btn_rect = Rect::from_min_size(new_query_rect.min, vec2(ui.available_width(), new_query_h));
                let new_query_resp = ui.allocate_rect(btn_rect, Sense::click());
                let is_hovered = new_query_resp.hovered();
                let bg_color = if is_hovered {
                    self.theme.surface_hover
                } else {
                    self.theme.surface_panel
                };
                ui.painter().rect_filled(btn_rect, Rounding::same(6.0), bg_color);
                ui.painter().rect_stroke(
                    btn_rect,
                    Rounding::same(6.0),
                    Stroke::new(1.0, if is_hovered { self.theme.border_default } else { self.theme.border_subtle }),
                );

                // Paint Icon + Text + Shortcut inside New Query button
                let left_center = Pos2::new(btn_rect.left() + 10.0, btn_rect.center().y);
                ui.painter().text(
                    left_center,
                    Align2::LEFT_CENTER,
                    char::from(Icon::SquarePen).to_string(),
                    FontId::new(13.0, FontFamily::Name("lucide".into())),
                    self.theme.text_primary,
                );
                ui.painter().text(
                    Pos2::new(left_center.x + 18.0, left_center.y),
                    Align2::LEFT_CENTER,
                    "New query",
                    FontId::proportional(12.5),
                    self.theme.text_primary,
                );
                ui.painter().text(
                    Pos2::new(btn_rect.right() - 10.0, left_center.y),
                    Align2::RIGHT_CENTER,
                    format!("{}N", Self::primary_modifier_label()),
                    FontId::proportional(11.0),
                    self.theme.text_muted,
                );

                if new_query_resp.clicked() {
                    self.new_query_document();
                    self.active_tab = WorkspaceTab::Query;
                }
                new_query_resp.on_hover_cursor(egui::CursorIcon::PointingHand);

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                // ── 3. Per-activity content ────────────────────────────────────
                match self.activity {
                    Activity::Explorer => self.draw_explorer_sub_panes(ui),
                    _ => {
                        egui::ScrollArea::vertical()
                            .id_salt("sidebar_scroll")
                            .show(ui, |ui| {
                                ui.add_space(4.0);
                                match self.activity {
                                    Activity::Queries => self.draw_queries(ui),
                                    Activity::History => self.draw_history(ui),
                                    Activity::Transfers => self.draw_activity_placeholder(
                                        ui,
                                        "TRANSFERS",
                                        Icon::Upload,
                                        "Background transfer jobs will appear here when transfers are available.",
                                    ),
                                    Activity::Monitor => self.draw_activity_placeholder(
                                        ui,
                                        "MONITOR",
                                        Icon::Gauge,
                                        "Connection health and query activity will appear here when monitoring is available.",
                                    ),
                                    Activity::Settings => self.draw_settings(ui),
                                    Activity::Diagram => self.draw_diagram_sidebar(ui),
                                    Activity::Explorer => unreachable!(),
                                }
                            });
                    }
                }
            });
        self.sidebar_width = response
            .response
            .rect
            .width()
            .clamp(SIDEBAR_MIN_WIDTH, SIDEBAR_MAX_WIDTH);
    }

    fn draw_activity_placeholder(&self, ui: &mut egui::Ui, title: &str, icon: Icon, description: &str) {
        ui.add_space(36.0);
        ui.vertical_centered(|ui| {
            ui.label(icon_text(icon, "", self.theme.accent));
            ui.add_space(10.0);
            ui.label(RichText::new(title).strong().color(self.theme.text_primary));
            ui.add_space(6.0);
            badge(ui, "COMING SOON", self.theme.surface_active, self.theme.text_secondary);
            ui.add_space(8.0);
            ui.label(RichText::new(description).small().color(self.theme.text_muted));
        });
    }

    fn draw_diagram_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            section_label(ui, "SCHEMA MAP", self.theme);
            ui.add_space(8.0);
            ui.label(RichText::new("Tables and foreign-key relationships").color(self.theme.text_primary));
            ui.add_space(4.0);
            ui.label(
                RichText::new(format!(
                    "{} tables · {} relationships",
                    self.schema.table_details.len(),
                    self.schema
                        .table_details
                        .iter()
                        .map(|table| table.foreign_keys.len())
                        .sum::<usize>()
                ))
                .small()
                .color(self.theme.text_muted),
            );
        });
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);
        ui.vertical(|ui| {
            section_label(ui, "NAVIGATION", self.theme);
            ui.add_space(8.0);
            ui.label(
                RichText::new(
                    "Drag the canvas to pan. Use the floating controls in the map to zoom or fit the schema.",
                )
                .small()
                .color(self.theme.text_secondary),
            );
            ui.add_space(10.0);
            if secondary_button_with_icon(ui, Icon::Database, "Back to Explorer", self.theme).clicked() {
                self.activity = Activity::Explorer;
                self.sidebar_open = true;
            }
        });
    }

    fn draw_queries(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            section_label(ui, "OPEN DOCUMENTS", self.theme);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if compact_icon_button(ui, Icon::Plus, self.theme)
                    .on_hover_text("New query")
                    .clicked()
                {
                    self.new_query_document();
                }
            });
        });
        ui.add_space(8.0);

        for (index, document) in self.query_documents.clone().into_iter().enumerate() {
            let selected = self.active_tab == WorkspaceTab::Query && self.active_query_document == index;
            let unsaved = selected && document.content != self.query_text;
            let title = if unsaved {
                format!("{}  •", document.title)
            } else {
                document.title.clone()
            };
            let response = sidebar_item(ui, Icon::FileCode2, &title, selected, self.theme);
            let is_ctx = is_context_menu_triggered(&response, ui);
            let mut close_requested = false;
            let mut duplicate_requested = false;
            let theme = self.theme;
            context_action_menu(ui, &response, theme, |ui, close_menu| {
                if ctx_menu_item(ui, Some(Icon::Copy), "Duplicate query", None, theme.text_primary, theme).clicked() {
                    duplicate_requested = true;
                    *close_menu = true;
                }
                if self.query_documents.len() > 1
                    && ctx_menu_item(ui, Some(Icon::Trash2), "Close query", None, theme.danger, theme).clicked()
                {
                    close_requested = true;
                    *close_menu = true;
                }
            });
            if response.clicked() && !is_ctx {
                self.switch_query_document(index);
                self.active_tab = WorkspaceTab::Query;
            }
            if duplicate_requested {
                self.new_query_document();
                self.query_text = document.content.clone();
            }
            if close_requested {
                self.close_query_document(index);
            }
        }

        ui.add_space(12.0);
        ui.label(
            RichText::new("Right-click a document for actions")
                .small()
                .color(self.theme.text_muted),
        );
    }

    fn draw_history(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Saved queries")
                .small()
                .strong()
                .color(self.theme.text_muted),
        );
        self.draw_saved_queries_section(ui);
        ui.separator();
        ui.label(
            RichText::new("Local history")
                .small()
                .strong()
                .color(self.theme.text_muted),
        );
        self.draw_local_history_section(ui);
    }

    /// Saved queries, grouped by folder, plus the pending-delete confirmation.
    fn draw_saved_queries_section(&mut self, ui: &mut egui::Ui) {
        if self.saved_queries.is_empty() {
            self.draw_empty_saved_queries(ui);
            return;
        }
        let saved = self.saved_queries.clone();
        let mut groups: Vec<(String, Vec<UiSavedQuerySummary>)> = Vec::new();
        for query in saved {
            let folder = query.folder.clone().unwrap_or_else(|| "Unfiled".to_owned());
            if let Some((_, queries)) = groups.iter_mut().find(|(name, _)| name == &folder) {
                queries.push(query);
            } else {
                groups.push((folder, vec![query]));
            }
        }
        for (folder, queries) in groups {
            self.draw_saved_query_folder(ui, folder, queries);
        }
        self.draw_delete_saved_query_confirmation(ui);
    }

    fn draw_empty_saved_queries(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(icon_text(Icon::Bookmark, "", self.theme.accent));
                ui.add_space(6.0);
                ui.label(RichText::new("No saved queries yet").strong());
                ui.label(
                    RichText::new("Save a query to keep it close at hand.")
                        .small()
                        .color(self.theme.text_muted),
                );
                ui.add_space(8.0);
                if compact_button_with_icon(ui, Icon::Plus, "New query", self.theme).clicked() {
                    self.new_query_document();
                }
            });
        });
    }

    /// One collapsible folder of saved queries, with a folder-level context menu.
    fn draw_saved_query_folder(&mut self, ui: &mut egui::Ui, folder: String, queries: Vec<UiSavedQuerySummary>) {
        let folder_id = self
            .query_folders
            .iter()
            .find(|item| item.name == folder)
            .map(|item| item.id.clone());
        let mut delete_requested = false;
        let header = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id(("saved-query-folder", folder.as_str())),
            true,
        )
        .show_header(ui, |ui| {
            ui.label(icon_text(Icon::FolderOpen, &folder, self.theme.text_primary));
            ui.label(
                RichText::new(format!("{} queries", queries.len()))
                    .small()
                    .color(self.theme.text_muted),
            );
        });
        let (_, header_response, _) = header.body(|ui| {
            for query in queries {
                self.draw_saved_query_entry(ui, &query);
            }
        });
        let theme = self.theme;
        context_action_menu(ui, &header_response.response, theme, |ui, close_menu| {
            if folder_id.is_some()
                && ctx_menu_item(ui, Some(Icon::Trash2), "Delete folder", None, theme.danger, theme).clicked()
            {
                delete_requested = true;
                *close_menu = true;
            }
        });
        if delete_requested {
            self.folder_delete_confirmation = folder_id;
        }
    }

    fn draw_saved_query_entry(&mut self, ui: &mut egui::Ui, query: &UiSavedQuerySummary) {
        let query_response = sidebar_item(ui, Icon::FileCode2, &query.name, false, self.theme);
        let is_ctx = is_context_menu_triggered(&query_response, ui);
        let mut rename_requested = false;
        let mut delete_requested = false;
        let mut copy_sql = false;
        let theme = self.theme;
        context_action_menu(ui, &query_response, theme, |ui, close_menu| {
            if ctx_menu_item(ui, Some(Icon::Play), "Open in Editor", None, theme.text_primary, theme).clicked() {
                *close_menu = true;
            }
            if ctx_menu_item(ui, Some(Icon::Copy), "Copy SQL", None, theme.text_primary, theme).clicked() {
                copy_sql = true;
                *close_menu = true;
            }
            ui.separator();
            if ctx_menu_item(ui, Some(Icon::Pencil), "Rename query", None, theme.text_primary, theme).clicked() {
                rename_requested = true;
                *close_menu = true;
            }
            if ctx_menu_item(ui, Some(Icon::Trash2), "Delete query", None, theme.danger, theme).clicked() {
                delete_requested = true;
                *close_menu = true;
            }
        });
        if query_response.clicked() && !is_ctx {
            self.query_text = query.sql.clone();
            self.active_tab = WorkspaceTab::Query;
        }
        if copy_sql {
            ui.output_mut(|o| o.copied_text = query.sql.clone());
            self.runtime_message = format!("Copied SQL for `{}`", query.name);
        }
        if rename_requested {
            self.rename_saved_query(query);
        }
        if delete_requested {
            self.delete_confirmation_id = Some(query.id.clone());
        }
    }

    fn rename_saved_query(&mut self, query: &UiSavedQuerySummary) {
        let request_id = self.task_bridge.next_request_id();
        let name = if self.query_folder.trim().is_empty() {
            format!("{} (renamed)", query.name)
        } else {
            self.query_folder.trim().to_owned()
        };
        self.dispatch_command(UiCommand::RenameSavedQuery {
            request_id,
            id: query.id.clone(),
            name,
        });
    }

    fn draw_delete_saved_query_confirmation(&mut self, ui: &mut egui::Ui) {
        let Some(id) = self.delete_confirmation_id.clone() else {
            return;
        };
        ui.colored_label(self.theme.warning, "Delete this saved query?");
        ui.horizontal(|ui| {
            if compact_button(ui, "Confirm delete", self.theme).clicked() {
                let request_id = self.task_bridge.next_request_id();
                self.dispatch_command(UiCommand::DeleteSavedQuery { request_id, id });
                self.delete_confirmation_id = None;
            }
            if compact_button(ui, "Cancel", self.theme).clicked() {
                self.delete_confirmation_id = None;
            }
        });
    }

    fn draw_local_history_section(&mut self, ui: &mut egui::Ui) {
        if self.query_history.is_empty() {
            ui.label(RichText::new("No queries run yet").color(self.theme.text_muted));
            return;
        }
        for query in self.query_history.iter().rev() {
            let title = query.lines().next().unwrap_or("query");
            if sidebar_item(ui, Icon::History, title, false, self.theme)
                .on_hover_text("Open query from local history")
                .clicked()
            {
                self.query_text = query.clone();
                self.active_tab = WorkspaceTab::Query;
            }
            ui.add_space(12.0);
        }
    }

    fn draw_settings(&mut self, ui: &mut egui::Ui) {
        self.draw_appearance_settings(ui);
        ui.add_space(12.0);
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "DATABASE FILES", self.theme);
            if !self.supports_backup_restore() {
                ui.label(
                    RichText::new("Backup and restore are unavailable for the active provider")
                        .small()
                        .color(self.theme.text_muted),
                );
                return;
            }
            ui.add_space(10.0);
            self.draw_backup_settings(ui);
            ui.add_space(14.0);
            self.draw_restore_settings(ui);
        });
    }

    fn draw_appearance_settings(&mut self, ui: &mut egui::Ui) {
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "APPEARANCE", self.theme);
            ui.add_space(10.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(icon_text(
                    if self.dark_mode { Icon::Moon } else { Icon::Sun },
                    "",
                    self.theme.accent,
                ));
                ui.selectable_value(&mut self.dark_mode, false, "Light");
                ui.selectable_value(&mut self.dark_mode, true, "Dark");
            });
            ui.label(
                RichText::new(
                    "Quiet surfaces, violet focus states, and high-contrast data. The choice is saved locally.",
                )
                .small()
                .color(self.theme.text_muted),
            );
            ui.checkbox(&mut self.reduce_motion, "Reduce motion");
            ui.label(
                RichText::new("Loading states keep a static status icon instead of a spinner.")
                    .small()
                    .color(self.theme.text_muted),
            );
        });
    }

    fn supports_backup_restore(&self) -> bool {
        self.active_capabilities()
            .is_some_and(|capabilities| capabilities.features.backup)
    }

    fn draw_backup_settings(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Backup destination")
                .small()
                .color(self.theme.text_secondary),
        );
        input_full_width(
            ui,
            &mut self.backup_output_path,
            "Choose a .sql backup path",
            self.theme,
        );
        ui.horizontal_wrapped(|ui| {
            if compact_button_with_icon(ui, Icon::FolderOpen, "Choose path", self.theme).clicked() {
                let request_id = self.task_bridge.next_request_id();
                self.dispatch_command(UiCommand::PickBackupFile { request_id });
            }
            if secondary_button_with_icon(ui, Icon::Archive, "Create backup", self.theme).clicked() {
                if let Some(connection) = self.active_connection().cloned() {
                    let request_id = self.task_bridge.next_request_id();
                    self.dispatch_command(UiCommand::Backup {
                        request_id,
                        connection_id: connection.id,
                        output_path: self.backup_output_path.clone(),
                        custom_format: false,
                    });
                }
            }
        });
    }

    fn draw_restore_settings(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Restore from backup")
                .small()
                .color(self.theme.text_secondary),
        );
        input_full_width(ui, &mut self.restore_input_path, "Choose a backup file", self.theme);
        ui.horizontal_wrapped(|ui| {
            if compact_button_with_icon(ui, Icon::FolderOpen, "Choose file", self.theme).clicked() {
                let request_id = self.task_bridge.next_request_id();
                self.dispatch_command(UiCommand::PickRestoreFile { request_id });
            }
            if secondary_button_with_icon(ui, Icon::RotateCcw, "Restore database", self.theme).clicked() {
                self.restore_confirmation = true;
            }
        });
        if self.restore_confirmation {
            ui.add_space(10.0);
            ui.colored_label(self.theme.warning, "Overwrite the active database?");
            ui.horizontal(|ui| {
                if danger_button(ui, "Confirm restore", self.theme).clicked() {
                    if let Some(connection) = self.active_connection().cloned() {
                        let request_id = self.task_bridge.next_request_id();
                        self.dispatch_command(UiCommand::Restore {
                            request_id,
                            connection_id: connection.id,
                            input_path: self.restore_input_path.clone(),
                            custom_format: false,
                        });
                    }
                    self.restore_confirmation = false;
                }
                if ghost_button_with_icon(ui, Icon::X, "Cancel", self.theme).clicked() {
                    self.restore_confirmation = false;
                }
            });
        }
    }
}
