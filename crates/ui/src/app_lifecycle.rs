use super::*;

impl eframe::App for DbProApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Match panel chrome so any sub-pixel seam between SidePanel and
        // CentralPanel cannot flash as a white strip (surface_app).
        self.theme.surface_panel.to_normalized_gamma_f32()
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.persist_layout_and_connection_state(storage);
        self.persist_query_and_schema_state(storage);
        self.persist_workspace_state(storage);
        self.persist_ui_preferences(storage);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.normalize_input_modifiers(ctx);
        self.prepare_frame(ctx);
        self.draw_shell(ctx);
        self.draw_overlays(ctx);
    }
}
impl DbProApp {
    fn persist_layout_and_connection_state(&mut self, storage: &mut dyn eframe::Storage) {
        let scope = TableDataState::layout_scope(
            self.connection.lifecycle.active_connection_id(),
            self.active_schema(),
            self.schema_explorer.selected_table.as_deref(),
        );
        self.table_data.persist_layout(scope);
        self.sync_settings_from_runtime();
        if let Ok(settings) = serde_json::to_string(&self.preferences.settings) {
            storage.set_string(SETTINGS_STORAGE_KEY, settings);
        }
        self.persist_saved_tasks(storage);
        self.persist_workspace_sessions(storage);
        if let Ok(raw) = serde_json::to_string(self.connection.dialog.ssh_profiles()) {
            storage.set_string("dbpro.native.ssh-profiles-v1", raw);
        }
        if let Ok(layouts) = serde_json::to_string(&self.table_data.grid_layout_preferences) {
            storage.set_string("dbpro.native.grid-layouts", layouts);
        }
        if let Ok(widths) = serde_json::to_string(&self.table_data.grid_column_widths) {
            storage.set_string("dbpro.native.grid-widths", widths);
        }
        storage.set_string(
            "dbpro.native.grid-widths-customized",
            self.table_data.grid_columns_user_resized.to_string(),
        );
    }

    fn persist_query_and_schema_state(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(documents) = serde_json::to_string(&self.query_session_state.documents) {
            storage.set_string("dbpro.native.query-documents", documents);
        }
        if let Ok(history) = serde_json::to_string(&self.query_editor.query_history_entries) {
            storage.set_string("dbpro.native.query-history-v1", history);
        }
        if let Ok(pinned) = serde_json::to_string(&self.schema_explorer.pinned_tables) {
            storage.set_string("dbpro.native.pinned-tables-v1", pinned);
        }
        if let Ok(recent) = serde_json::to_string(&self.schema_explorer.recent_tables) {
            storage.set_string("dbpro.native.recent-tables-v1", recent);
        }
    }

