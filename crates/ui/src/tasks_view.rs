//! Saved database tasks activity (#206).
use super::*;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use db_pro_core::domain::saved_task::{
    SavedTask, SavedTaskPayload, SavedTaskRun, SavedTaskRunStatus, SavedTaskRunTrigger, SavedTaskStore, TaskSchedule,
};
use egui::RichText;
use lucide_icons::Icon;
use uuid::Uuid;

pub(crate) const SAVED_TASKS_STORAGE_KEY: &str = "dbpro.native.saved-tasks-v1";

impl DbProApp {
    pub(crate) fn load_saved_tasks_from_storage(&mut self, storage: &dyn eframe::Storage) {
        if let Some(raw) = storage.get_string(SAVED_TASKS_STORAGE_KEY) {
            if let Ok(store) = serde_json::from_str::<SavedTaskStore>(&raw) {
                self.saved_tasks.store = store;
            }
        }
    }

    pub(crate) fn persist_saved_tasks(&self, storage: &mut dyn eframe::Storage) {
        if let Ok(raw) = serde_json::to_string(&self.saved_tasks.store) {
            storage.set_string(SAVED_TASKS_STORAGE_KEY, raw);
        }
    }

    pub(super) fn draw_tasks_activity(&mut self, ui: &mut egui::Ui) {
        section_label(ui, "SAVED TASKS", self.theme);
        ui.add_space(6.0);
        ui.label(
            RichText::new(
                "Scheduler runs only while this app is open — there is no background daemon. Secrets stay on the connection.",
            )
            .small()
            .color(self.theme.text_muted),
        );
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            if Button::new(self.theme)
                .text("New SQL task")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.begin_new_sql_task();
            }
            if Button::new(self.theme)
                .text("New backup task")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.begin_new_backup_task();
            }
        });
        ui.add_space(8.0);

        if self.saved_tasks.draft.is_some() {
            self.draw_task_draft_form(ui);
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);
        }

        if self.saved_tasks.store.tasks.is_empty() {
            empty_state(
                ui,
                Icon::ListTodo,
                "No saved tasks",
                "Create a SQL, export, backup, or maintenance task to run manually.",
                self.theme,
            );
            return;
        }

        let tasks = self.saved_tasks.store.tasks.clone();
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
                    if Button::new(self.theme)
                        .text("Run")
                        .variant(ButtonVariant::Default)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.run_saved_task(task.id, SavedTaskRunTrigger::Manual);
                    }
                    if task.schedule.as_ref().is_some_and(|s| s.enabled) {
                        if Button::new(self.theme)
                            .text("Disable schedule")
                            .variant(ButtonVariant::Secondary)
                            .size(ButtonSize::Sm)
                            .show(ui)
                            .clicked()
                        {
                            let _ = self.saved_tasks.store.set_schedule_enabled(&task.id, false);
                            self.saved_tasks.dirty = true;
                        }
                    } else if Button::new(self.theme)
                        .text("Schedule 60s")
                        .variant(ButtonVariant::Secondary)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.enable_task_schedule(task.id, 60);
                    }
                    if Button::new(self.theme)
                        .text("Delete")
                        .variant(ButtonVariant::Destructive)
                        .size(ButtonSize::Sm)
                        .show(ui)
                        .clicked()
                    {
                        self.saved_tasks.store.remove(&task.id);
                        self.saved_tasks.dirty = true;
                    }
                });
                if let Some(schedule) = &task.schedule {
                    let next = schedule
                        .next_run_at
                        .map(|t| t.to_rfc3339())
                        .unwrap_or_else(|| "—".into());
                    ui.label(
                        RichText::new(format!(
                            "schedule · {} · next {next}",
                            if schedule.enabled { "on" } else { "off" }
                        ))
                        .small()
                        .color(self.theme.text_muted),
                    );
                }
            });
            ui.add_space(6.0);
        }

        if !self.saved_tasks.store.runs.is_empty() {
            ui.separator();
            ui.add_space(6.0);
            section_label(ui, "RECENT RUNS", self.theme);
            for run in self.saved_tasks.store.runs.iter().take(12) {
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
        let Some(draft) = self.saved_tasks.draft.as_mut() else {
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
            if Button::new(self.theme)
                .text("Save task")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.commit_task_draft();
            }
            if Button::new(self.theme)
                .text("Cancel")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .show(ui)
                .clicked()
            {
                self.saved_tasks.draft = None;
            }
        });
    }

    fn begin_new_sql_task(&mut self) {
        let connection_id = self
            .connection
            .lifecycle
            .active_connection_id()
            .map(str::to_owned)
            .unwrap_or_else(|| self.connection.catalog.get(0).map(|c| c.id.clone()).unwrap_or_default());
        self.saved_tasks.draft = Some(SavedTask {
            id: Uuid::new_v4(),
            name: "SQL task".into(),
            description: String::new(),
            connection_id,
            payload: SavedTaskPayload::Sql {
                sql: self.query_session_state.active_text().to_owned(),
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_run: None,
            schedule: None,
        });
    }

    fn begin_new_backup_task(&mut self) {
        let connection_id = self
            .connection
            .lifecycle
            .active_connection_id()
            .map(str::to_owned)
            .unwrap_or_else(|| self.connection.catalog.get(0).map(|c| c.id.clone()).unwrap_or_default());
        self.saved_tasks.draft = Some(SavedTask {
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
            schedule: None,
        });
    }

    fn commit_task_draft(&mut self) {
        let Some(mut draft) = self.saved_tasks.draft.take() else {
            return;
        };
        draft.updated_at = chrono::Utc::now();
        match self.saved_tasks.store.upsert(draft) {
            Ok(()) => {
                self.saved_tasks.dirty = true;
                self.feedback.runtime_message = "Saved task".to_owned();
            }
            Err(err) => {
                self.feedback.runtime_message = err;
            }
        }
    }

    pub(crate) fn enable_task_schedule(&mut self, task_id: Uuid, interval_secs: u64) {
        let Some(task) = self.saved_tasks.store.tasks.iter_mut().find(|t| t.id == task_id) else {
            return;
        };
        let mut schedule = TaskSchedule::every_secs(interval_secs);
        if task.payload.is_destructive() {
            self.feedback.runtime_message =
                "Destructive tasks cannot be scheduled without an explicit policy allow".to_owned();
            return;
        }
        schedule.enabled = true;
        task.schedule = Some(schedule);
        task.updated_at = chrono::Utc::now();
        self.saved_tasks.dirty = true;
        self.feedback.runtime_message = format!("Schedule enabled every {interval_secs}s (app must stay open)");
    }

    pub(crate) fn tick_saved_task_scheduler(&mut self) {
        let due = self.saved_tasks.store.due_scheduled_task_ids(chrono::Utc::now());
        if due.is_empty() {
            return;
        }
        self.saved_tasks.dirty = true;
        for task_id in due {
            let trigger = {
                let retry = self
                    .saved_tasks
                    .store
                    .tasks
                    .iter()
                    .find(|t| t.id == task_id)
                    .and_then(|t| t.schedule.as_ref())
                    .is_some_and(|s| s.retry_count > 0);
                if retry {
                    SavedTaskRunTrigger::Retry
                } else {
                    SavedTaskRunTrigger::Scheduled
                }
            };
            self.run_saved_task(task_id, trigger);
        }
    }

    pub(crate) fn run_saved_task(&mut self, task_id: Uuid, trigger: SavedTaskRunTrigger) {
        let Some(task) = self.saved_tasks.store.tasks.iter().find(|t| t.id == task_id).cloned() else {
            self.feedback.runtime_message = "Saved task not found".to_owned();
            return;
        };
        if trigger == SavedTaskRunTrigger::Manual
            && task.payload.is_destructive()
            && self.preferences.settings.general.confirm_destructive_queries
            && !self.saved_tasks.confirm_destructive
        {
            self.saved_tasks.pending_destructive_task_id = Some(task_id);
            self.feedback.runtime_message =
                "Destructive task requires confirmation — check Confirm in the Tasks pane".to_owned();
            return;
        }
        if trigger != SavedTaskRunTrigger::Manual
            && task.payload.is_destructive()
            && !task.schedule.as_ref().is_some_and(|s| s.allow_destructive)
        {
            self.feedback.runtime_message = "Scheduled destructive task blocked by policy".to_owned();
            return;
        }
        self.saved_tasks.pending_destructive_task_id = None;
        self.saved_tasks.confirm_destructive = false;

        let started = chrono::Utc::now();
        let result = self.dispatch_saved_task_payload(&task);
        let finished = chrono::Utc::now();
        let duration_ms = (finished - started).num_milliseconds().max(0) as u64;
        let (status, message) = match result {
            Ok(msg) => (SavedTaskRunStatus::Success, msg),
            Err(msg) => (SavedTaskRunStatus::Failed, msg),
        };
        self.saved_tasks.store.record_run(SavedTaskRun {
            id: Uuid::new_v4(),
            task_id,
            status,
            started_at: started,
            finished_at: finished,
            duration_ms,
            message: message.clone(),
            trigger,
        });
        self.saved_tasks.dirty = true;
        if status == SavedTaskRunStatus::Failed && trigger != SavedTaskRunTrigger::Manual {
            self.feedback.runtime_message = format!("Scheduled task failed: {message}");
        } else {
            self.feedback.runtime_message = message;
        }
    }

    fn dispatch_saved_task_payload(&mut self, task: &SavedTask) -> Result<String, String> {
        match &task.payload {
            SavedTaskPayload::Sql { sql } => {
                if sql.trim().is_empty() {
                    return Err("SQL payload is empty".to_owned());
                }
                *self.connection.lifecycle.active_connection_id_mut() = Some(task.connection_id.clone());
                self.set_active_query_text(sql);
                self.workspace.active_tab = WorkspaceTab::Query;
                // Task-level confirmation already satisfied destructive policy (#206).
                let version = self.query_session_state.active_buffer_version();
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
                    *self.connection.lifecycle.active_connection_id_mut() = Some(task.connection_id.clone());
                    self.set_active_query_text(format!("SELECT * FROM {table} LIMIT 1000"));
                    self.workspace.active_tab = WorkspaceTab::Query;
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
                *self.connection.lifecycle.active_connection_id_mut() = Some(task.connection_id.clone());
                self.set_active_query_text(&sql);
                self.workspace.active_tab = WorkspaceTab::Query;
                self.dispatch_query();
                Ok(format!("Dispatched maintenance: {sql}"))
            }
        }
    }
}
