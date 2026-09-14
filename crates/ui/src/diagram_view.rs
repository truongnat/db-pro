use super::*;
pub use crate::diagram::*;

impl DbProApp {
    pub(super) fn draw_diagram(&mut self, ui: &mut egui::Ui) {
        let all_table_count = self.schema.table_details.len();
        let large_schema = all_table_count > ER_LARGE_SCHEMA_THRESHOLD;
        let search_query = self.diagram_search.trim().to_ascii_lowercase();
        let search_mode = diagram_search_mode(large_schema, self.diagram_show_all);

        if self.schema.table_details.is_empty() {
            self.draw_diagram_empty_state(ui, 0, false, false);
            return;
        }

        let render_limit = if !large_schema || self.diagram_show_all {
            all_table_count
        } else {
            ER_MAX_TABLES
        };

        let (_candidate_count, tables) =
            diagram_candidates(&self.schema.table_details, &search_query, search_mode, render_limit);

        let grid_columns = if tables.len() <= 3 {
            tables.len().max(1)
        } else {
            ((all_table_count as f32).sqrt().ceil() as usize).clamp(3, 10)
        };

        let max_visible_columns = self
            .schema
            .table_details
            .iter()
            .take(50)
            .map(|table| table.columns.len().clamp(1, ER_MAX_COLUMNS))
            .max()
            .unwrap_or(1);
        let node_height = ER_HEADER_HEIGHT + ER_ROW_HEIGHT * max_visible_columns as f32;

        self.ensure_diagram_graph(grid_columns, node_height);

        // Determine active table subset:
        let active_node_indices: Option<Vec<usize>> = if search_mode && !search_query.is_empty() {
            let seed_indices: Vec<usize> = self
                .diagram_graph
                .nodes
                .iter()
                .filter(|node| matches_diagram_search(&node.table, &search_query))
                .map(|node| node.id)
                .collect();
            if seed_indices.is_empty() {
                self.draw_diagram_toolbar(ui, large_schema, all_table_count, &tables, render_limit);
                ui.add_space(10.0);
                self.draw_diagram_empty_state(ui, all_table_count, true, true);
                return;
            }
            Some(
                self.diagram_graph
                    .bfs_neighborhood(&seed_indices, self.diagram_neighborhood_depth, 100),
            )
        } else if search_mode && search_query.is_empty() {
            self.draw_diagram_toolbar(ui, large_schema, all_table_count, &tables, render_limit);
            ui.add_space(10.0);
            self.draw_diagram_empty_state(ui, all_table_count, true, false);
            return;
        } else {
            None
        };

        self.draw_diagram_toolbar(ui, large_schema, all_table_count, &tables, render_limit);
        ui.add_space(10.0);

        self.draw_diagram_canvas(ui, active_node_indices.as_deref());
    }

    fn ensure_diagram_graph(&mut self, grid_columns: usize, node_height: f32) {
        let current_count = self.schema.table_details.len();
        let graph_dirty = self.diagram_graph.nodes.len() != current_count
            || self.diagram_graph.schema_version != self.diagram_schema_version;

        if graph_dirty && current_count > 0 {
            self.diagram_graph = ErGraph::build(
                &self.schema.table_details,
                self.diagram_schema_version,
                grid_columns,
                node_height,
            );
            self.diagram_spatial_index = ErSpatialIndex::build(
                &self.diagram_graph.nodes,
                &self.diagram_graph.edges,
                DEFAULT_SPATIAL_CELL_SIZE,
            );
        }
    }
}

