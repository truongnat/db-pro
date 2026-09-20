use super::palette_search_view::PaletteSearchContext;
use super::*;
use crate::components::{kbd_badge, Dialog};

impl DbProApp {
    fn palette_entries(&self, mode: PaletteMode) -> Vec<(SearchKind, PaletteItem)> {
        let active_schema = self.active_schema().to_owned();
        let table_names = self.active_schema_table_names();
        let column_names = self.active_schema_column_names();
        PaletteSearchContext {
            workspace: &self.workspace,
            schema: &self.schema,
            query: &self.query,
            connection: &self.connection,
            active_schema: &active_schema,
            table_names: &table_names,
            column_names: &column_names,
        }
        .entries(mode)
    }

    fn search_fingerprint(&self) -> String {
        let workspace_files = self
            .workspace
            .files
            .ide_workspace
            .index()
            .into_iter()
            .filter(|entry| entry.is_sql)
            .count();
        SearchService::build_fingerprint(SearchFingerprintParts {
            connection_id: self.connection.lifecycle.active_connection_id(),
            schema: self.active_schema(),
            tables: self.schema.explorer.schema.tables.len(),
            views: self.schema.explorer.schema.views.len(),
            functions: self.schema.explorer.schema.functions.len(),
            columns: self.active_schema_column_names().len(),
            saved_queries: self.query.library.saved_queries.len(),
            history: self.query.editor.query_history_entries.len(),
            connections: self.connection.catalog.len(),
            workspace_files,
        })
    }

    fn ensure_search_index(&mut self, mode: PaletteMode) {
        let fingerprint = format!("{}|{:?}", self.search_fingerprint(), mode);
        if self.palette.search_index.fingerprint() == fingerprint && !self.palette.search_index.is_empty() {
            return;
        }
        let entries = self.palette_entries(mode);
        self.palette.search_index.replace(fingerprint, entries);
    }

    pub(crate) fn filtered_palette_items(&self, mode: PaletteMode) -> Vec<PaletteItem> {
        let entries = if self.palette.search_index.fingerprint().contains(&format!("{mode:?}"))
            && !self.palette.search_index.is_empty()
        {
            self.palette.search_index.entries().to_vec()
        } else {
            self.palette_entries(mode)
        };
        SearchService::filter_rank(&entries, &self.palette.query, self.palette.scope, 120)
    }

    /// Rebuild + rank for mutable callers (palette draw path).
    pub(crate) fn filtered_palette_items_fresh(&mut self, mode: PaletteMode) -> Vec<PaletteItem> {
        self.ensure_search_index(mode);
        self.filtered_palette_items(mode)
    }

    pub(super) fn refresh_schema_palette(&mut self) {
        if let Some(connection_id) = self.connection.lifecycle.active_connection_id().map(str::to_owned) {
            self.table.state.refresh_table_info_after_schema = self.schema.explorer.selected_table.is_some();
            self.request_schema_introspection(connection_id, true);
        } else {
            self.feedback.runtime_message = "Connect to a database before refreshing schema".to_owned();
        }
    }

    pub(crate) fn open_table_from_palette(&mut self, table: String) {
        if self.schema.explorer.selected_table.as_deref() != Some(table.as_str())
            && !self.table.mutation.staged_changes.is_empty()
        {
            self.feedback.runtime_message = "Apply or discard staged changes before opening another table".to_owned();
            return;
        }
        let scope = TableDataState::layout_scope(
            self.connection.lifecycle.active_connection_id(),
            self.active_schema(),
            self.schema.explorer.selected_table.as_deref(),
        );
        self.table.data.persist_layout(scope);
        self.schema.explorer.record_recent_table(&table);
        self.schema.explorer.selected_table = Some(table.clone());
        let scope = TableDataState::layout_scope(
            self.connection.lifecycle.active_connection_id(),
            self.active_schema(),
            self.schema.explorer.selected_table.as_deref(),
        );
        self.table.data.restore_layout(scope);
        self.schema.explorer.selected_schema_object = None;
        self.table.state.table_view = TableView::Structure;
        self.table.state.table_info = None;
        self.table.state.table_ddl = None;
        self.table.data_query.result = None;
        self.request_table_info();
        self.workspace.active_tab = WorkspaceTab::Table;
        self.feedback.runtime_message = format!("Opening table {table}");
    }