    fn persist_workspace_state(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(recent_ws) = serde_json::to_string(
            &self
                .workspace
                .files
                .ide_workspace
                .recent_roots
                .iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
        ) {
            storage.set_string("dbpro.native.workspace-recent-v1", recent_ws);
        }
        if let Ok(roots) = serde_json::to_string(
            &self
                .workspace
                .files
                .ide_workspace
                .roots
                .iter()
                .map(|root| root.path.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
        ) {
            storage.set_string("dbpro.native.workspace-roots-v1", roots);
        }
        storage.set_string(
            "dbpro.native.workspace-trusted-v1",
            self.workspace.files.ide_workspace.is_trusted().to_string(),
        );
    }

    fn persist_ui_preferences(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("dbpro.native.theme-version", "light-first-v1".to_owned());
        storage.set_string("dbpro.native.dark-mode", self.preferences.dark_mode.to_string());
        storage.set_string("dbpro.native.reduce-motion", self.preferences.reduce_motion.to_string());
        if let Ok(prediction_mode) = serde_json::to_string(&self.preferences.prediction_mode) {
            storage.set_string("dbpro.native.prediction-mode", prediction_mode);
        }
        storage.set_string("dbpro.native.sidebar-width", self.workspace.sidebar_width.to_string());
        storage.set_string("dbpro.native.agent-width", self.workspace.agent_width.to_string());
        storage.set_string("dbpro.native.output-open", self.workspace.bottom_panel_open.to_string());
        storage.set_string(
            "dbpro.native.output-height",
            self.workspace.bottom_panel_height.to_string(),
        );
        storage.set_string(
            "dbpro.native.connections-pane-height",
            self.schema_explorer.connections_pane_height.to_string(),
        );
        storage.set_string(
            "dbpro.native.schemas-pane-height",
            self.schema_explorer.schemas_pane_height.to_string(),
        );
    }

    fn normalize_input_modifiers(&mut self, ctx: &egui::Context) {
        // Map Ctrl to Command in input events so Ctrl+A/C/V/X/Z work seamlessly on macOS
        ctx.input_mut(|i| {
            if i.modifiers.ctrl {
                i.modifiers.command = true;
            }
            for event in &mut i.events {
                if let egui::Event::Key { modifiers, .. } = event {
                    if modifiers.ctrl {
                        modifiers.command = true;
                    }
                }
            }
        });
    }

    fn prepare_frame(&mut self, ctx: &egui::Context) {
        if self.initial_frames_count < 3 {
            self.initial_frames_count += 1;
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
        }
        self.request_connections_once();
        let runtime_events_pending = self.apply_runtime_events();
        self.tick_saved_task_scheduler();
        if runtime_events_pending
            || self.runtime_work_pending()
            || self
                .saved_tasks
                .store
                .tasks
                .iter()
                .any(|t| t.schedule.as_ref().is_some_and(|s| s.enabled))
        {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
        self.theme = if self.preferences.dark_mode {
            DbProTheme::dark()
        } else {
            DbProTheme::light()
        };
        self.theme.apply(ctx);
        self.handle_shortcuts(ctx);
    }

    fn draw_shell(&mut self, ctx: &egui::Context) {
        self.draw_topbar(ctx);
        // Query owns its rich output dock; the shell panel is for other tabs.
        if self.workspace.active_tab != WorkspaceTab::Query {
            self.draw_output_panel(ctx);
        }
        self.draw_statusbar(ctx);
        if let Some(action) = activity_bar_view::draw_activity_bar(ctx, self.theme, &mut self.workspace) {
            match action {
                activity_bar_view::ActivityBarAction::OpenSchemaWorkbench => self.open_schema_workbench(),
                activity_bar_view::ActivityBarAction::ToggleAgent => {
                    self.set_agent_open(!self.workspace.agent_open, ctx)
                }
            }
        }

        if self.workspace.sidebar_open {
            self.draw_sidebar(ctx);
        }

        if self.workspace.agent_open {
            self.draw_agent_panel(ctx);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame {
                // Flush to the sidebar splitter; match `SHELL_SPLIT_INSET` / sidebar
                // `pad_right` so the body lines up with the navigator across the divider.
                fill: self.theme.surface_panel,
                inner_margin: egui::Margin {
                    left: SHELL_SPLIT_INSET,
                    right: SHELL_SPLIT_INSET,
                    top: 0.0,
                    bottom: 0.0,
                },
                outer_margin: egui::Margin::ZERO,
                stroke: egui::Stroke::NONE,
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.set_min_size(ui.available_size());
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                self.draw_workspace(ui);
            });
    }

    fn draw_overlays(&mut self, ctx: &egui::Context) {
        if self.connection.dialog.is_open() {
            connection::view::draw_connection_dialog(
                ctx,
                self.theme,
                &mut self.connection.dialog,
                &mut self.connection.lifecycle,
                &mut self.task_bridge,
                &mut self.feedback,
            );
        }
        if self.overlay.delete_confirmation_id.is_some() {
            connection::delete_dialog::draw(
                ctx,
                self.theme,
                &mut self.overlay,
                &self.connection.catalog,
                &mut self.connection.lifecycle,
                &mut self.task_bridge,
                &mut self.feedback,
            );
        }
        if self.overlay.folder_delete_confirmation.is_some() {
            query_folder_delete_dialog::draw(
                ctx,
                self.theme,
                &mut self.overlay,
                &self.query_library,
                &mut self.task_bridge,
                &mut self.feedback,
            );
        }
        if self.table_data.insert_row_open {
            self.draw_insert_row_dialog(ctx);
        }
        if self.palette.mode.is_some() {
            self.draw_palette(ctx);
        }

        self.feedback.toasts.render_ctx(ctx, self.theme);
        if !self.feedback.toasts.is_empty() {
            ctx.request_repaint_after(Duration::from_millis(50));
        }
    }
}