pub(super) fn diagram_candidates(
    all_tables: &[UiTableSummary],
    search_query: &str,
    search_mode: bool,
    render_limit: usize,
) -> (usize, Vec<UiTableSummary>) {
    let candidate_count = if search_mode && !search_query.is_empty() {
        all_tables
            .iter()
            .filter(|table| matches_diagram_search(table, search_query))
            .count()
    } else {
        all_tables.len()
    };
    let tables = if search_mode {
        all_tables
            .iter()
            .filter(|table| !search_query.is_empty() && matches_diagram_search(table, search_query))
            .take(render_limit + 1)
            .cloned()
            .collect()
    } else {
        all_tables
            .iter()
            .take(render_limit.saturating_add(1))
            .cloned()
            .collect()
    };
    (candidate_count, tables)
}

pub(super) fn diagram_search_mode(large_schema: bool, show_all: bool) -> bool {
    large_schema && !show_all
}

pub(super) fn diagram_show_all_after_search_edit(show_all: bool, search_query: &str, changed: bool) -> bool {
    if changed && !search_query.trim().is_empty() {
        false
    } else {
        show_all
    }
}

impl DbProApp {
    fn draw_diagram_toolbar(
        &mut self,
        ui: &mut egui::Ui,
        large_schema: bool,
        all_table_count: usize,
        tables: &[UiTableSummary],
        render_limit: usize,
    ) {
        let visible_tables = if self.diagram_show_all || !large_schema {
            all_table_count
        } else {
            tables.len().min(render_limit)
        };
        let relationship_count: usize = self.diagram_graph.edges.len();

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
                &format!("{relationship_count} relationships"),
                self.theme.surface_hover,
                self.theme.text_secondary,
            );
            if large_schema {
                ui.separator();
                let search_changed =
                    input(ui, &mut self.diagram_search, "Find table or column…", 220.0, self.theme).changed();
                self.diagram_show_all =
                    diagram_show_all_after_search_edit(self.diagram_show_all, &self.diagram_search, search_changed);
                let search_mode = diagram_search_mode(large_schema, self.diagram_show_all);
                if search_mode {
                    ui.label(RichText::new("Neighborhood:").small().color(self.theme.text_muted));
                    if ui
                        .selectable_label(self.diagram_neighborhood_depth == 1, "1 hop")
                        .clicked()
                    {
                        self.diagram_neighborhood_depth = 1;
                    }
                    if ui
                        .selectable_label(self.diagram_neighborhood_depth == 2, "2 hops")
                        .clicked()
                    {
                        self.diagram_neighborhood_depth = 2;
                    }
                    ui.add_space(4.0);
                    if secondary_button_with_icon(
                        ui,
                        Icon::Workflow,
                        &format!("Show all {all_table_count} tables"),
                        self.theme,
                    )
                    .clicked()
                    {
                        self.diagram_show_all = true;
                    }
                } else {
                    badge(ui, "All tables", self.theme.warning, self.theme.text_inverse);
                    if compact_button_with_icon(ui, Icon::Search, "Focus search", self.theme).clicked() {
                        self.diagram_show_all = false;
                    }
                }
            }
        });
    }

    fn draw_diagram_empty_state(
        &mut self,
        ui: &mut egui::Ui,
        all_table_count: usize,
        search_mode: bool,
        no_matches: bool,
    ) {
        let canvas_size = egui::vec2(ui.available_width(), ui.available_height().max(360.0));
        egui::Frame {
            fill: self.theme.surface_editor,
            inner_margin: egui::Margin::ZERO,
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.set_min_size(canvas_size);
            paint_diagram_grid(ui.painter(), ui.max_rect(), 1.0, self.theme);
            ui.vertical_centered(|ui| {
                let top_space = if no_matches { 56.0 } else { 40.0 };
                ui.add_space(top_space);
                ui.label(icon_text(
                    if no_matches { Icon::Search } else { Icon::Workflow },
                    if no_matches {
                        "No matching tables"
                    } else if search_mode {
                        "Focus the schema map"
                    } else {
                        "No schema map yet"
                    },
                    self.theme.text_secondary,
                ));
                ui.add_space(8.0);
                let description = if no_matches {
                    "Try a different table or column name.".to_owned()
                } else if search_mode {
                    format!(
                        "This schema has {all_table_count} tables. Search by table or column to open a focused neighborhood map, or show all tables explicitly."
                    )
                } else {
                    "Connect to a database and load its tables to see the relationship map.".to_owned()
                };
                ui.label(RichText::new(description).small().color(self.theme.text_muted));
                if no_matches {
                    ui.add_space(10.0);
                    if compact_button(ui, "Clear search", self.theme).clicked() {
                        self.diagram_search.clear();
                    }
                }
                ui.add_space(56.0);
            });
        });
    }
}

