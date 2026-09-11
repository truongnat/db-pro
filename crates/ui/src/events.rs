use super::*;

impl DbProApp {
    pub(super) fn apply_runtime_events(&mut self) {
        let events: Vec<UiEvent> = self.task_bridge.drain_events().collect();
        for event in events {
            match event {
                UiEvent::ConnectionsLoaded { connections, .. } => {
                    self.connections_request_pending = false;
                    self.connections = connections;
                    if self.active_connection_id.is_none() {
                        self.active_connection_id = self.connections.first().map(|connection| connection.id.clone());
                    }
                    self.runtime_message = format!("Loaded {} connections", self.connections.len());
                }
                UiEvent::SavedQueriesLoaded { queries, .. } => {
                    self.saved_queries = queries;
                }
                UiEvent::QueryFoldersLoaded { folders, .. } => {
                    self.query_folders = folders;
                }
                UiEvent::SchemaLoaded { request_id, schema } => {
                    if self
                        .schema_request
                        .is_some_and(|expected_request| expected_request != request_id)
                    {
                        continue;
                    }
                    self.schema_request = None;
                    self.schema_error = None;
                    let refresh_selected_table = self.refresh_table_info_after_schema;
                    self.refresh_table_info_after_schema = false;
                    self.schema = schema;
                    if self
                        .selected_schema
                        .as_ref()
                        .is_none_or(|selected| !self.schema.schemas.iter().any(|schema| schema == selected))
                    {
                        self.selected_schema = self.schema.schemas.first().cloned();
                    }
                    if self
                        .selected_table
                        .as_ref()
                        .is_some_and(|table| !self.schema.tables.iter().any(|candidate| candidate == table))
                    {
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
                        if self.active_tab == WorkspaceTab::Table {
                            self.active_tab = WorkspaceTab::Welcome;
                        }
                    }
                    let object_exists = match self.selected_schema_object.as_ref() {
                        Some(SchemaObjectSelection::View(name)) => {
                            self.schema.views.iter().any(|view| &view.name == name)
                        }
                        Some(SchemaObjectSelection::Trigger(name)) => {
                            self.schema.triggers.iter().any(|trigger| &trigger.name == name)
                        }
                        Some(SchemaObjectSelection::Function(name)) => {
                            self.schema.functions.iter().any(|function| &function.name == name)
                        }
                        None => true,
                    };
                    if !object_exists {
                        self.selected_schema_object = None;
                        if self.active_tab == WorkspaceTab::SchemaObject {
                            self.active_tab = WorkspaceTab::Welcome;
                        }
                    }
                    self.runtime_message = format!(
                        "Schema loaded · {} tables · {} views · {} triggers · {} functions",
                        self.schema.tables.len(),
                        self.schema.views.len(),
                        self.schema.triggers.len(),
                        self.schema.functions.len()
                    );
                    if refresh_selected_table && self.active_tab == WorkspaceTab::Table && self.selected_table.is_some()
                    {
                        self.request_table_info();
                    }
                }
                UiEvent::AgentCompleted {
                    request_id,
                    provider,
                    message,
                } => {
                    if self.agent_request == Some(request_id) {
                        self.agent_request = None;
                        self.agent_pending_prompt = None;
                        self.agent_pending_context = None;
                        let provider_detail = format!("{provider} Responses API · SQL drafts stay unexecuted");
                        self.agent_provider_label = provider;
                        self.agent_provider_detail = provider_detail;
                        self.agent_messages.push(message);
                        self.runtime_message = "Agent response received".to_owned();
                    }
                }
                UiEvent::AgentProviderReady { provider, detail } => {
                    self.agent_provider_label = provider;
                    self.agent_provider_detail = detail;
                }
                UiEvent::AgentFailed { request_id, message } => {
                    if self.agent_request == Some(request_id) {
                        self.agent_request = None;
                        self.runtime_message = "Agent unavailable · switched to offline draft".to_owned();
                        self.fallback_agent_response(Some(&format!("Agent unavailable: {message}")));
                    }
                }
                UiEvent::TableInfoLoaded { request_id, table_info } => {
                    if self.table_info_request == Some(request_id) {
                        if self.table_data_filter_column.is_empty() {
                            self.table_data_filter_column = table_info
                                .columns
                                .first()
                                .map(|column| column.name.clone())
                                .unwrap_or_default();
                        }
                        if self.table_data_sort_column.is_none() {
                            self.table_data_sort_column = table_info
                                .primary_key
                                .as_ref()
                                .and_then(|columns| columns.first().cloned())
                                .or_else(|| table_info.columns.first().map(|column| column.name.clone()));
                        }
                        self.table_info = Some(table_info);
                        self.table_info_error = None;
                        self.table_info_request = None;
                        self.runtime_message = "Table structure loaded".to_owned();
                    }
                }
                UiEvent::TableDdlLoaded { request_id, sql } => {
                    if self.table_ddl_request == Some(request_id) {
                        self.table_ddl = Some(sql);
                        self.ddl_execute_confirmation = false;
                        self.table_ddl_error = None;
                        self.table_ddl_request = None;
                        self.runtime_message = "Table DDL loaded".to_owned();
                    }
                }
                UiEvent::TableDataLoaded {
                    request_id,
                    result,
                    total_rows,
                } => {
                    if self.table_data_request == Some(request_id) {
                        if self.table_data_filter_column.is_empty() {
                            self.table_data_filter_column = result
                                .columns
                                .first()
                                .map(|column| column.name.clone())
                                .unwrap_or_default();
                        }
                        if self.table_data_sort_column.is_none() {
                            self.table_data_sort_column = result.columns.first().map(|column| column.name.clone());
                        }
                        self.table_data_result = Some(result);
                        self.table_data_total_rows = Some(total_rows);
                        self.table_data_error = None;
                        self.table_data_request = None;
                        self.runtime_message = format!("Table data loaded · {total_rows} rows");
                    }
                }
                UiEvent::FilePicked { kind, path, .. } => {
                    if let Some(path) = path {
                        if kind == "sqlite" {
                            self.connection_draft.database = path;
                        } else if kind == "ssh-key" {
                            self.connection_draft.ssh_private_key = path;
                        } else if kind == "backup" {
                            self.backup_output_path = path;
                        } else if kind == "restore" {
                            self.restore_input_path = path;
                        }
                        self.connection_error.clear();
                        self.connection_test_valid = false;
                    } else if kind == "sqlite" || kind == "ssh-key" {
                        self.connection_error = "File selection was cancelled".to_owned();
                        self.connection_test_valid = false;
                    }
                }
                UiEvent::OperationProgress { operation, status, .. } => {
                    self.runtime_message = format!("{operation}: {status}");
                }
                UiEvent::BackupCompleted {
                    output_path,
                    size_bytes,
                    ..
                } => {
                    self.runtime_message = format!("Backup completed · {output_path} · {size_bytes} bytes");
                }
                UiEvent::DdlCompleted {
                    request_id,
                    affected_rows,
                } => {
                    if self.ddl_execution_request == Some(request_id) {
                        self.ddl_execution_request = None;
                        self.ddl_execute_confirmation = false;
                        self.table_ddl_error = None;
                        self.refresh_table_info_after_schema = self.selected_table.is_some();
                        self.runtime_message = format!("DDL applied · {affected_rows} affected rows");
                        if let Some(connection_id) = self.active_connection_id.clone() {
                            self.request_schema_introspection(connection_id, true);
                        }
                    }
                }
                UiEvent::OperationCompleted { request_id, operation } => {
                    let pending_connection_request = self.pending_connection_request == Some(request_id);
                    if matches!(
                        operation.as_str(),
                        "connection.created" | "connection.updated" | "connection.deleted" | "connection.tested"
                    ) && !pending_connection_request
                    {
                        continue;
                    }
                    self.runtime_message = operation.clone();
                    if pending_connection_request {
                        self.pending_connection_request = None;
                    }
                    if matches!(
                        operation.as_str(),
                        "connection.created" | "connection.updated" | "connection.deleted"
                    ) {
                        self.connections_requested = false;
                        self.request_connections_once();
                    }
                    if operation == "connection.tested" && pending_connection_request {
                        self.connection_error.clear();
                        if self.connection_test_draft.as_ref() == Some(&self.connection_draft) {
                            self.connection_test_valid = true;
                            self.runtime_message = "Connection test succeeded".to_owned();
                        } else {
                            self.connection_test_valid = false;
                            self.runtime_message = "Connection changed · test again before saving".to_owned();
                        }
                    }
                    if operation.starts_with("table-row.") {
                        self.data_editing_cell = None;
                        self.data_edit_value.clear();
                        self.data_delete_confirmation = false;
                        if self.staged_apply_request == Some(request_id) {
                            self.staged_apply_completed();
                        } else if self.table_mutation_request == Some(request_id) {
                            self.table_mutation_request = None;
                            self.table_data_result = None;
                            self.table_data_total_rows = None;
                            self.table_data_error = None;
                            self.table_data_request = None;
                            if self.active_tab == WorkspaceTab::Table {
                                self.request_table_data();
                            }
                        }
                    }
                    if operation == "connection.created" || operation == "connection.updated" {
                        self.connection_dialog_open = false;
                        self.editing_connection_id = None;
                    }
                    if operation.starts_with("query") || operation.starts_with("query-folder") {
                        if let Some(connection_id) = self.active_connection_id.clone() {
                            let request_id = self.task_bridge.next_request_id();
                            let _ = self.task_bridge.send(UiCommand::ListSavedQueries {
                                request_id,
                                connection_id,
                            });
                        }
                    }
                    if operation == "connection.deleted" {
                        self.active_connection_id = None;
                        self.connected = false;
                    }
                }
                UiEvent::Connected {
                    request_id,
                    connection_id,
                } => {
                    if self
                        .pending_connection_request
                        .is_some_and(|expected_request| expected_request != request_id)
                    {
                        continue;
                    }
                    self.pending_connection_request = None;
                    self.active_connection_id = Some(connection_id);
                    self.connected = true;
                    self.runtime_message = "Connection established".to_owned();
                    if let Some(connection_id) = self.active_connection_id.clone() {
                        self.request_schema_introspection(connection_id.clone(), false);
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::ListSavedQueries {
                            request_id,
                            connection_id: connection_id.clone(),
                        });
                        let request_id = self.task_bridge.next_request_id();
                        let _ = self.task_bridge.send(UiCommand::ListQueryFolders {
                            request_id,
                            connection_id,
                        });
                    }
                }
                UiEvent::QueryQueued { request_id } => {
                    self.next_query_request = Some(request_id);
                    self.runtime_message = format!("Query queued · request {}", request_id.0);
                }
                UiEvent::QueryCompleted { request_id, result } => {
                    if self.next_query_request == Some(request_id) {
                        self.runtime_message = format!("Query completed · {} rows", result.row_count);
                        self.query_messages.push(self.runtime_message.clone());
                        self.grid_sort_column = None;
                        self.grid_column_widths = vec![180.0; result.columns.len()];
                        self.selected_cell = None;
                        self.selected_row = None;
                        self.copy_status.clear();
                        self.query_result = Some(result);
                        self.output_tab = OutputTab::Results;
                        self.next_query_request = None;
                    }
                }
                UiEvent::ExplainCompleted { request_id, plan } => {
                    if self.explain_request == Some(request_id) {
                        self.explain_request = None;
                        self.explain_plan = Some(plan);
                        self.output_tab = OutputTab::Explain;
                        self.runtime_message = "Query plan ready".to_owned();
                        self.query_messages.push(self.runtime_message.clone());
                    }
                }
                UiEvent::QueryCancelled { request_id } => {
                    if self.next_query_request == Some(request_id) {
                        self.runtime_message = "Query cancelled".to_owned();
                        self.next_query_request = None;
                    }
                }
                UiEvent::QueryFailed { request_id, message } => {
                    if self.pending_connection_request == Some(request_id) {
                        self.pending_connection_request = None;
                        if !self.connection_dialog_open {
                            self.connected = false;
                            self.schema_request = None;
                            self.schema_error = None;
                        }
                        self.connection_error = message.clone();
                        self.runtime_message = format!("Connection failed · {message}");
                    } else if self.schema_request == Some(request_id) {
                        self.schema_request = None;
                        self.schema_error = Some(message.clone());
                        self.runtime_message = format!("Schema introspection failed · {message}");
                    } else if self.staged_apply_request == Some(request_id) {
                        self.staged_apply_failed(&message);
                    } else if self.table_mutation_request == Some(request_id) {
                        self.table_mutation_request = None;
                        self.data_editing_cell = None;
                        self.data_edit_value.clear();
                        self.data_delete_confirmation = false;
                        self.runtime_message = format!("Row mutation failed · {message}");
                    } else if self.table_info_request == Some(request_id) {
                        self.table_info_request = None;
                        self.table_info_error = Some(message.clone());
                        self.runtime_message = format!("Table structure failed · {message}");
                    } else if self.table_ddl_request == Some(request_id) {
                        self.table_ddl_request = None;
                        self.table_ddl_error = Some(message.clone());
                        self.runtime_message = format!("Table DDL failed · {message}");
                    } else if self.table_data_request == Some(request_id) {
                        self.table_data_request = None;
                        self.table_data_error = Some(message.clone());
                        self.runtime_message = format!("Table data failed · {message}");
                    } else if self.ddl_execution_request == Some(request_id) {
                        self.ddl_execution_request = None;
                        self.ddl_execute_confirmation = false;
                        self.table_ddl_error = Some(message.clone());
                        self.runtime_message = format!("DDL execution failed · {message}");
                    } else if self.next_query_request == Some(request_id) {
                        self.runtime_message = format!("Query failed · {message}");
                        self.query_messages.push(self.runtime_message.clone());
                        self.next_query_request = None;
                    } else if self.explain_request == Some(request_id) {
                        self.explain_request = None;
                        self.explain_plan = None;
                        self.output_tab = OutputTab::Messages;
                        self.runtime_message = format!("Explain failed · {message}");
                        self.query_messages.push(self.runtime_message.clone());
                    } else {
                        self.runtime_message = format!("Operation failed · {message}");
                    }
                }
            }
        }
    }

    pub(super) fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if self.palette_mode.is_some() {
            return;
        }
        let text_input_has_focus = ctx.wants_keyboard_input();
        if !text_input_has_focus
            && ctx.input(|i| i.key_pressed(egui::Key::P) && Self::primary_modifier_pressed(i) && i.modifiers.shift)
        {
            self.open_palette(PaletteMode::Commands);
            return;
        }
        if !text_input_has_focus
            && (ctx.input(|i| i.key_pressed(egui::Key::K) && Self::primary_modifier_pressed(i))
                || ctx.input(|i| i.key_pressed(egui::Key::P) && Self::primary_modifier_pressed(i)))
        {
            self.open_palette(PaletteMode::QuickOpen);
            return;
        }
        if !text_input_has_focus && ctx.input(|i| i.key_pressed(egui::Key::B) && Self::primary_modifier_pressed(i)) {
            self.sidebar_open = !self.sidebar_open;
        }
        if !text_input_has_focus && ctx.input(|i| i.key_pressed(egui::Key::F) && Self::primary_modifier_pressed(i)) {
            self.editor_search_open = true;
        }
        if ctx.input(|i| {
            i.key_pressed(egui::Key::F5)
                || (self.query_editor_focused
                    && !self.agent_open
                    && i.key_pressed(egui::Key::Enter)
                    && Self::primary_modifier_pressed(i))
        }) {
            self.dispatch_query();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if let Some(request_id) = self.next_query_request {
                self.cancel_query(request_id);
            } else if self.query_tools_open {
                self.query_tools_open = false;
            } else if self.editor_search_open {
                self.editor_search_open = false;
            } else {
                self.set_agent_open(false, ctx);
            }
        }
    }

    pub(super) fn cancel_query(&mut self, request_id: crate::RequestId) {
        let _ = self.task_bridge.send(UiCommand::CancelQuery { request_id });
        self.runtime_message = "Cancelling query…".to_owned();
    }

    pub(super) fn dispatch_query(&mut self) {
        if self.next_query_request.is_some() {
            return;
        }
        let Some(connection_id) = self.active_connection().map(|connection| connection.id.clone()) else {
            self.runtime_message = "Create or select a connection first".to_owned();
            return;
        };
        let sql = if self.selected_query.trim().is_empty() {
            self.query_text.clone()
        } else {
            self.selected_query.clone()
        };
        if !self.query_history.iter().any(|query| query == &sql) {
            self.query_history.push(sql.clone());
            if self.query_history.len() > 20 {
                self.query_history.remove(0);
            }
        }
        let request_id = self.task_bridge.next_request_id();
        self.next_query_request = Some(request_id);
        self.runtime_message = "Sending query to runtime…".to_owned();
        let _ = self.task_bridge.send(UiCommand::RunQuery {
            request_id,
            connection_id,
            sql,
        });
    }
}