    pub(super) fn open_saved_query_from_palette(&mut self, query_id: String) {
        let Some(query) = self
            .query
            .library
            .saved_queries
            .iter()
            .find(|item| item.id == query_id)
            .cloned()
        else {
            self.feedback.runtime_message = "Saved query is no longer available".to_owned();
            return;
        };
        self.new_query_document();
        if let Some(doc) = self
            .query
            .session
            .documents
            .get_mut(self.query.session.active_document_index)
        {
            doc.set_text(query.sql.clone());
            doc.title = query.name.clone();
            doc.saved_query_id = Some(query.id.clone());
            doc.mark_saved();
        }
        self.workspace.active_tab = WorkspaceTab::Query;
        self.feedback.runtime_message = format!("Opened saved query {}", query.name);
    }

    pub(crate) fn toggle_pinned_table(&mut self, table: String) {
        let target = if table.is_empty() {
            self.schema.explorer.selected_table.clone()
        } else {
            Some(table)
        };
        let Some(table) = target else {
            self.feedback.runtime_message = "Select a table before pinning".to_owned();
            return;
        };
        if let Some(index) = self
            .schema
            .explorer
            .pinned_tables
            .iter()
            .position(|item| item == &table)
        {
            self.schema.explorer.pinned_tables.remove(index);
            self.feedback.runtime_message = format!("Unpinned table {table}");
        } else {
            self.schema.explorer.pinned_tables.push(table.clone());
            self.feedback.runtime_message = format!("Pinned table {table}");
        }
    }

    pub(super) fn export_results_from_palette(&mut self) {
        if self.query.session.active_result().is_some() {
            self.query.output.active_tab = OutputTab::Results;
            self.overlay.export_open = true;
            self.workspace.active_tab = WorkspaceTab::Query;
        } else {
            self.feedback.runtime_message = "Run a query before exporting results".to_owned();
        }
    }

    pub(super) fn switch_connection_from_palette(&mut self, connection_id: String) {
        let connection = self.connection.catalog.find(&connection_id).cloned();
        if let Some(connection) = connection {
            *self.connection.lifecycle.active_connection_id_mut() = Some(connection.id.clone());
            self.connection.lifecycle.set_connected(false);
            let request_id = self.task_bridge.next_request_id();
            self.connection.lifecycle.set_pending_request(Some(request_id));
            self.dispatch_command(self.connection.lifecycle.connect_command(request_id, connection.id));
            self.feedback.runtime_message = format!("Connecting to {}…", connection.name);
        }
    }