impl DbProApp {
    fn draw_diagram_canvas(&mut self, ui: &mut egui::Ui, active_filter: Option<&[usize]>) {
        let world_size = self.diagram_graph.world_bounds.size();
        let viewport_size = egui::vec2(ui.available_width(), ui.available_height());
        let canvas_size = diagram_canvas_size(world_size * self.diagram_zoom, viewport_size);
        let zoom = self.diagram_zoom;
        let pan = self.diagram_pan;
        let theme = self.theme;

        egui::Frame {
            fill: theme.surface_editor,
            inner_margin: egui::Margin::ZERO,
            ..Default::default()
        }
        .show(ui, |ui| {
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                let (response, painter) = ui.allocate_painter(canvas_size, Sense::click_and_drag());
                paint_diagram_grid(&painter, response.rect, zoom, theme);

                let viewport = ErViewport::new(pan, zoom, response.rect.min);
                let scene = prepare_render_scene(
                    &self.diagram_graph,
                    &self.diagram_spatial_index,
                    &viewport,
                    response.rect,
                    active_filter,
                );

                // Paint visible edges:
                paint_scene_edges(&painter, &self.diagram_graph, &scene, &viewport, theme);

                // Paint visible nodes:
                let selected_table_name = self.selected_table.as_deref();
                for &node_id in &scene.visible_nodes {
                    if let Some(node) = self.diagram_graph.nodes.get(node_id) {
                        let screen_rect = viewport.world_to_screen_rect(node.world_rect);
                        let selected = selected_table_name == Some(node.table.name.as_str());
                        paint_er_node_lod(&painter, node, screen_rect, selected, scene.lod, zoom, theme);
                    }
                }

                draw_diagram_zoom_controls(ui, response.rect, &mut self.diagram_zoom, &mut self.diagram_pan, theme);
                self.update_diagram_pan(&response);

                if response.clicked() {
                    if let Some(pointer) = response.interact_pointer_pos() {
                        let world_pos = viewport.screen_to_world_pos(pointer);
                        if let Some(hit_id) = self
                            .diagram_spatial_index
                            .hit_test_node(world_pos, &self.diagram_graph.nodes)
                        {
                            let table_name = self.diagram_graph.nodes.get(hit_id).map(|node| node.table.name.clone());
                            if let Some(name) = table_name {
                                self.open_diagram_table(&name);
                            }
                        }
                    }
                }
            });
        });
    }

    fn update_diagram_pan(&mut self, response: &egui::Response) {
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
    }

    fn open_diagram_table(&mut self, table: &str) {
        if self.selected_table.as_deref() != Some(table) && !self.staged_changes.is_empty() {
            self.runtime_message = "Apply or discard staged changes before opening another table".to_owned();
            return;
        }
        self.persist_current_grid_layout();
        self.selected_table = Some(table.to_owned());
        self.restore_grid_layout_for_active_table();
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
        self.table_data_filter_operator = UiTableFilterOperator::default();
        self.table_data_filter_value.clear();
        self.table_data_filters.clear();
        self.table_data_sorts.clear();
        self.table_data_error = None;
        self.table_info_request = None;
        self.table_ddl_request = None;
        self.table_data_request = None;
        self.selected_cell = None;
        self.selected_row = None;
        self.selected_rows.clear();
        self.selection_anchor_row = None;
        self.selection_anchor_cell = None;
        self.data_editing_cell = None;
        self.data_edit_value.clear();
        self.data_edit_error = None;
        self.data_delete_confirmation = false;
        self.discard_changes_confirmation = false;
        self.table_view = TableView::Structure;
        self.set_active_query_text(format!("SELECT *\nFROM {table}\nLIMIT 100;"));
        self.request_table_info();
        self.activity = Activity::Explorer;
        self.sidebar_open = true;
        self.active_tab = WorkspaceTab::Table;
    }
}

