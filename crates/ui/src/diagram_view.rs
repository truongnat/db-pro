use super::*;

impl DbProApp {
    pub(super) fn draw_diagram(&mut self, ui: &mut egui::Ui) {
        let all_tables = self.schema.table_details.clone();
        let large_schema = all_tables.len() > ER_LARGE_SCHEMA_THRESHOLD;
        let search_query = self.diagram_search.trim().to_ascii_lowercase();
        let search_mode = large_schema && !self.diagram_show_all;
        let tables = if search_mode && !search_query.is_empty() {
            all_tables
                .iter()
                .filter(|table| matches_diagram_search(table, &search_query))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            all_tables.clone()
        };
        let render_limit = if large_schema && self.diagram_show_all {
            tables.len()
        } else {
            ER_MAX_TABLES
        };
        let relationship_count: usize = tables.iter().map(|table| table.foreign_keys.len()).sum();
        let visible_tables = tables.len().min(render_limit);

        ui.horizontal_wrapped(|ui| {
            section_label(ui, "ER DIAGRAM", self.theme);
            ui.add_space(8.0);
            badge(
                ui,
                &format!("{visible_tables} tables"),
                self.theme.accent_soft,
                self.theme.accent,
            );
            badge(
                ui,
                &format!("{} relationships", relationship_count.min(ER_MAX_EDGES)),
                self.theme.surface_hover,
                self.theme.text_secondary,
            );
            if tables.len() > render_limit {
                ui.label(
                    RichText::new(format!("+ {} more outside view", tables.len() - render_limit))
                        .small()
                        .color(self.theme.text_muted),
                );
            }
            if large_schema {
                ui.separator();
                input(ui, &mut self.diagram_search, "Find table or column…", 220.0, self.theme);
                if search_mode {
                    if secondary_button_with_icon(
                        ui,
                        Icon::Workflow,
                        &format!("Show all {} tables", all_tables.len()),
                        self.theme,
                    )
                    .clicked()
                    {
                        self.diagram_show_all = true;
                    }
                } else {
                    badge(ui, "All tables", self.theme.warning, self.theme.text_inverse);
                }
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if compact_button(ui, "+", self.theme).clicked() {
                    self.diagram_zoom = (self.diagram_zoom + 0.1).clamp(0.7, 1.5);
                }
                ui.label(
                    RichText::new(format!("{:.0}%", self.diagram_zoom * 100.0))
                        .small()
                        .color(self.theme.text_secondary),
                );
                if compact_button(ui, "−", self.theme).clicked() {
                    self.diagram_zoom = (self.diagram_zoom - 0.1).clamp(0.7, 1.5);
                }
                if secondary_button_with_icon(ui, Icon::Square, "Fit", self.theme).clicked() {
                    self.diagram_zoom = 1.0;
                    self.diagram_pan = egui::Vec2::ZERO;
                }
            });
        });
        ui.add_space(10.0);

