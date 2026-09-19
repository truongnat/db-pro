use super::*;
pub use crate::diagram::*;

impl DbProApp {
    pub(super) fn draw_diagram(&mut self, ui: &mut egui::Ui) {
        let all_table_count = self.schema_explorer.schema.table_details.len();
        let large_schema = all_table_count > ER_LARGE_SCHEMA_THRESHOLD;
        let search_query = self.diagram.search.trim().to_ascii_lowercase();
        let search_mode = diagram_search_mode(large_schema, self.diagram.show_all);

        // Poll background layout worker:
        if let Some(res) = self.diagram.layout_worker.poll_result() {
            if res.graph_version == self.diagram.schema_version && res.request_id == self.diagram.latest_layout_request
            {
                self.diagram.graph = res.graph;
                self.diagram.spatial_index = res.spatial_index;
                self.diagram.layout_state = ErLayoutState::Ready;
            }
        }

        if self.schema_explorer.schema.table_details.is_empty() {
            self.draw_diagram_empty_state(ui, 0, false, false);
            return;
        }

        let render_limit = if !large_schema || self.diagram.show_all {
            all_table_count
        } else {
            ER_MAX_TABLES
        };

        let (_candidate_count, tables) = diagram_candidates(
            &self.schema_explorer.schema.table_details,
            &search_query,
            search_mode,
            render_limit,
        );

        let grid_columns = if tables.len() <= 3 {
            tables.len().max(1)
        } else {
            ((all_table_count as f32).sqrt().ceil() as usize).clamp(3, 10)
        };

        let max_visible_columns = self
            .schema_explorer
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
                .diagram
                .graph
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
                self.diagram
                    .graph
                    .bfs_neighborhood(&seed_indices, self.diagram.neighborhood_depth, 100),
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
        self.draw_er_design_panel(ui);
        ui.add_space(10.0);

        self.draw_diagram_canvas(ui, active_node_indices.as_deref());
    }