pub(super) fn diagram_canvas_size(content_size: egui::Vec2, viewport_size: egui::Vec2) -> egui::Vec2 {
    egui::vec2(
        content_size.x.max(viewport_size.x).max(640.0),
        content_size.y.max(viewport_size.y).max(360.0),
    )
}

fn paint_diagram_grid(painter: &egui::Painter, rect: egui::Rect, zoom: f32, theme: DbProTheme) {
    let grid_step = (24.0 * zoom).max(8.0);
    let visible = painter.clip_rect().intersect(rect);
    if visible.is_negative() {
        return;
    }
    let grid_color = theme.border_subtle.linear_multiply(0.55);
    let start_x = rect.left() + ((visible.left() - rect.left()) / grid_step).floor() * grid_step;
    let mut grid_x = start_x;
    while grid_x <= visible.right() {
        if grid_x >= visible.left() {
            painter.line_segment(
                [egui::pos2(grid_x, visible.top()), egui::pos2(grid_x, visible.bottom())],
                egui::Stroke::new(1.0, grid_color),
            );
        }
        grid_x += grid_step;
    }
    let start_y = rect.top() + ((visible.top() - rect.top()) / grid_step).floor() * grid_step;
    let mut grid_y = start_y;
    while grid_y <= visible.bottom() {
        if grid_y >= visible.top() {
            painter.line_segment(
                [egui::pos2(visible.left(), grid_y), egui::pos2(visible.right(), grid_y)],
                egui::Stroke::new(1.0, grid_color),
            );
        }
        grid_y += grid_step;
    }
}

#[allow(dead_code)]
pub(super) fn diagram_edge_bounding_box(source_rect: egui::Rect, target_rect: egui::Rect, zoom: f32) -> egui::Rect {
    let min_x = source_rect.min.x.min(target_rect.min.x) - 40.0 * zoom;
    let max_x = source_rect.max.x.max(target_rect.max.x) + 40.0 * zoom;
    let min_y = source_rect.min.y.min(target_rect.min.y) - 20.0 * zoom;
    let max_y = source_rect.max.y.max(target_rect.max.y) + 20.0 * zoom;
    egui::Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y))
}

pub(super) fn diagram_foreign_key_label(foreign_key: &UiSchemaForeignKey) -> String {
    if foreign_key.from_columns.len() > 1 || foreign_key.to_columns.len() > 1 {
        format!(
            "[{}] → [{}]",
            foreign_key.from_columns.join(", "),
            foreign_key.to_columns.join(", ")
        )
    } else {
        format!(
            "{} → {}",
            foreign_key.from_columns.first().map(String::as_str).unwrap_or("key"),
            foreign_key.to_columns.first().map(String::as_str).unwrap_or("key"),
        )
    }
}