        if search_mode && search_query.is_empty() {
            card_frame(self.theme).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.label(icon_text(
                        Icon::Search,
                        "Focus the schema map",
                        self.theme.text_secondary,
                    ));
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!(
                            "This schema has {} tables. Search by table or column to open a focused map, or show all tables explicitly.",
                            all_tables.len()
                        ))
                        .small()
                        .color(self.theme.text_muted),
                    );
                    ui.add_space(40.0);
                });
            });
            return;
        }

        if tables.is_empty() {
            let no_matches = search_mode && !search_query.is_empty();
            card_frame(self.theme).show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(56.0);
                    ui.label(icon_text(
                        if no_matches { Icon::Search } else { Icon::Workflow },
                        if no_matches {
                            "No matching tables"
                        } else {
                            "No schema map yet"
                        },
                        self.theme.text_secondary,
                    ));
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(if no_matches {
                            "Try a different table or column name."
                        } else {
                            "Connect to a database and load its tables to see the relationship map."
                        })
                        .small()
                        .color(self.theme.text_muted),
                    );
                    if no_matches {
                        ui.add_space(10.0);
                        if compact_button(ui, "Clear search", self.theme).clicked() {
                            self.diagram_search.clear();
                        }
                    }
                    ui.add_space(56.0);
                });
            });
            return;
        }

        let grid_columns = visible_tables.clamp(1, 3);
        let grid_rows = visible_tables.div_ceil(grid_columns);
        let max_visible_columns = tables
            .iter()
            .take(render_limit)
            .map(|table| table.columns.len().clamp(1, ER_MAX_COLUMNS))
            .max()
            .unwrap_or(1);
        let node_height = ER_HEADER_HEIGHT + ER_ROW_HEIGHT * max_visible_columns as f32;
        let canvas_width = (ER_CANVAS_MARGIN * 2.0
            + grid_columns as f32 * ER_NODE_WIDTH
            + (grid_columns.saturating_sub(1)) as f32 * ER_GAP_X)
            * self.diagram_zoom;
        let canvas_height =
            (ER_CANVAS_MARGIN * 2.0 + grid_rows as f32 * node_height + (grid_rows.saturating_sub(1)) as f32 * ER_GAP_Y)
                * self.diagram_zoom;
        let zoom = self.diagram_zoom;
        let pan = self.diagram_pan;
        let theme = self.theme;

        panel_frame(self.theme).show(ui, |ui| {
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                let (response, painter) = ui.allocate_painter(
                    egui::vec2(canvas_width.max(640.0), canvas_height.max(360.0)),
                    Sense::click_and_drag(),
                );
                let origin = response.rect.min + pan;
                let nodes = tables
                    .iter()
                    .take(render_limit)
                    .enumerate()
                    .map(|(index, table)| {
                        let column = index % grid_columns;
                        let row = index / grid_columns;
                        let position = origin
                            + egui::vec2(
                                (ER_CANVAS_MARGIN + column as f32 * (ER_NODE_WIDTH + ER_GAP_X)) * zoom,
                                (ER_CANVAS_MARGIN + row as f32 * (node_height + ER_GAP_Y)) * zoom,
                            );
                        ErNode {
                            table: table.clone(),
                            rect: egui::Rect::from_min_size(
                                position,
                                egui::vec2(ER_NODE_WIDTH * zoom, node_height * zoom),
                            ),
                        }
                    })
                    .collect::<Vec<_>>();

                let node_lookup = nodes
                    .iter()
                    .enumerate()
                    .map(|(index, node)| ((node.table.schema.as_str(), node.table.name.as_str()), index))
                    .collect::<HashMap<_, _>>();
                let mut drawn_edges = 0;
                for source_node in &nodes {
                    for foreign_key in &source_node.table.foreign_keys {
                        if drawn_edges >= ER_MAX_EDGES {
                            break;
                        }
                        let Some(target_index) =
                            node_lookup.get(&(foreign_key.to_schema.as_str(), foreign_key.to_table.as_str()))
                        else {
                            continue;
                        };
                        let target_node = &nodes[*target_index];
                        let source_on_right = source_node.rect.center().x < target_node.rect.center().x;
                        let from = er_column_anchor(
                            source_node,
                            foreign_key.from_columns.first().map(String::as_str),
                            source_on_right,
                            zoom,
                        );
                        let to = er_column_anchor(
                            target_node,
                            foreign_key.to_columns.first().map(String::as_str),
                            !source_on_right,
                            zoom,
                        );
                        let bend_x = (from.x + to.x) / 2.0;
                        let bend_a = egui::pos2(bend_x, from.y);
                        let bend_b = egui::pos2(bend_x, to.y);
                        let stroke = egui::Stroke::new(1.2, theme.accent);
                        painter.line_segment([from, bend_a], stroke);
                        painter.line_segment([bend_a, bend_b], stroke);
                        painter.line_segment([bend_b, to], stroke);
                        let label = format!(
                            "{} → {}",
                            foreign_key.from_columns.first().map(String::as_str).unwrap_or("key"),
                            foreign_key.to_columns.first().map(String::as_str).unwrap_or("key"),
                        );
                        let label_position = egui::pos2(bend_x, (from.y + to.y) / 2.0);
                        let label_galley =
                            painter.layout_no_wrap(label.clone(), FontId::proportional(10.0), theme.text_secondary);
                        let label_rect =
                            egui::Rect::from_center_size(label_position, label_galley.size() + egui::vec2(8.0, 4.0));
                        painter.rect_filled(label_rect, egui::Rounding::same(3.0), theme.surface_panel);
                        painter.text(
                            label_position,
                            egui::Align2::CENTER_CENTER,
                            label,
                            FontId::proportional(10.0),
                            theme.text_secondary,
                        );
                        drawn_edges += 1;
                    }
                }

                for node in &nodes {
                    let selected = self.selected_table.as_deref() == Some(node.table.name.as_str());
                    painter.rect_filled(node.rect, egui::Rounding::same(8.0), theme.surface_panel);
                    painter.rect_stroke(
                        node.rect,
                        egui::Rounding::same(8.0),
                        egui::Stroke::new(1.0, if selected { theme.accent } else { theme.border_default }),
                    );
                    let header_rect = egui::Rect::from_min_max(
                        node.rect.min,
                        egui::pos2(node.rect.max.x, node.rect.min.y + ER_HEADER_HEIGHT * zoom),
                    );
                    painter.rect_filled(
                        header_rect,
                        egui::Rounding::same(8.0),
                        if selected {
                            theme.accent_soft
                        } else {
                            theme.surface_hover
                        },
                    );
                    painter.rect_filled(
                        egui::Rect::from_min_max(
                            egui::pos2(header_rect.min.x, header_rect.max.y - 8.0 * zoom),
                            header_rect.max,
                        ),
                        egui::Rounding::ZERO,
                        if selected {
                            theme.accent_soft
                        } else {
                            theme.surface_hover
                        },
                    );
                    painter.text(
                        node.rect.min + egui::vec2(14.0 * zoom, 20.0 * zoom),
                        egui::Align2::LEFT_CENTER,
                        format!("{}.{}", node.table.schema, node.table.name),
                        FontId::proportional(13.0 * zoom),
                        theme.text_primary,
                    );
                    painter.text(
                        egui::pos2(node.rect.max.x - 12.0 * zoom, node.rect.min.y + 20.0 * zoom),
                        egui::Align2::RIGHT_CENTER,
                        "TABLE",
                        FontId::proportional(9.0 * zoom),
                        theme.text_muted,
                    );

                    for (index, column) in node.table.columns.iter().take(ER_MAX_COLUMNS).enumerate() {
                        let row_top = node.rect.min.y + (ER_HEADER_HEIGHT + index as f32 * ER_ROW_HEIGHT) * zoom;
                        let row_rect = egui::Rect::from_min_max(
                            egui::pos2(node.rect.min.x, row_top),
                            egui::pos2(node.rect.max.x, row_top + ER_ROW_HEIGHT * zoom),
                        );
                        if index % 2 == 0 {
                            painter.rect_filled(row_rect, egui::Rounding::ZERO, theme.surface_app);
                        }
                        let is_foreign_key = node
                            .table
                            .foreign_keys
                            .iter()
                            .any(|foreign_key| foreign_key.from_columns.iter().any(|name| name == &column.name));
                        let marker_color = if column.is_primary_key {
                            theme.warning
                        } else if is_foreign_key {
                            theme.accent
                        } else {
                            theme.border_strong
                        };
                        painter.circle_filled(
                            egui::pos2(row_rect.min.x + 13.0 * zoom, row_rect.center().y),
                            3.0 * zoom,
                            marker_color,
                        );
                        painter.text(
                            egui::pos2(row_rect.min.x + 23.0 * zoom, row_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            &column.name,
                            FontId::proportional(11.0 * zoom),
                            theme.text_primary,
                        );
                        painter.text(
                            egui::pos2(row_rect.max.x - 12.0 * zoom, row_rect.center().y),
                            egui::Align2::RIGHT_CENTER,
                            &column.data_type,
                            FontId::monospace(10.0 * zoom),
                            theme.text_secondary,
                        );
                    }
                    if node.table.columns.len() > ER_MAX_COLUMNS {
                        painter.text(
                            egui::pos2(
                                node.rect.min.x + 14.0 * zoom,
                                node.rect.min.y + (ER_HEADER_HEIGHT + ER_ROW_HEIGHT * 7.5) * zoom,
                            ),
                            egui::Align2::LEFT_CENTER,
                            format!("+ {} more columns", node.table.columns.len() - ER_MAX_COLUMNS),
                            FontId::proportional(10.0 * zoom),
                            theme.text_muted,
                        );
                    }
                }

                if response.drag_started() {
                    self.diagram_pan_origin = Some(self.diagram_pan);
                }
                if response.dragged() {
                    if let Some(origin) = self.diagram_pan_origin {
                        self.diagram_pan = origin + response.drag_delta();
                    }
                }
                if response.drag_stopped() {
                    self.diagram_pan_origin = None;
                }

                if response.clicked() {
                    if let Some(pointer) = response.interact_pointer_pos() {
                        if let Some(node) = nodes.iter().find(|node| node.rect.contains(pointer)) {
                            self.selected_table = Some(node.table.name.clone());
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
                            self.selected_cell = None;
                            self.selected_row = None;
                            self.data_editing_cell = None;
                            self.data_edit_value.clear();
                            self.data_delete_confirmation = false;
                            self.table_view = TableView::Structure;
                            self.query_text = format!("SELECT *\nFROM {}\nLIMIT 100;", node.table.name);
                            self.request_table_info();
                            self.activity = Activity::Explorer;
                            self.sidebar_open = true;
                            self.active_tab = WorkspaceTab::Table;
                        }
                    }
                }
            });
        });
    }
}