    fn ensure_diagram_graph(&mut self, grid_columns: usize, node_height: f32) {
        let current_count = self.schema_explorer.schema.table_details.len();
        let graph_dirty = self.diagram.graph.nodes.len() != current_count
            || self.diagram.graph.schema_version != self.diagram.schema_version;

        if graph_dirty && current_count > 0 {
            if self.diagram.graph.nodes.is_empty() {
                // First load: build immediately so canvas starts populated without blank frame
                self.diagram.graph = ErGraph::build(
                    &self.schema_explorer.schema.table_details,
                    self.diagram.schema_version,
                    grid_columns,
                    node_height,
                );
                self.diagram.spatial_index = ErSpatialIndex::build(
                    &self.diagram.graph.nodes,
                    &self.diagram.graph.edges,
                    DEFAULT_SPATIAL_CELL_SIZE,
                );
                self.diagram.layout_state = ErLayoutState::Ready;
            } else {
                // Background worker update: keep old graph renderable and dispatch async request.
                // saturating_add: at u64::MAX the version stays MAX; subsequent invalidations
                // all produce the same version but the graph node count check (graph_dirty)
                // prevents re-dispatch unless the actual table list changes.
                self.diagram.schema_version = self.diagram.schema_version.saturating_add(1);
                let request_id = self.diagram.layout_worker.request_layout(
                    self.diagram.schema_version,
                    self.schema_explorer.schema.table_details.clone(),
                    grid_columns,
                    node_height,
                );
                self.diagram.latest_layout_request = request_id;

                // If the worker is in degraded mode (spawn failed), the request was
                // silently dropped. Transition to Failed state so the UI shows a
                // concise error while retaining the last valid graph.
                if self.diagram.layout_worker.dispatch_succeeded() {
                    self.diagram.layout_state = ErLayoutState::Computing {
                        request_id,
                        graph_version: self.diagram.schema_version,
                    };
                } else {
                    self.diagram.layout_state = ErLayoutState::Failed("ER layout worker unavailable".to_owned());
                }
            }
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
        let visible_tables = if self.diagram.show_all || !large_schema {
            all_table_count
        } else {
            tables.len().min(render_limit)
        };
        let relationship_count: usize = self.diagram.graph.edges.len();

        ui.horizontal_wrapped(|ui| {
            section_label(ui, "ER DIAGRAM", self.theme);
            ui.add_space(8.0);
            badge(
                ui,
                &plural_count(visible_tables, "table", "tables"),
                self.theme.accent_soft,
                self.theme.accent,
            );
            badge(
                ui,
                &plural_count(relationship_count, "relationship", "relationships"),
                self.theme.surface_hover,
                self.theme.text_secondary,
            );
            if matches!(self.diagram.layout_state, ErLayoutState::Computing { .. }) {
                badge(
                    ui,
                    "Arranging map…",
                    self.theme.surface_hover,
                    self.theme.text_secondary,
                );
            }
            if large_schema {
                ui.separator();
                let search_changed =
                    input(ui, &mut self.diagram.search, "Find table or column…", 220.0, self.theme).changed();
                self.diagram.show_all =
                    diagram_show_all_after_search_edit(self.diagram.show_all, &self.diagram.search, search_changed);
                let search_mode = diagram_search_mode(large_schema, self.diagram.show_all);
                if search_mode {
                    ui.label(RichText::new("Neighborhood:").small().color(self.theme.text_muted));
                    if ui
                        .selectable_label(self.diagram.neighborhood_depth == 1, "1 hop")
                        .clicked()
                    {
                        self.diagram.neighborhood_depth = 1;
                    }
                    if ui
                        .selectable_label(self.diagram.neighborhood_depth == 2, "2 hops")
                        .clicked()
                    {
                        self.diagram.neighborhood_depth = 2;
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
                        self.diagram.show_all = true;
                    }
                } else {
                    badge(ui, "All tables", self.theme.warning, self.theme.text_inverse);
                    if compact_button_with_icon(ui, Icon::Search, "Focus search", self.theme).clicked() {
                        self.diagram.show_all = false;
                    }
                }
            }
            ui.separator();
            let design_label = if self.diagram.design.enabled {
                "Design Mode ✓"
            } else {
                "Design Mode"
            };
            if secondary_button(ui, design_label, self.theme)
                .on_hover_text("Draft schema edits — never mutates DB until Apply")
                .clicked()
            {
                self.diagram.design.enabled = !self.diagram.design.enabled;
                if self.diagram.design.enabled {
                    let names: Vec<String> = self
                        .schema_explorer
                        .schema
                        .table_details
                        .iter()
                        .map(|t| format!("{}.{}", t.schema, t.name))
                        .collect();
                    self.diagram
                        .design
                        .set_schema_fingerprint(crate::diagram::design_mode::schema_fingerprint_from_names(&names));
                }
            }
        });
    }

    pub(super) fn draw_er_design_panel(&mut self, ui: &mut egui::Ui) {
        if !self.diagram.design.enabled {
            return;
        }
        ui.add_space(SPACE_SM);
        card_frame(self.theme).show(ui, |ui| {
            section_label(ui, "DESIGN DRAFT", self.theme);
            ui.label(
                RichText::new("Draft only · ObjectMutationService plan · fingerprint-gated apply")
                    .small()
                    .color(self.theme.text_muted),
            );
            let live_fp = {
                let names: Vec<String> = self
                    .schema_explorer
                    .schema
                    .table_details
                    .iter()
                    .map(|t| format!("{}.{}", t.schema, t.name))
                    .collect();
                crate::diagram::design_mode::schema_fingerprint_from_names(&names)
            };
            if self.diagram.design.fingerprint_stale(&live_fp) {
                ui.colored_label(
                    self.theme.warning,
                    "Live schema changed — re-open Design Mode or discard before apply",
                );
            }
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.diagram.new_schema).hint_text("schema"));
                ui.add(egui::TextEdit::singleline(&mut self.diagram.new_table).hint_text("table"));
                if secondary_button(ui, "Add draft table", self.theme).clicked() {
                    self.diagram
                        .design
                        .add_draft_table(&self.diagram.new_schema, &self.diagram.new_table);
                    self.diagram.new_table.clear();
                }
                if ghost_button(ui, "Undo", self.theme).clicked() {
                    self.diagram.design.undo();
                }
                if ghost_button(ui, "Redo", self.theme).clicked() {
                    self.diagram.design.redo();
                }
                if danger_button(ui, "Discard", self.theme).clicked() {
                    self.diagram.design.discard();
                }
            });
            for (idx, table) in self.diagram.design.draft.tables.clone().into_iter().enumerate() {
                ui.label(
                    RichText::new(format!(
                        "draft {}.{} · {} cols",
                        table.schema,
                        table.name,
                        table.columns.len()
                    ))
                    .strong()
                    .monospace(),
                );
                for col in &table.columns {
                    ui.label(
                        RichText::new(format!(
                            "  {} {}{}{}",
                            col.name,
                            col.data_type,
                            if col.is_pk { " PK" } else { "" },
                            if col.is_unique { " UNIQUE" } else { "" }
                        ))
                        .small()
                        .monospace(),
                    );
                }
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.diagram.column_name).hint_text("col"));
                    ui.add(egui::TextEdit::singleline(&mut self.diagram.column_type).hint_text("type"));
                    if ghost_button(ui, "Add col", self.theme).clicked() {
                        self.diagram.design.add_column(
                            idx,
                            crate::diagram::design_mode::DraftColumn {
                                name: self.diagram.column_name.clone(),
                                data_type: self.diagram.column_type.clone(),
                                nullable: true,
                                is_pk: false,
                                is_unique: false,
                            },
                        );
                        self.diagram.column_name.clear();
                    }
                });
            }
            ui.add_space(SPACE_XS);
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.diagram.foreign_key_name).hint_text("fk name"));
                ui.add(
                    egui::TextEdit::singleline(&mut self.diagram.foreign_key_from).hint_text("from schema.table.col"),
                );
                ui.add(egui::TextEdit::singleline(&mut self.diagram.foreign_key_to).hint_text("to schema.table.col"));
                if secondary_button(ui, "Add FK", self.theme).clicked() {
                    self.er_design_add_fk();
                }
            });
            for fk in &self.diagram.design.draft.foreign_keys {
                ui.label(
                    RichText::new(format!(
                        "FK {} · {}.{}({}) → {}.{}({})",
                        fk.name,
                        fk.from_schema,
                        fk.from_table,
                        fk.from_columns.join(","),
                        fk.to_schema,
                        fk.to_table,
                        fk.to_columns.join(",")
                    ))
                    .small()
                    .monospace(),
                );
            }
            ui.horizontal(|ui| {
                if secondary_button(ui, "Preview mutation plan", self.theme).clicked() {
                    self.er_design_preview_plan();
                }
                if primary_button(ui, "Apply (confirm)", self.theme).clicked() {
                    self.er_design_apply_plan();
                }
            });
            if let Some(error) = &self.diagram.design.error {
                ui.colored_label(self.theme.danger, error);
            }
            if !self.diagram.design.preview_sql.is_empty() {
                ui.label(
                    RichText::new(format!("fingerprint {}", self.diagram.design.preview_fingerprint))
                        .small()
                        .color(self.theme.text_muted),
                );
                for effect in &self.diagram.design.preview_effects {
                    ui.label(RichText::new(effect).small().color(self.theme.text_secondary));
                }
                egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                    ui.label(RichText::new(&self.diagram.design.preview_sql).monospace());
                });
            }
        });
    }

    fn er_design_add_fk(&mut self) {
        let Some((from_schema, from_table, from_col)) = split_three(&self.diagram.foreign_key_from) else {
            self.diagram.design.error = Some("FK from must be schema.table.column".into());
            return;
        };
        let Some((to_schema, to_table, to_col)) = split_three(&self.diagram.foreign_key_to) else {
            self.diagram.design.error = Some("FK to must be schema.table.column".into());
            return;
        };
        let name = if self.diagram.foreign_key_name.trim().is_empty() {
            format!("fk_{from_table}_{to_table}")
        } else {
            self.diagram.foreign_key_name.trim().to_owned()
        };
        self.diagram
            .design
            .add_fk(crate::diagram::design_mode::DraftForeignKey {
                name,
                from_schema,
                from_table,
                from_columns: vec![from_col],
                to_schema,
                to_table,
                to_columns: vec![to_col],
            });
    }

    fn er_design_preview_plan(&mut self) {
        struct QuoteDialect;
        impl db_pro_core::ports::SqlDialect for QuoteDialect {
            fn placeholder(&self, index: usize) -> String {
                format!("${index}")
            }
            fn quote_identifier(&self, name: &str) -> String {
                format!("\"{}\"", name.replace('"', "\"\""))
            }
        }
        let driver = self.active_driver().to_owned();
        match crate::diagram::design_mode::plan_design_draft(&self.diagram.design.draft, &driver, &QuoteDialect) {
            Ok(previews) => {
                let (sql, fp, effects) = crate::diagram::design_mode::merge_preview_sql(&previews);
                self.diagram.design.preview_sql = sql;
                self.diagram.design.preview_fingerprint = fp;
                self.diagram.design.preview_effects = effects;
                self.diagram.design.error = None;
                self.diagram.design.apply_confirm = true;
            }
            Err(err) => {
                self.diagram.design.error = Some(err);
                self.diagram.design.clear_preview();
            }
        }
    }

    fn er_design_apply_plan(&mut self) {
        let live_fp = {
            let names: Vec<String> = self
                .schema_explorer
                .schema
                .table_details
                .iter()
                .map(|t| format!("{}.{}", t.schema, t.name))
                .collect();
            crate::diagram::design_mode::schema_fingerprint_from_names(&names)
        };
        if self.diagram.design.fingerprint_stale(&live_fp) {
            self.diagram.design.error = Some("schema fingerprint stale — refresh Design Mode".into());
            return;
        }
        if self.diagram.design.preview_sql.is_empty() || !self.diagram.design.apply_confirm {
            self.er_design_preview_plan();
            if self.diagram.design.preview_sql.is_empty() {
                return;
            }
        }
        if self.connection.lifecycle.active_connection_id().is_none() || !self.connection.lifecycle.is_connected() {
            self.diagram.design.error = Some("connect before applying design plan".into());
            return;
        }
        self.set_active_query_text(self.diagram.design.preview_sql.clone());
        self.workspace.active_tab = WorkspaceTab::Query;
        self.dispatch_query();
        self.feedback.runtime_message = "Design Mode mutation plan applied via query runtime".into();
        self.diagram.design.discard();
    }
}