fn paint_scene_edges(
    painter: &egui::Painter,
    graph: &ErGraph,
    scene: &ErRenderScene,
    viewport: &ErViewport,
    theme: DbProTheme,
) {
    for &edge_id in &scene.visible_edges {
        let Some(edge) = graph.edges.get(edge_id) else {
            continue;
        };
        let Some(source_node) = graph.nodes.get(edge.source) else {
            continue;
        };
        let Some(target_node) = graph.nodes.get(edge.target) else {
            continue;
        };

        let source_on_right = source_node.world_rect.center().x < target_node.world_rect.center().x;
        let from_world = source_node.column_anchor(
            edge.foreign_key.from_columns.first().map(String::as_str),
            source_on_right,
            scene.lod.max_columns(),
        );
        let to_world = target_node.column_anchor(
            edge.foreign_key.to_columns.first().map(String::as_str),
            !source_on_right,
            scene.lod.max_columns(),
        );

        let from = viewport.world_to_screen_pos(from_world);
        let to = viewport.world_to_screen_pos(to_world);

        let bend_x = (from.x + to.x) / 2.0;
        let bend_a = egui::pos2(bend_x, from.y);
        let bend_b = egui::pos2(bend_x, to.y);
        let stroke = egui::Stroke::new(1.2, theme.accent);
        painter.line_segment([from, bend_a], stroke);
        painter.line_segment([bend_a, bend_b], stroke);
        painter.line_segment([bend_b, to], stroke);

        if scene.lod.shows_edge_labels() {
            let label = diagram_foreign_key_label(&edge.foreign_key);
            let display_label = if label.len() > 36 {
                format!("{}…", &label[..34])
            } else {
                label
            };
            let label_position = egui::pos2(bend_x, (from.y + to.y) / 2.0);
            let label_galley =
                painter.layout_no_wrap(display_label.clone(), FontId::proportional(10.0), theme.text_secondary);
            let label_rect = egui::Rect::from_center_size(label_position, label_galley.size() + egui::vec2(8.0, 4.0));
            painter.rect_filled(label_rect, egui::Rounding::same(3.0), theme.surface_panel);
            painter.text(
                label_position,
                egui::Align2::CENTER_CENTER,
                display_label,
                FontId::proportional(10.0),
                theme.text_secondary,
            );
        }
    }
}

