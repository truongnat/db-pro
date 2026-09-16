//! Saved database tasks activity (#206).
use super::*;
use db_pro_core::domain::saved_task::{SavedTask, SavedTaskPayload, SavedTaskRun, SavedTaskRunStatus, SavedTaskStore};
use egui::RichText;
use lucide_icons::Icon;
use uuid::Uuid;

pub(crate) const SAVED_TASKS_STORAGE_KEY: &str = "dbpro.native.saved-tasks-v1";

impl DbProApp {
    pub(crate) fn load_saved_tasks_from_storage(&mut self, storage: &dyn eframe::Storage) {
        if let Some(raw) = storage.get_string(SAVED_TASKS_STORAGE_KEY) {
            if let Ok(store) = serde_json::from_str::<SavedTaskStore>(&raw) {
                self.saved_task_store = store;
            }
        }
    }

    pub(crate) fn persist_saved_tasks(&self, storage: &mut dyn eframe::Storage) {
        if let Ok(raw) = serde_json::to_string(&self.saved_task_store) {
            storage.set_string(SAVED_TASKS_STORAGE_KEY, raw);
        }
    }

    pub(super) fn draw_tasks_activity(&mut self, ui: &mut egui::Ui) {
        section_label(ui, "SAVED TASKS", self.theme);
        ui.add_space(6.0);
        ui.label(
            RichText::new("Manual run only — no scheduler. Secrets stay on the connection.")
                .small()
                .color(self.theme.text_muted),
        );
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if compact_button(ui, "New SQL task", self.theme).clicked() {
                self.begin_new_sql_task();
            }
            if compact_button(ui, "New backup task", self.theme).clicked() {
                self.begin_new_backup_task();
            }
        });
        ui.add_space(8.0);

        if self.saved_task_draft.is_some() {
            self.draw_task_draft_form(ui);
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);
        }

        if self.saved_task_store.tasks.is_empty() {
            empty_state(
                ui,
                Icon::ListTodo,
                "No saved tasks",
                "Create a SQL, export, backup, or maintenance task to run manually.",
                self.theme,
            );
            return;
        }

        let tasks = self.saved_task_store.tasks.clone();
        for task in tasks {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&task.name).strong().color(self.theme.text_primary));
                    ui.label(RichText::new(task.kind().label()).small().color(self.theme.text_muted));
                });
                if !task.description.is_empty() {
                    ui.label(
                        RichText::new(&task.description)
                            .small()
                            .color(self.theme.text_secondary),
                    );
                }
                ui.label(
                    RichText::new(format!("connection · {}", task.connection_id))
                        .small()
                        .color(self.theme.text_muted),
                );
                if let Some(run) = &task.last_run {
                    let status = match run.status {
                        SavedTaskRunStatus::Success => "success",
                        SavedTaskRunStatus::Failed => "failed",
                        SavedTaskRunStatus::Cancelled => "cancelled",
                    };
                    ui.label(
                        RichText::new(format!("last · {status} · {} ms · {}", run.duration_ms, run.message))
                            .small()
                            .color(self.theme.text_muted),
                    );
                }
                ui.horizontal(|ui| {
                    if primary_button(ui, "Run", self.theme).clicked() {
                        self.run_saved_task(task.id);
                    }
                    if compact_button(ui, "Delete", self.theme).clicked() {
                        self.saved_task_store.remove(&task.id);
                        self.saved_tasks_dirty = true;
                    }
                });
            });
            ui.add_space(6.0);
        }

        if !self.saved_task_store.runs.is_empty() {
            ui.separator();
            ui.add_space(6.0);
            section_label(ui, "RECENT RUNS", self.theme);
            for run in self.saved_task_store.runs.iter().take(12) {
                let status = match run.status {
                    SavedTaskRunStatus::Success => "OK",
                    SavedTaskRunStatus::Failed => "FAIL",
                    SavedTaskRunStatus::Cancelled => "CANCEL",
                };
                ui.label(
                    RichText::new(format!(
                        "{status} · {} ms · {}",
                        run.duration_ms,
                        run.message.chars().take(80).collect::<String>()
                    ))
                    .small()
                    .monospace()
                    .color(self.theme.text_muted),
                );
            }
        }
    }

    fn draw_task_draft_form(&mut self, ui: &mut egui::Ui) {
        let Some(draft) = self.saved_task_draft.as_mut() else {
            return;
        };
        ui.label(RichText::new("New task").strong().color(self.theme.text_primary));
        input_full_width(ui, &mut draft.name, "Name", self.theme);
        input_full_width(ui, &mut draft.description, "Description", self.theme);
        match &mut draft.payload {
            SavedTaskPayload::Sql { sql } => {
                ui.label(RichText::new("SQL").small().color(self.theme.text_muted));
                ui.add(
                    egui::TextEdit::multiline(sql)
                        .desired_width(ui.available_width())
                        .desired_rows(4)
                        .code_editor(),
                );
            }
            SavedTaskPayload::Backup {
                output_path,
                custom_format,
            } => {
                input_full_width(ui, output_path, "Output path", self.theme);
                ui.checkbox(custom_format, "Custom (pg_dump -Fc) format");
            }
            SavedTaskPayload::Export { table, format } => {
                let mut table_text = table.clone().unwrap_or_default();
                input_full_width(ui, &mut table_text, "Table (optional)", self.theme);
                *table = if table_text.trim().is_empty() {
                    None
                } else {
                    Some(table_text)
                };
                input_full_width(ui, format, "Format (csv/tsv/json)", self.theme);
            }
            SavedTaskPayload::Maintenance { operation, target } => {
                input_full_width(ui, operation, "Operation (vacuum/analyze)", self.theme);
                let mut target_text = target.clone().unwrap_or_default();
                input_full_width(ui, &mut target_text, "Target (optional)", self.theme);
                *target = if target_text.trim().is_empty() {
                    None
                } else {
                    Some(target_text)
                };
            }
        }
        ui.horizontal(|ui| {
            if primary_button(ui, "Save task", self.theme).clicked() {
                self.commit_task_draft();
            }
            if compact_button(ui, "Cancel", self.theme).clicked() {
                self.saved_task_draft = None;
            }
        });
    }

    fn begin_new_sql_task(&mut self) {
        let connection_id = self
            .active_connection_id
            .clone()
            .unwrap_or_else(|| self.connections.first().map(|c| c.id.clone()).unwrap_or_default());
        self.saved_task_draft = Some(SavedTask {
            id: Uuid::new_v4(),
            name: "SQL task".into(),
            description: String::new(),
            connection_id,
            payload: SavedTaskPayload::Sql {
                sql: self.active_query_text().to_owned(),
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_run: None,
        });
    }

    fn begin_new_backup_task(&mut self) {
        let connection_id = self
            .active_connection_id
            .clone()
            .unwrap_or_else(|| self.connections.first().map(|c| c.id.clone()).unwrap_or_default());
        self.saved_task_draft = Some(SavedTask {
            id: Uuid::new_v4(),
            name: "Backup".into(),
            description: String::new(),
            connection_id,
            payload: SavedTaskPayload::Backup {
                output_path: String::new(),
                custom_format: false,
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_run: None,
        });
    }

    fn commit_task_draft(&mut self) {
        let Some(mut draft) = self.saved_task_draft.take() else {
            return;
        };
        draft.updated_at = chrono::Utc::now();
        match self.saved_task_store.upsert(draft) {
            Ok(()) => {
                self.saved_tasks_dirty = true;
                self.runtime_message = "Saved task".to_owned();
            }
            Err(err) => {
                self.runtime_message = err;
            }
        }
    }

    pub(crate) fn run_saved_task(&mut self, task_id: Uuid) {
        let Some(task) = self.saved_task_store.tasks.iter().find(|t| t.id == task_id).cloned() else {
            self.runtime_message = "Saved task not found".to_owned();
            return;
        };
        if task.payload.is_destructive()
            && self.settings.general.confirm_destructive_queries
            && !self.saved_task_confirm_destructive
        {
            self.pending_destructive_task_id = Some(task_id);
            self.runtime_message =
                "Destructive task requires confirmation — check Confirm in the Tasks pane".to_owned();
            return;
        }
        self.pending_destructive_task_id = None;
        self.saved_task_confirm_destructive = false;

        let started = chrono::Utc::now();
        let result = self.dispatch_saved_task_payload(&task);
        let finished = chrono::Utc::now();
        let duration_ms = (finished - started).num_milliseconds().max(0) as u64;
        let (status, message) = match result {
            Ok(msg) => (SavedTaskRunStatus::Success, msg),
            Err(msg) => (SavedTaskRunStatus::Failed, msg),
        };
        self.saved_task_store.record_run(SavedTaskRun {
            id: Uuid::new_v4(),
            task_id,
            status,
            started_at: started,
            finished_at: finished,
            duration_ms,
            message: message.clone(),
        });
        self.saved_tasks_dirty = true;
        self.runtime_message = message;
    }

    fn dispatch_saved_task_payload(&mut self, task: &SavedTask) -> Result<String, String> {
        match &task.payload {
            SavedTaskPayload::Sql { sql } => {
                if sql.trim().is_empty() {
                    return Err("SQL payload is empty".to_owned());
                }
                self.active_connection_id = Some(task.connection_id.clone());
                self.set_active_query_text(sql);
                self.active_tab = WorkspaceTab::Query;
                // Task-level confirmation already satisfied destructive policy (#206).
                let version = self.active_query_buffer_version();
                let execution_range = (0, sql.len());
                self.send_query_run(task.connection_id.clone(), sql.clone(), execution_range, version, false);
                Ok("Dispatched SQL task to query runtime".to_owned())
            }
            SavedTaskPayload::Backup {
                output_path,
                custom_format,
            } => {
                if output_path.trim().is_empty() {
                    return Err("backup output path is required".to_owned());
                }
                let request_id = self.task_bridge.next_request_id();
                self.task_bridge
                    .send(UiCommand::Backup {
                        request_id,
                        connection_id: task.connection_id.clone(),
                        output_path: output_path.clone(),
                        custom_format: *custom_format,
                    })
                    .map_err(|e| e.to_string())?;
                Ok("Dispatched backup task".to_owned())
            }
            SavedTaskPayload::Export { table, format } => {
                let fmt = format.to_ascii_lowercase();
                if let Some(table) = table {
                    self.active_connection_id = Some(task.connection_id.clone());
                    self.set_active_query_text(format!("SELECT * FROM {table} LIMIT 1000"));
                    self.active_tab = WorkspaceTab::Query;
                    self.dispatch_query();
                    Ok(format!(
                        "Opened export source for {table} ({fmt}) — use Export on results"
                    ))
                } else {
                    Err("export task needs a table or an active result set".to_owned())
                }
            }
            SavedTaskPayload::Maintenance { operation, target } => {
                let op = operation.trim().to_ascii_lowercase();
                let sql = match (op.as_str(), target.as_deref()) {
                    ("vacuum", Some(t)) => format!("VACUUM {t}"),
                    ("vacuum", None) => "VACUUM".to_owned(),
                    ("analyze", Some(t)) => format!("ANALYZE {t}"),
                    ("analyze", None) => "ANALYZE".to_owned(),
                    _ => return Err(format!("unsupported maintenance operation: {operation}")),
                };
                self.active_connection_id = Some(task.connection_id.clone());
                self.set_active_query_text(&sql);
                self.active_tab = WorkspaceTab::Query;
                self.dispatch_query();
                Ok(format!("Dispatched maintenance: {sql}"))
            }
        }
    }
}
