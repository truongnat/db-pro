//! Saved database tasks activity (#206).
use super::*;
use db_pro_core::domain::saved_task::{
    SavedTask, SavedTaskPayload, SavedTaskRun, SavedTaskRunStatus, SavedTaskRunTrigger, SavedTaskStore, TaskSchedule,
};
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
        let action = saved_tasks_surface_view::SavedTasksSurfaceContext {
            theme: self.theme,
            saved_tasks: &mut self.saved_tasks,
        }
        .draw(ui);
        if let Some(action) = action {
            self.apply_saved_tasks_surface_action(action);
        }
    }

    fn apply_saved_tasks_surface_action(&mut self, action: saved_tasks_surface_view::SavedTasksSurfaceAction) {
        use saved_tasks_surface_view::SavedTasksSurfaceAction;

        match action {
            SavedTasksSurfaceAction::NewSqlTask => self.begin_new_sql_task(),
            SavedTasksSurfaceAction::NewBackupTask => self.begin_new_backup_task(),
            SavedTasksSurfaceAction::SaveDraft => self.commit_task_draft(),
            SavedTasksSurfaceAction::CancelDraft => self.saved_tasks.draft = None,
            SavedTasksSurfaceAction::RunTask(task_id) => {
                self.run_saved_task(task_id, SavedTaskRunTrigger::Manual);
            }
            SavedTasksSurfaceAction::DisableSchedule(task_id) => {
                let _ = self.saved_tasks.store.set_schedule_enabled(&task_id, false);
                self.saved_tasks.dirty = true;
            }
            SavedTasksSurfaceAction::EnableSchedule(task_id) => self.enable_task_schedule(task_id, 60),
            SavedTasksSurfaceAction::DeleteTask(task_id) => {
                self.saved_tasks.store.remove(&task_id);
                self.saved_tasks.dirty = true;
            }
        }
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
                sql: self.query.session.active_text().to_owned(),
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
            SavedTaskPayload::Sql { sql } => self.dispatch_sql_task(task, sql),
            SavedTaskPayload::Backup {
                output_path,
                custom_format,
            } => self.dispatch_backup_task(task, output_path, *custom_format),
            SavedTaskPayload::Export { table, format } => self.dispatch_export_task(task, table.as_deref(), format),
            SavedTaskPayload::Maintenance { operation, target } => {
                self.dispatch_maintenance_task(task, operation, target.as_deref())
            }
        }
    }

    fn dispatch_sql_task(&mut self, task: &SavedTask, sql: &str) -> Result<String, String> {
        if sql.trim().is_empty() {
            return Err("SQL payload is empty".to_owned());
        }
        *self.connection.lifecycle.active_connection_id_mut() = Some(task.connection_id.clone());
        self.set_active_query_text(sql);
        self.workspace.active_tab = WorkspaceTab::Query;
        // Task-level confirmation already satisfied destructive policy (#206).
        let version = self.query.session.active_buffer_version();
        let execution_range = (0, sql.len());
        if self.send_query_run(
            task.connection_id.clone(),
            sql.to_owned(),
            execution_range,
            version,
            false,
        ) {
            Ok("Dispatched SQL task to query runtime".to_owned())
        } else {
            Err(self.feedback.runtime_message.clone())
        }
    }

    fn dispatch_backup_task(
        &mut self,
        task: &SavedTask,
        output_path: &str,
        custom_format: bool,
    ) -> Result<String, String> {
        if output_path.trim().is_empty() {
            return Err("backup output path is required".to_owned());
        }
        let request_id = self.task_bridge.next_request_id();
        if !self.dispatch_command(UiCommand::Backup {
            request_id,
            connection_id: task.connection_id.clone(),
            output_path: output_path.to_owned(),
            custom_format,
        }) {
            return Err("Runtime worker unavailable".to_owned());
        }
        Ok("Dispatched backup task".to_owned())
    }

    fn dispatch_export_task(&mut self, task: &SavedTask, table: Option<&str>, format: &str) -> Result<String, String> {
        let Some(table) = table else {
            return Err("export task needs a table or an active result set".to_owned());
        };
        let fmt = format.to_ascii_lowercase();
        *self.connection.lifecycle.active_connection_id_mut() = Some(task.connection_id.clone());
        self.set_active_query_text(format!("SELECT * FROM {table} LIMIT 1000"));
        self.workspace.active_tab = WorkspaceTab::Query;
        if self.dispatch_query() {
            Ok(format!(
                "Opened export source for {table} ({fmt}) — use Export on results"
            ))
        } else {
            Err(self.feedback.runtime_message.clone())
        }
    }

    fn dispatch_maintenance_task(
        &mut self,
        task: &SavedTask,
        operation: &str,
        target: Option<&str>,
    ) -> Result<String, String> {
        let op = operation.trim().to_ascii_lowercase();
        let sql = match (op.as_str(), target) {
            ("vacuum", Some(target)) => format!("VACUUM {target}"),
            ("vacuum", None) => "VACUUM".to_owned(),
            ("analyze", Some(target)) => format!("ANALYZE {target}"),
            ("analyze", None) => "ANALYZE".to_owned(),
            _ => return Err(format!("unsupported maintenance operation: {operation}")),
        };
        *self.connection.lifecycle.active_connection_id_mut() = Some(task.connection_id.clone());
        self.set_active_query_text(&sql);
        self.workspace.active_tab = WorkspaceTab::Query;
        if self.dispatch_query() {
            Ok(format!("Dispatched maintenance: {sql}"))
        } else {
            Err(self.feedback.runtime_message.clone())
        }
    }
}
