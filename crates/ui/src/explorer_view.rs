use super::*;
use egui::{pos2, vec2, Align2, Color32, FontFamily, FontId, Margin, Rect, Rounding, Stroke};
use lucide_icons::Icon;

/// Properties for rendering an ultra-clean Codex-style tree row.
struct CodexTreeRow<'a> {
    depth: usize,
    is_expandable: bool,
    is_expanded: bool,
    icon: Icon,
    icon_color: Color32,
    label: &'a str,
    is_selected: bool,
    is_dimmed: bool,
    status_dot: Option<Color32>,
    badge_text: Option<&'a str>,
    badge_accent: bool,
    count_text: Option<String>,
    detail_text: Option<&'a str>,
}

/// Renders a single pixel-aligned, elegant tree row inspired by OpenAI Codex and modern developer tools.
fn draw_codex_tree_row(ui: &mut egui::Ui, theme: &DbProTheme, row: CodexTreeRow<'_>) -> (egui::Response, bool) {
    let row_height = 26.0;
    let available_width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(vec2(available_width, row_height), egui::Sense::click());

    let is_hovered = response.hovered();
    let painter = ui.painter();

    // 1. Hover & Selection states
    if row.is_selected {
        painter.rect_filled(rect, Rounding::same(4.0), theme.surface_active);
        // Signature Codex active pill indicator on left edge
        let pill_rect = Rect::from_min_max(
            pos2(rect.min.x + 1.0, rect.min.y + 4.0),
            pos2(rect.min.x + 3.5, rect.max.y - 4.0),
        );
        painter.rect_filled(pill_rect, Rounding::same(1.2), theme.accent);
    } else if is_hovered {
        painter.rect_filled(rect, Rounding::same(4.0), theme.surface_hover);
    }

    let center_y = rect.center().y;
    let mut curr_x = rect.min.x + 6.0 + (row.depth as f32) * 14.0;

    // 2. Chevron slot (width 14.0 px)
    let chevron_rect = Rect::from_min_size(pos2(curr_x, rect.min.y), vec2(14.0, row_height));
    let mut chevron_clicked = false;
    if row.is_expandable {
        let expand_t = ui.ctx().animate_bool_with_time(
            response.id.with("chev_anim"),
            row.is_expanded,
            crate::components::animation::OVERLAY_DURATION_SECS,
        );
        let chevron_color = if is_hovered || row.is_selected {
            theme.text_secondary
        } else {
            theme.text_muted
        };
        if expand_t < 0.99 {
            let alpha = ((1.0 - expand_t) * 255.0) as u8;
            let c = Color32::from_rgba_unmultiplied(chevron_color.r(), chevron_color.g(), chevron_color.b(), alpha);
            painter.text(
                chevron_rect.center(),
                Align2::CENTER_CENTER,
                char::from(Icon::ChevronRight).to_string(),
                FontId::new(10.5, FontFamily::Name("lucide".into())),
                c,
            );
        }
        if expand_t > 0.01 {
            let alpha = (expand_t * 255.0) as u8;
            let c = Color32::from_rgba_unmultiplied(chevron_color.r(), chevron_color.g(), chevron_color.b(), alpha);
            painter.text(
                chevron_rect.center(),
                Align2::CENTER_CENTER,
                char::from(Icon::ChevronDown).to_string(),
                FontId::new(10.5, FontFamily::Name("lucide".into())),
                c,
            );
        }
    }
    curr_x += 14.0;

    // 3. Status dot (e.g. connection status)
    if let Some(dot_color) = row.status_dot {
        painter.circle_filled(pos2(curr_x + 3.0, center_y), 3.0, dot_color);
        curr_x += 10.0;
    }

    // 4. Node Icon (Lucide vector icon)
    painter.text(
        pos2(curr_x + 7.0, center_y),
        Align2::CENTER_CENTER,
        char::from(row.icon).to_string(),
        FontId::new(13.0, FontFamily::Name("lucide".into())),
        row.icon_color,
    );
    curr_x += 17.0;

    // 5. Right-side items (render right-to-left)
    let mut right_x = rect.max.x - 6.0;

    // 5a. Driver badge (e.g. PG / SQLITE)
    if let Some(badge) = row.badge_text {
        let badge_color = if row.badge_accent {
            theme.accent
        } else {
            theme.text_muted
        };
        let badge_bg = if row.badge_accent {
            theme.accent_soft
        } else {
            theme.surface_hover
        };
        let badge_w = (badge.len() as f32) * 6.5 + 8.0;
        let badge_h = 16.0;
        let badge_rect = Rect::from_min_size(
            pos2(right_x - badge_w, center_y - badge_h * 0.5),
            vec2(badge_w, badge_h),
        );
        painter.rect_filled(badge_rect, Rounding::same(3.0), badge_bg);
        painter.text(
            badge_rect.center(),
            Align2::CENTER_CENTER,
            badge,
            FontId::proportional(9.5),
            badge_color,
        );
        right_x -= badge_w + 5.0;
    }

    // 5b. Count text (e.g. 68)
    if let Some(count) = row.count_text.as_deref() {
        let count_rect = painter.text(
            pos2(right_x, center_y),
            Align2::RIGHT_CENTER,
            count,
            FontId::proportional(11.0),
            theme.text_muted,
        );
        right_x -= count_rect.width() + 5.0;
    }

    // 5c. Detail text (e.g. column data type: varchar, int8)
    if let Some(detail) = row.detail_text {
        let detail_rect = painter.text(
            pos2(right_x, center_y),
            Align2::RIGHT_CENTER,
            detail,
            FontId::monospace(10.5),
            theme.text_muted,
        );
        right_x -= detail_rect.width() + 5.0;
    }

    // 6. Label (clipped)
    let label_color = if row.is_selected {
        theme.accent
    } else if row.is_dimmed {
        theme.text_muted
    } else {
        theme.text_primary
    };

    let clip_rect = Rect::from_min_max(pos2(curr_x, rect.min.y), pos2(right_x.max(curr_x + 10.0), rect.max.y));
    painter.with_clip_rect(clip_rect).text(
        pos2(curr_x, center_y),
        Align2::LEFT_CENTER,
        row.label,
        FontId::proportional(12.5),
        label_color,
    );

    if response.clicked() {
        if let Some(click_pos) = ui.input(|i| i.pointer.hover_pos()) {
            if chevron_rect.expand(2.0).contains(click_pos) {
                chevron_clicked = true;
            }
        }
    }

    (response, chevron_clicked)
}