fn split_three(raw: &str) -> Option<(String, String, String)> {
    let parts: Vec<&str> = raw.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((parts[0].to_owned(), parts[1].to_owned(), parts[2].to_owned()))
}

impl DbProApp {
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
                        self.diagram.search.clear();
                    }
                }
                ui.add_space(56.0);
            });
        });
    }
}

impl DbProApp {
    fn draw_diagram_canvas(&mut self, ui: &mut egui::Ui, active_filter: Option<&[usize]>) {
        let world_size = if let Some(filter) = active_filter {
            self.diagram.graph.active_subset_bounds(filter).size()
        } else {
            self.diagram.graph.world_bounds.size()
        };
        let viewport_size = egui::vec2(ui.available_width(), ui.available_height());
        let canvas_size = diagram_canvas_size(world_size * self.diagram.zoom, viewport_size);
        let zoom = self.diagram.zoom;
        let pan = self.diagram.pan;
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
                    &self.diagram.graph,
                    &self.diagram.spatial_index,
                    &viewport,
                    response.rect,
                    active_filter,
                );

                // Paint visible edges:
                paint_scene_edges(&painter, &self.diagram.graph, &scene, &viewport, zoom, theme);

                // Paint visible nodes:
                let selected_table_name = self.schema_explorer.selected_table.as_deref();
                for &node_id in &scene.visible_nodes {
                    if let Some(node) = self.diagram.graph.nodes.get(node_id) {
                        let screen_rect = viewport.world_to_screen_rect(node.world_rect);
                        let selected = selected_table_name == Some(node.table.name.as_str());
                        paint_er_node_lod(&painter, node, screen_rect, selected, scene.lod, zoom, theme);
                    }
                }