fn paint_er_node_lod(
    painter: &egui::Painter,
    node: &ErNode,
    screen_rect: egui::Rect,
    selected: bool,
    lod: ErLod,
    zoom: f32,
    theme: DbProTheme,
) {
    match lod {
        ErLod::Compact => {
            // Compact pill card: only header with table name and PK count
            painter.rect_filled(screen_rect, egui::Rounding::same(6.0), theme.surface_panel);
            painter.rect_stroke(
                screen_rect,
                egui::Rounding::same(6.0),
                egui::Stroke::new(1.0, if selected { theme.accent } else { theme.border_default }),
            );
            let header_fill = if selected {
                theme.accent_soft
            } else {
                theme.surface_hover
            };
            painter.rect_filled(screen_rect, egui::Rounding::same(6.0), header_fill);
            painter.text(
                screen_rect.center_top() + egui::vec2(0.0, 14.0 * zoom),
                egui::Align2::CENTER_CENTER,
                &node.table.name,
                FontId::proportional((12.0 * zoom).clamp(8.0, 14.0)),
                theme.text_primary,
            );
            let pk_count = node.table.columns.iter().filter(|c| c.is_primary_key).count();
            if pk_count > 0 {
                painter.text(
                    egui::pos2(screen_rect.right() - 8.0 * zoom, screen_rect.center().y),
                    egui::Align2::RIGHT_CENTER,
                    format!("{pk_count} PK"),
                    FontId::proportional((9.0 * zoom).clamp(7.0, 11.0)),
                    theme.warning,
                );
            }
        }
        ErLod::Standard | ErLod::Detailed => {
            // Header:
            painter.rect_filled(screen_rect, egui::Rounding::same(8.0), theme.surface_panel);
            painter.rect_stroke(
                screen_rect,
                egui::Rounding::same(8.0),
                egui::Stroke::new(1.0, if selected { theme.accent } else { theme.border_default }),
            );
            let header_height = ER_HEADER_HEIGHT * zoom;
            let header_rect = egui::Rect::from_min_max(
                screen_rect.min,
                egui::pos2(screen_rect.max.x, screen_rect.min.y + header_height),
            );
            let header_fill = if selected {
                theme.accent_soft
            } else {
                theme.surface_hover
            };
            painter.rect_filled(header_rect, egui::Rounding::same(8.0), header_fill);
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(header_rect.min.x, header_rect.max.y - 8.0 * zoom),
                    header_rect.max,
                ),
                egui::Rounding::ZERO,
                header_fill,
            );
            painter.text(
                screen_rect.min + egui::vec2(14.0 * zoom, 20.0 * zoom),
                egui::Align2::LEFT_CENTER,
                format!("{}.{}", node.table.schema, node.table.name),
                FontId::proportional(13.0 * zoom),
                theme.text_primary,
            );
            painter.text(
                egui::pos2(screen_rect.max.x - 12.0 * zoom, screen_rect.min.y + 20.0 * zoom),
                egui::Align2::RIGHT_CENTER,
                "TABLE",
                FontId::proportional(9.0 * zoom),
                theme.text_muted,
            );

            // Columns:
            let max_cols = lod.max_columns();
            for (index, column) in node.table.columns.iter().take(max_cols).enumerate() {
                let row_top = screen_rect.min.y + (ER_HEADER_HEIGHT + index as f32 * ER_ROW_HEIGHT) * zoom;
                let row_rect = egui::Rect::from_min_max(
                    egui::pos2(screen_rect.min.x, row_top),
                    egui::pos2(screen_rect.max.x, row_top + ER_ROW_HEIGHT * zoom),
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
                if lod.shows_data_types() {
                    painter.text(
                        egui::pos2(row_rect.max.x - 12.0 * zoom, row_rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        &column.data_type,
                        FontId::monospace(10.0 * zoom),
                        theme.text_secondary,
                    );
                }
            }
            if node.table.columns.len() > max_cols {
                painter.text(
                    egui::pos2(
                        screen_rect.min.x + 14.0 * zoom,
                        screen_rect.min.y + (ER_HEADER_HEIGHT + ER_ROW_HEIGHT * (max_cols as f32 - 0.5)) * zoom,
                    ),
                    egui::Align2::LEFT_CENTER,
                    format!("+ {} more columns", node.table.columns.len() - max_cols),
                    FontId::proportional(10.0 * zoom),
                    theme.text_muted,
                );
            }
        }
    }
}

fn draw_diagram_zoom_controls(
    ui: &mut egui::Ui,
    canvas_rect: egui::Rect,
    zoom: &mut f32,
    pan: &mut egui::Vec2,
    theme: DbProTheme,
) {
    let controls_rect = egui::Rect::from_min_size(
        egui::pos2(canvas_rect.right() - 142.0, canvas_rect.top() + 12.0),
        egui::vec2(130.0, 32.0),
    );
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(controls_rect), |ui| {
        egui::Frame {
            fill: theme.surface_floating,
            inner_margin: egui::Margin::symmetric(5.0, 4.0),
            rounding: egui::Rounding::same(6.0),
            stroke: egui::Stroke::new(1.0, theme.border_subtle),
            ..Default::default()
        }
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if compact_icon_button(ui, Icon::Minus, theme)
                    .on_hover_text("Zoom out")
                    .clicked()
                {
                    *zoom = (*zoom - 0.1).clamp(0.5, 2.0);
                }
                ui.label(
                    RichText::new(format!("{:.0}%", *zoom * 100.0))
                        .small()
                        .color(theme.text_secondary),
                );
                if compact_icon_button(ui, Icon::Plus, theme)
                    .on_hover_text("Zoom in")
                    .clicked()
                {
                    *zoom = (*zoom + 0.1).clamp(0.5, 2.0);
                }
                if compact_icon_button(ui, Icon::Square, theme)
                    .on_hover_text("Fit diagram")
                    .clicked()
                {
                    *zoom = 1.0;
                    *pan = egui::Vec2::ZERO;
                }
            });
        });
    });
}