/// Determines the best semantic Lucide icon and color for a column.
fn column_icon_and_color(data_type: &str, is_pk: bool, is_fk: bool, theme: &DbProTheme) -> (Icon, Color32) {
    if is_pk {
        (Icon::Key, theme.warning)
    } else if is_fk {
        (Icon::Link, theme.info)
    } else {
        let dt = data_type.to_ascii_lowercase();
        if dt.contains("int")
            || dt.contains("serial")
            || dt.contains("num")
            || dt.contains("dec")
            || dt.contains("float")
            || dt.contains("double")
        {
            (Icon::Hash, theme.text_muted)
        } else if dt.contains("char") || dt.contains("text") || dt.contains("uuid") {
            (Icon::Type, theme.text_muted)
        } else if dt.contains("date") || dt.contains("time") {
            (Icon::Calendar, theme.text_muted)
        } else if dt.contains("bool") {
            (Icon::ToggleLeft, theme.text_muted)
        } else {
            (Icon::Columns3, theme.text_muted)
        }
    }
}

impl DbProApp {
    /// Entry-point for Database Navigator in Codex / DBeaver style:
    /// Unified hierarchical tree where connections are root nodes.
    pub(super) fn draw_explorer_sub_panes(&mut self, ui: &mut egui::Ui) {
        // ── Toolbar: Codex Search Bar + Action Buttons ──────────────────
        ui.horizontal(|ui| {
            let clear_width = if self.explorer_search.is_empty() { 0.0 } else { 22.0 };
            let search_width = (ui.available_width() - clear_width - 56.0).max(80.0);

            egui::Frame {
                fill: self.theme.surface_hover,
                rounding: Rounding::same(6.0),
                stroke: Stroke::new(1.0, self.theme.border_subtle),
                inner_margin: Margin::symmetric(6.0, 3.0),
                ..Default::default()
            }
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(char::from(Icon::Search).to_string())
                            .family(FontFamily::Name("lucide".into()))
                            .size(12.0)
                            .color(self.theme.text_muted),
                    );
                    ui.add_sized(
                        vec2(search_width - 24.0, 20.0),
                        egui::TextEdit::singleline(&mut self.explorer_search)
                            .hint_text(RichText::new("Filter objects…").size(12.0).color(self.theme.text_muted))
                            .frame(false)
                            .text_color(self.theme.text_primary),
                    );
                });
            });

            if !self.explorer_search.is_empty()
                && compact_icon_button(ui, Icon::X, self.theme)
                    .on_hover_text("Clear filter")
                    .clicked()
            {
                self.explorer_search.clear();
            }

            if compact_icon_button(ui, Icon::Plus, self.theme)
                .on_hover_text("New connection")
                .clicked()
            {
                self.open_new_connection();
            }

            let mut refresh_schema = false;
            let refresh_btn =
                compact_icon_button(ui, Icon::RotateCcw, self.theme).on_hover_text("Refresh active schema");
            refresh_btn.context_menu(|ui| {
                if ui.button("Refresh Schema").clicked() {
                    refresh_schema = true;
                    ui.close_menu();
                }
            });
            if refresh_btn.clicked() || refresh_schema {
                if let Some(connection_id) = self.active_connection_id.clone() {
                    self.request_schema_introspection(connection_id, true);
                }
            }
        });
        ui.add_space(6.0);

        // ── Unified Database Navigator Tree ─────────────────────────────
        egui::ScrollArea::vertical()
            .id_salt("codex_navigator_scroll")
            .show(ui, |ui| {
                if self.connections.is_empty() {
                    self.draw_dbeaver_empty_state(ui);
                } else {
                    self.draw_dbeaver_connections_tree(ui);
                }
            });
    }

    /// Empty state shown when no connections exist yet.
    fn draw_dbeaver_empty_state(&mut self, ui: &mut egui::Ui) {
        ui.add_space(36.0);
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(char::from(Icon::Database).to_string())
                    .family(FontFamily::Name("lucide".into()))
                    .size(28.0)
                    .color(self.theme.text_muted),
            );
            ui.add_space(8.0);
            ui.label(RichText::new("No connections").strong().color(self.theme.text_primary));
            ui.add_space(3.0);
            ui.label(
                RichText::new("Create a database connection to begin.")
                    .small()
                    .color(self.theme.text_muted),
            );
            ui.add_space(12.0);
            if compact_button_with_icon(ui, Icon::Plus, "New connection", self.theme).clicked() {
                self.open_new_connection();
            }
        });
    }

    /// Renders the list of connections as expandable root nodes (Codex / DBeaver style).
    fn draw_dbeaver_connections_tree(&mut self, ui: &mut egui::Ui) {
        let connections = self.connections.clone();
        for connection in connections {
            let is_active = self.active_connection_id.as_deref() == Some(&connection.id);
            let is_connected = self.connected && is_active;
            let is_connecting = self.pending_connection_request.is_some() && is_active;
            let id = ui.make_persistent_id(("codex_conn_node", &connection.id));

            let mut connect_now = false;
            let mut disconnect_now = false;
            let mut refresh_now = false;
            let mut edit_now = false;
            let mut delete_now = false;

            let mut collapsing =
                egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, is_connected);
            let is_open = collapsing.is_open();

            let status_dot = if is_connected {
                Some(self.theme.success)
            } else if is_connecting {
                Some(self.theme.accent)
            } else {
                Some(self.theme.text_tertiary)
            };

            let badge_text = if connection.driver.eq_ignore_ascii_case("postgresql") {
                "PG"
            } else {
                "SQLITE"
            };

            let (response, chevron_clicked) = draw_codex_tree_row(
                ui,
                &self.theme,
                CodexTreeRow {
                    depth: 0,
                    is_expandable: true,
                    is_expanded: is_open,
                    icon: Icon::Database,
                    icon_color: if is_connected {
                        self.theme.accent
                    } else {
                        self.theme.text_muted
                    },
                    label: &connection.name,
                    is_selected: is_active,
                    is_dimmed: !is_connected,
                    status_dot,
                    badge_text: Some(badge_text),
                    badge_accent: is_connected,
                    count_text: None,
                    detail_text: None,
                },
            );

            if chevron_clicked {
                collapsing.set_open(!is_open);
                collapsing.store(ui.ctx());
            } else if response.clicked() {
                if !is_connected {
                    connect_now = true;
                    collapsing.set_open(true);
                    collapsing.store(ui.ctx());
                } else {
                    collapsing.set_open(!is_open);
                    collapsing.store(ui.ctx());
                }
            }

            response.context_menu(|ui| {
                if is_connected {
                    if ui.button("Disconnect").clicked() {
                        disconnect_now = true;
                        ui.close_menu();
                    }
                    if ui.button("Refresh Schema").clicked() {
                        refresh_now = true;
                        ui.close_menu();
                    }
                } else if ui.button("Connect").clicked() {
                    connect_now = true;
                    ui.close_menu();
                }
                if ui.button("Edit Connection").clicked() {
                    edit_now = true;
                    ui.close_menu();
                }
                if ui.button("Delete Connection").clicked() {
                    delete_now = true;
                    ui.close_menu();
                }
            });

            if collapsing.is_open() {
                if is_connected {
                    self.draw_dbeaver_connected_body(ui, &connection);
                } else {
                    let (sub_resp, _) = draw_codex_tree_row(
                        ui,
                        &self.theme,
                        CodexTreeRow {
                            depth: 1,
                            is_expandable: false,
                            is_expanded: false,
                            icon: Icon::Circle,
                            icon_color: self.theme.text_muted,
                            label: "Disconnected — click to connect",
                            is_selected: false,
                            is_dimmed: true,
                            status_dot: None,
                            badge_text: None,
                            badge_accent: false,
                            count_text: None,
                            detail_text: None,
                        },
                    );
                    if sub_resp.clicked() {
                        connect_now = true;
                    }
                }
            }

            if connect_now {
                self.connect_to_connection(&connection);
            }
            if disconnect_now {
                self.connected = false;
                self.schema = UiSchemaSummary::default();
                self.selected_table = None;
                self.selected_schema_object = None;
                self.runtime_message = format!("Disconnected from {}", connection.name);
            }
            if refresh_now {
                self.request_schema_introspection(connection.id.clone(), true);
            }
            if edit_now {
                self.open_edit_connection(&connection);
            }
            if delete_now {
                self.delete_confirmation_id = Some(connection.id.clone());
            }
            ui.add_space(2.0);
        }
    }

    /// Body rendered when a connection node is expanded.
    fn draw_dbeaver_connected_body(&mut self, ui: &mut egui::Ui, connection: &UiConnectionSummary) {
        // Schema feedback (progress / error)
        self.draw_explorer_schema_feedback(ui);

        // Database node
        let db_id = ui.make_persistent_id(("codex_db_node", &connection.id, &connection.database));
        let db_name = if connection.database.is_empty() {
            "database".to_owned()
        } else {
            connection.database.clone()
        };

        let mut collapsing = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), db_id, true);
        let is_open = collapsing.is_open();

        let (resp, chevron_clicked) = draw_codex_tree_row(
            ui,
            &self.theme,
            CodexTreeRow {
                depth: 1,
                is_expandable: true,
                is_expanded: is_open,
                icon: Icon::Database,
                icon_color: self.theme.accent,
                label: &db_name,
                is_selected: false,
                is_dimmed: false,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: None,
                detail_text: None,
            },
        );

        if resp.clicked() || chevron_clicked {
            collapsing.set_open(!is_open);
            collapsing.store(ui.ctx());
        }

        if collapsing.is_open() {
            let schemas = self.schema.schemas.clone();
            if schemas.is_empty() {
                // Flat tables/views (e.g. SQLite)
                self.draw_dbeaver_schema_objects(ui, "");
            } else {
                // Nested Schemas (e.g. PostgreSQL: public, information_schema, etc.)
                for schema in &schemas {
                    self.draw_dbeaver_schema_node(ui, &connection.id, schema);
                }
            }
        }
    }

    /// A schema folder node inside the Database node (e.g. `public`).
    fn draw_dbeaver_schema_node(&mut self, ui: &mut egui::Ui, connection_id: &str, schema: &str) {
        let is_active_schema = self.active_schema() == schema;
        let schema_id = ui.make_persistent_id(("codex_schema_node", connection_id, schema));
        let table_count = self.schema.table_details.iter().filter(|t| t.schema == schema).count();

        let mut collapsing =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), schema_id, is_active_schema);
        let is_open = collapsing.is_open();

        let (resp, chevron_clicked) = draw_codex_tree_row(
            ui,
            &self.theme,
            CodexTreeRow {
                depth: 2,
                is_expandable: true,
                is_expanded: is_open,
                icon: if is_open { Icon::FolderOpen } else { Icon::Folder },
                icon_color: if is_active_schema {
                    self.theme.warning
                } else {
                    self.theme.text_secondary
                },
                label: schema,
                is_selected: is_active_schema,
                is_dimmed: !is_active_schema,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: if table_count > 0 {
                    Some(table_count.to_string())
                } else {
                    None
                },
                detail_text: None,
            },
        );

        let mut activate_schema = false;
        if chevron_clicked {
            collapsing.set_open(!is_open);
            collapsing.store(ui.ctx());
        } else if resp.clicked() {
            if !is_active_schema {
                activate_schema = true;
                collapsing.set_open(true);
                collapsing.store(ui.ctx());
            } else {
                collapsing.set_open(!is_open);
                collapsing.store(ui.ctx());
            }
        }

        if collapsing.is_open() {
            if is_active_schema {
                self.draw_dbeaver_schema_objects(ui, schema);
            } else {
                let (act_resp, _) = draw_codex_tree_row(
                    ui,
                    &self.theme,
                    CodexTreeRow {
                        depth: 3,
                        is_expandable: false,
                        is_expanded: false,
                        icon: Icon::Circle,
                        icon_color: self.theme.text_muted,
                        label: "Inactive schema — click to activate",
                        is_selected: false,
                        is_dimmed: true,
                        status_dot: None,
                        badge_text: None,
                        badge_accent: false,
                        count_text: None,
                        detail_text: None,
                    },
                );
                if act_resp.clicked() {
                    activate_schema = true;
                }
            }
        }

        if activate_schema {
            self.selected_schema = Some(schema.to_owned());
            self.selected_table = None;
            self.selected_schema_object = None;
            self.table_info = None;
            self.table_ddl = None;
            self.table_data_result = None;
            self.staged_changes.clear();
            self.active_tab = WorkspaceTab::Welcome;
        }
    }

    /// Renders the folders for a schema: Tables, Views, Functions, Triggers.
    fn draw_dbeaver_schema_objects(&mut self, ui: &mut egui::Ui, schema: &str) {
        let search_query = self.explorer_search.trim().to_ascii_lowercase();
        let all_tables = self.active_schema_table_names();
        let total_tables = all_tables.len();
        let (matching_table_count, tables) = filtered_explorer_tables(&all_tables, &search_query);

        // ── Tables Category Folder ──────────────────────────────────────
        let tables_folder_id = ui.make_persistent_id(("codex_tbl_folder", schema));
        let count_str = if search_query.is_empty() {
            total_tables.to_string()
        } else {
            format!("{matching_table_count}/{total_tables}")
        };

        let mut collapsing =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), tables_folder_id, true);
        let is_open = collapsing.is_open();

        let (resp, chevron_clicked) = draw_codex_tree_row(
            ui,
            &self.theme,
            CodexTreeRow {
                depth: 3,
                is_expandable: true,
                is_expanded: is_open,
                icon: Icon::Table2,
                icon_color: self.theme.info,
                label: "Tables",
                is_selected: false,
                is_dimmed: total_tables == 0,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: Some(count_str),
                detail_text: None,
            },
        );

        if resp.clicked() || chevron_clicked {
            collapsing.set_open(!is_open);
            collapsing.store(ui.ctx());
        }

        if collapsing.is_open() {
            if tables.is_empty() {
                draw_codex_tree_row(
                    ui,
                    &self.theme,
                    CodexTreeRow {
                        depth: 4,
                        is_expandable: false,
                        is_expanded: false,
                        icon: Icon::Info,
                        icon_color: self.theme.text_muted,
                        label: if total_tables == 0 {
                            "No tables in schema"
                        } else {
                            "No matching tables"
                        },
                        is_selected: false,
                        is_dimmed: true,
                        status_dot: None,
                        badge_text: None,
                        badge_accent: false,
                        count_text: None,
                        detail_text: None,
                    },
                );
            } else {
                for table in &tables {
                    self.draw_dbeaver_table_item(ui, table);
                }
            }
        }

        // ── Views Category Folder ────────────────────────────────────────
        let schema_name = schema.to_owned();
        let views: Vec<_> = if self.schema.schemas.is_empty() {
            self.schema.views.clone()
        } else {
            self.schema
                .views
                .iter()
                .filter(|v| v.schema == schema_name)
                .cloned()
                .collect()
        };
        self.draw_dbeaver_views_folder(ui, &views);

        // ── Functions Category Folder (PostgreSQL only) ──────────────────
        let supports_functions = self.active_capabilities().is_some_and(|c| c.schema.functions);
        if supports_functions {
            let functions: Vec<_> = if self.schema.schemas.is_empty() {
                self.schema.functions.clone()
            } else {
                self.schema
                    .functions
                    .iter()
                    .filter(|f| f.schema == schema_name)
                    .cloned()
                    .collect()
            };
            self.draw_dbeaver_functions_folder(ui, &functions);
        }

        // ── Triggers Category Folder ─────────────────────────────────────
        let triggers: Vec<_> = if self.schema.schemas.is_empty() {
            self.schema.triggers.clone()
        } else {
            self.schema
                .triggers
                .iter()
                .filter(|t| t.schema == schema_name)
                .cloned()
                .collect()
        };
        self.draw_dbeaver_triggers_folder(ui, &triggers);
    }

    /// Renders an individual table item in the tree with selection and expandable details.
    fn draw_dbeaver_table_item(&mut self, ui: &mut egui::Ui, table: &str) {
        let is_selected = self.selected_table.as_deref() == Some(table);
        let table_details_id = ui.make_persistent_id(("codex_tbl_details", table));
        let has_details = is_selected && self.table_info.is_some();

        let mut collapsing =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), table_details_id, true);
        let is_open = collapsing.is_open();

        let (resp, chevron_clicked) = draw_codex_tree_row(
            ui,
            &self.theme,
            CodexTreeRow {
                depth: 4,
                is_expandable: has_details,
                is_expanded: is_open,
                icon: Icon::Table2,
                icon_color: if is_selected {
                    self.theme.accent
                } else {
                    self.theme.text_secondary
                },
                label: table,
                is_selected,
                is_dimmed: false,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: None,
                detail_text: None,
            },
        );

        let mut open_query = false;
        let mut ask_agent = false;
        let mut refresh_schema = false;

        resp.context_menu(|ui| {
            if ui.button("Open in Query").clicked() {
                open_query = true;
                ui.close_menu();
            }
            if ui.button("Ask Agent about table").clicked() {
                ask_agent = true;
                ui.close_menu();
            }
            if ui.button("Refresh Schema").clicked() {
                refresh_schema = true;
                ui.close_menu();
            }
        });

        if chevron_clicked && has_details {
            collapsing.set_open(!is_open);
            collapsing.store(ui.ctx());
        } else if resp.clicked() || open_query || ask_agent {
            self.selected_table = Some(table.to_owned());
            self.selected_schema_object = None;
            self.schema_object_view = SchemaObjectView::Definition;
            self.table_info = None;
            self.table_ddl = None;
            self.table_info_error = None;
            self.table_ddl_error = None;
            self.ddl_execute_confirmation = false;
            self.ddl_execution_request = None;
            self.table_data_result = None;
            self.table_data_total_rows = None;
            self.table_data_offset = 0;
            self.table_data_filter_column.clear();
            self.table_data_filter_value.clear();
            self.table_data_sort_column = None;
            self.table_data_sort_desc = false;
            self.table_data_error = None;
            self.table_info_request = None;
            self.table_ddl_request = None;
            self.table_data_request = None;
            self.table_view = TableView::Structure;
            self.table_mutation_request = None;
            self.staged_changes.clear();
            self.staged_apply_request = None;
            self.selected_cell = None;
            self.selected_row = None;
            self.data_editing_cell = None;
            self.data_edit_value.clear();
            self.data_delete_confirmation = false;
            self.query_text = format!("SELECT *\nFROM {table}\nLIMIT 100;");
            self.request_table_info();
            self.active_tab = WorkspaceTab::Table;
        }

        if open_query {
            self.query_text = format!("SELECT *\nFROM {table}\nLIMIT 100;");
            self.active_tab = WorkspaceTab::Query;
        }
        if ask_agent {
            self.open_agent_prompt(
                format!("Explain the `{table}` table and suggest useful read-only queries"),
                ui.ctx(),
            );
        }
        if refresh_schema {
            if let Some(connection_id) = self.active_connection_id.clone() {
                self.request_schema_introspection(connection_id, true);
            }
        }

        // If table is selected and expanded, show nested details (Columns, Foreign keys, Indexes)
        if is_selected && collapsing.is_open() {
            if let Some(info) = self.table_info.clone() {
                // ── Columns Folder ──────────────────────────────────────
                let col_id = ui.make_persistent_id(("codex_tbl_col_folder", table));
                let mut col_collapsing =
                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), col_id, false);
                let col_open = col_collapsing.is_open();

                let (col_resp, col_chevron) = draw_codex_tree_row(
                    ui,
                    &self.theme,
                    CodexTreeRow {
                        depth: 5,
                        is_expandable: true,
                        is_expanded: col_open,
                        icon: Icon::Columns3,
                        icon_color: self.theme.text_secondary,
                        label: "Columns",
                        is_selected: false,
                        is_dimmed: info.columns.is_empty(),
                        status_dot: None,
                        badge_text: None,
                        badge_accent: false,
                        count_text: Some(info.columns.len().to_string()),
                        detail_text: None,
                    },
                );
                if col_resp.clicked() || col_chevron {
                    col_collapsing.set_open(!col_open);
                    col_collapsing.store(ui.ctx());
                }

                if col_collapsing.is_open() {
                    for column in &info.columns {
                        let is_fk = info
                            .foreign_keys
                            .iter()
                            .any(|fk| fk.from_columns.contains(&column.name));
                        let (c_icon, c_color) =
                            column_icon_and_color(&column.data_type, column.is_primary_key, is_fk, &self.theme);

                        draw_codex_tree_row(
                            ui,
                            &self.theme,
                            CodexTreeRow {
                                depth: 6,
                                is_expandable: false,
                                is_expanded: false,
                                icon: c_icon,
                                icon_color: c_color,
                                label: &column.name,
                                is_selected: false,
                                is_dimmed: false,
                                status_dot: None,
                                badge_text: None,
                                badge_accent: false,
                                count_text: None,
                                detail_text: Some(&column.data_type),
                            },
                        );
                    }
                }

                // ── Foreign Keys Folder ─────────────────────────────────
                let fk_id = ui.make_persistent_id(("codex_tbl_fk_folder", table));
                let mut fk_collapsing =
                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), fk_id, false);
                let fk_open = fk_collapsing.is_open();

                let (fk_resp, fk_chevron) = draw_codex_tree_row(
                    ui,
                    &self.theme,
                    CodexTreeRow {
                        depth: 5,
                        is_expandable: true,
                        is_expanded: fk_open,
                        icon: Icon::ArrowRightLeft,
                        icon_color: self.theme.text_secondary,
                        label: "Foreign keys",
                        is_selected: false,
                        is_dimmed: info.foreign_keys.is_empty(),
                        status_dot: None,
                        badge_text: None,
                        badge_accent: false,
                        count_text: Some(info.foreign_keys.len().to_string()),
                        detail_text: None,
                    },
                );
                if fk_resp.clicked() || fk_chevron {
                    fk_collapsing.set_open(!fk_open);
                    fk_collapsing.store(ui.ctx());
                }

                if fk_collapsing.is_open() {
                    if info.foreign_keys.is_empty() {
                        draw_codex_tree_row(
                            ui,
                            &self.theme,
                            CodexTreeRow {
                                depth: 6,
                                is_expandable: false,
                                is_expanded: false,
                                icon: Icon::Info,
                                icon_color: self.theme.text_muted,
                                label: "No foreign keys",
                                is_selected: false,
                                is_dimmed: true,
                                status_dot: None,
                                badge_text: None,
                                badge_accent: false,
                                count_text: None,
                                detail_text: None,
                            },
                        );
                    } else {
                        for fk in &info.foreign_keys {
                            draw_codex_tree_row(
                                ui,
                                &self.theme,
                                CodexTreeRow {
                                    depth: 6,
                                    is_expandable: false,
                                    is_expanded: false,
                                    icon: Icon::Link,
                                    icon_color: self.theme.info,
                                    label: &fk.name,
                                    is_selected: false,
                                    is_dimmed: false,
                                    status_dot: None,
                                    badge_text: None,
                                    badge_accent: false,
                                    count_text: None,
                                    detail_text: Some(&fk.to_table),
                                },
                            );
                        }
                    }
                }

                // ── Indexes Folder ──────────────────────────────────────
                let idx_id = ui.make_persistent_id(("codex_tbl_idx_folder", table));
                let mut idx_collapsing =
                    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), idx_id, false);
                let idx_open = idx_collapsing.is_open();

                let (idx_resp, idx_chevron) = draw_codex_tree_row(
                    ui,
                    &self.theme,
                    CodexTreeRow {
                        depth: 5,
                        is_expandable: true,
                        is_expanded: idx_open,
                        icon: Icon::List,
                        icon_color: self.theme.text_secondary,
                        label: "Indexes",
                        is_selected: false,
                        is_dimmed: info.indexes.is_empty(),
                        status_dot: None,
                        badge_text: None,
                        badge_accent: false,
                        count_text: Some(info.indexes.len().to_string()),
                        detail_text: None,
                    },
                );
                if idx_resp.clicked() || idx_chevron {
                    idx_collapsing.set_open(!idx_open);
                    idx_collapsing.store(ui.ctx());
                }

                if idx_collapsing.is_open() {
                    if info.indexes.is_empty() {
                        draw_codex_tree_row(
                            ui,
                            &self.theme,
                            CodexTreeRow {
                                depth: 6,
                                is_expandable: false,
                                is_expanded: false,
                                icon: Icon::Info,
                                icon_color: self.theme.text_muted,
                                label: "No indexes",
                                is_selected: false,
                                is_dimmed: true,
                                status_dot: None,
                                badge_text: None,
                                badge_accent: false,
                                count_text: None,
                                detail_text: None,
                            },
                        );
                    } else {
                        for idx in &info.indexes {
                            draw_codex_tree_row(
                                ui,
                                &self.theme,
                                CodexTreeRow {
                                    depth: 6,
                                    is_expandable: false,
                                    is_expanded: false,
                                    icon: Icon::Zap,
                                    icon_color: self.theme.text_muted,
                                    label: &idx.name,
                                    is_selected: false,
                                    is_dimmed: false,
                                    status_dot: None,
                                    badge_text: None,
                                    badge_accent: false,
                                    count_text: None,
                                    detail_text: None,
                                },
                            );
                        }
                    }
                }
            }
        }
    }

    /// Views folder in Codex tree.
    fn draw_dbeaver_views_folder(&mut self, ui: &mut egui::Ui, views: &[UiViewSummary]) {
        let dimmed = views.is_empty();
        let folder_id = ui.make_persistent_id("codex_views_folder");

        let mut collapsing =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), folder_id, false);
        let is_open = collapsing.is_open();

        let (resp, chevron_clicked) = draw_codex_tree_row(
            ui,
            &self.theme,
            CodexTreeRow {
                depth: 3,
                is_expandable: true,
                is_expanded: is_open,
                icon: Icon::Eye,
                icon_color: self.theme.success,
                label: "Views",
                is_selected: false,
                is_dimmed: dimmed,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: Some(views.len().to_string()),
                detail_text: None,
            },
        );

        if resp.clicked() || chevron_clicked {
            collapsing.set_open(!is_open);
            collapsing.store(ui.ctx());
        }

        if collapsing.is_open() {
            if dimmed {
                draw_codex_tree_row(
                    ui,
                    &self.theme,
                    CodexTreeRow {
                        depth: 4,
                        is_expandable: false,
                        is_expanded: false,
                        icon: Icon::Info,
                        icon_color: self.theme.text_muted,
                        label: "No views in schema",
                        is_selected: false,
                        is_dimmed: true,
                        status_dot: None,
                        badge_text: None,
                        badge_accent: false,
                        count_text: None,
                        detail_text: None,
                    },
                );
            } else {
                for view in views {
                    let is_selected = matches!(
                        self.selected_schema_object.as_ref(),
                        Some(SchemaObjectSelection::View(s)) if s == &view.name
                    );

                    let (v_resp, _) = draw_codex_tree_row(
                        ui,
                        &self.theme,
                        CodexTreeRow {
                            depth: 4,
                            is_expandable: false,
                            is_expanded: false,
                            icon: Icon::Eye,
                            icon_color: if is_selected {
                                self.theme.accent
                            } else {
                                self.theme.success
                            },
                            label: &view.name,
                            is_selected,
                            is_dimmed: false,
                            status_dot: None,
                            badge_text: None,
                            badge_accent: false,
                            count_text: None,
                            detail_text: None,
                        },
                    );

                    let mut open_query = false;
                    v_resp.context_menu(|ui| {
                        if ui.button("Open in Query").clicked() {
                            open_query = true;
                            ui.close_menu();
                        }
                    });

                    if v_resp.clicked() {
                        self.selected_schema_object = Some(SchemaObjectSelection::View(view.name.clone()));
                        self.schema_object_view = SchemaObjectView::Definition;
                        self.selected_table = None;
                        self.table_info = None;
                        self.table_ddl = None;
                        self.table_view = TableView::Ddl;
                        self.active_tab = WorkspaceTab::SchemaObject;
                        self.runtime_message = format!("Opened view {}.{}", view.schema, view.name);
                    }
                    if open_query {
                        self.query_text = format!("SELECT *\nFROM {}\nLIMIT 100;", view.name);
                        self.active_tab = WorkspaceTab::Query;
                    }
                }
            }
        }
    }

    /// Functions folder in Codex tree.
    fn draw_dbeaver_functions_folder(&mut self, ui: &mut egui::Ui, functions: &[UiFunctionSummary]) {
        let dimmed = functions.is_empty();
        let folder_id = ui.make_persistent_id("codex_functions_folder");

        let mut collapsing =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), folder_id, false);
        let is_open = collapsing.is_open();

        let (resp, chevron_clicked) = draw_codex_tree_row(
            ui,
            &self.theme,
            CodexTreeRow {
                depth: 3,
                is_expandable: true,
                is_expanded: is_open,
                icon: Icon::Code2,
                icon_color: self.theme.accent,
                label: "Functions",
                is_selected: false,
                is_dimmed: dimmed,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: Some(functions.len().to_string()),
                detail_text: None,
            },
        );

        if resp.clicked() || chevron_clicked {
            collapsing.set_open(!is_open);
            collapsing.store(ui.ctx());
        }

        if collapsing.is_open() {
            if dimmed {
                draw_codex_tree_row(
                    ui,
                    &self.theme,
                    CodexTreeRow {
                        depth: 4,
                        is_expandable: false,
                        is_expanded: false,
                        icon: Icon::Info,
                        icon_color: self.theme.text_muted,
                        label: "No functions in schema",
                        is_selected: false,
                        is_dimmed: true,
                        status_dot: None,
                        badge_text: None,
                        badge_accent: false,
                        count_text: None,
                        detail_text: None,
                    },
                );
            } else {
                for function in functions {
                    let is_selected = matches!(
                        self.selected_schema_object.as_ref(),
                        Some(SchemaObjectSelection::Function(s)) if s == &function.name
                    );
                    let icon = if function.routine_type.eq_ignore_ascii_case("procedure") {
                        Icon::GitBranch
                    } else {
                        Icon::Code2
                    };
                    let label = format!("{} · {}", function.name, function.routine_type);

                    let (f_resp, _) = draw_codex_tree_row(
                        ui,
                        &self.theme,
                        CodexTreeRow {
                            depth: 4,
                            is_expandable: false,
                            is_expanded: false,
                            icon,
                            icon_color: self.theme.accent,
                            label: &label,
                            is_selected,
                            is_dimmed: false,
                            status_dot: None,
                            badge_text: None,
                            badge_accent: false,
                            count_text: None,
                            detail_text: None,
                        },
                    );

                    let mut open_query = false;
                    f_resp.context_menu(|ui| {
                        if ui.button("Open call in Query").clicked() {
                            open_query = true;
                            ui.close_menu();
                        }
                    });

                    if f_resp.clicked() {
                        self.selected_schema_object = Some(SchemaObjectSelection::Function(function.name.clone()));
                        self.schema_object_view = SchemaObjectView::Definition;
                        self.selected_table = None;
                        self.table_info = None;
                        self.table_ddl = None;
                        self.table_view = TableView::Ddl;
                        self.active_tab = WorkspaceTab::SchemaObject;
                        self.runtime_message = format!("Opened function {}.{}", function.schema, function.name);
                    }
                    if open_query {
                        self.query_text = format!("SELECT * FROM {}.{}();", function.schema, function.name);
                        self.active_tab = WorkspaceTab::Query;
                    }
                }
            }
        }
    }

    /// Triggers folder in Codex tree.
    fn draw_dbeaver_triggers_folder(&mut self, ui: &mut egui::Ui, triggers: &[UiTriggerSummary]) {
        let dimmed = triggers.is_empty();
        let folder_id = ui.make_persistent_id("codex_triggers_folder");

        let mut collapsing =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), folder_id, false);
        let is_open = collapsing.is_open();

        let (resp, chevron_clicked) = draw_codex_tree_row(
            ui,
            &self.theme,
            CodexTreeRow {
                depth: 3,
                is_expandable: true,
                is_expanded: is_open,
                icon: Icon::Zap,
                icon_color: self.theme.warning,
                label: "Triggers",
                is_selected: false,
                is_dimmed: dimmed,
                status_dot: None,
                badge_text: None,
                badge_accent: false,
                count_text: Some(triggers.len().to_string()),
                detail_text: None,
            },
        );

        if resp.clicked() || chevron_clicked {
            collapsing.set_open(!is_open);
            collapsing.store(ui.ctx());
        }

        if collapsing.is_open() {
            if dimmed {
                draw_codex_tree_row(
                    ui,
                    &self.theme,
                    CodexTreeRow {
                        depth: 4,
                        is_expandable: false,
                        is_expanded: false,
                        icon: Icon::Info,
                        icon_color: self.theme.text_muted,
                        label: "No triggers in schema",
                        is_selected: false,
                        is_dimmed: true,
                        status_dot: None,
                        badge_text: None,
                        badge_accent: false,
                        count_text: None,
                        detail_text: None,
                    },
                );
            } else {
                for trigger in triggers {
                    let is_selected = matches!(
                        self.selected_schema_object.as_ref(),
                        Some(SchemaObjectSelection::Trigger(s)) if s == &trigger.name
                    );
                    let label = format!("{} · {}", trigger.name, trigger.event);

                    let (t_resp, _) = draw_codex_tree_row(
                        ui,
                        &self.theme,
                        CodexTreeRow {
                            depth: 4,
                            is_expandable: false,
                            is_expanded: false,
                            icon: Icon::Zap,
                            icon_color: if is_selected {
                                self.theme.accent
                            } else {
                                self.theme.warning
                            },
                            label: &label,
                            is_selected,
                            is_dimmed: false,
                            status_dot: None,
                            badge_text: None,
                            badge_accent: false,
                            count_text: None,
                            detail_text: None,
                        },
                    );

                    if t_resp.clicked() {
                        self.selected_schema_object = Some(SchemaObjectSelection::Trigger(trigger.name.clone()));
                        self.schema_object_view = SchemaObjectView::Definition;
                        self.selected_table = None;
                        self.table_info = None;
                        self.table_ddl = None;
                        self.table_view = TableView::Ddl;
                        self.active_tab = WorkspaceTab::SchemaObject;
                        self.runtime_message = format!("Opened trigger {}", trigger.name);
                    }
                }
            }
        }
    }

    /// Schema loading progress and error banner.
    fn draw_explorer_schema_feedback(&mut self, ui: &mut egui::Ui) {
        let schema_error = self.schema_error.clone();
        if let Some(error) = schema_error.as_deref() {
            grid_frame(self.theme).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(icon_text(Icon::TriangleAlert, "Schema load failed", self.theme.danger));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if let Some(connection_id) = self.active_connection_id.clone() {
                            if secondary_button_with_icon(ui, Icon::RotateCcw, "Refresh schema", self.theme).clicked() {
                                self.request_schema_introspection(connection_id, true);
                            }
                        }
                    });
                });
                ui.add_space(6.0);
                egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                    ui.label(
                        RichText::new(error)
                            .small()
                            .monospace()
                            .color(self.theme.text_secondary),
                    );
                });
            });
            ui.add_space(8.0);
        } else if self.schema_request.is_some() {
            grid_frame(self.theme).show(ui, |ui| {
                ui.horizontal(|ui| {
                    if self.reduce_motion {
                        ui.label(icon_text(Icon::LoaderCircle, "Loading schema…", self.theme.accent));
                    } else {
                        ui.spinner();
                        ui.label(RichText::new("Loading schema…").color(self.theme.accent));
                    }
                    ui.label(
                        RichText::new("Large databases may take a moment.")
                            .small()
                            .color(self.theme.text_muted),
                    );
                });
            });
            ui.add_space(8.0);
        }
    }

    /// Helper to initiate connection logic.
    fn connect_to_connection(&mut self, connection: &UiConnectionSummary) {
        self.reset_agent_context();
        self.active_connection_id = Some(connection.id.clone());
        self.selected_schema = None;
        self.schema = UiSchemaSummary::default();
        self.selected_table = None;
        self.selected_schema_object = None;
        self.table_info = None;
        self.table_ddl = None;
        self.table_info_error = None;
        self.table_ddl_error = None;
        self.ddl_execute_confirmation = false;
        self.ddl_execution_request = None;
        self.table_data_result = None;
        self.table_data_total_rows = None;
        self.table_data_offset = 0;
        self.table_data_filter_column.clear();
        self.table_data_filter_value.clear();
        self.table_data_sort_column = None;
        self.table_data_sort_desc = false;
        self.table_data_error = None;
        self.table_info_request = None;
        self.table_ddl_request = None;
        self.table_data_request = None;
        self.table_mutation_request = None;
        self.staged_changes.clear();
        self.staged_apply_request = None;
        self.selected_cell = None;
        self.selected_row = None;
        self.data_editing_cell = None;
        self.data_edit_value.clear();
        self.data_delete_confirmation = false;
        self.table_view = TableView::Structure;
        self.explorer_search.clear();
        let request_id = self.task_bridge.next_request_id();
        self.connected = false;
        self.pending_connection_request = Some(request_id);
        self.schema_request = None;
        self.schema_error = None;
        let _ = self.task_bridge.send(UiCommand::Connect {
            request_id,
            connection_id: connection.id.clone(),
        });
        self.runtime_message = format!("Connecting to {}…", connection.name);
    }
}
