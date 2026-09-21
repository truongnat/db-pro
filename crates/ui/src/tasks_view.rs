//! Saved database tasks activity (#206).
use super::*;
use db_pro_core::domain::saved_task::{SavedTask, SavedTaskPayload, SavedTaskRunTrigger, SavedTaskStore};
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
                if let Err(error) = self.saved_tasks.disable_schedule(task_id) {
                    self.feedback.runtime_message = error;
                }
            }
            SavedTasksSurfaceAction::EnableSchedule(task_id) => self.enable_task_schedule(task_id, 60),
            SavedTasksSurfaceAction::DeleteTask(task_id) => {
                self.saved_tasks.delete_task(task_id);
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
        self.saved_tasks.begin_sql_draft(
            connection_id,
            self.query.session.active_text().to_owned(),
            chrono::Utc::now(),
        );
    }

    fn begin_new_backup_task(&mut self) {
        let connection_id = self
            .connection
            .lifecycle
            .active_connection_id()
            .map(str::to_owned)
            .unwrap_or_else(|| self.connection.catalog.get(0).map(|c| c.id.clone()).unwrap_or_default());
        self.saved_tasks.begin_backup_draft(connection_id, chrono::Utc::now());
    }

    fn commit_task_draft(&mut self) {
        match self.saved_tasks.commit_draft(chrono::Utc::now()) {
            Ok(()) => {
                self.feedback.runtime_message = "Saved task".to_owned();
            }
            Err(err) => {
                self.feedback.runtime_message = err;
            }
        }
    }

    pub(crate) fn enable_task_schedule(&mut self, task_id: Uuid, interval_secs: u64) {
        match self
            .saved_tasks
            .enable_schedule(task_id, interval_secs, chrono::Utc::now())
        {
            Ok(()) => {
                self.feedback.runtime_message = format!("Schedule enabled every {interval_secs}s (app must stay open)");
            }
            Err(error) => self.feedback.runtime_message = error,
        }
    }

    pub(crate) fn tick_saved_task_scheduler(&mut self) {
        for (task_id, trigger) in self.saved_tasks.take_due_tasks(chrono::Utc::now()) {
            self.run_saved_task(task_id, trigger);
        }
    }

    pub(crate) fn run_saved_task(&mut self, task_id: Uuid, trigger: SavedTaskRunTrigger) {
        let task = match self.saved_tasks.prepare_run(
            task_id,
            trigger,
            self.preferences.settings.general.confirm_destructive_queries,
        ) {
            saved_task_state::SavedTaskRunPreparation::Ready(task) => *task,
            saved_task_state::SavedTaskRunPreparation::NeedsConfirmation => {
                self.feedback.runtime_message =
                    "Destructive task requires confirmation — check Confirm in the Tasks pane".to_owned();
                return;
            }
            saved_task_state::SavedTaskRunPreparation::Blocked(message) => {
                self.feedback.runtime_message = message;
                return;
            }
            saved_task_state::SavedTaskRunPreparation::NotFound => {
                self.feedback.runtime_message = "Saved task not found".to_owned();
                return;
            }
        };

        let started = chrono::Utc::now();
        let result = self.dispatch_saved_task_payload(&task);
        let finished = chrono::Utc::now();
        let (status, message) = self.saved_tasks.record_run(task_id, trigger, started, finished, result);
        if status == db_pro_core::domain::saved_task::SavedTaskRunStatus::Failed
            && trigger != SavedTaskRunTrigger::Manual
        {
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