    pub(super) fn draw_palette(&mut self, ctx: &egui::Context) {
        let Some(mode) = self.palette.mode else {
            return;
        };
        let items = self.filtered_palette_items_fresh(mode);
        self.clamp_palette_selection(&items);
        let mut activate = false;
        let mut open = true;
        let title = if mode == PaletteMode::QuickOpen {
            "Quick Open"
        } else if self.palette.scope == SearchScope::Connections {
            "Switch Connection"
        } else {
            "Command Palette"
        };
        let description = if mode == PaletteMode::QuickOpen {
            "Switch workspaces, tabs, or open editors"
        } else if self.palette.scope == SearchScope::Connections {
            "Pick a saved connection to open"
        } else {
            "Search commands, actions, and database tools"
        };

        Dialog::new(&mut open, title, self.theme)
            .description(description)
            .width(580.0)
            .id_salt("palette_dialog")
            .show_ctx(ctx, |ui| {
                let response = ui.add(
                    TextEdit::singleline(&mut self.palette.query)
                        .hint_text(RichText::new("Type a command or search…").color(self.theme.text_muted))
                        .desired_width(ui.available_width())
                        .margin(egui::Margin::symmetric(12.0, 8.0))
                        .font(egui::FontId::proportional(13.5))
                        .text_color(self.theme.text_primary),
                );
                if self.palette.focus_requested {
                    response.request_focus();
                    self.palette.focus_requested = false;
                }

                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    for scope in SearchScope::all() {
                        let selected = self.palette.scope == *scope;
                        if ui.selectable_label(selected, scope.label()).clicked() {
                            self.palette.scope = *scope;
                            self.palette.selected = 0;
                        }
                    }
                });

                if ui.input(|input| input.key_pressed(egui::Key::ArrowDown)) && !items.is_empty() {
                    self.palette.selected = (self.palette.selected + 1) % items.len();
                }
                if ui.input(|input| input.key_pressed(egui::Key::ArrowUp)) && !items.is_empty() {
                    self.palette.selected = if self.palette.selected == 0 {
                        items.len() - 1
                    } else {
                        self.palette.selected - 1
                    };
                }
                if ui.input(|input| input.key_pressed(egui::Key::Enter)) && !items.is_empty() {
                    activate = true;
                }

                ui.add_space(8.0);
                egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                    if items.is_empty() {
                        ui.add_space(16.0);
                        ui.vertical_centered(|ui| {
                            ui.label(RichText::new("No matching commands found").color(self.theme.text_muted));
                        });
                        ui.add_space(16.0);
                    }
                    for (index, item) in items.iter().enumerate() {
                        let selected = index == self.palette.selected;
                        let item_fill = if selected {
                            self.theme.surface_hover
                        } else {
                            egui::Color32::TRANSPARENT
                        };
                        let (rect, item_resp) =
                            ui.allocate_exact_size(egui::vec2(ui.available_width(), 44.0), egui::Sense::click());
                        if item_resp.hovered() {
                            self.palette.selected = index;
                        }
                        if item_resp.clicked() {
                            self.palette.selected = index;
                            activate = true;
                        }

                        if selected || item_resp.hovered() {
                            ui.painter().rect_filled(rect, egui::Rounding::same(6.0), item_fill);
                            if selected {
                                ui.painter().rect_stroke(
                                    rect,
                                    egui::Rounding::same(6.0),
                                    egui::Stroke::new(1.0, self.theme.border_subtle),
                                );
                            }
                        }

                        // Icon
                        let icon_char = char::from(item.icon).to_string();
                        ui.painter().text(
                            egui::pos2(rect.left() + 12.0, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            icon_char,
                            egui::FontId::new(15.0, egui::FontFamily::Name("lucide".into())),
                            if selected {
                                self.theme.text_primary
                            } else {
                                self.theme.text_secondary
                            },
                        );

                        // Title and Subtitle
                        let text_x = rect.left() + 38.0;
                        ui.painter().text(
                            egui::pos2(text_x, rect.center().y - 8.0),
                            egui::Align2::LEFT_CENTER,
                            &item.title,
                            crate::DbProTheme::ui_medium_font(13.0),
                            self.theme.text_primary,
                        );
                        ui.painter().text(
                            egui::pos2(text_x, rect.center().y + 8.0),
                            egui::Align2::LEFT_CENTER,
                            &item.subtitle,
                            egui::FontId::proportional(11.5),
                            self.theme.text_muted,
                        );

                        if let Some(shortcut) = &item.shortcut {
                            ui.allocate_new_ui(
                                egui::UiBuilder::new().max_rect(egui::Rect::from_min_max(
                                    egui::pos2(rect.right() - 80.0, rect.top()),
                                    rect.right_bottom(),
                                )),
                                |ui| {
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add_space(8.0);
                                        kbd_badge(ui, shortcut, self.theme);
                                    });
                                },
                            );
                        }
                    }
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("↑↓ Navigate · ↵ Select · Esc Close")
                            .size(11.0)
                            .color(self.theme.text_muted),
                    );
                });
            });

        if !open {
            self.palette.mode = None;
        }

        if activate {
            if let Some(item) = items.get(self.palette.selected) {
                self.execute_palette_action(item.action.clone(), ctx);
            }
        }
    }

    fn clamp_palette_selection(&mut self, items: &[PaletteItem]) {
        if items.is_empty() {
            self.palette.selected = 0;
        } else {
            self.palette.selected = self.palette.selected.min(items.len() - 1);
        }
    }
}