                draw_diagram_zoom_controls(
                    ui,
                    response.rect,
                    &mut self.diagram.zoom,
                    &mut self.diagram.pan,
                    &self.diagram.graph,
                    active_filter,
                    theme,
                );
                self.update_diagram_pan(&response);

                if response.clicked() {
                    if let Some(pointer) = response.interact_pointer_pos() {
                        let world_pos = viewport.screen_to_world_pos(pointer);
                        if let Some(hit_id) = self
                            .diagram
                            .spatial_index
                            .hit_test_node(world_pos, &self.diagram.graph.nodes)
                        {
                            let table_name = self.diagram.graph.nodes.get(hit_id).map(|node| node.table.name.clone());
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
            self.diagram.pan_origin = Some(self.diagram.pan);
        }
        if response.dragged() {
            if let Some(origin) = self.diagram.pan_origin {
                self.diagram.pan = origin + response.drag_delta();
            }
        }
        if response.drag_stopped() {
            self.diagram.pan_origin = None;
        }
    }

    fn open_diagram_table(&mut self, table: &str) {
        if self.schema_explorer.selected_table.as_deref() != Some(table)
            && !self.table_mutation.staged_changes.is_empty()
        {
            self.feedback.runtime_message = "Apply or discard staged changes before opening another table".to_owned();
            return;
        }
        self.persist_current_grid_layout();
        self.schema_explorer.selected_table = Some(table.to_owned());
        self.restore_grid_layout_for_active_table();
        self.schema_explorer.selected_schema_object = None;
        self.schema_explorer.schema_object_view = SchemaObjectView::Definition;
        self.table_state.table_info = None;
        self.table_state.table_ddl = None;
        self.table_state.table_info_error = None;
        self.table_state.table_ddl_error = None;
        self.table_state.ddl_execute_confirmation = false;
        self.table_state.ddl_execution_request = None;
        self.table_state.table_data_result = None;
        self.table_state.table_data_total_rows = None;
        self.table_state.table_data_offset = 0;
        self.table_state.table_data_filter_column.clear();
        self.table_state.table_data_filter_operator = UiTableFilterOperator::default();
        self.table_state.table_data_filter_value.clear();
        self.table_state.table_data_filters.clear();
        self.table_state.table_data_sorts.clear();
        self.table_state.table_data_error = None;
        self.table_state.table_info_request = None;
        self.table_state.table_ddl_request = None;
        self.table_state.table_data_request = None;
        self.table_data.selected_cell = None;
        self.table_data.selected_row = None;
        self.table_data.selected_rows.clear();
        self.table_data.selection_anchor_row = None;
        self.table_data.selection_anchor_cell = None;
        self.table_data.data_editing_cell = None;
        self.table_data.data_edit_value.clear();
        self.table_data.data_edit_error = None;
        self.table_data.data_delete_confirmation = false;
        self.table_data.discard_changes_confirmation = false;
        self.table_state.table_view = TableView::Structure;
        self.set_active_query_text(format!("SELECT *\nFROM {table}\nLIMIT 100;"));
        self.request_table_info();
        self.workspace.activity = Activity::Explorer;
        self.workspace.sidebar_open = true;
        self.workspace.active_tab = WorkspaceTab::Table;
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
    zoom: f32,
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
        let stroke = egui::Stroke::new((1.2 * zoom).clamp(1.0, 2.0), theme.accent);
        painter.line_segment([from, bend_a], stroke);
        painter.line_segment([bend_a, bend_b], stroke);
        painter.line_segment([bend_b, to], stroke);

        // Directional arrowhead at the destination connection point
        let arrow_len = (6.0 * zoom).clamp(4.0, 8.0);
        let arrow_half_w = arrow_len * 0.55;
        let (p1, p2) = if source_on_right {
            (
                egui::pos2(to.x - arrow_len, to.y - arrow_half_w),
                egui::pos2(to.x - arrow_len, to.y + arrow_half_w),
            )
        } else {
            (
                egui::pos2(to.x + arrow_len, to.y - arrow_half_w),
                egui::pos2(to.x + arrow_len, to.y + arrow_half_w),
            )
        };
        painter.add(egui::Shape::convex_polygon(
            vec![to, p1, p2],
            theme.accent,
            egui::Stroke::NONE,
        ));

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
            let label_rect = egui::Rect::from_center_size(label_position, label_galley.size() + egui::vec2(10.0, 6.0));
            painter.rect_filled(label_rect, egui::Rounding::same(4.0), theme.surface_panel);
            painter.rect_stroke(
                label_rect,
                egui::Rounding::same(4.0),
                egui::Stroke::new(1.0, theme.border_subtle),
            );
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
        ErLod::Compact if !selected => {
            // Compact pill card: only header with table name and PK count
            painter.rect_filled(screen_rect, egui::Rounding::same(6.0), theme.surface_panel);
            painter.rect_stroke(
                screen_rect,
                egui::Rounding::same(6.0),
                egui::Stroke::new(1.0, theme.border_default),
            );
            painter.rect_filled(screen_rect, egui::Rounding::same(6.0), theme.surface_hover);
            let title = crate::components::truncate_ellipsis(&node.table.name, 20);
            painter.text(
                screen_rect.center_top() + egui::vec2(0.0, 14.0 * zoom),
                egui::Align2::CENTER_CENTER,
                title,
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
        _ => {
            // Header:
            painter.rect_filled(screen_rect, egui::Rounding::same(8.0), theme.surface_panel);
            let border_stroke = if selected {
                egui::Stroke::new(1.5, theme.accent)
            } else {
                egui::Stroke::new(1.0, theme.border_default)
            };
            painter.rect_stroke(screen_rect, egui::Rounding::same(8.0), border_stroke);

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

            let full_table_name = if node.table.schema.is_empty() {
                node.table.name.clone()
            } else {
                format!("{}.{}", node.table.schema, node.table.name)
            };
            let max_title_chars = if zoom > 1.0 { 24 } else { 18 };
            let display_title = crate::components::truncate_ellipsis(&full_table_name, max_title_chars);
            painter.text(
                screen_rect.min + egui::vec2(14.0 * zoom, 20.0 * zoom),
                egui::Align2::LEFT_CENTER,
                display_title,
                FontId::proportional(13.0 * zoom),
                theme.text_primary,
            );
            painter.text(
                egui::pos2(screen_rect.max.x - 12.0 * zoom, screen_rect.min.y + 20.0 * zoom),
                egui::Align2::RIGHT_CENTER,
                "TABLE",
                FontId::proportional(9.0 * zoom),
                if selected { theme.accent } else { theme.text_muted },
            );

            // Columns:
            let max_cols = if selected && lod == ErLod::Compact {
                3
            } else {
                lod.max_columns()
            };
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
                    2.5 * zoom,
                    marker_color,
                );

                let col_name_max = if lod.shows_data_types() { 16 } else { 24 };
                let col_name = crate::components::truncate_ellipsis(&column.name, col_name_max);
                painter.text(
                    egui::pos2(row_rect.min.x + 23.0 * zoom, row_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    col_name,
                    FontId::proportional(11.0 * zoom),
                    theme.text_primary,
                );
                if lod.shows_data_types() {
                    let type_display = if column.is_primary_key {
                        format!("PK · {}", column.data_type)
                    } else if is_foreign_key {
                        format!("FK · {}", column.data_type)
                    } else if !column.nullable {
                        format!("{} · NN", column.data_type)
                    } else {
                        column.data_type.clone()
                    };
                    painter.text(
                        egui::pos2(row_rect.max.x - 12.0 * zoom, row_rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        type_display,
                        FontId::monospace(9.5 * zoom),
                        if column.is_primary_key {
                            theme.warning
                        } else if is_foreign_key {
                            theme.accent
                        } else {
                            theme.text_secondary
                        },
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
    graph: &ErGraph,
    active_filter: Option<&[usize]>,
    theme: DbProTheme,
) {
    let controls_rect = egui::Rect::from_min_size(
        egui::pos2(canvas_rect.right() - 176.0, canvas_rect.top() + 12.0),
        egui::vec2(164.0, 32.0),
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
                if ui
                    .add(
                        egui::Label::new(
                            RichText::new(format!("{:.0}%", *zoom * 100.0))
                                .small()
                                .color(theme.text_secondary),
                        )
                        .sense(Sense::click()),
                    )
                    .on_hover_text("Click to reset zoom to 100%")
                    .clicked()
                {
                    *zoom = 1.0;
                }
                if compact_icon_button(ui, Icon::Plus, theme)
                    .on_hover_text("Zoom in")
                    .clicked()
                {
                    *zoom = (*zoom + 0.1).clamp(0.5, 2.0);
                }
                if compact_icon_button(ui, Icon::RotateCcw, theme)
                    .on_hover_text("Reset zoom (100%)")
                    .clicked()
                {
                    *zoom = 1.0;
                    *pan = egui::Vec2::ZERO;
                }
                if compact_icon_button(ui, Icon::Maximize2, theme)
                    .on_hover_text("Fit diagram to viewport")
                    .clicked()
                {
                    let target_bounds = if let Some(filter) = active_filter {
                        graph.active_subset_bounds(filter)
                    } else {
                        graph.world_bounds
                    };
                    let bounds_size = target_bounds.size();
                    if bounds_size.x > 10.0 && bounds_size.y > 10.0 {
                        let zoom_x = (canvas_rect.width() - 80.0) / bounds_size.x;
                        let zoom_y = (canvas_rect.height() - 80.0) / bounds_size.y;
                        *zoom = zoom_x.min(zoom_y).clamp(0.5, 1.5);
                        *pan = egui::vec2(
                            (canvas_rect.width() - bounds_size.x * *zoom) / 2.0 - target_bounds.left() * *zoom,
                            (canvas_rect.height() - bounds_size.y * *zoom) / 2.0 - target_bounds.top() * *zoom,
                        );
                    } else {
                        *zoom = 1.0;
                        *pan = egui::Vec2::ZERO;
                    }
                }
            });
        });
    });
}
